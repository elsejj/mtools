pub mod cli;
pub mod commands;
pub mod decoder;
pub mod models;
pub mod post_action;
pub mod sendkey;
pub mod sniffer;
pub mod storage;
pub mod tray;

use base64::prelude::*;
use models::*;
use sniffer::{SniffInput, SnifferRegistry};
use storage::AppStorage;
use tauri::{Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

pub fn rgba_to_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
  use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
  let mut buf = Vec::new();
  let encoder = PngEncoder::new(&mut buf);
  encoder
    .write_image(rgba, width, height, ExtendedColorType::Rgba8)
    .map_err(|e| format!("PNG encode error: {}", e))?;
  Ok(buf)
}

pub fn process_clipboard_internal(app: &tauri::AppHandle) -> Result<EnrichedPayload, String> {
  let storage = app.state::<AppStorage>();
  let registry = app.state::<SnifferRegistry>();
  let clipboard = app.clipboard();

  // 1. 尝试读取文本
  if let Ok(text) = clipboard.read_text() {
    if !text.trim().is_empty() {
      let decoded = decoder::DecoderPipeline::decode(&text);
      return match decoded {
        decoder::DecodedOutput::Text {
          text: dec_text,
          trace,
        } => {
          let input = SniffInput::Text(&dec_text);
          Ok(registry.execute(&input, text.clone(), dec_text.clone(), trace, None))
        }
        decoder::DecodedOutput::Image {
          bytes,
          mime_type: _,
          trace,
        } => {
          let rel_path = storage::cache::save_image_cache(&storage.data_dir, &bytes).ok();
          let input = SniffInput::Image(&bytes);
          let base64_str = format!("data:image/png;base64,{}", BASE64_STANDARD.encode(&bytes));
          Ok(registry.execute(&input, text.clone(), base64_str, trace, rel_path))
        }
        decoder::DecodedOutput::PassThrough => {
          let input = SniffInput::Text(&text);
          Ok(registry.execute(&input, text.clone(), text.clone(), Vec::new(), None))
        }
      };
    }
  }

  // 2. 尝试读取剪贴板图像
  if let Ok(img) = clipboard.read_image() {
    let rgba = img.rgba();
    let png_bytes = rgba_to_png(img.width(), img.height(), rgba).unwrap_or_else(|_| rgba.to_vec());
    let rel_path = storage::cache::save_image_cache(&storage.data_dir, &png_bytes).ok();
    let base64_str = format!(
      "data:image/png;base64,{}",
      BASE64_STANDARD.encode(&png_bytes)
    );
    let input = SniffInput::Image(&png_bytes);
    return Ok(registry.execute(&input, base64_str.clone(), base64_str, Vec::new(), rel_path));
  }

  Err("Clipboard is empty or unsupported format".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
      if let Some(cmd) = args.get(1) {
        if cmd.eq_ignore_ascii_case("copy") {
          let _ = sendkey::send_keys(sendkey::COPY_KEY);
          // 等待前台软件写入系统剪贴板
          std::thread::sleep(std::time::Duration::from_millis(200));
        }
      }

      if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        let _ = app.emit("launch", &args);

        // 唤起后若存在有效剪切板内容，直接推送到前端
        if let Ok(payload) = process_clipboard_internal(app) {
          // println!("Got clipboard: {:?}", payload);
          let _ = app.emit("payload-ready", payload);
        }
      }
    }))
    .plugin(tauri_plugin_opener::init())
    .on_window_event(|window, event| {
      match event {
        tauri::WindowEvent::Resized(_size) => {
          //storage::window::record_window_geometry(window);
        }
        tauri::WindowEvent::Moved(_pos) => {}
        tauri::WindowEvent::CloseRequested { api, .. } => {
          // 点击关闭按钮时隐藏至系统托盘，不直接杀掉应用进程
          api.prevent_close();
          let _ = window.hide();
          storage::window::record_window_geometry(window);
        }
        tauri::WindowEvent::Destroyed | tauri::WindowEvent::Focused(false) => {
          storage::window::record_window_geometry(window);
        }
        _ => {}
      }
    })
    .setup(|app| {
      let data_dir = dirs::data_dir()
        .map(|p| p.join("mtools"))
        .unwrap_or_else(|| std::path::PathBuf::from("./data"));

      let storage = AppStorage::init(data_dir)?;

      // 启动时自动恢复上次记忆的窗口尺寸与位置
      if let Some(_window) = app.get_webview_window("main") {
        if let Ok(conn) = storage.db.lock() {
          if let Ok(Some(_geom)) = storage::window::get_window_geometry(&conn) {
            //storage::window::apply_window_geometry(&window, &geom);
          }
        }
      }
      app.manage(storage);

      let registry = SnifferRegistry::new();
      app.manage(registry);

      // 初始化系统托盘（图标、菜单、点击交互）
      if let Err(err) = tray::setup_system_tray(app) {
        eprintln!("Failed to setup system tray: {}", err);
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::simulate_copy,
      commands::simulate_paste,
      commands::write_clipboard_html,
      commands::read_clipboard_image_base64,
      commands::fetch_and_process_clipboard,
      commands::process_custom_content,
      commands::execute_cli_command,
      commands::save_content_to_file,
      commands::show_in_folder,
      commands::add_history_record,
      commands::get_history_records,
      commands::delete_history_record,
      commands::clear_all_history,
      commands::save_image_cache,
      commands::get_image_cache_stats,
      commands::cleanup_image_cache,
      commands::load_system_settings,
      commands::save_system_settings,
      commands::load_tools_config,
      commands::save_tool_config,
      commands::delete_tool_config,
      commands::save_window_geometry,
      commands::restore_window_geometry,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
