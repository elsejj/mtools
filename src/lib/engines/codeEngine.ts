import type { PreprocessedResult } from "@/types";

/**
 * 1. JSON 格式化与校验引擎
 */
export function formatJson(input: string, minify: boolean = false): string {
  const trimmed = input.trim();
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) {
    throw new Error("无效的 JSON 格式 (内容必须以 { 或 [ 开头)");
  }
  try {
    const parsed = JSON.parse(trimmed);
    if (minify) {
      return JSON.stringify(parsed);
    }
    return JSON.stringify(parsed, null, 2);
  } catch (err: any) {
    throw new Error(`JSON 解析失败: ${err?.message || "语法错误"}`);
  }
}

/**
 * 2. URL 编解码与 Query 参数结构化解析
 */
export function parseUrl(input: string): string {
  const trimmed = input.trim();
  if (trimmed.length > 2048) {
    throw new Error("输入的 URL 长度超出常规限制");
  }
  try {
    // If not starts with protocol, attempt standard http prepend for parsing
    let parseTarget = trimmed;
    if (!trimmed.includes("://") && !trimmed.startsWith("/")) {
      parseTarget = "http://" + trimmed;
    }

    const url = new URL(parseTarget);
    const queryParams: Record<string, string | string[]> = {};

    url.searchParams.forEach((val, key) => {
      if (queryParams[key]) {
        if (Array.isArray(queryParams[key])) {
          (queryParams[key] as string[]).push(val);
        } else {
          queryParams[key] = [queryParams[key] as string, val];
        }
      } else {
        queryParams[key] = val;
      }
    });

    const result = {
      original: trimmed.length > 120 ? `${trimmed.slice(0, 120)}...` : trimmed,
      decodedUrl: decodeURI(trimmed),
      protocol: url.protocol,
      hostname: url.hostname,
      port: url.port || (url.protocol === "https:" ? "443" : "80"),
      pathname: url.pathname,
      queryParams: Object.keys(queryParams).length > 0 ? queryParams : undefined,
      hash: url.hash || undefined,
    };

    return JSON.stringify(result, null, 2);
  } catch {
    // If not a full URL, attempt standard decodeURIComponent
    try {
      const decoded = decodeURIComponent(trimmed);
      return JSON.stringify(
        { decoded, original: trimmed.length > 120 ? `${trimmed.slice(0, 120)}...` : trimmed },
        null,
        2,
      );
    } catch {
      throw new Error("无效的 URL 格式");
    }
  }
}

/**
 * 3. JWT 结构化解析器
 */
export function inspectJwt(token: string): string {
  const trimmed = token.trim();
  const parts = trimmed.split(".");
  if (parts.length !== 3) {
    throw new Error("无效的 JWT 格式 (必须由两点分隔的三段组成)");
  }

  function b64urlDecode(str: string): any {
    try {
      let base64 = str.replace(/-/g, "+").replace(/_/g, "/");
      while (base64.length % 4) {
        base64 += "=";
      }
      const json = atob(base64);
      return JSON.parse(decodeURIComponent(escape(json)));
    } catch {
      try {
        let base64 = str.replace(/-/g, "+").replace(/_/g, "/");
        while (base64.length % 4) {
          base64 += "=";
        }
        return JSON.parse(atob(base64));
      } catch (e: any) {
        return { error: `解码失败: ${e?.message || e}` };
      }
    }
  }

  const header = b64urlDecode(parts[0]);
  const payload = b64urlDecode(parts[1]);

  // Transform timestamps to human readable
  const enrichedPayload = { ...payload };
  const nowSec = Math.floor(Date.now() / 1000);

  if (typeof payload.exp === "number") {
    enrichedPayload["_exp_formatted"] = new Date(payload.exp * 1000).toLocaleString();
    enrichedPayload["_is_expired"] = payload.exp < nowSec;
  }
  if (typeof payload.iat === "number") {
    enrichedPayload["_iat_formatted"] = new Date(payload.iat * 1000).toLocaleString();
  }
  if (typeof payload.nbf === "number") {
    enrichedPayload["_nbf_formatted"] = new Date(payload.nbf * 1000).toLocaleString();
  }

  const result = {
    header,
    payload: enrichedPayload,
    signature: parts[2],
  };

  return JSON.stringify(result, null, 2);
}

/**
 * 4. 时间戳双向转换器
 */
export function convertTimestamp(input: string): string {
  const trimmed = input.trim();

  // Fast-fail if input is excessively long or has image/base64 characteristics
  if (
    trimmed.length > 100 ||
    trimmed.startsWith("data:") ||
    trimmed.startsWith("{") ||
    trimmed.startsWith("[")
  ) {
    throw new Error("无法识别的时间戳格式 (输入内容不符合时间戳或日期特征)");
  }

  const num = Number(trimmed);

  if (!isNaN(num) && trimmed.length >= 9 && trimmed.length <= 16) {
    // Numeric timestamp
    const ms = trimmed.length === 10 ? num * 1000 : num;
    const date = new Date(ms);

    if (isNaN(date.getTime())) {
      throw new Error("无效的时间戳数值");
    }

    const now = Date.now();
    const diffSec = Math.floor((now - ms) / 1000);
    let relative = "";
    if (diffSec >= 0) {
      if (diffSec < 60) relative = `${diffSec} 秒前`;
      else if (diffSec < 3600) relative = `${Math.floor(diffSec / 60)} 分钟前`;
      else if (diffSec < 86400) relative = `${Math.floor(diffSec / 3600)} 小时前`;
      else relative = `${Math.floor(diffSec / 86400)} 天前`;
    } else {
      const future = Math.abs(diffSec);
      if (future < 60) relative = `${future} 秒后`;
      else if (future < 3600) relative = `${Math.floor(future / 60)} 分钟后`;
      else relative = `${Math.floor(future / 3600)} 小时后`;
    }

    const pad = (n: number) => String(n).padStart(2, "0");
    const localFormatted = `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;

    return JSON.stringify(
      {
        timestampSeconds: Math.floor(ms / 1000),
        timestampMilliseconds: ms,
        localTime: localFormatted,
        utcTime: date.toUTCString(),
        iso8601: date.toISOString(),
        relative,
      },
      null,
      2,
    );
  }

  // Date String to Timestamp
  const parsedDate = new Date(trimmed);
  if (!isNaN(parsedDate.getTime())) {
    const ms = parsedDate.getTime();
    return JSON.stringify(
      {
        input: trimmed,
        timestampSeconds: Math.floor(ms / 1000),
        timestampMilliseconds: ms,
        localTime: parsedDate.toLocaleString(),
        iso8601: parsedDate.toISOString(),
      },
      null,
      2,
    );
  }

  throw new Error("无法识别的时间戳或日期格式");
}

/**
 * 统一代码型工具调度器
 */
export function executeCodeTool(
  toolId: string,
  input: string,
  preprocessed?: PreprocessedResult,
): string {
  switch (toolId) {
    case "json-formatter":
      // If Rust already preprocessed pretty json, use it directly, else fallback to formatJson
      if (preprocessed?.formattedText) {
        return preprocessed.formattedText;
      }
      return formatJson(input);

    case "url-codec":
      return parseUrl(input);

    case "jwt-inspector":
      return inspectJwt(input);

    case "timestamp-converter":
      return convertTimestamp(input);

    default:
      if (preprocessed?.formattedText) {
        return preprocessed.formattedText;
      }
      return input;
  }
}
