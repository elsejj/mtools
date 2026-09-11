use base64::prelude::*;

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
  /// 对输入的文本执行前置解码探测（Base64 / Hex / DataURL / URL-Percent）
  pub fn decode(raw: &str) -> DecodedOutput {
    let trimmed = raw.trim();

    // 1. 尝试 DataURL 或 Base64 图片/文本
    if let Some(res) = Self::try_decode_base64(trimmed) {
      return res;
    }

    // 2. 尝试 Hex 十六进制
    if let Some(res) = Self::try_decode_hex(trimmed) {
      return res;
    }

    // 3. 尝试 URL percent-encoding (如 %7B%22foo%22... 等)
    if let Some(res) = Self::try_decode_url(trimmed) {
      return res;
    }

    DecodedOutput::PassThrough
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

/// 识别常见图片格式魔数
pub fn detect_image_mime(bytes: &[u8]) -> Option<String> {
  if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
    Some("image/png".to_string())
  } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
    Some("image/jpeg".to_string())
  } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
    Some("image/webp".to_string())
  } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
    Some("image/gif".to_string())
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
}
