use crate::models::{HistoryQuery, HistoryRecordItem, NewHistoryRecord};
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

pub fn add_record(conn: &Connection, record: NewHistoryRecord) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    conn.execute(
        "INSERT INTO history_records (
            id, tool_id, tool_name, payload_type, input_summary, input_text,
            input_image_path, output_content, post_action_type, output_file_path,
            status, duration_ms, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            id,
            record.tool_id,
            record.tool_name,
            record.payload_type,
            record.input_summary,
            record.input_text,
            record.input_image_path,
            record.output_content,
            record.post_action_type,
            record.output_file_path,
            record.status,
            record.duration_ms,
            now,
        ],
    )
    .map_err(|e| format!("Failed to insert history record: {}", e))?;

    Ok(id)
}

pub fn get_records(conn: &Connection, query: HistoryQuery) -> Result<Vec<HistoryRecordItem>, String> {
    let mut sql = String::from(
        "SELECT id, tool_id, tool_name, payload_type, input_summary, input_text,
                input_image_path, output_content, post_action_type, output_file_path,
                status, duration_ms, created_at
         FROM history_records WHERE 1=1 ",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref tool_id) = query.tool_id {
        if !tool_id.is_empty() {
            sql.push_str("AND tool_id = ? ");
            params_vec.push(Box::new(tool_id.clone()));
        }
    }

    if let Some(ref kw) = query.keyword {
        if !kw.is_empty() {
            sql.push_str("AND (input_summary LIKE ? OR output_content LIKE ?) ");
            let pattern = format!("%{}%", kw);
            params_vec.push(Box::new(pattern.clone()));
            params_vec.push(Box::new(pattern));
        }
    }

    sql.push_str("ORDER BY created_at DESC LIMIT ? OFFSET ?");
    params_vec.push(Box::new(query.limit));
    params_vec.push(Box::new(query.offset));

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("Query prepare error: {}", e))?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(HistoryRecordItem {
                id: row.get(0)?,
                tool_id: row.get(1)?,
                tool_name: row.get(2)?,
                payload_type: row.get(3)?,
                input_summary: row.get(4)?,
                input_text: row.get(5)?,
                input_image_path: row.get(6)?,
                output_content: row.get(7)?,
                post_action_type: row.get(8)?,
                output_file_path: row.get(9)?,
                status: row.get(10)?,
                duration_ms: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?;

    let mut result = Vec::new();
    for item in rows {
        if let Ok(record) = item {
            result.push(record);
        }
    }

    Ok(result)
}

pub fn delete_record(conn: &Connection, id: &str, data_dir: &Path) -> Result<(), String> {
    // 检查是否有本地关联图片
    let mut stmt = conn
        .prepare("SELECT input_image_path FROM history_records WHERE id = ?")
        .map_err(|e| e.to_string())?;

    let image_path: Option<String> = stmt
        .query_row(params![id], |row| row.get(0))
        .unwrap_or(None);

    if let Some(rel_path) = image_path {
        let full_path = data_dir.join(&rel_path);
        if full_path.exists() {
            let _ = fs::remove_file(full_path);
        }
    }

    conn.execute("DELETE FROM history_records WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete history record: {}", e))?;

    Ok(())
}

pub fn clear_all(conn: &Connection, data_dir: &Path) -> Result<(), String> {
    // 找出所有图片并清理
    let mut stmt = conn
        .prepare("SELECT input_image_path FROM history_records WHERE input_image_path IS NOT NULL")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| row.get::<_, Option<String>>(0))
        .map_err(|e| e.to_string())?;

    for r in rows {
        if let Ok(Some(rel_path)) = r {
            let full_path = data_dir.join(&rel_path);
            if full_path.exists() {
                let _ = fs::remove_file(full_path);
            }
        }
    }

    conn.execute("DELETE FROM history_records", [])
        .map_err(|e| format!("Failed to clear history: {}", e))?;

    Ok(())
}

