export type PayloadType = "text" | "image" | "files";

export interface TextMetadata {
  charCount: number;
  lineCount: number;
  detectedFormat: string; // 'json' | 'url' | 'jwt' | 'xml' | 'sql' | 'cron' | 'plain'
  decodingTrace: string[];
}

export interface ImageMetadata {
  width: number;
  height: number;
  mimeType: string;
  byteSize: number;
  localCachePath: string;
  decodingTrace: string[];
}

export interface PreprocessedResult {
  formattedText?: string;
  diffSource?: string;
  suggestedOutputType: "json" | "text" | "markdown" | "image";
}

export interface ToolScoreItem {
  toolId: string;
  score: number;
}

export interface EnrichedPayload {
  id: string;
  payloadType: PayloadType;
  rawOriginal: string;
  actualContent: string;
  metadata: TextMetadata | ImageMetadata | Record<string, any>;
  tags: string[];
  preprocessedResult?: PreprocessedResult;
  recommendedToolId: string;
  candidateToolScores: ToolScoreItem[];
  createdAt: number;
}

// ----------------- Tool Definition -----------------

export type ToolType = "code" | "llm" | "cli";

export type PostActionType = "none" | "copy_to_clipboard" | "save_to_file";

export interface SaveToFileConfig {
  directory: string;
  extension?: string;
  timeFormat?: string;
}

export interface PostActionConfig {
  type: PostActionType;
  saveConfig?: SaveToFileConfig;
  notifyOnSuccess?: boolean;
}

export interface ToolMatcher {
  acceptedTypes: PayloadType[];
  patterns?: string[];
  requiredFormats?: string[];
  customPredicate?: string;
  basePriority: number;
}

export interface CodeToolConfig {
  script: string;
  outputType: "text" | "json" | "markdown" | "diff";
}

export interface LLMToolConfig {
  useSystemProvider: boolean;
  customProviderId?: string;
  customModel?: string;
  temperature?: number;
  systemPrompt: string;
  userPromptTemplate: string;
  stream: boolean;
}

export interface CLIToolConfig {
  command: string;
  args: string[];
  workingDir?: string;
  stdinMode: "pipe" | "none";
  timeoutMs: number;
  env?: Record<string, string>;
}

export interface ToolDefinition {
  id: string;
  name: string;
  icon: string;
  description: string;
  category: "developer" | "text" | "ai" | "utilities";
  isCustom: boolean;
  enabled: boolean;
  sortOrder: number;
  matcher: ToolMatcher;
  type: ToolType;
  postAction: PostActionConfig;
  codeConfig?: CodeToolConfig;
  llmConfig?: LLMToolConfig;
  cliConfig?: CLIToolConfig;
}

// ----------------- History & System Config -----------------

export interface HistoryRecordItem {
  id: string;
  toolId: string;
  toolName: string;
  payloadType: string;
  inputSummary: string;
  inputText?: string;
  inputImagePath?: string;
  outputContent?: string;
  postActionType: string;
  outputFilePath?: string;
  status: "success" | "error" | "running";
  durationMs: number;
  createdAt: number;
}

export interface NewHistoryRecord {
  toolId: string;
  toolName: string;
  payloadType: string;
  inputSummary: string;
  inputText?: string;
  inputImagePath?: string;
  outputContent?: string;
  postActionType: string;
  outputFilePath?: string;
  status: string;
  durationMs: number;
}

export interface HistoryQuery {
  toolId?: string;
  keyword?: string;
  limit: number;
  offset: number;
}

export interface LLMProvider {
  id: string;
  name: string;
  baseUrl: string;
  apiKey: string;
  defaultModel: string;
}

export interface EvaluationModelConfig {
  baseUrl: string;
  apiKey: string;
  model: string;
}

export interface SystemSettings {
  theme: "light" | "dark" | "system";
  autoCopyResult: boolean;
  closeWindowOnCopy: boolean;
  defaultProviderId: string;
  providers: LLMProvider[];
  evaluationModel?: EvaluationModelConfig;
}

export interface WindowGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
  isMaximized: boolean;
}

export interface CliExecuteRequest {
  command: string;
  args: string[];
  workingDir?: string;
  stdinContent?: string;
  envVars?: Record<string, string>;
  timeoutMs?: number;
}

export interface CliExecuteResponse {
  exitCode?: number;
  stdout: string;
  stderr: string;
  durationMs: number;
}

export interface SaveFileRequest {
  targetDirectory: string;
  extension: string;
  content: string;
  customFilename?: string;
}

export interface SaveFileResponse {
  fullPath: string;
  fileName: string;
  byteSize: number;
}

export interface ImageCacheStats {
  totalBytes: number;
  fileCount: number;
  directoryPath: string;
}
