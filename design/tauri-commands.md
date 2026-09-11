# Tauri Rust IPC Command 接口规范 (Tauri Backend Commands API)

本文档整理并定义了 `src-tauri` 后端拟提供给前端 Vue 页面的所有 `#[tauri::command]` 接口。
所有命令均采用强类型输入与输出，异步非阻塞设计，统一错误处理规范。

---

## 1. 剪贴板与模拟按键交互 (Clipboard & Key Injection)

### 1.1 `simulate_copy`

- **说明**: 主动向当前操作系统前台窗口模拟发送 `Ctrl+C` (Windows/Linux) 或 `Cmd+C` (macOS)。通常在用户点击“重新抓取选区”时调用。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn simulate_copy() -> Result<(), String>;
  ```
- **TS 调用**: `await invoke('simulate_copy');`

### 1.2 `simulate_paste`

- **说明**: 隐藏/最小化当前应用窗口后，向之前的目标应用模拟发送 `Ctrl+V` (或 `Cmd+V`)，实现“处理结果一键回填到源文档”。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn simulate_paste(content: Option<String>) -> Result<(), String>;
  ```
- **TS 调用**: `await invoke('simulate_paste', { content: "..." });`

### 1.3 `read_clipboard_image_base64`

- **说明**: 从系统剪贴板读取图像并转码为 Base64（或带 DataURL 前缀），供前端富文本或多模态大模型即时预览。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn read_clipboard_image_base64(app: tauri::AppHandle) -> Result<Option<String>, String>;
  ```
- **TS 调用**: `const base64 = await invoke<string | null>('read_clipboard_image_base64');`

---

## 2. 内容嗅探与预处理引擎 (Sniffer & Preprocessor Engine)

根据“Web 前端尽量不处理数据，做好展示即可”的核心原则，所有内容特征分析与内置格式化（如 JSON 格式化、URL 解码、JWT 拆包等）全部在 Rust 端原生执行。

### 2.1 `fetch_and_process_clipboard`

- **说明**: 主动触发 Rust 读取系统剪贴板 (`app.clipboard()`)，并在 Rust 端直接完成内容嗅探（Sniffing）、数据预处理（如 JSON Pretty 格式化）及路由匹配打分，返回完整的 `EnrichedPayload`。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn fetch_and_process_clipboard(app: tauri::AppHandle) -> Result<EnrichedPayload, String>;
  ```
- **TS 调用**: `const payload = await invoke<EnrichedPayload>('fetch_and_process_clipboard');`

### 2.2 `process_custom_content`

- **说明**: 当用户在前端主工作台输入区手动编辑、键盘输入或粘贴自定义文本时，调用 Rust 后端执行嗅探与格式化预处理（例如将未格式化的 JSON 一键转为 Pretty JSON），前端无需运行重度计算。
- **Rust 签名与类型**:
  ```rust
  #[derive(serde::Deserialize)]
  pub struct ProcessContentRequest {
      pub content: String,
      pub content_type: Option<String>, // 可选提示如 "text" | "json"
  }

  #[tauri::command]
  pub async fn process_custom_content(request: ProcessContentRequest) -> Result<EnrichedPayload, String>;
  ```
- **TS 调用**:
  ```typescript
  const result = await invoke<EnrichedPayload>("process_custom_content", {
    request: { content: rawInput },
  });
  ```

---

## 3. 外部命令执行引擎 (CLI Execution Engine)

### 3.1 `execute_cli_command`

- **说明**: 执行用户指定的外部系统 CLI 命令或脚本（如 `jq`, `prettier`, `python` 等），支持管道输入 `stdin`、参数替换、超时限制及工作目录。
- **Rust 签名与类型**:
  ```rust
  #[derive(serde::Deserialize)]
  pub struct CliExecuteRequest {
      pub command: String,
      pub args: Vec<String>,
      pub working_dir: Option<String>,
      pub stdin_content: Option<String>,
      pub env_vars: Option<std::collections::HashMap<String, String>>,
      pub timeout_ms: Option<u64>,
  }

  #[derive(serde::Serialize)]
  pub struct CliExecuteResponse {
      pub exit_code: Option<i32>,
      pub stdout: String,
      pub stderr: String,
      pub duration_ms: u64,
  }

  #[tauri::command]
  pub async fn execute_cli_command(request: CliExecuteRequest) -> Result<CliExecuteResponse, String>;
  ```
- **TS 调用**:
  ```typescript
  const res = await invoke<CliExecuteResponse>("execute_cli_command", {
    request: {
      command: "jq",
      args: ["."],
      stdinContent: '{"foo": "bar"}',
      timeoutMs: 5000,
    },
  });
  ```

---

## 4. 文件系统与后置处理 (File Operations & Post Actions)

### 3.1 `save_content_to_file`

