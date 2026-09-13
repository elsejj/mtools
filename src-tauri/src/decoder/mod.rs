use base64::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub enum DecodedOutput {
  Text {
    text: String,
    trace: Vec<String>,
  },
  Image {
    bytes: Vec<u8>,
    mime_type: String,
    trace: Vec<String>,
  },
  PassThrough,
}

pub struct DecoderPipeline;

impl DecoderPipeline {
  /// 对输入的文本执行前置解码探测（多行文件路径 / 引号JSON转义 / Base64 / Hex / DataURL / URL-Percent）
  pub fn decode(raw: &str) -> DecodedOutput {
    let trimmed = raw.trim();

    // 0. 尝试文件路径探测与首个支持文件的读取（单行或多行）
    if let Some(res) = Self::try_decode_file_paths(trimmed) {
      return res;
    }

    // 1. 尝试去除外层单/双引号并反转义 JSON 字符串
    if let Some(res) = Self::try_decode_quoted_string(trimmed) {
      return res;
    }

    // 2. 尝试 DataURL 或 Base64 图片/文本
    if let Some(res) = Self::try_decode_base64(trimmed) {
      return res;
    }

    // 3. 尝试 Hex 十六进制
    if let Some(res) = Self::try_decode_hex(trimmed) {
      return res;
    }

    // 4. 尝试 URL percent-encoding (如 %7B%22foo%22... 等)
    if let Some(res) = Self::try_decode_url(trimmed) {
      return res;
    }

    DecodedOutput::PassThrough
  }

  /// 针对文本内容进行后续前置解码流水线探测（排除多行文件路径，避免循环递归）
  fn decode_content(text: &str) -> DecodedOutput {
    let trimmed = text.trim();

    if let Some(res) = Self::try_decode_quoted_string(trimmed) {
      return res;
    }
    if let Some(res) = Self::try_decode_base64(trimmed) {
      return res;
    }
    if let Some(res) = Self::try_decode_hex(trimmed) {
      return res;
    }
    if let Some(res) = Self::try_decode_url(trimmed) {
      return res;
    }

    DecodedOutput::PassThrough
  }

  /// 若输入为文件路径（单行或多行），且每一行都是有效文件路径，则读取第一个可支持的文件作为输入
  fn try_decode_file_paths(input: &str) -> Option<DecodedOutput> {
    let lines: Vec<&str> = input
      .lines()
      .map(|l| l.trim())
      .filter(|l| !l.is_empty())
      .collect();

    if lines.is_empty() {
      return None;
    }

    // 每一行都必须是一个文件路径（先正则快速判定，通过后用 stat 再次确认）
    let mut valid_paths = Vec::with_capacity(lines.len());
    for line in lines {
      if let Some(path) = check_file_path(line) {
        valid_paths.push(path);
      } else {
        return None;
      }
    }

    // 顺序读取第一个可支持的文件作为输入
    for path in valid_paths {
      if let Some(output) = try_read_supported_file(&path) {
        return Some(output);
      }
    }

    None
  }

  fn try_decode_quoted_string(input: &str) -> Option<DecodedOutput> {
    let unquoted = unquote_and_unescape(input)?;
    if unquoted == input {
      return None;
    }

    // 将去掉转义后的字符串作为输入，继续通过后续解码流水线探测
    let inner_decoded = Self::decode(&unquoted);
    match inner_decoded {
      DecodedOutput::Text { text, mut trace } => {
        trace.insert(0, "unquote".to_string());
        Some(DecodedOutput::Text { text, trace })
      }
      DecodedOutput::Image {
        bytes,
        mime_type,
        mut trace,
      } => {
        trace.insert(0, "unquote".to_string());
        Some(DecodedOutput::Image {
          bytes,
          mime_type,
          trace,
        })
      }
      DecodedOutput::PassThrough => Some(DecodedOutput::Text {
        text: unquoted,
        trace: vec!["unquote".to_string()],
      }),
    }
  }

