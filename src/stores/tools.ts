import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type {
  ToolDefinition,
  ToolScoreItem,
  EnrichedPayload,
  PostActionConfig,
  CliExecuteResponse,
  SaveFileResponse,
} from '@/types';
import { tauriApi } from '@/lib/tauri';

export const DEFAULT_TOOLS: ToolDefinition[] = [
  {
    id: 'json-formatter',
    name: 'JSON 格式化',
    icon: 'IconCode',
    description: '格式化并高亮 JSON 字符串，验证语法有效性',
    category: 'developer',
    isCustom: false,
    enabled: true,
    sortOrder: 1,
    matcher: {
      acceptedTypes: ['text'],
      requiredFormats: ['json'],
      basePriority: 90,
    },
    type: 'code',
    postAction: { type: 'none' },
    codeConfig: {
      script: '',
      outputType: 'json',
    },
  },
  {
    id: 'jwt-inspector',
    name: 'JWT 解析',
    icon: 'IconKey',
    description: '解析 JWT Token 结构，查看 Header 与 Payload 声明',
    category: 'developer',
    isCustom: false,
    enabled: true,
    sortOrder: 2,
    matcher: {
      acceptedTypes: ['text'],
      requiredFormats: ['jwt'],
      basePriority: 85,
    },
    type: 'code',
    postAction: { type: 'none' },
    codeConfig: {
      script: '',
      outputType: 'json',
    },
  },
  {
    id: 'timestamp-converter',
    name: '时间戳转换',
    icon: 'IconClock',
    description: 'Unix 秒/毫秒时间戳与本地可读时间互转',
    category: 'developer',
    isCustom: false,
    enabled: true,
    sortOrder: 3,
    matcher: {
      acceptedTypes: ['text'],
      requiredFormats: ['time'],
      basePriority: 80,
    },
    type: 'code',
    postAction: { type: 'none' },
    codeConfig: {
      script: '',
      outputType: 'text',
    },
  },
  {
    id: 'url-codec',
    name: 'URL 编解码',
    icon: 'IconLink',
    description: 'URL Encode/Decode 与 Query 参数结构化解析',
    category: 'developer',
    isCustom: false,
    enabled: true,
    sortOrder: 4,
    matcher: {
      acceptedTypes: ['text'],
      requiredFormats: ['url'],
      basePriority: 75,
    },
    type: 'code',
    postAction: { type: 'none' },
    codeConfig: {
      script: '',
      outputType: 'json',
    },
  },
  {
    id: 'llm-assistant',
    name: 'AI 助手',
    icon: 'IconSparkles',
    description: '智能文本分析、翻译或图像多模态处理',
    category: 'ai',
    isCustom: false,
    enabled: true,
    sortOrder: 5,
    matcher: {
      acceptedTypes: ['text', 'image'],
      basePriority: 50,
    },
    type: 'llm',
    postAction: { type: 'none' },
    llmConfig: {
      useSystemProvider: true,
      systemPrompt: '你是一个高效精准的个人桌面生产力助手。请直接输出分析或处理结果，保持严谨简洁。',
      userPromptTemplate: '{{input}}',
      stream: true,
    },
  },
  {
    id: 'cli-runner',
    name: '外部 CLI',
    icon: 'IconTerminal2',
    description: '通过管道将输入数据传递给本地可执行程序',
    category: 'utilities',
    isCustom: false,
    enabled: true,
    sortOrder: 6,
    matcher: {
      acceptedTypes: ['text'],
      basePriority: 20,
    },
    type: 'cli',
    postAction: { type: 'none' },
    cliConfig: {
      command: 'cat',
      args: [],
      stdinMode: 'pipe',
      timeoutMs: 5000,
    },
  },
];

