import { invoke } from '@tauri-apps/api/core';
import type {
  CliExecuteRequest,
  CliExecuteResponse,
  EnrichedPayload,
  HistoryQuery,
  HistoryRecordItem,
  ImageCacheStats,
  NewHistoryRecord,
  SaveFileRequest,
  SaveFileResponse,
  SystemSettings,
  ToolDefinition,
  WindowGeometry,
} from '@/types';

export const tauriApi = {
  // 1. 剪贴板与模拟按键
  simulateCopy: () => invoke<void>('simulate_copy'),
  simulatePaste: (content?: string) => invoke<void>('simulate_paste', { content }),
  readClipboardImageBase64: () => invoke<string | null>('read_clipboard_image_base64'),

  // 2. 嗅探与预处理
  fetchAndProcessClipboard: () => invoke<EnrichedPayload>('fetch_and_process_clipboard'),
  processCustomContent: (content: string) =>
    invoke<EnrichedPayload>('process_custom_content', { request: { content } }),

  // 3. 外部命令执行
  executeCliCommand: (request: CliExecuteRequest) =>
    invoke<CliExecuteResponse>('execute_cli_command', { request }),

  // 4. 后置处理与文件系统
  saveContentToFile: (request: SaveFileRequest) =>
    invoke<SaveFileResponse>('save_content_to_file', { request }),
  showInFolder: (path: string) => invoke<void>('show_in_folder', { path }),

  // 5. 历史记录管理
  addHistoryRecord: (record: NewHistoryRecord) =>
    invoke<string>('add_history_record', { record }),
  getHistoryRecords: (query: HistoryQuery) =>
    invoke<HistoryRecordItem[]>('get_history_records', { query }),
  deleteHistoryRecord: (id: string) => invoke<void>('delete_history_record', { id }),
  clearAllHistory: () => invoke<void>('clear_all_history'),

  // 6. 媒体缓存管控
  saveImageCache: (imageBytes: number[]) =>
    invoke<string>('save_image_cache', { imageBytes }),
  getImageCacheStats: () => invoke<ImageCacheStats>('get_image_cache_stats'),
  cleanupImageCache: (maxAgeDays?: number, forceAll: boolean = false) =>
    invoke<number>('cleanup_image_cache', { maxAgeDays, forceAll }),

  // 7. 配置中心持久化
  loadSystemSettings: () => invoke<SystemSettings>('load_system_settings'),
  saveSystemSettings: (settings: SystemSettings) =>
    invoke<void>('save_system_settings', { settings }),
  loadToolsConfig: () => invoke<ToolDefinition[]>('load_tools_config'),
  saveToolConfig: (tool: ToolDefinition) => invoke<void>('save_tool_config', { tool }),
  deleteToolConfig: (toolId: string) => invoke<void>('delete_tool_config', { toolId }),

  // 8. 窗口尺寸与位置记忆
  saveWindowGeometry: (geometry: WindowGeometry) =>
    invoke<void>('save_window_geometry', { geometry }),
  restoreWindowGeometry: () => invoke<void>('restore_window_geometry'),
};