  fn try_decode_base64(input: &str) -> Option<DecodedOutput> {
    let (candidate_b64, explicit_image_mime) = if let Some(stripped) = input.strip_prefix("data:") {
      if let Some((header, body)) = stripped.split_once(";base64,") {
        (body.trim(), Some(header.trim().to_string()))
      } else {
        return None;
      }
    } else {
      // 普通纯文本: 长度至少需要 >= 8，且为有效 Base64 字符
      if input.len() < 8 || input.contains(char::is_whitespace) {
        return None;
      }
      (input, None)
    };

    let decoded_bytes = BASE64_STANDARD.decode(candidate_b64).ok()?;
    if decoded_bytes.is_empty() {
      return None;
    }

    // 检查图片魔数
    if let Some(mime) = explicit_image_mime.or_else(|| detect_image_mime(&decoded_bytes)) {
      return Some(DecodedOutput::Image {
        bytes: decoded_bytes,
        mime_type: mime,
        trace: vec!["base64".to_string()],
      });
    }

    // 检查是否为有效且可读的 UTF-8 文本
    if let Ok(text) = String::from_utf8(decoded_bytes) {
      // 过滤无意义随机二进制解码出来的极端控制字符
      let control_count = text
        .chars()
        .filter(|c| c.is_control() && *c != '\n' && *c != '\r' && *c != '\t')
        .count();
      if control_count == 0 && text.trim().len() >= 2 {
        return Some(DecodedOutput::Text {
          text,
          trace: vec!["base64".to_string()],
        });
      }
    }

    None
  }

  fn try_decode_hex(input: &str) -> Option<DecodedOutput> {
    let clean_hex = if let Some(stripped) = input
      .strip_prefix("0x")
      .or_else(|| input.strip_prefix("0X"))
    {
      stripped.trim()
    } else {
      input
    };

    // 长度必须为偶数，且至少 8 字符，且全为 hex 字符
    if clean_hex.len() < 8
      || clean_hex.len() % 2 != 0
      || !clean_hex.chars().all(|c| c.is_ascii_hexdigit())
    {
      return None;
    }

    let decoded_bytes = hex::decode(clean_hex).ok()?;
    if decoded_bytes.is_empty() {
      return None;
    }

    // 检查图片魔数
    if let Some(mime) = detect_image_mime(&decoded_bytes) {
      return Some(DecodedOutput::Image {
        bytes: decoded_bytes,
        mime_type: mime,
        trace: vec!["hex".to_string()],
      });
    }

    // 检查 UTF-8 文本
    if let Ok(text) = String::from_utf8(decoded_bytes) {
      let control_count = text
        .chars()
        .filter(|c| c.is_control() && *c != '\n' && *c != '\r' && *c != '\t')
        .count();
      if control_count == 0 && text.trim().len() >= 2 {
        return Some(DecodedOutput::Text {
          text,
          trace: vec!["hex".to_string()],
        });
      }
    }

    None
  }

  fn try_decode_url(input: &str) -> Option<DecodedOutput> {
    // 含有两个以上 %XX 序列才尝试
    if input.matches('%').count() < 2 {
      return None;
    }
    let decoded = urlencoding_decode(input)?;
    if decoded != input && !decoded.trim().is_empty() {
      return Some(DecodedOutput::Text {
        text: decoded,
        trace: vec!["url-percent".to_string()],
      });
    }
    None
  }
}

/// 简单轻量的 percent-decode 实现，无需额外重量级依赖
fn urlencoding_decode(input: &str) -> Option<String> {
  let mut bytes = Vec::with_capacity(input.len());
  let mut chars = input.bytes();
  while let Some(b) = chars.next() {
    if b == b'%' {
      let h1 = chars.next()?;
      let h2 = chars.next()?;
      let hex_str = [h1, h2];
      let byte = u8::from_str_radix(std::str::from_utf8(&hex_str).ok()?, 16).ok()?;
      bytes.push(byte);
    } else if b == b'+' {
      bytes.push(b' ');
    } else {
      bytes.push(b);
    }
  }
  String::from_utf8(bytes).ok()
}

static PATH_REGEX: OnceLock<Regex> = OnceLock::new();

/// 判定字符串在语法层面是否符合常见文件路径形态
fn is_path_syntax(candidate: &str) -> bool {
  let re = PATH_REGEX.get_or_init(|| {
    // 匹配典型文件路径特征：
    // 1. Unix 绝对路径: /...
    // 2. 家目录路径: ~/... 或 ~
    // 3. 相对路径前缀: ./... 或 ../...
    // 4. Windows 驱动器路径: C:\... 或 C:/...
    // 5. Windows UNC 路径: \\...
    // 6. 包含目录分隔符的子路径: foo/bar 或 foo\bar
    // 7. 带有文件扩展名的文件名: filename.ext
    Regex::new(
      r#"^(?:[a-zA-Z]:[\\/]|/|~[\\/]|(?:\.\.?[\\/])|(?:[^\x00<>"|?*]+[\\/])|[a-zA-Z0-9_.\-]+\.[a-zA-Z0-9]{1,8})[^\x00<>"|?*]*$"#,
    )
    .expect("valid path regex")
  });
  re.is_match(candidate)
}

