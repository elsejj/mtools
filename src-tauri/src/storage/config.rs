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

pub const DEFAULT_TOOLS_INFO: &[(&str, &str, &str)] = &[
  (
    "json-formatter",
    "JSON 格式化",
    "格式化并高亮 JSON 字符串，验证语法有效性",
  ),
  (
    "jwt-inspector",
    "JWT 解析",
    "解析 JWT Token 结构，查看 Header 与 Payload 声明",
  ),
  (
    "timestamp-converter",
    "时间戳转换",
    "Unix 秒/毫秒时间戳与本地可读时间互转",
  ),
  (
    "calculator",
    "计算器",
    "支持中英文数量单位（K/M/G/万/亿等）的智能表达式计算器",
  ),
  (
    "url-codec",
    "URL 编解码",
    "URL Encode/Decode 与 Query 参数结构化解析",
  ),
  (
    "ocr-extractor",
    "OCR 识图提取",
    "多模态 AI 识别并提取图片中的所有排版文字",
  ),
  (
    "llm-translate",
    "AI 翻译与润色",
    "中英双语即时翻译与文案表达润色",
  ),
  (
    "cli-runner",
    "外部 CLI",
    "通过管道将输入数据传递给本地命令行工具 (如 jq/cat)",
  ),
];

pub fn get_evaluation_config(conn: &Connection) -> crate::models::EvaluationModelConfig {
  if let Ok(val) = get_system_config(conn) {
    if let Some(eval_val) = val.get("evaluationModel") {
      if let Ok(cfg) =
        serde_json::from_value::<crate::models::EvaluationModelConfig>(eval_val.clone())
      {
        return cfg;
      }
    }
  }
  crate::models::EvaluationModelConfig::default()
}

pub fn get_enabled_tool_descriptions(conn: &Connection) -> Vec<(String, String)> {
  use std::collections::HashMap;

  let mut tool_map: HashMap<String, (String, bool)> = HashMap::new();
  for (id, _name, desc) in DEFAULT_TOOLS_INFO {
    tool_map.insert(id.to_string(), (desc.to_string(), true));
  }

  if let Ok(val) = get_tools_config(conn) {
    if let Some(arr) = val.as_array() {
      for item in arr {
        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
          let enabled = item
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
          let desc = item
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
          if !desc.is_empty() {
            tool_map.insert(id.to_string(), (desc.to_string(), enabled));
          } else if let Some(existing) = tool_map.get_mut(id) {
            existing.1 = enabled;
          }
        }
      }
    }
  }

  let mut list: Vec<(String, String)> = tool_map
    .into_iter()
    .filter(|(_, (_, enabled))| *enabled)
    .map(|(id, (desc, _))| (id, desc))
    .collect();
  list.sort_by(|a, b| a.0.cmp(&b.0));
  list
}
