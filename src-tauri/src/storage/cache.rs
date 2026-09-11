use crate::models::ImageCacheStats;
use chrono::{Local, Utc};
use std::fs;
use std::path::{Path, PathBuf};

/// 将图片二进制保存至 cache/images/YYYY-MM/YYYY-MM-DD_HH-mm-ss.png
pub fn save_image_cache(data_dir: &Path, bytes: &[u8]) -> Result<String, String> {
  let now = Local::now();
  let month_dir_name = now.format("%Y-%m").to_string();
  let base_filename = now.format("%Y-%m-%d_%H-%M-%S").to_string();

  let target_dir = data_dir.join("cache").join("images").join(&month_dir_name);
  fs::create_dir_all(&target_dir)
    .map_err(|e| format!("Failed to create image cache directory: {}", e))?;

  // 冲突消解
  let mut counter = 0;
  let mut file_path: PathBuf;
  let mut final_filename: String;

  loop {
    final_filename = if counter == 0 {
      format!("{}.png", base_filename)
    } else {
      format!("{}_{}.png", base_filename, counter)
    };
    file_path = target_dir.join(&final_filename);
    if !file_path.exists() {
      break;
    }
    counter += 1;
  }

  fs::write(&file_path, bytes).map_err(|e| format!("Failed to write cache image: {}", e))?;

  // 返回相对路径，便于移动与跨设备展示
  let rel_path = format!("cache/images/{}/{}", month_dir_name, final_filename);
  Ok(rel_path)
}

/// 统计图片缓存的占用大小与文件总数
pub fn get_cache_stats(data_dir: &Path) -> Result<ImageCacheStats, String> {
  let images_dir = data_dir.join("cache").join("images");
  if !images_dir.exists() {
    return Ok(ImageCacheStats {
      total_bytes: 0,
      file_count: 0,
      directory_path: images_dir.to_string_lossy().to_string(),
    });
  }

  let mut total_bytes = 0u64;
  let mut file_count = 0usize;

  count_dir_recursive(&images_dir, &mut total_bytes, &mut file_count);

  Ok(ImageCacheStats {
    total_bytes,
    file_count,
    directory_path: images_dir.to_string_lossy().to_string(),
  })
}

fn count_dir_recursive(dir: &Path, total_bytes: &mut u64, file_count: &mut usize) {
  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        count_dir_recursive(&path, total_bytes, file_count);
      } else if path.is_file() {
        if let Ok(meta) = entry.metadata() {
          *total_bytes += meta.len();
          *file_count += 1;
        }
      }
    }
  }
}

/// 清理图片缓存，支持 TTL 过期天数清理或强制全部清空
pub fn cleanup_cache(
  data_dir: &Path,
  max_age_days: Option<u32>,
  force_all: bool,
) -> Result<u64, String> {
  let images_dir = data_dir.join("cache").join("images");
  if !images_dir.exists() {
    return Ok(0);
  }

  let mut freed_bytes = 0u64;

  if force_all {
    let _ = fs::remove_dir_all(&images_dir);
    let _ = fs::create_dir_all(&images_dir);
    return Ok(freed_bytes);
  }

  if let Some(days) = max_age_days {
    let threshold_duration = chrono::Duration::days(days as i64);
    let now = Utc::now();

    if let Ok(entries) = fs::read_dir(&images_dir) {
      for entry in entries.flatten() {
        let month_path = entry.path();
        if month_path.is_dir() {
          if let Ok(files) = fs::read_dir(&month_path) {
            for f in files.flatten() {
              let file_path = f.path();
              if file_path.is_file() {
                if let Ok(meta) = f.metadata() {
                  if let Ok(modified) = meta.modified() {
                    let modified_dt: chrono::DateTime<Utc> = modified.into();
                    if now.signed_duration_since(modified_dt) > threshold_duration {
                      freed_bytes += meta.len();
                      let _ = fs::remove_file(file_path);
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  Ok(freed_bytes)
}
