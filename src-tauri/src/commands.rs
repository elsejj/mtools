use crate::models::*;
use crate::sniffer::SnifferRegistry;
use crate::storage::AppStorage;
use crate::{cli, decoder, post_action, sendkey, sniffer, storage};
use base64::prelude::*;
use tauri::{Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;

// 1. 剪贴板与模拟按键
#[tauri::command]
pub async fn simulate_copy() -> Result<(), String> {
  sendkey::send_keys(sendkey::COPY_KEY)?;
  tokio::time::sleep(tokio::time::Duration::from_millis(90)).await;
  Ok(())
}

#[tauri::command]
pub async fn simulate_paste(
  app: tauri::AppHandle,
  content: Option<String>,
  html: Option<String>,
) -> Result<(), String> {
  if let Some(html_content) = html {
    let _ = app.clipboard().write_html(html_content, content.clone());
  } else if let Some(text) = content {
    let _ = app.clipboard().write_text(text);
  }
  if let Some(w) = app.get_webview_window("main") {
    let _ = w.hide();
  }
  tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
  sendkey::send_keys(sendkey::PASTE_KEY)?;
  Ok(())
}

#[tauri::command]
pub async fn write_clipboard_html(
  app: tauri::AppHandle,
  html: String,
  alt_text: Option<String>,
) -> Result<(), String> {
  app
    .clipboard()
    .write_html(html, alt_text)
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_clipboard_image_base64(app: tauri::AppHandle) -> Result<Option<String>, String> {
  let clipboard = app.clipboard();
  if let Ok(img) = clipboard.read_image() {
    let rgba = img.rgba();
    let png_bytes =
      crate::rgba_to_png(img.width(), img.height(), rgba).map_err(|e| e.to_string())?;
    Ok(Some(format!(
      "data:image/png;base64,{}",
      BASE64_STANDARD.encode(&png_bytes)
    )))
  } else {
    Ok(None)
  }
}

// 2. 嗅探与预处理
#[tauri::command]
pub async fn fetch_and_process_clipboard(app: tauri::AppHandle) -> Result<EnrichedPayload, String> {
  crate::process_clipboard_internal(&app)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessContentRequest {
  pub content: String,
}

#[tauri::command]
pub async fn process_custom_content(
  app: tauri::AppHandle,
  request: ProcessContentRequest,
) -> Result<EnrichedPayload, String> {
  let storage = app.state::<AppStorage>();
  let registry = app.state::<SnifferRegistry>();
  let text = request.content;

  let decoded = decoder::DecoderPipeline::decode(&text);
  match decoded {
    decoder::DecodedOutput::Text {
      text: dec_text,
      trace,
    } => {
      let input = sniffer::SniffInput::Text(&dec_text);
      Ok(registry.execute(&input, text.clone(), dec_text.clone(), trace, None))
    }
    decoder::DecodedOutput::Image {
      bytes,
      mime_type: _,
      trace,
    } => {
      let rel_path = storage::cache::save_image_cache(&storage.data_dir, &bytes).ok();
      let input = sniffer::SniffInput::Image(&bytes);
      let base64_str = format!("data:image/png;base64,{}", BASE64_STANDARD.encode(&bytes));
      Ok(registry.execute(&input, text.clone(), base64_str, trace, rel_path))
    }
    decoder::DecodedOutput::PassThrough => {
      let input = sniffer::SniffInput::Text(&text);
      Ok(registry.execute(&input, text.clone(), text.clone(), Vec::new(), None))
    }
  }
}

// 3. 外部 CLI 命令执行
#[tauri::command]
pub async fn execute_cli_command(request: CliExecuteRequest) -> Result<CliExecuteResponse, String> {
  cli::run_cli_command(request).await
}

// 4. 后置处理与文件系统
#[tauri::command]
pub async fn save_content_to_file(request: SaveFileRequest) -> Result<SaveFileResponse, String> {
  post_action::save_content(request)
}

#[tauri::command]
pub async fn show_in_folder(path: String) -> Result<(), String> {
  post_action::reveal_in_folder(&path)
}

// 5. 历史记录管理
#[tauri::command]
pub async fn add_history_record(
  storage: State<'_, AppStorage>,
  record: NewHistoryRecord,
) -> Result<String, String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::history::add_record(&conn, record)
}

#[tauri::command]
pub async fn get_history_records(
  storage: State<'_, AppStorage>,
  query: HistoryQuery,
) -> Result<Vec<HistoryRecordItem>, String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::history::get_records(&conn, query)
}

#[tauri::command]
pub async fn delete_history_record(
  storage: State<'_, AppStorage>,
  id: String,
) -> Result<(), String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::history::delete_record(&conn, &id, &storage.data_dir)
}

#[tauri::command]
pub async fn clear_all_history(storage: State<'_, AppStorage>) -> Result<(), String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::history::clear_all(&conn, &storage.data_dir)
}

// 6. 媒体缓存管控
#[tauri::command]
pub async fn save_image_cache(
  storage: State<'_, AppStorage>,
  image_bytes: Vec<u8>,
) -> Result<String, String> {
  storage::cache::save_image_cache(&storage.data_dir, &image_bytes)
}

#[tauri::command]
pub async fn get_image_cache_stats(
  storage: State<'_, AppStorage>,
) -> Result<ImageCacheStats, String> {
  storage::cache::get_cache_stats(&storage.data_dir)
}

#[tauri::command]
pub async fn cleanup_image_cache(
  storage: State<'_, AppStorage>,
  max_age_days: Option<u32>,
  force_all: bool,
) -> Result<u64, String> {
  storage::cache::cleanup_cache(&storage.data_dir, max_age_days, force_all)
}

// 7. 配置中心持久化
#[tauri::command]
pub async fn load_system_settings(
  storage: State<'_, AppStorage>,
) -> Result<serde_json::Value, String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::config::get_system_config(&conn)
}

#[tauri::command]
pub async fn save_system_settings(
  storage: State<'_, AppStorage>,
  settings: serde_json::Value,
) -> Result<(), String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::config::save_system_config(&conn, &settings)
}

#[tauri::command]
pub async fn load_tools_config(
  storage: State<'_, AppStorage>,
) -> Result<serde_json::Value, String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::config::get_tools_config(&conn)
}

#[tauri::command]
pub async fn save_tool_config(
  storage: State<'_, AppStorage>,
  tool: serde_json::Value,
) -> Result<(), String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::config::save_tool_config(&conn, &tool)
}

#[tauri::command]
pub async fn delete_tool_config(
  storage: State<'_, AppStorage>,
  tool_id: String,
) -> Result<(), String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::config::delete_tool_config(&conn, &tool_id)
}

// 8. 窗口尺寸与位置记忆
#[tauri::command]
pub async fn save_window_geometry(
  storage: State<'_, AppStorage>,
  geometry: WindowGeometry,
) -> Result<(), String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  storage::window::save_window_geometry(&conn, &geometry)
}

#[tauri::command]
pub async fn restore_window_geometry(
  storage: State<'_, AppStorage>,
  window: WebviewWindow,
) -> Result<(), String> {
  let conn = storage.db.lock().map_err(|e| e.to_string())?;
  if let Ok(Some(geom)) = storage::window::get_window_geometry(&conn) {
    storage::window::apply_window_geometry(&window, &geom);
  }
  Ok(())
}
