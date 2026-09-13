use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};

#[derive(Default)]
pub struct TextSniffer;

impl ContentSniffer for TextSniffer {
  fn id(&self) -> &'static str {
    "text-sniffer"
  }

  fn name(&self) -> &'static str {
    "Text & Natural Language Sniffer"
  }

  fn priority(&self) -> u32 {
    10 // 低于特定格式嗅探器 (JSON 90, URL 85, JWT 85, Time 80)，作为自然语言保底
  }

  fn supports(&self, input: &SniffInput) -> bool {
    matches!(input, SniffInput::Text(_))
  }

  fn sniff(&self, input: &SniffInput) -> SniffOutput {
    let text = match input {
      SniffInput::Text(t) => t.trim(),
      _ => return SniffOutput::default(),
    };

    if text.is_empty() {
      return SniffOutput::default();
    }

    let mut metadata = serde_json::Map::new();
    let has_cjk = text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
    let lang_hint = if has_cjk { "zh" } else { "en" };
    metadata.insert("langHint".to_string(), lang_hint.into());

    SniffOutput {
      matched: true,
      confidence: 0.65, // 通用文本保底置信度，低于任何特定命中格式 (0.85 ~ 0.95)
      tags: vec!["format-text".to_string(), "natural-language".to_string()],
      preprocessed_text: None,
      suggested_tool_id: Some("llm-translate".to_string()),
      suggested_output_type: Some("markdown".to_string()),
      metadata,
    }
  }
}
