use rusqlite::{params, Connection};
use serde_json::Value;

pub fn get_system_config(conn: &Connection) -> Result<Value, String> {
  let mut stmt = conn
    .prepare("SELECT value FROM system_config WHERE key = 'settings'")
    .map_err(|e| e.to_string())?;

  let res = stmt.query_row([], |row| {
    let val_str: String = row.get(0)?;
    Ok(val_str)
  });

  match res {
    Ok(str_val) => serde_json::from_str(&str_val).map_err(|e| e.to_string()),
    Err(_) => Ok(serde_json::json!({
        "theme": "system",
        "autoCopyResult": false,
        "closeWindowOnCopy": false,
        "defaultProviderId": "openai",
        "providers": []
    })),
  }
}

pub fn save_system_config(conn: &Connection, settings: &Value) -> Result<(), String> {
  let str_val = serde_json::to_string(settings).map_err(|e| e.to_string())?;
  conn
    .execute(
      "INSERT INTO system_config (key, value) VALUES ('settings', ?1)
         ON CONFLICT(key) DO UPDATE SET value = ?1",
      params![str_val],
    )
    .map_err(|e| format!("Failed to save system config: {}", e))?;
  Ok(())
}

pub fn get_tools_config(conn: &Connection) -> Result<Value, String> {
  let mut stmt = conn
        .prepare("SELECT id, name, tool_type, json_config, is_custom, sort_order, enabled FROM tools_config ORDER BY sort_order ASC")
        .map_err(|e| e.to_string())?;

  let rows = stmt
    .query_map([], |row| {
      let json_str: String = row.get(3)?;
      Ok(json_str)
    })
    .map_err(|e| e.to_string())?;

  let mut list = Vec::new();
  for r in rows {
    if let Ok(json_str) = r {
      if let Ok(val) = serde_json::from_str::<Value>(&json_str) {
        list.push(val);
      }
    }
  }

  Ok(Value::Array(list))
}

pub fn save_tool_config(conn: &Connection, tool: &Value) -> Result<(), String> {
  let id = tool
    .get("id")
    .and_then(|v| v.as_str())
    .ok_or("Missing tool id")?;
  let name = tool
    .get("name")
    .and_then(|v| v.as_str())
    .unwrap_or("Untitled Tool");
  let tool_type = tool.get("type").and_then(|v| v.as_str()).unwrap_or("code");
  let is_custom = if tool
    .get("isCustom")
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
  {
    1
  } else {
    0
  };
  let enabled = if tool
    .get("enabled")
    .and_then(|v| v.as_bool())
    .unwrap_or(true)
  {
    1
  } else {
    0
  };
  let sort_order = tool.get("sortOrder").and_then(|v| v.as_i64()).unwrap_or(0);
  let json_str = serde_json::to_string(tool).map_err(|e| e.to_string())?;
  let now = chrono::Utc::now().timestamp_millis();

  conn.execute(
        "INSERT INTO tools_config (id, name, tool_type, json_config, is_custom, sort_order, enabled, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
            name = ?2,
            tool_type = ?3,
            json_config = ?4,
            is_custom = ?5,
            sort_order = ?6,
            enabled = ?7,
            updated_at = ?8",
        params![id, name, tool_type, json_str, is_custom, sort_order, enabled, now],
    )
    .map_err(|e| format!("Failed to save tool config: {}", e))?;

  Ok(())
}

pub fn delete_tool_config(conn: &Connection, tool_id: &str) -> Result<(), String> {
  conn
    .execute(
      "DELETE FROM tools_config WHERE id = ? AND is_custom = 1",
      params![tool_id],
    )
    .map_err(|e| format!("Failed to delete custom tool: {}", e))?;
  Ok(())
}
