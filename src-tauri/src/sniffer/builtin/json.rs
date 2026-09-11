use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};
use serde_json::Value;

#[derive(Default)]
pub struct JsonSniffer;

impl ContentSniffer for JsonSniffer {
  fn id(&self) -> &'static str {
    "json-sniffer"
  }

  fn name(&self) -> &'static str {
    "JSON Format & Pretty Preprocessor"
  }

  fn priority(&self) -> u32 {
    90 // 较高优先级
  }

  fn supports(&self, input: &SniffInput) -> bool {
    matches!(input, SniffInput::Text(_))
  }

  fn sniff(&self, input: &SniffInput) -> SniffOutput {
    let text = match input {
      SniffInput::Text(t) => t.trim(),
      _ => return SniffOutput::default(),
    };

    // 快速边界预判，避免非 JSON 长文本进行无意义的反序列化
    if !(text.starts_with('{') && text.ends_with('}'))
      && !(text.starts_with('[') && text.ends_with(']'))
    {
      return SniffOutput::default();
    }

    // 尝试解析并生成 pretty 格式化
    if let Ok(parsed) = serde_json::from_str::<Value>(text) {
      let pretty_json = serde_json::to_string_pretty(&parsed).ok();
      let mut metadata = serde_json::Map::new();
      if let Value::Object(ref map) = parsed {
        metadata.insert("isObject".to_string(), Value::Bool(true));
        metadata.insert("keyCount".to_string(), Value::Number(map.len().into()));
      } else if let Value::Array(ref arr) = parsed {
        metadata.insert("isArray".to_string(), Value::Bool(true));
        metadata.insert("itemCount".to_string(), Value::Number(arr.len().into()));
      }

      SniffOutput {
        matched: true,
        confidence: 0.95,
        tags: vec!["format-json".to_string(), "valid-syntax".to_string()],
        preprocessed_text: pretty_json,
        suggested_tool_id: Some("json-formatter".to_string()),
        suggested_output_type: Some("json".to_string()),
        metadata,
      }
    } else {
      SniffOutput::default()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_json_sniffer_valid() {
    let sniffer = JsonSniffer;
    let input = SniffInput::Text(r#"{"name":"mtools","age":1}"#);
    let output = sniffer.sniff(&input);
    assert!(output.matched);
    assert_eq!(output.confidence, 0.95);
    assert_eq!(output.suggested_tool_id, Some("json-formatter".to_string()));
    assert!(output.preprocessed_text.unwrap().contains('\n'));
  }

  #[test]
  fn test_json_sniffer_invalid() {
    let sniffer = JsonSniffer;
    let input = SniffInput::Text("Not a json string");
    let output = sniffer.sniff(&input);
    assert!(!output.matched);
  }
}
