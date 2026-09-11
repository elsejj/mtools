pub mod cache;
pub mod config;
pub mod history;
pub mod window;

use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppStorage {
  pub db: Mutex<Connection>,
  pub data_dir: PathBuf,
}

impl AppStorage {
  /// 初始化存储管理器，确保数据目录与数据库表结构建立
  pub fn init(data_dir: PathBuf) -> Result<Self, String> {
    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

    let db_path = data_dir.join("mtools.db");
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open db: {}", e))?;

    // 初始化所有核心数据表
    conn
      .execute_batch(
        "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            -- 历史记录表
            CREATE TABLE IF NOT EXISTS history_records (
                id TEXT PRIMARY KEY,
                tool_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                payload_type TEXT NOT NULL,
                input_summary TEXT NOT NULL,
                input_text TEXT,
                input_image_path TEXT,
                output_content TEXT,
                post_action_type TEXT DEFAULT 'none',
                output_file_path TEXT,
                status TEXT NOT NULL,
                duration_ms INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_history_tool_id ON history_records(tool_id);
            CREATE INDEX IF NOT EXISTS idx_history_created_at ON history_records(created_at DESC);

            -- 配置表
            CREATE TABLE IF NOT EXISTS system_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- 自定义工具表
            CREATE TABLE IF NOT EXISTS tools_config (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                tool_type TEXT NOT NULL,
                json_config TEXT NOT NULL,
                is_custom INTEGER DEFAULT 0,
                sort_order INTEGER DEFAULT 0,
                enabled INTEGER DEFAULT 1,
                updated_at INTEGER NOT NULL
            );

            -- 窗口几何尺寸表
            CREATE TABLE IF NOT EXISTS window_geometry (
                id TEXT PRIMARY KEY,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                is_maximized INTEGER NOT NULL
            );
            ",
      )
      .map_err(|e| format!("Failed to initialize database tables: {}", e))?;

    // 初始化图片缓存根目录
    let image_cache_dir = data_dir.join("cache").join("images");
    fs::create_dir_all(&image_cache_dir)
      .map_err(|e| format!("Failed to create image cache dir: {}", e))?;

    Ok(Self {
      db: Mutex::new(conn),
      data_dir,
    })
  }
}
