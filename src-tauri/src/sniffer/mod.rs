pub mod builtin;

use crate::models::{
  EnrichedPayload, PayloadType, PreprocessedResult, TextMetadata, ToolScoreItem,
};
use serde_json::json;

/// 嗅探输入源抽象
pub enum SniffInput<'a> {
  Text(&'a str),
  Image(&'a [u8]),
}

/// 嗅探产物
#[derive(Debug, Clone, Default)]
pub struct SniffOutput {
  pub matched: bool,
  pub confidence: f32,                       // 0.0 ~ 1.0 置信度
  pub tags: Vec<String>,                     // 命中标签，如 ["format-json"]
  pub preprocessed_text: Option<String>,     // 可选: Rust 预处理产物 (如 Pretty JSON)
  pub suggested_tool_id: Option<String>,     // 可选: 建议工具 ID (如 "json-formatter")
  pub suggested_output_type: Option<String>, // 可选: 推荐渲染类型 ("json", "text", "markdown")
  pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// 嗅探器标准 Trait，供系统内置或后续灵活扩展
pub trait ContentSniffer: Send + Sync {
  fn id(&self) -> &'static str;
  fn name(&self) -> &'static str;
  fn priority(&self) -> u32 {
    100
  }
  fn supports(&self, input: &SniffInput) -> bool;
  fn sniff(&self, input: &SniffInput) -> SniffOutput;
}

/// 嗅探器单例管理与调度流水线
pub struct SnifferRegistry {
  sniffers: Vec<Box<dyn ContentSniffer>>,
}

impl SnifferRegistry {
  pub fn new() -> Self {
    let mut registry = Self {
      sniffers: Vec::new(),
    };
    registry.register(Box::new(builtin::json::JsonSniffer::default()));
    registry.register(Box::new(builtin::url::UrlSniffer::default()));
    registry.register(Box::new(builtin::jwt::JwtSniffer::default()));
    registry.register(Box::new(builtin::time::TimestampSniffer::default()));
    registry.register(Box::new(builtin::image::ImageSniffer::default()));
    registry.register(Box::new(builtin::text::TextSniffer::default()));
    registry
  }

  /// 注册新的嗅探器，并根据 priority 降序排列
  pub fn register(&mut self, sniffer: Box<dyn ContentSniffer>) {
    self.sniffers.push(sniffer);
    self
      .sniffers
      .sort_by(|a, b| b.priority().cmp(&a.priority()));
  }

  /// 调度执行责任链并聚合产物为 EnrichedPayload
  pub fn execute(
    &self,
    input: &SniffInput,
    raw_original: String,
    actual_content: String,
    decoding_trace: Vec<String>,
    image_local_cache_path: Option<String>,
  ) -> EnrichedPayload {
    let mut combined_tags = Vec::new();
    let mut best_confidence = 0.0f32;
    let mut selected_preprocessed = None;
    let mut recommended_tool_id = match input {
      SniffInput::Image(_) => "ocr-extractor".to_string(),
      SniffInput::Text(_) => "llm-translate".to_string(),
    };
    let mut candidate_scores = Vec::new();
    let mut detected_format = "plain".to_string();

    for sniffer in &self.sniffers {
      if sniffer.supports(input) {
        let output = sniffer.sniff(input);
        if output.matched {
          combined_tags.extend(output.tags);
          if let Some(tool_id) = output.suggested_tool_id.clone() {
            candidate_scores.push(ToolScoreItem {
              tool_id,
              score: output.confidence * 100.0,
            });
          }

          if output.confidence > best_confidence {
            best_confidence = output.confidence;
            if let Some(prep) = output.preprocessed_text {
              selected_preprocessed = Some(PreprocessedResult {
                formatted_text: Some(prep),
                diff_source: None,
                suggested_output_type: output
                  .suggested_output_type
                  .unwrap_or_else(|| "text".to_string()),
              });
            }
            if let Some(tool_id) = output.suggested_tool_id {
              recommended_tool_id = tool_id;
            }
            detected_format = sniffer.id().replace("-sniffer", "");
          }
        }
      }
    }

    if !decoding_trace.is_empty() {
      for trace in &decoding_trace {
        combined_tags.push(format!("decoded-from-{}", trace));
      }
    }

    let now = chrono::Utc::now().timestamp_millis();
    let id = uuid::Uuid::new_v4().to_string();

    match input {
      SniffInput::Text(t) => {
        let meta = TextMetadata {
          char_count: t.chars().count(),
          line_count: t.lines().count().max(1),
          detected_format,
          decoding_trace,
        };
        EnrichedPayload {
          id,
          payload_type: PayloadType::Text,
          raw_original,
          actual_content,
          metadata: json!(meta),
          tags: combined_tags,
          preprocessed_result: selected_preprocessed,
          recommended_tool_id,
          candidate_tool_scores: candidate_scores,
          created_at: now,
        }
      }
      SniffInput::Image(bytes) => {
        let local_path = image_local_cache_path.unwrap_or_default();
        let meta = crate::models::ImageMetadata {
          width: 0,
          height: 0,
          mime_type: "image/png".to_string(),
          byte_size: bytes.len(),
          local_cache_path: local_path,
          decoding_trace,
        };
        EnrichedPayload {
          id,
          payload_type: PayloadType::Image,
          raw_original,
          actual_content,
          metadata: json!(meta),
          tags: combined_tags,
          preprocessed_result: selected_preprocessed,
          recommended_tool_id: "ai-ocr".to_string(),
          candidate_tool_scores: candidate_scores,
          created_at: now,
        }
      }
    }
  }
}