/// 提取去除外层单/双引号后的候选路径字符串
fn extract_path_candidate(line: &str) -> &str {
  let trimmed = line.trim();
  if (trimmed.starts_with('"') && trimmed.ends_with('"'))
    || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
  {
    if trimmed.len() >= 2 {
      return trimmed[1..trimmed.len() - 1].trim();
    }
  }
  trimmed
}

/// 标准化 file:// URI 协议头
fn normalize_candidate(path_str: &str) -> &str {
  if let Some(rest) = path_str.strip_prefix("file://") {
    #[cfg(windows)]
    if let Some(w) = rest.strip_prefix('/') {
      return w;
    }
    rest
  } else {
    path_str
  }
}

/// 解析路径中的波浪号 ~ 为真实家目录
fn resolve_path(candidate: &str) -> PathBuf {
  if let Some(stripped) = candidate.strip_prefix("~/") {
    if let Some(home) = dirs::home_dir() {
      return home.join(stripped);
    }
  } else if candidate == "~" {
    if let Some(home) = dirs::home_dir() {
      return home;
    }
  }
  PathBuf::from(candidate)
}

/// 对单行进行文件路径检测：先正则快速判定，通过后用 stat 再次确认
fn check_file_path(line: &str) -> Option<PathBuf> {
  let unquoted = extract_path_candidate(line);
  let normalized = normalize_candidate(unquoted);
  if normalized.is_empty() {
    return None;
  }

  // 1. 正则快速判定
  if !is_path_syntax(normalized) {
    return None;
  }

  // 2. 通过后用 stat 进行再次确认
  let resolved = resolve_path(normalized);
  if std::fs::metadata(&resolved).is_ok() {
    Some(resolved)
  } else {
    None
  }
}

/// 读取可以支持的文件作为输入（普通文件、体积 <= 20MB、支持图片格式或可读 UTF-8 文本）
fn try_read_supported_file(path: &Path) -> Option<DecodedOutput> {
  let meta = std::fs::metadata(path).ok()?;
  if !meta.is_file() {
    return None;
  }

  // 文件体积保护（大于 20MB 跳过）
  if meta.len() > 20 * 1024 * 1024 {
    return None;
  }

  let bytes = std::fs::read(path).ok()?;
  if bytes.is_empty() {
    return None;
  }

  // 1. 优先检查是否为受支持的魔数图片 (PNG, JPEG, WEBP, GIF, BMP)
  if let Some(mime) = detect_image_mime(&bytes) {
    return Some(DecodedOutput::Image {
      bytes,
      mime_type: mime,
      trace: vec!["file".to_string()],
    });
  }

  // 1.1 依据扩展名保底识别常见图片
  let ext_opt = path
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| e.to_ascii_lowercase());
  if let Some(ext) = ext_opt {
    let mime = match ext.as_str() {
      "jpg" | "jpeg" => Some("image/jpeg"),
      "png" => Some("image/png"),
      "webp" => Some("image/webp"),
      "gif" => Some("image/gif"),
      "bmp" => Some("image/bmp"),
      "ico" => Some("image/x-icon"),
      _ => None,
    };
    if let Some(m) = mime {
      return Some(DecodedOutput::Image {
        bytes,
        mime_type: m.to_string(),
        trace: vec!["file".to_string()],
      });
    }
  }

  // 2. 检查是否为有效 UTF-8 文本
  if let Ok(text) = String::from_utf8(bytes) {
    let has_invalid_controls = text
      .chars()
      .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t');
    if !has_invalid_controls {
      // 进一步检查文件文本内容是否包含下级编码（如 Base64/Hex/JSON转义）
      let inner = DecoderPipeline::decode_content(&text);
      return match inner {
        DecodedOutput::Text {
          text: dec_text,
          mut trace,
        } => {
          trace.insert(0, "file".to_string());
          Some(DecodedOutput::Text {
            text: dec_text,
            trace,
          })
        }
        DecodedOutput::Image {
          bytes: img_bytes,
          mime_type,
          mut trace,
        } => {
          trace.insert(0, "file".to_string());
          Some(DecodedOutput::Image {
            bytes: img_bytes,
            mime_type,
            trace,
          })
        }
        DecodedOutput::PassThrough => Some(DecodedOutput::Text {
          text,
          trace: vec!["file".to_string()],
        }),
      };
    }
  }

  None
}

