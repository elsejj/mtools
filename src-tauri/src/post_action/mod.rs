use crate::models::{SaveFileRequest, SaveFileResponse};
use chrono::Local;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// 解析包含 "~" 的路径为绝对路径
pub fn expand_tilde(path_str: &str) -> PathBuf {
  if let Some(stripped) = path_str.strip_prefix("~/") {
    if let Some(home) = dirs::home_dir() {
      return home.join(stripped);
    }
  } else if path_str == "~" {
    if let Some(home) = dirs::home_dir() {
      return home;
    }
  }
  PathBuf::from(path_str)
}

/// 保存处理结果为文件（按时间戳命名、自动冲突消解）
pub fn save_content(req: SaveFileRequest) -> Result<SaveFileResponse, String> {
  let target_dir = expand_tilde(&req.target_directory);
  fs::create_dir_all(&target_dir)
    .map_err(|e| format!("Failed to create target directory: {}", e))?;

  let base_name = req
    .custom_filename
    .unwrap_or_else(|| Local::now().format("%Y-%m-%d_%H-%M-%S").to_string());

  let ext = req.extension.trim_start_matches('.');
  let mut counter = 0;
  let mut file_path: PathBuf;
  let mut final_filename: String;

  loop {
    final_filename = if counter == 0 {
      format!("{}.{}", base_name, ext)
    } else {
      format!("{}_{}.{}", base_name, counter, ext)
    };
    file_path = target_dir.join(&final_filename);
    if !file_path.exists() {
      break;
    }
    counter += 1;
  }

  fs::write(&file_path, req.content.as_bytes())
    .map_err(|e| format!("Failed to write file: {}", e))?;

  let byte_size = req.content.len() as u64;
  let full_path = file_path.to_string_lossy().to_string();

  Ok(SaveFileResponse {
    full_path,
    file_name: final_filename,
    byte_size,
  })
}

/// 在原生文件管理器中定位指定文件或打开目录
pub fn reveal_in_folder(path_str: &str) -> Result<(), String> {
  let target = expand_tilde(path_str);
  if !target.exists() {
    return Err(format!("Path does not exist: {}", path_str));
  }

  #[cfg(target_os = "windows")]
  {
    Command::new("explorer")
      .arg(format!("/select,\"{}\"", target.display()))
      .spawn()
      .map_err(|e| e.to_string())?;
  }

  #[cfg(target_os = "macos")]
  {
    Command::new("open")
      .arg("-R")
      .arg(&target)
      .spawn()
      .map_err(|e| e.to_string())?;
  }

  #[cfg(target_os = "linux")]
  {
    let parent = if target.is_file() {
      target.parent().unwrap_or(&target)
    } else {
      &target
    };
    Command::new("xdg-open")
      .arg(parent)
      .spawn()
      .map_err(|e| e.to_string())?;
  }

  Ok(())
}