- **说明**: 工具后置处理专用命令。根据工具专属目录与时间戳规则保存结果至文件。若目标目录不存在，自动递归创建。
- **Rust 签名与类型**:
  ```rust
  #[derive(serde::Deserialize)]
  pub struct SaveFileRequest {
      pub target_directory: String,      // 目标目录 (如 "~/Documents/mtools/ocr")
      pub extension: String,             // 扩展名 (如 "txt", "md", "json")
      pub content: String,               // 文本内容
      pub custom_filename: Option<String>, // 可选自定义名，默认按当前时间 YYYY-MM-DD_HH-mm-ss
  }

  #[derive(serde::Serialize)]
  pub struct SaveFileResponse {
      pub full_path: String,
      pub file_name: String,
      pub byte_size: u64,
  }

  #[tauri::command]
  pub async fn save_content_to_file(request: SaveFileRequest) -> Result<SaveFileResponse, String>;
  ```
- **TS 调用**:
  ```typescript
  const res = await invoke<SaveFileResponse>("save_content_to_file", {
    request: {
      targetDirectory: "~/Documents/mtools/ocr",
      extension: "txt",
      content: "识别结果...",
    },
  });
  ```

### 3.2 `show_in_folder`

- **说明**: 在操作系统的原生文件管理器（Finder / Explorer / Nautilus）中高亮定位指定文件或打开目录。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn show_in_folder(path: String) -> Result<(), String>;
  ```
- **TS 调用**: `await invoke('show_in_folder', { path: '/home/user/file.txt' });`

### 3.3 `pick_directory_dialog`

- **说明**: 打开原生系统目录选择对话框，用于在配置工具后置处理或媒体保存时便捷选择专属目录。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn pick_directory_dialog(default_path: Option<String>) -> Result<Option<String>, String>;
  ```
- **TS 调用**: `const dir = await invoke<string | null>('pick_directory_dialog', { defaultPath: '...' });`

---

## 5. 历史记录管理 (SQLite History Storage)

### 5.1 `add_history_record`

- **说明**: 处理完成后异步插入一条历史记录快照。
- **Rust 签名与类型**:
  ```rust
  #[derive(serde::Deserialize)]
  pub struct NewHistoryRecord {
      pub tool_id: String,
      pub tool_name: String,
      pub payload_type: String, // "text" | "image" | "files"
      pub input_summary: String,
      pub input_text: Option<String>,
      pub input_image_path: Option<String>,
      pub output_content: Option<String>,
      pub post_action_type: String, // "none" | "copy_to_clipboard" | "save_to_file"
      pub output_file_path: Option<String>,
      pub status: String,       // "success" | "error"
      pub duration_ms: u64,
  }

  #[tauri::command]
  pub async fn add_history_record(record: NewHistoryRecord) -> Result<String, String>; // 返回 UUID
  ```

### 5.2 `get_history_records`

- **说明**: 分页查询历史列表，支持工具分类过滤与关键词检索。
- **Rust 签名与类型**:
  ```rust
  #[derive(serde::Deserialize)]
  pub struct HistoryQuery {
      pub tool_id: Option<String>,
      pub keyword: Option<String>,
      pub limit: u32,
      pub offset: u32,
  }

  #[derive(serde::Serialize)]
  pub struct HistoryRecordItem {
      pub id: String,
      pub tool_id: String,
      pub tool_name: String,
      pub payload_type: String,
      pub input_summary: String,
      pub input_text: Option<String>,
      pub input_image_path: Option<String>,
      pub output_content: Option<String>,
      pub post_action_type: String,
      pub output_file_path: Option<String>,
      pub status: String,
      pub duration_ms: u64,
      pub created_at: i64,
  }

  #[tauri::command]
  pub async fn get_history_records(query: HistoryQuery) -> Result<Vec<HistoryRecordItem>, String>;
  ```

### 5.3 `delete_history_record`

- **说明**: 删除单条历史记录，若存在关联的本地缓存图片，联动物理删除对应图片文件。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn delete_history_record(id: String) -> Result<(), String>;
  ```

### 5.4 `clear_all_history`

- **说明**: 清空全部历史记录，并批量删除已关联的历史图片缓存。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn clear_all_history() -> Result<(), String>;
  ```

---

## 6. 图片媒体缓存与存储管控 (Image Cache & Pruning)

### 6.1 `save_image_cache`

