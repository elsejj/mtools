use crate::models::WindowGeometry;
use rusqlite::{params, Connection};

pub fn save_window_geometry(conn: &Connection, geometry: &WindowGeometry) -> Result<(), String> {
  //println!("save window size: {:?}", geometry);
  if geometry.is_maximized {
    // 窗口最大化时，更新 is_maximized 为 1，并保留最大化前的正常尺寸与位置
    conn
      .execute(
        "INSERT INTO window_geometry (id, x, y, width, height, is_maximized)
           VALUES ('main', ?1, ?2, ?3, ?4, 1)
           ON CONFLICT(id) DO UPDATE SET is_maximized = 1",
        params![geometry.x, geometry.y, geometry.width, geometry.height],
      )
      .map_err(|e| format!("Failed to save window geometry: {}", e))?;
  } else {
    // 过滤异常极小尺寸（如最小化状态）
    if geometry.width < 400 || geometry.height < 300 {
      return Ok(());
    }
    conn
      .execute(
        "INSERT INTO window_geometry (id, x, y, width, height, is_maximized)
           VALUES ('main', ?1, ?2, ?3, ?4, 0)
           ON CONFLICT(id) DO UPDATE SET
              x = ?1, y = ?2, width = ?3, height = ?4, is_maximized = 0",
        params![geometry.x, geometry.y, geometry.width, geometry.height],
      )
      .map_err(|e| format!("Failed to save window geometry: {}", e))?;
  }
  Ok(())
}

pub fn get_window_geometry(conn: &Connection) -> Result<Option<WindowGeometry>, String> {
  let mut stmt = conn
    .prepare("SELECT x, y, width, height, is_maximized FROM window_geometry WHERE id = 'main'")
    .map_err(|e| e.to_string())?;

  let res = stmt.query_row([], |row| {
    let is_max: i32 = row.get(4)?;
    Ok(WindowGeometry {
      x: row.get(0)?,
      y: row.get(1)?,
      width: row.get(2)?,
      height: row.get(3)?,
      is_maximized: is_max != 0,
    })
  });

  match res {
    Ok(geom) => Ok(Some(geom)),
    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
    Err(e) => Err(e.to_string()),
  }
}

pub fn apply_window_geometry(window: &tauri::WebviewWindow, geom: &WindowGeometry) {
  //println!("set window size: {:?}", geom);
  if geom.is_maximized {
    let _ = window.maximize();
  } else {
    let _ = window.unmaximize();
    if geom.width >= 400 && geom.height >= 300 {
      let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: geom.width,
        height: geom.height,
      }));
    }
  }
}

pub fn record_window_geometry(window: &tauri::Window) {
  use tauri::Manager;
  if let Some(storage) = window
    .app_handle()
    .try_state::<crate::storage::AppStorage>()
  {
    if let Ok(conn) = storage.db.lock() {
      let is_max = window.is_maximized().unwrap_or(false);
      let pos = window.outer_position().unwrap_or_default();
      let size = window.outer_size().unwrap_or_default();
      if size.width >= 400 && size.height >= 300 {
        let _ = save_window_geometry(
          &conn,
          &WindowGeometry {
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
            is_maximized: is_max,
          },
        );
      }
    }
  }
}
