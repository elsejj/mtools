import { fetch } from "@tauri-apps/plugin-http";
import type { ToolDefinition, EnrichedPayload, SystemSettings, LLMProvider } from "@/types";

export interface LLMStreamOptions {
  onToken: (chunk: string) => void;
  onComplete?: (fullText: string) => void;
  onError?: (err: Error) => void;
  signal?: AbortSignal;
}

/**
 * 解析级联配置获取生效的 Provider 与模型
 */
export function resolveLLMConfig(
  tool: ToolDefinition,
  settings: SystemSettings,
): { provider: LLMProvider; model: string; temperature: number } {
  const llmConfig = tool.llmConfig;
  const providers = settings.providers || [];

  let provider: LLMProvider | undefined;

  if (llmConfig && !llmConfig.useSystemProvider && llmConfig.customProviderId) {
    provider = providers.find((p) => p.id === llmConfig.customProviderId);
  }

  if (!provider) {
    provider = providers.find((p) => p.id === settings.defaultProviderId) ||
      providers[0] || {
        id: "default",
        name: "Default",
        baseUrl: "https://api.openai.com/v1",
        apiKey: "",
        defaultModel: "gpt-4o",
      };
  }

  const model = llmConfig?.customModel || provider.defaultModel || "gpt-4o";
  const temperature = llmConfig?.temperature ?? 0.7;

  return { provider, model, temperature };
}

/**
 * 组装多模态图文消息体
 */
export function buildMessages(tool: ToolDefinition, payload: EnrichedPayload): any[] {
  const llmConfig = tool.llmConfig;
  const systemPrompt =
    llmConfig?.systemPrompt || "你是一个专业高效的助手。请严谨、准确地直接输出处理结果。";
  const userTemplate = llmConfig?.userPromptTemplate || "{{input}}";

  const messages: any[] = [
    {
      role: "system",
      content: systemPrompt,
    },
  ];

  if (payload.payloadType === "image") {
    // Multimodal image payload
    let imageUrl = "";
    if (payload.rawOriginal && payload.rawOriginal.startsWith("data:image/")) {
      imageUrl = payload.rawOriginal;
    } else if (payload.actualContent && payload.actualContent.startsWith("data:image/")) {
      imageUrl = payload.actualContent;
    } else {
      // If cached image, use placeholder or data URL
      imageUrl = payload.actualContent;
    }

    const textPrompt = userTemplate.replace("{{input}}", "").trim() || "请识别并处理以下图片内容：";

    messages.push({
      role: "user",
      content: [
        {
          type: "text",
          text: textPrompt,
        },
        {
          type: "image_url",
          image_url: {
            url: imageUrl,
            detail: "high",
          },
        },
      ],
    });
  } else {
    // Pure text payload
    const textContent = userTemplate.includes("{{input}}")
      ? userTemplate.replace("{{input}}", payload.actualContent)
      : `${userTemplate}\n\n${payload.actualContent}`;

    messages.push({
      role: "user",
      content: textContent,
    });
  }

  return messages;
}

/**
 * 发起统一 OpenAI 兼容多模态流式请求
 */
export async function streamLLMCompletion(
  tool: ToolDefinition,
  payload: EnrichedPayload,
  settings: SystemSettings,
  options: LLMStreamOptions,
): Promise<string> {
  const { provider, model, temperature } = resolveLLMConfig(tool, settings);
  const messages = buildMessages(tool, payload);

  let fullResponse = "";

  // Clean baseUrl
  let url = provider.baseUrl.trim();
  if (url.endsWith("/")) {
    url = url.slice(0, -1);
  }
  if (!url.endsWith("/chat/completions")) {
    url = `${url}/chat/completions`;
  }

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (provider.apiKey) {
    headers["Authorization"] = `Bearer ${provider.apiKey}`;
  }

  try {
    const response = await fetch(url, {
      method: "POST",
      headers,
      body: JSON.stringify({
        model,
        messages,
        temperature,
        stream: true,
      }),
      signal: options.signal,
    });

    if (!response.ok) {
      const errorText = await response.text();
      let errorMsg = `LLM 请求失败 (${response.status} ${response.statusText})`;
      try {
        const errJson = JSON.parse(errorText);
        if (errJson.error?.message) {
          errorMsg = `${errorMsg}: ${errJson.error.message}`;
        }
      } catch {
        errorMsg = `${errorMsg}: ${errorText}`;
      }
      throw new Error(errorMsg);
    }

    if (!response.body) {
      throw new Error("服务商未返回可读数据流");
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder("utf-8");
    let buffer = "";

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split("\n");
      buffer = lines.pop() || "";

      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith(":")) continue;

        if (trimmed === "data: [DONE]") {
          break;
        }

        if (trimmed.startsWith("data: ")) {
          const jsonStr = trimmed.slice(6).trim();
          try {
            const data = JSON.parse(jsonStr);
            const delta = data.choices?.[0]?.delta?.content;
            if (delta) {
              fullResponse += delta;
              options.onToken(delta);
            }
          } catch {
            // Ignore parse errors on partial chunks
          }
        }
      }
    }

    options.onComplete?.(fullResponse);
    return fullResponse;
  } catch (err: any) {
    if (err.name === "AbortError") {
      console.log("LLM 请求被用户中断");
      options.onComplete?.(fullResponse);
      return fullResponse;
    }
    options.onError?.(err);
    throw err;
  }
}