- **说明**: 将剪贴板图片保存到 `$APP_DATA_DIR/cache/images/YYYY-MM/YYYY-MM-DD_HH-mm-ss.png`，自动消解重名，返回相对路径。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn save_image_cache(image_bytes: Vec<u8>) -> Result<String, String>;
  ```

### 6.2 `get_image_cache_stats`

- **说明**: 统计当前图片缓存的总物理占用（字节数）和总文件数，供设置界面展示。
- **Rust 签名与类型**:
  ```rust
  #[derive(serde::Serialize)]
  pub struct ImageCacheStats {
      pub total_bytes: u64,
      pub file_count: usize,
      pub directory_path: String,
  }

  #[tauri::command]
  pub async fn get_image_cache_stats() -> Result<ImageCacheStats, String>;
  ```

### 6.3 `cleanup_image_cache`

- **说明**: 执行图片缓存清理。支持按过期天数 (TTL) 或强制全部清空。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn cleanup_image_cache(max_age_days: Option<u32>, force_all: bool) -> Result<u64, String>; // 返回清理释放的字节数
  ```

---

## 7. 配置中心持久化 (Settings & Tool Config)

### 7.1 `load_system_settings` / `save_system_settings`

- **说明**: 加载与保存全局系统设置（主题、服务商列表、默认模型、快捷行为等）。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn load_system_settings() -> Result<serde_json::Value, String>;

  #[tauri::command]
  pub async fn save_system_settings(settings: serde_json::Value) -> Result<(), String>;
  ```

### 7.2 `load_tools_config` / `save_tool_config` / `delete_tool_config`

- **说明**: 读取所有工具列表、更新单个工具（内置或自定义）、删除自定义工具。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn load_tools_config() -> Result<serde_json::Value, String>;

  #[tauri::command]
  pub async fn save_tool_config(tool: serde_json::Value) -> Result<(), String>;

  #[tauri::command]
  pub async fn delete_tool_config(tool_id: String) -> Result<(), String>;
  ```

---

## 8. 窗口尺寸与位置记忆 (Window Geometry Persistence)

### 8.1 `save_window_geometry`

- **说明**: 页面监听到窗口尺寸或位置调整时，通过防抖调用该命令，在 Rust 端持久化物理矩形信息。
- **Rust 签名与类型**:
  ```rust
  #[derive(serde::Deserialize)]
  pub struct WindowGeometry {
      pub x: i32,
      pub y: i32,
      pub width: u32,
      pub height: u32,
      pub is_maximized: bool,
  }

  #[tauri::command]
  pub async fn save_window_geometry(geometry: WindowGeometry) -> Result<(), String>;
  ```

### 8.2 `restore_window_geometry`

- **说明**: 实例唤起时读取保存的矩形数据并应用至主窗口。
- **Rust 签名**:
  ```rust
  #[tauri::command]
  pub async fn restore_window_geometry(window: tauri::WebviewWindow) -> Result<(), String>;
  ```

---

## 9. 命令速查与分类索引汇总表

| 分类             | Command 名称                                    | 主要职责                                           |
| :--------------- | :---------------------------------------------- | :------------------------------------------------- |
| **剪贴板/按键**  | `simulate_copy`                                 | 向系统发送复制按键                                 |
|                  | `simulate_paste`                                | 向前台目标应用回贴结果                             |
|                  | `read_clipboard_image_base64`                   | 提取剪贴板图像 Base64                              |
| **嗅探与预处理** | `fetch_and_process_clipboard`                   | Rust 直接读取剪贴板并嗅探/格式化（如 Pretty JSON） |
|                  | `process_custom_content`                        | 对用户手动输入的内容在 Rust 端执行嗅探与格式化     |
| **外部命令**     | `execute_cli_command`                           | 执行用户配置的 CLI / 脚本命令                      |
| **后置文件操作** | `save_content_to_file`                          | 按时间戳将结果存入专属目录                         |
|                  | `show_in_folder`                                | 原生文件管理器定位文件/文件夹                      |
|                  | `pick_directory_dialog`                         | 唤起原生目录选择对话框                             |
| **历史记录**     | `add_history_record`                            | 新增一条历史快照记录                               |
|                  | `get_history_records`                           | 分页过滤查询历史记录列表                           |
|                  | `delete_history_record`                         | 删除历史并联动清理图片                             |
|                  | `clear_all_history`                             | 清空全部历史记录                                   |
| **媒体缓存**     | `save_image_cache`                              | 图片存盘（年月分级 + 时间命名）                    |
|                  | `get_image_cache_stats`                         | 查询图片缓存占用大小与数量                         |
|                  | `cleanup_image_cache`                           | 缓存淘汰清理（TTL/配额）                           |
| **配置存储**     | `load_system_settings` / `save_system_settings` | 全局系统配置存取                                   |
|                  | `load_tools_config` / `save_tool_config`        | 工具配置列表与修改                                 |
|                  | `delete_tool_config`                            | 删除自定义工具                                     |
| **窗口管理**     | `save_window_geometry`                          | 持久化记忆窗口大小与位置                           |
|                  | `restore_window_geometry`                       | 恢复窗口大小与位置                                 |