/// 若输入被单引号 `'...'` 或双引号 `"..."` 完整包裹，提取并反转义内部字符串
fn unquote_and_unescape(input: &str) -> Option<String> {
  let trimmed = input.trim();
  if trimmed.len() < 2 {
    return None;
  }

  let quote = trimmed.chars().next()?;
  if quote != '"' && quote != '\'' {
    return None;
  }

  let inner = get_enclosed_quoted_content(trimmed)?;

  if quote == '"' {
    // 优先尝试标准 serde_json 反转义
    if let Ok(unescaped) = serde_json::from_str::<String>(trimmed) {
      return Some(unescaped);
    }
  }

  // 兜底或单引号字符串反转义
  Some(unescape_string_content(inner))
}

/// 检查字符串是否完整由首尾对应的引号包裹（中间未被非转义引号提前闭合）
fn get_enclosed_quoted_content(s: &str) -> Option<&str> {
  let trimmed = s.trim();
  if trimmed.len() < 2 {
    return None;
  }
  let quote = trimmed.chars().next()?;
  if quote != '"' && quote != '\'' {
    return None;
  }
  if !trimmed.ends_with(quote) {
    return None;
  }

  // 检查开头引号是否在最后一个字符前被提前闭合
  let inner_with_last = &trimmed[quote.len_utf8()..];
  let mut chars = inner_with_last.char_indices();
  let mut escaped = false;

  while let Some((idx, c)) = chars.next() {
    if escaped {
      escaped = false;
      continue;
    }
    if c == '\\' {
      escaped = true;
      continue;
    }
    if c == quote {
      // 若闭合引号位于最后一个字符处，说明整体被完整包裹
      if idx == inner_with_last.len() - quote.len_utf8() {
        let inner = &trimmed[quote.len_utf8()..trimmed.len() - quote.len_utf8()];
        return Some(inner);
      } else {
        // 中途闭合（如 "foo" "bar"），非单一包裹字符串
        return None;
      }
    }
  }

  None
}

/// 手动反转义通用 JSON / 字符串转义序列
fn unescape_string_content(inner: &str) -> String {
  let mut result = String::with_capacity(inner.len());
  let mut chars = inner.chars().peekable();

  while let Some(c) = chars.next() {
    if c == '\\' {
      match chars.next() {
        Some('"') => result.push('"'),
        Some('\'') => result.push('\''),
        Some('\\') => result.push('\\'),
        Some('/') => result.push('/'),
        Some('n') => result.push('\n'),
        Some('r') => result.push('\r'),
        Some('t') => result.push('\t'),
        Some('b') => result.push('\x08'),
        Some('f') => result.push('\x0C'),
        Some('u') => {
          if let Some(ch) = parse_unicode_escape(&mut chars) {
            result.push(ch);
          } else {
            result.push_str("\\u");
          }
        }
        Some(other) => {
          result.push(other);
        }
        None => {
          result.push('\\');
        }
      }
    } else {
      result.push(c);
    }
  }

  result
}

/// 解析 \uXXXX 形式的 Unicode 转义，支持代理对 (Surrogate Pairs)
fn parse_unicode_escape<I>(chars: &mut std::iter::Peekable<I>) -> Option<char>
where
  I: Iterator<Item = char> + Clone,
{
  let mut hex = String::with_capacity(4);
  for _ in 0..4 {
    let &c = chars.peek()?;
    if c.is_ascii_hexdigit() {
      hex.push(chars.next()?);
    } else {
      return None;
    }
  }
  let code = u16::from_str_radix(&hex, 16).ok()?;

  // 处理 UTF-16 高代理 (0xD800..=0xDBFF)
  if (0xD800..=0xDBFF).contains(&code) {
    let mut clone = chars.clone();
    if clone.next() == Some('\\') && clone.next() == Some('u') {
      let mut low_hex = String::with_capacity(4);
      let mut valid = true;
      for _ in 0..4 {
        if let Some(c) = clone.next() {
          if c.is_ascii_hexdigit() {
            low_hex.push(c);
            continue;
          }
        }
        valid = false;
        break;
      }
      if valid {
        if let Ok(low_code) = u16::from_str_radix(&low_hex, 16) {
          if (0xDC00..=0xDFFF).contains(&low_code) {
            for _ in 0..6 {
              chars.next();
            }
            if let Some(Ok(ch)) = char::decode_utf16([code, low_code]).next() {
              return Some(ch);
            }
          }
        }
      }
    }
  }

  char::from_u32(code as u32)
}