export const useToolStore = defineStore('tools', () => {
  const tools = ref<ToolDefinition[]>(DEFAULT_TOOLS);
  const activeToolId = ref<string>('json-formatter');
  const isExecuting = ref<boolean>(false);
  const executionError = ref<string | null>(null);
  const executionOutput = ref<string>('');
  const lastSavedFilePath = ref<string | null>(null);

  // Getters
  const activeTool = computed(() => {
    return tools.value.find((t) => t.id === activeToolId.value) || tools.value[0];
  });

  const toolsByCategory = computed(() => {
    const map: Record<string, ToolDefinition[]> = {
      developer: [],
      text: [],
      ai: [],
      utilities: [],
    };
    for (const tool of tools.value) {
      if (tool.enabled) {
        if (!map[tool.category]) {
          map[tool.category] = [];
        }
        map[tool.category].push(tool);
      }
    }
    return map;
  });

  function getRecommendedTools(candidates: ToolScoreItem[]): ToolDefinition[] {
    if (!candidates || candidates.length === 0) {
      return tools.value.filter((t) => t.enabled).slice(0, 5);
    }
    const scoreMap = new Map<string, number>();
    for (const c of candidates) {
      scoreMap.set(c.toolId, c.score);
    }

    return [...tools.value]
      .filter((t) => t.enabled)
      .sort((a, b) => {
        const scoreA = scoreMap.get(a.id) ?? 0;
        const scoreB = scoreMap.get(b.id) ?? 0;
        return scoreB - scoreA;
      });
  }

  // Actions
  async function loadTools() {
    try {
      const savedTools = await tauriApi.loadToolsConfig();
      if (savedTools && savedTools.length > 0) {
        // Merge or replace
        const toolMap = new Map<string, ToolDefinition>();
        for (const t of DEFAULT_TOOLS) {
          toolMap.set(t.id, t);
        }
        for (const t of savedTools) {
          toolMap.set(t.id, t);
        }
        tools.value = Array.from(toolMap.values()).sort((a, b) => a.sortOrder - b.sortOrder);
      }
    } catch (err) {
      console.warn('Failed to load tools from database, using defaults:', err);
    }
  }

  async function saveTool(tool: ToolDefinition) {
    const idx = tools.value.findIndex((t) => t.id === tool.id);
    if (idx >= 0) {
      tools.value[idx] = tool;
    } else {
      tools.value.push(tool);
    }
    try {
      await tauriApi.saveToolConfig(tool);
    } catch (err) {
      console.error('Failed to persist tool to database:', err);
    }
  }

  async function deleteTool(toolId: string) {
    tools.value = tools.value.filter((t) => t.id !== toolId);
    try {
      await tauriApi.deleteToolConfig(toolId);
    } catch (err) {
      console.error('Failed to delete tool from database:', err);
    }
    if (activeToolId.value === toolId && tools.value.length > 0) {
      activeToolId.value = tools.value[0].id;
    }
  }

  function setActiveTool(toolId: string) {
    const target = tools.value.find((t) => t.id === toolId);
    if (target) {
      activeToolId.value = toolId;
      executionError.value = null;
      lastSavedFilePath.value = null;
    }
  }

  function autoSelectBestTool(recommendedToolId?: string, candidateScores?: ToolScoreItem[]) {
    if (recommendedToolId && tools.value.some((t) => t.id === recommendedToolId && t.enabled)) {
      setActiveTool(recommendedToolId);
      return;
    }
    if (candidateScores && candidateScores.length > 0) {
      const highest = candidateScores[0];
      if (highest && highest.score > 0 && tools.value.some((t) => t.id === highest.toolId && t.enabled)) {
        setActiveTool(highest.toolId);
      }
    }
  }

  // Trigger post-actions like copy or save to file
  async function handlePostAction(
    postAction: PostActionConfig,
    content: string,
    toolName: string
  ): Promise<{ savedPath?: string; copied?: boolean }> {
    const result: { savedPath?: string; copied?: boolean } = {};
    if (postAction.type === 'copy_to_clipboard') {
      try {
        await navigator.clipboard.writeText(content);
        result.copied = true;
      } catch (e) {
        console.error('PostAction: Failed to copy to clipboard', e);
      }
    } else if (postAction.type === 'save_to_file' && postAction.saveConfig) {
      try {
        const ext = postAction.saveConfig.extension || 'txt';
        const dir = postAction.saveConfig.directory || toolName.toLowerCase().replace(/\s+/g, '_');
        const saveRes: SaveFileResponse = await tauriApi.saveContentToFile({
          targetDirectory: dir,
          extension: ext,
          content,
        });
        result.savedPath = saveRes.fullPath;
        lastSavedFilePath.value = saveRes.fullPath;
      } catch (e) {
        console.error('PostAction: Failed to save file', e);
      }
    }
    return result;
  }

  // Tool execution dispatcher
  async function executeTool(payload: EnrichedPayload): Promise<string> {
    const tool = activeTool.value;
    if (!tool) return '';

    isExecuting.value = true;
    executionError.value = null;
    lastSavedFilePath.value = null;
    const startTime = Date.now();

    try {
      let output = '';

      if (tool.type === 'code') {
        // For code tools, frontend uses preprocessedResult or simple transform
        if (payload.preprocessedResult?.formattedText) {
          output = payload.preprocessedResult.formattedText;
        } else {
          output = payload.actualContent;
        }
      } else if (tool.type === 'cli' && tool.cliConfig) {
        const cliRes: CliExecuteResponse = await tauriApi.executeCliCommand({
          command: tool.cliConfig.command,
          args: tool.cliConfig.args || [],
          workingDir: tool.cliConfig.workingDir,
          stdinContent: tool.cliConfig.stdinMode === 'pipe' ? payload.actualContent : undefined,
          envVars: tool.cliConfig.env,
          timeoutMs: tool.cliConfig.timeoutMs || 5000,
        });
        output = cliRes.stdout || cliRes.stderr;
        if (cliRes.exitCode !== 0 && !cliRes.stdout && cliRes.stderr) {
          executionError.value = `CLI failed with exit code ${cliRes.exitCode}: ${cliRes.stderr}`;
        }
      } else if (tool.type === 'llm') {
        // LLM tool execution placeholder (will be fully integrated in Phase 4)
        output = `[AI 响应预览]:\n输入内容已就绪 (${payload.actualContent.slice(0, 100)}...)`;
      }

      executionOutput.value = output;

      // Handle Post Action
      const postActionResult = await handlePostAction(tool.postAction, output, tool.name);

      // Record in history
      const durationMs = Date.now() - startTime;
      await tauriApi.addHistoryRecord({
        toolId: tool.id,
        toolName: tool.name,
        payloadType: payload.payloadType,
        inputSummary: payload.actualContent.slice(0, 100),
        inputText: payload.actualContent,
        inputImagePath: (payload.metadata as any)?.localCachePath,
        outputContent: output,
        postActionType: tool.postAction.type,
        outputFilePath: postActionResult.savedPath,
        status: executionError.value ? 'error' : 'success',
        durationMs,
      }).catch((e) => console.warn('Failed to record history item:', e));

      return output;
    } catch (err: any) {
      executionError.value = err?.message || String(err);
      console.error('Execution error:', err);
      return '';
    } finally {
      isExecuting.value = false;
    }
  }

  return {
    // State
    tools,
    activeToolId,
    isExecuting,
    executionError,
    executionOutput,
    lastSavedFilePath,

    // Getters
    activeTool,
    toolsByCategory,
    getRecommendedTools,

    // Actions
    loadTools,
    saveTool,
    deleteTool,
    setActiveTool,
    autoSelectBestTool,
    handlePostAction,
    executeTool,
  };
});

