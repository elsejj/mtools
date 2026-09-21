use super::jev::JevSniffer;
use crate::models::EvaluationModelConfig;
use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};

#[derive(Default)]
pub struct TextSniffer;

impl TextSniffer {
  pub async fn sniff_async(
    &self,
    input: &SniffInput<'_>,
    eval_config: Option<&EvaluationModelConfig>,
    tools: &[(String, String)],
  ) -> SniffOutput {
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

    // 当配置了判定模型且候选工具列表非空时，调用 jev-latest
    if let Some(config) = eval_config {
      if config.is_enabled() && !tools.is_empty() {
        let jev = JevSniffer::new();
        match jev.evaluate(text, config, tools).await {
          Ok(result) => {
            let tags = vec![
              "format-text".to_string(),
              "natural-language".to_string(),
              "jev-choice".to_string(),
            ];
            let prob_val = serde_json::to_value(&result.probabilities).unwrap_or_default();
            metadata.insert("choiceProbabilities".to_string(), prob_val);

            // Jev 判定结果赋予较高置信度 (>= 0.85)，以进入相应工具
            let confidence = if result.confidence > 0.0 {
              result.confidence.max(0.85)
            } else {
              0.85
            };

            return SniffOutput {
              matched: true,
              confidence,
              tags,
              preprocessed_text: None,
              suggested_tool_id: Some(result.choice),
              suggested_output_type: Some("markdown".to_string()),
              metadata,
            };
          }
          Err(err) => {
            eprintln!(
              "[TextSniffer] Jev evaluation failed, falling back to default: {}",
              err
            );
          }
        }
      }
    }

    // 未启用判定模型或网络判定失败时的保底策略
    SniffOutput {
      matched: true,
      confidence: 0.65,
      tags: vec!["format-text".to_string(), "natural-language".to_string()],
      preprocessed_text: None,
      suggested_tool_id: Some("llm-translate".to_string()),
      suggested_output_type: Some("markdown".to_string()),
      metadata,
    }
  }
}

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