/// 识别常见图片格式魔数
pub fn detect_image_mime(bytes: &[u8]) -> Option<String> {
  if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
    Some("image/png".to_string())
  } else if bytes.starts_with(&[0xFF, 0xD8]) {
    Some("image/jpeg".to_string())
  } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
    Some("image/webp".to_string())
  } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
    Some("image/gif".to_string())
  } else if bytes.starts_with(b"BM") {
    Some("image/bmp".to_string())
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_base64_json_decode() {
    // {"name":"mtools"} in base64 is eyJuYW1lIjoibXRvb2xzIn0=
    let b64 = "eyJuYW1lIjoibXRvb2xzIn0=";
    match DecoderPipeline::decode(b64) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, "{\"name\":\"mtools\"}");
        assert_eq!(trace, vec!["base64"]);
      }
      _ => panic!("Expected DecodedOutput::Text"),
    }
  }

  #[test]
  fn test_hex_decode() {
    // "hello" in hex is 68656c6c6f
    let hex_str = "0x68656c6c6f";
    match DecoderPipeline::decode(hex_str) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, "hello");
        assert_eq!(trace, vec!["hex"]);
      }
      _ => panic!("Expected DecodedOutput::Text"),
    }
  }

  #[test]
  fn test_png_magic_upgrade() {
    // PNG header in base64: iVBORw0KGgo=
    let png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    match DecoderPipeline::decode(png_base64) {
      DecodedOutput::Image {
        mime_type, trace, ..
      } => {
        assert_eq!(mime_type, "image/png");
        assert_eq!(trace, vec!["base64"]);
      }
      _ => panic!("Expected DecodedOutput::Image"),
    }
  }

  #[test]
  fn test_double_quoted_json_string() {
    let quoted = r#""{\"name\": \"mtools\", \"version\": 2}""#;
    match DecoderPipeline::decode(quoted) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, r#"{"name": "mtools", "version": 2}"#);
        assert_eq!(trace, vec!["unquote"]);
      }
      _ => panic!("Expected DecodedOutput::Text"),
    }
  }

  #[test]
  fn test_single_quoted_json_string() {
    let quoted = r#"'{\"name\": \"mtools\", \"status\": true}'"#;
    match DecoderPipeline::decode(quoted) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, r#"{"name": "mtools", "status": true}"#);
        assert_eq!(trace, vec!["unquote"]);
      }
      _ => panic!("Expected DecodedOutput::Text"),
    }
  }

  #[test]
  fn test_single_quoted_unescaped_json() {
    let quoted = r#"'{"action": "query", "limit": 100}'"#;
    match DecoderPipeline::decode(quoted) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, r#"{"action": "query", "limit": 100}"#);
        assert_eq!(trace, vec!["unquote"]);
      }
      _ => panic!("Expected DecodedOutput::Text"),
    }
  }

  #[test]
  fn test_quoted_base64_string() {
    // "eyJuYW1lIjoibXRvb2xzIn0=" wrapped in quotes
    let quoted_b64 = r#""eyJuYW1lIjoibXRvb2xzIn0=""#;
    match DecoderPipeline::decode(quoted_b64) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, "{\"name\":\"mtools\"}");
        assert_eq!(trace, vec!["unquote", "base64"]);
      }
      _ => panic!("Expected DecodedOutput::Text with unquote + base64 trace"),
    }
  }

  #[test]
  fn test_quoted_escapes_and_unicode() {
    let quoted = r#""Hello\nWorld\t\u4e2d\u6587\uD83D\uDE00""#;
    match DecoderPipeline::decode(quoted) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, "Hello\nWorld\t中文😀");
        assert_eq!(trace, vec!["unquote"]);
      }
      _ => panic!("Expected DecodedOutput::Text"),
    }
  }

  #[test]
  fn test_unclosed_or_mismatched_quotes() {
    assert!(matches!(
      DecoderPipeline::decode(r#""hello"#),
      DecodedOutput::PassThrough
    ));
    assert!(matches!(
      DecoderPipeline::decode(r#""hello'"#),
      DecodedOutput::PassThrough
    ));
    assert!(matches!(
      DecoderPipeline::decode(r#""foo" "bar""#),
      DecodedOutput::PassThrough
    ));
  }

  #[test]
  fn test_multiline_file_paths_first_supported_read() {
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let json_file = temp_dir.join("mtools_test_multiline_1.json");
    let txt_file = temp_dir.join("mtools_test_multiline_2.txt");

    let mut f1 = std::fs::File::create(&json_file).unwrap();
    write!(f1, "{{\"tool\":\"mtools\",\"ok\":true}}").unwrap();

    let mut f2 = std::fs::File::create(&txt_file).unwrap();
    write!(f2, "second file content").unwrap();

    let input = format!(
      "{}\n{}",
      json_file.to_str().unwrap(),
      txt_file.to_str().unwrap()
    );

    match DecoderPipeline::decode(&input) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, "{\"tool\":\"mtools\",\"ok\":true}");
        assert_eq!(trace, vec!["file"]);
      }
      _ => panic!("Expected DecodedOutput::Text from first supported file"),
    }

    let _ = std::fs::remove_file(json_file);
    let _ = std::fs::remove_file(txt_file);
  }

  #[test]
  fn test_multiline_file_paths_skip_dir_and_binary() {
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let bin_file = temp_dir.join("mtools_test_binary.bin");
    let txt_file = temp_dir.join("mtools_test_target.txt");

    // 写入包含非法控制字符的二进制数据（NUL 字节）
    let mut f_bin = std::fs::File::create(&bin_file).unwrap();
    f_bin.write_all(&[0x00, 0x01, 0x02, 0xFF]).unwrap();

    let mut f_txt = std::fs::File::create(&txt_file).unwrap();
    write!(f_txt, "hello from supported file").unwrap();

    // 行1是目录（temp_dir），行2是非支持二进制文件，行3是支持的文本文件
    let input = format!(
      "{}\n{}\n{}",
      temp_dir.to_str().unwrap(),
      bin_file.to_str().unwrap(),
      txt_file.to_str().unwrap()
    );

    match DecoderPipeline::decode(&input) {
      DecodedOutput::Text { text, trace } => {
        assert_eq!(text, "hello from supported file");
        assert_eq!(trace, vec!["file"]);
      }
      _ => panic!("Expected to skip directory and binary file to read target text file"),
    }

    let _ = std::fs::remove_file(bin_file);
    let _ = std::fs::remove_file(txt_file);
  }

  #[test]
  fn test_multiline_non_path_rejected() {
    let input = "/tmp\nThis is not a file path because it has spaces and no separators";
    assert!(matches!(
      DecoderPipeline::decode(input),
      DecodedOutput::PassThrough
    ));
  }

  #[test]
  fn test_single_line_path_dir_passthrough() {
    let input = "/tmp";
    assert!(matches!(
      DecoderPipeline::decode(input),
      DecodedOutput::PassThrough
    ));
  }

  #[test]
  fn test_single_line_image_path_decodes_to_image() {
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let img_path = temp_dir.join("mtools_test_single_image.jpg");

    // 写入标准 JPEG 头部 SOI (0xFF, 0xD8, 0xFF, 0xE0...)
    let mut f = std::fs::File::create(&img_path).unwrap();
    f.write_all(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46])
      .unwrap();

    let input = format!("  {}  ", img_path.to_str().unwrap());

    match DecoderPipeline::decode(&input) {
      DecodedOutput::Image {
        mime_type,
        trace,
        bytes,
      } => {
        assert_eq!(mime_type, "image/jpeg");
        assert_eq!(trace, vec!["file"]);
        assert_eq!(bytes.len(), 10);
      }
      _ => panic!("Expected single line image path to decode into DecodedOutput::Image"),
    }

    let _ = std::fs::remove_file(img_path);
  }
}
