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
import { executeCodeTool } from '@/lib/engines/codeEngine';
import { streamLLMCompletion } from '@/lib/engines/llmEngine';
import { useSettingsStore } from './settings';

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
      outputType: 'json',
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
    id: 'ocr-extractor',
    name: 'OCR 识图提取',
    icon: 'IconScan',
    description: '多模态 AI 识别并提取图片中的所有排版文字',
    category: 'ai',
    isCustom: false,
    enabled: true,
    sortOrder: 5,
    matcher: {
      acceptedTypes: ['image'],
      basePriority: 95,
    },
    type: 'llm',
    postAction: { type: 'copy_to_clipboard' },
    llmConfig: {
      useSystemProvider: true,
      systemPrompt: '请精准提取图片中的所有文字，忠实保留原始分段与排版格式，直接输出文字，无需寒暄。',
      userPromptTemplate: '请提取该图片中的全部文字内容：',
      stream: true,
      temperature: 0.1,
    },
  },
  {
    id: 'llm-translate',
    name: 'AI 翻译与润色',
    icon: 'IconLanguage',
    description: '中英双语即时翻译与文案表达润色',
    category: 'ai',
    isCustom: false,
    enabled: true,
    sortOrder: 6,
    matcher: {
      acceptedTypes: ['text'],
      basePriority: 45,
    },
    type: 'llm',
    postAction: { type: 'none' },
    llmConfig: {
      useSystemProvider: true,
      systemPrompt: '你是一位精通多语言翻译与专业文案润色的大师。若用户输入中文，请翻译为地道流利的英文；若输入为其他语言，请翻译为通顺规范的中文。直接输出翻译结果。',
      userPromptTemplate: '{{input}}',
      stream: true,
      temperature: 0.3,
    },
  },
  {
    id: 'cli-runner',
    name: '外部 CLI',
    icon: 'IconTerminal2',
    description: '通过管道将输入数据传递给本地命令行工具 (如 jq/cat)',
    category: 'utilities',
    isCustom: false,
    enabled: true,
    sortOrder: 7,
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

/**
 * 实时计算工具匹配得分 (0 ~ 100)
 */
export function calculateToolMatchScore(
  tool: ToolDefinition,
  sample: string,
  sampleType: 'text' | 'image'
): number {
  if (!tool.matcher.acceptedTypes.includes(sampleType)) {
    return 0;
  }
  let score = tool.matcher.basePriority || 10;
  const content = sample.trim();

  // Pattern matching
  if (tool.matcher.patterns && tool.matcher.patterns.length > 0) {
    let matchedPattern = false;
    for (const pat of tool.matcher.patterns) {
      try {
        const reg = new RegExp(pat);
        if (reg.test(content)) {
          matchedPattern = true;
          score += 20;
          break;
        }
      } catch {}
    }
    if (!matchedPattern && tool.matcher.patterns.length > 0) {
      score -= 10;
    }
  }

  // Format checks
  if (tool.matcher.requiredFormats && tool.matcher.requiredFormats.length > 0) {
    for (const fmt of tool.matcher.requiredFormats) {
      if (fmt === 'json') {
        if (
          (content.startsWith('{') && content.endsWith('}')) ||
          (content.startsWith('[') && content.endsWith(']'))
        ) {
          try {
            JSON.parse(content);
            score += 25;
          } catch {}
        }
      } else if (fmt === 'url') {
        if (content.startsWith('http://') || content.startsWith('https://') || content.includes('?')) {
          score += 20;
        }
      } else if (fmt === 'jwt') {
        if (content.split('.').length === 3) {
          score += 25;
        }
      } else if (fmt === 'time') {
        const num = Number(content);
        if (!isNaN(num) && (content.length === 10 || content.length === 13)) {
          score += 20;
        }
      }
    }
  }

  return Math.min(Math.max(score, 0), 100);
}

export const useToolStore = defineStore('tools', () => {
  const tools = ref<ToolDefinition[]>(DEFAULT_TOOLS);
  const activeToolId = ref<string>('json-formatter');
  const editingTool = ref<ToolDefinition | null>(null);
  const isExecuting = ref<boolean>(false);
  const isStreaming = ref<boolean>(false);
  const executionError = ref<string | null>(null);
  const executionOutput = ref<string>('');
  const lastSavedFilePath = ref<string | null>(null);

  let activeAbortController: AbortController | null = null;

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

  async function toggleToolEnabled(toolId: string) {
    const tool = tools.value.find((t) => t.id === toolId);
    if (tool) {
      tool.enabled = !tool.enabled;
      await saveTool(tool);
    }
  }

  async function reorderTool(toolId: string, direction: 'up' | 'down') {
    const idx = tools.value.findIndex((t) => t.id === toolId);
    if (idx < 0) return;

    if (direction === 'up' && idx > 0) {
      const temp = tools.value[idx];
      tools.value[idx] = tools.value[idx - 1];
      tools.value[idx - 1] = temp;
    } else if (direction === 'down' && idx < tools.value.length - 1) {
      const temp = tools.value[idx];
      tools.value[idx] = tools.value[idx + 1];
      tools.value[idx + 1] = temp;
    }

    // Re-index sortOrder
    for (let i = 0; i < tools.value.length; i++) {
      tools.value[i].sortOrder = i + 1;
      await tauriApi.saveToolConfig(tools.value[i]);
    }
  }

  function exportToolsToJson(): string {
    return JSON.stringify(tools.value, null, 2);
  }

  async function importToolsFromJson(jsonStr: string): Promise<{ count: number; error?: string }> {
    try {
      const list = JSON.parse(jsonStr);
      if (!Array.isArray(list)) {
        return { count: 0, error: '导入的 JSON 必须是工具数组' };
      }

      let count = 0;
      for (const item of list) {
        if (item.id && item.name && item.type) {
          await saveTool(item);
          count++;
        }
      }
      return { count };
    } catch (e: any) {
      return { count: 0, error: e?.message || String(e) };
    }
  }

  function setEditingTool(tool: ToolDefinition | null) {
    editingTool.value = tool;
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

  function stopExecution() {
    if (activeAbortController) {
      activeAbortController.abort();
      activeAbortController = null;
    }
    isExecuting.value = false;
    isStreaming.value = false;
  }

  async function handlePostAction(
    postAction: PostActionConfig,
    content: string,
    toolName: string
  ): Promise<{ savedPath?: string; copied?: boolean }> {
    const result: { savedPath?: string; copied?: boolean } = {};
    const settingsStore = useSettingsStore();

    if (postAction.type === 'copy_to_clipboard' || settingsStore.settings.autoCopyResult) {
      try {
        await navigator.clipboard.writeText(content);
        result.copied = true;
      } catch (e) {
        console.error('PostAction: Failed to copy to clipboard', e);
      }
    }

    if (postAction.type === 'save_to_file' && postAction.saveConfig) {
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

  async function executeTool(payload: EnrichedPayload): Promise<string> {
    const tool = activeTool.value;
    if (!tool) return '';

    stopExecution();

    isExecuting.value = true;
    executionError.value = null;
    lastSavedFilePath.value = null;
    executionOutput.value = '';
    const startTime = Date.now();

    try {
      let output = '';

      if (tool.type === 'code') {
        output = executeCodeTool(tool.id, payload.actualContent, payload.preprocessedResult);
        executionOutput.value = output;
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
          executionError.value = `CLI 执行失败 (退出码 ${cliRes.exitCode}): ${cliRes.stderr}`;
        }
        executionOutput.value = output;
      } else if (tool.type === 'llm') {
        const settingsStore = useSettingsStore();
        activeAbortController = new AbortController();
        isStreaming.value = true;

        output = await streamLLMCompletion(tool, payload, settingsStore.settings, {
          signal: activeAbortController.signal,
          onToken: (token) => {
            executionOutput.value += token;
          },
          onComplete: (full) => {
            output = full;
          },
        });
      }

      const postActionResult = await handlePostAction(tool.postAction, output, tool.name);

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
      isStreaming.value = false;
      activeAbortController = null;
    }
  }

  return {
    // State
    tools,
    activeToolId,
    editingTool,
    isExecuting,
    isStreaming,
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
    toggleToolEnabled,
    reorderTool,
    exportToolsToJson,
    importToolsFromJson,
    setEditingTool,
    setActiveTool,
    autoSelectBestTool,
    stopExecution,
    handlePostAction,
    executeTool,
  };
});
