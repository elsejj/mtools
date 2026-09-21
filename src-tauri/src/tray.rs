use tauri::{
  menu::{Menu, MenuItem, PredefinedMenuItem},
  tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
  Emitter, Manager,
};

pub fn setup_system_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
  // 1. 菜单项创建
  let toggle_i = MenuItem::with_id(app, "toggle", "显示 / 隐藏主窗口", true, None::<&str>)?;
  let clipboard_i = MenuItem::with_id(
    app,
    "process_clipboard",
    "读取剪贴板并处理",
    true,
    None::<&str>,
  )?;
  let sep = PredefinedMenuItem::separator(app)?;
  let quit_i = MenuItem::with_id(app, "quit", "退出 mtools", true, None::<&str>)?;

  let menu = Menu::with_items(app, &[&toggle_i, &clipboard_i, &sep, &quit_i])?;

  // 2. 托盘图标构建
  let mut builder = TrayIconBuilder::new()
    .tooltip("mtools - 智能快捷生产力工具")
    .menu(&menu)
    .show_menu_on_left_click(false);

  if let Some(icon) = app.default_window_icon() {
    builder = builder.icon(icon.clone());
  }

  builder
    .on_menu_event(|app, event| match event.id.as_ref() {
      "toggle" => {
        if let Some(w) = app.get_webview_window("main") {
          if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
          } else {
            let _ = w.show();
            let _ = w.set_focus();
          }
        }
      }
      "process_clipboard" => {
        if let Some(w) = app.get_webview_window("main") {
          let _ = w.show();
          let _ = w.set_focus();
          let app_clone = app.clone();
          tauri::async_runtime::spawn(async move {
            if let Ok(payload) = crate::process_clipboard_internal(&app_clone).await {
              let _ = app_clone.emit("payload-ready", payload);
            }
          });
        }
      }
      "quit" => {
        app.exit(0);
      }
      _ => {}
    })
    .on_tray_icon_event(|tray, event| match event {
      TrayIconEvent::DoubleClick {
        button: MouseButton::Left,
        ..
      } => {
        let app = tray.app_handle();
        if let Some(w) = app.get_webview_window("main") {
          if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
          } else {
            let _ = w.show();
            let _ = w.set_focus();
          }
        }
      }
      _ => {}
    })
    .build(app)?;

  Ok(())
}
