use crate::models::WindowGeometry;
use rusqlite::{params, Connection};

pub fn save_window_geometry(conn: &Connection, geometry: &WindowGeometry) -> Result<(), String> {
    let is_max = if geometry.is_maximized { 1 } else { 0 };
    conn.execute(
        "INSERT INTO window_geometry (id, x, y, width, height, is_maximized)
         VALUES ('main', ?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
            x = ?1, y = ?2, width = ?3, height = ?4, is_maximized = ?5",
        params![geometry.x, geometry.y, geometry.width, geometry.height, is_max],
    )
    .map_err(|e| format!("Failed to save window geometry: {}", e))?;
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

