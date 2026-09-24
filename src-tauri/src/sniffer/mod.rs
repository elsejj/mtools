pub mod builtin;

use crate::models::{
  EnrichedPayload, PayloadType, PreprocessedResult, TextMetadata, ToolScoreItem,
};
use log::{debug, info, trace};
use serde_json::json;

fn format_sniff_input(input: &SniffInput) -> String {
  match input {
    SniffInput::Text(t) => {
      let trimmed = t.trim();
      let char_count = trimmed.chars().count();
      let preview: String = trimmed.chars().take(80).collect();
      let preview_clean = preview.replace('\n', "\\n");
      if char_count <= 80 {
        format!("Text ({} chars): \"{}\"", char_count, preview_clean)
      } else {
        format!("Text ({} chars): \"{}...\"", char_count, preview_clean)
      }
    }
    SniffInput::Image(bytes) => {
      format!("Image ({} bytes)", bytes.len())
    }
  }
}

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
    registry.register(Box::new(builtin::calc::CalculatorSniffer::default()));
    registry.register(Box::new(builtin::time::TimestampSniffer::default()));
    registry.register(Box::new(builtin::image::ImageSniffer::default()));
    registry.register(Box::new(builtin::text::TextSniffer::default()));
    registry
  }

  /// 注册新的嗅探器，并根据 priority 降序排列
  pub fn register(&mut self, sniffer: Box<dyn ContentSniffer>) {
    debug!(
      "[SnifferRegistry] Registering sniffer '{}' (name: '{}', priority: {})",
      sniffer.id(),
      sniffer.name(),
      sniffer.priority()
    );
    self.sniffers.push(sniffer);
    self
      .sniffers
      .sort_by(|a, b| b.priority().cmp(&a.priority()));
  }

  /// 调度执行责任链并聚合产物为 EnrichedPayload (同步模式)
  pub fn execute(
    &self,
    input: &SniffInput,
    raw_original: String,
    actual_content: String,
    decoding_trace: Vec<String>,
    image_local_cache_path: Option<String>,
  ) -> EnrichedPayload {
    let input_desc = format_sniff_input(input);
    let mut combined_tags = Vec::new();
    let mut best_confidence = 0.0f32;
    let mut selected_preprocessed = None;
    let mut recommended_tool_id = match input {
      SniffInput::Image(_) => "ocr-extractor".to_string(),
      SniffInput::Text(_) => "llm-translate".to_string(),
    };
    let mut candidate_scores = Vec::new();
    let mut detected_format = "plain".to_string();

    info!(
      "[Sniffer] >>> Starting sync sniffing pipeline for {} | default fallback tool: '{}'",
      input_desc, recommended_tool_id
    );

    for sniffer in &self.sniffers {
      if sniffer.supports(input) {
        debug!(
          "[Sniffer] Evaluating sniffer '{}' (priority: {})...",
          sniffer.id(),
          sniffer.priority()
        );
        let output = sniffer.sniff(input);
        if output.matched {
          info!(
            "[Sniffer] Sniffer '{}' MATCHED (confidence: {:.2}, suggested_tool: {:?}, tags: {:?})",
            sniffer.id(),
            output.confidence,
            output.suggested_tool_id,
            output.tags
          );
          combined_tags.extend(output.tags);
          if let Some(tool_id) = output.suggested_tool_id.clone() {
            candidate_scores.push(ToolScoreItem {
              tool_id,
              score: output.confidence * 100.0,
            });
          }

          if output.confidence > best_confidence {
            info!(
              "[Sniffer] -> Updating candidate tool: '{}' -> '{:?}' (confidence: {:.2} > previous {:.2}, detected format: '{}')",
              recommended_tool_id,
              output.suggested_tool_id,
              output.confidence,
              best_confidence,
              sniffer.id().replace("-sniffer", "")
            );
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
          } else {
            debug!(
              "[Sniffer] -> Sniffer '{}' matched with confidence {:.2} <= current best {:.2}, keeping '{}'",
              sniffer.id(),
              output.confidence,
              best_confidence,
              recommended_tool_id
            );
          }
        } else {
          trace!("[Sniffer] Sniffer '{}' did not match", sniffer.id());
        }
      } else {
        trace!("[Sniffer] Sniffer '{}' does not support input", sniffer.id());
      }
    }

    if !decoding_trace.is_empty() {
      info!("[Sniffer] Appending decoding trace tags: {:?}", decoding_trace);
      for trace in &decoding_trace {
        combined_tags.push(format!("decoded-from-{}", trace));
      }
    }

    info!(
      "[Sniffer] <<< Sync sniffing complete: SELECTED TOOL='{}' | format='{}' | confidence={:.2} | candidates={:?} | tags={:?}",
      recommended_tool_id,
      detected_format,
      best_confidence,
      candidate_scores
        .iter()
        .map(|s| format!("{}:{:.1}", s.tool_id, s.score))
        .collect::<Vec<_>>(),
      combined_tags
    );

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
        let mime_type =
          crate::decoder::detect_image_mime(bytes).unwrap_or_else(|| "image/png".to_string());
        let (width, height) = if bytes.len() >= 24 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
          (
            u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
          )
        } else {
          (0, 0)
        };
        let meta = crate::models::ImageMetadata {
          width,
          height,
          mime_type,
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
          recommended_tool_id,
          candidate_tool_scores: candidate_scores,
          created_at: now,
        }
      }
    }
  }

  /// 调度执行责任链并聚合产物为 EnrichedPayload (异步模式，支持判定模型)
  pub async fn execute_async(
    &self,
    input: &SniffInput<'_>,
    raw_original: String,
    actual_content: String,
    decoding_trace: Vec<String>,
    image_local_cache_path: Option<String>,
    storage: Option<&crate::storage::AppStorage>,
  ) -> EnrichedPayload {
    let input_desc = format_sniff_input(input);
    let mut combined_tags = Vec::new();
    let mut best_confidence = 0.0f32;
    let mut selected_preprocessed = None;
    let mut recommended_tool_id = match input {
      SniffInput::Image(_) => "ocr-extractor".to_string(),
      SniffInput::Text(_) => "llm-translate".to_string(),
    };
    let mut candidate_scores = Vec::new();
    let mut detected_format = "plain".to_string();

    info!(
      "[Sniffer] >>> Starting async sniffing pipeline for {} | default fallback tool: '{}'",
      input_desc, recommended_tool_id
    );

    // 1. 运行固定规则嗅探器 (非 text-sniffer)
    debug!("[Sniffer] Stage 1: Running fixed-rule sniffers...");
    for sniffer in &self.sniffers {
      if sniffer.id() == "text-sniffer" {
        continue;
      }
      if sniffer.supports(input) {
        debug!(
          "[Sniffer] Evaluating fixed-rule sniffer '{}' (priority: {})...",
          sniffer.id(),
          sniffer.priority()
        );
        let output = sniffer.sniff(input);
        if output.matched {
          info!(
            "[Sniffer] Fixed-rule '{}' MATCHED (confidence: {:.2}, suggested_tool: {:?}, tags: {:?})",
            sniffer.id(),
            output.confidence,
            output.suggested_tool_id,
            output.tags
          );
          combined_tags.extend(output.tags);
          if let Some(tool_id) = output.suggested_tool_id.clone() {
            candidate_scores.push(ToolScoreItem {
              tool_id,
              score: output.confidence * 100.0,
            });
          }

          if output.confidence > best_confidence {
            info!(
              "[Sniffer] -> Updating candidate tool: '{}' -> '{:?}' (confidence: {:.2} > previous {:.2}, detected format: '{}')",
              recommended_tool_id,
              output.suggested_tool_id,
              output.confidence,
              best_confidence,
              sniffer.id().replace("-sniffer", "")
            );
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
          } else {
            debug!(
              "[Sniffer] -> Fixed-rule '{}' matched with confidence {:.2} <= current best {:.2}, keeping '{}'",
              sniffer.id(),
              output.confidence,
              best_confidence,
              recommended_tool_id
            );
          }
        } else {
          trace!("[Sniffer] Fixed-rule '{}' did not match", sniffer.id());
        }
      } else {
        trace!("[Sniffer] Fixed-rule '{}' does not support input", sniffer.id());
      }
    }

    // 2. 检查固定规则是否已命中 (置信度 >= 0.80 说明已判定)
    let fixed_rule_matched = best_confidence >= 0.80;
    info!(
      "[Sniffer] Stage 1 finished: fixed_rule_matched={}, best_confidence={:.2}, current tool='{}'",
      fixed_rule_matched, best_confidence, recommended_tool_id
    );

    // 3. 处理文本嗅探器 TextSniffer
    if matches!(input, SniffInput::Text(_)) {
      if fixed_rule_matched {
        // 固定规则已判定，TextSniffer 仅做常规语言分析补充，不覆盖高置信度推荐
        info!(
          "[Sniffer] Stage 2: Fixed rule hit confidence threshold (>= 0.80). Running TextSniffer only for auxiliary language tagging without overriding tool '{}'.",
          recommended_tool_id
        );
        let text_sniffer = builtin::text::TextSniffer::default();
        let output = text_sniffer.sniff(input);
        if output.matched {
          debug!("[Sniffer] TextSniffer auxiliary tags: {:?}", output.tags);
          combined_tags.extend(output.tags);
        }
      } else {
        // 固定规则无法判定时，尝试调用判定模型 (jev-latest)
        info!(
          "[Sniffer] Stage 2: No fixed rule reached confidence threshold (best: {:.2} < 0.80). Invoking TextSniffer / AI evaluation model...",
          best_confidence
        );
        let (eval_config, tool_descriptions) = if let Some(st) = storage {
          if let Ok(conn) = st.db.lock() {
            let cfg = crate::storage::config::get_evaluation_config(&conn);
            let tools = crate::storage::config::get_enabled_tool_descriptions(&conn);
            (Some(cfg), tools)
          } else {
            (None, Vec::new())
          }
        } else {
          (None, Vec::new())
        };

        let is_eval_enabled = eval_config.as_ref().map(|c| c.is_enabled()).unwrap_or(false);
        debug!(
          "[Sniffer] AI model status: eval_model_enabled={}, candidate_tools_count={}",
          is_eval_enabled,
          tool_descriptions.len()
        );

        let text_sniffer = builtin::text::TextSniffer::default();
        let output = text_sniffer
          .sniff_async(input, eval_config.as_ref(), &tool_descriptions)
          .await;

        if output.matched {
          info!(
            "[Sniffer] TextSniffer async returned: confidence={:.2}, suggested_tool={:?}, tags={:?}",
            output.confidence, output.suggested_tool_id, output.tags
          );
          combined_tags.extend(output.tags.clone());

          // 若存在 Jev 的概率分布，注入 candidate_scores
          if let Some(prob_val) = output.metadata.get("choiceProbabilities") {
            if let Some(map) = prob_val.as_object() {
              for (tid, prob) in map {
                if let Some(score_f) = prob.as_f64() {
                  candidate_scores.push(ToolScoreItem {
                    tool_id: tid.clone(),
                    score: (score_f * 100.0) as f32,
                  });
                }
              }
              info!(
                "[Sniffer] AI model probability distribution injected: {:?}",
                candidate_scores
                  .iter()
                  .map(|s| format!("{}:{:.1}", s.tool_id, s.score))
                  .collect::<Vec<_>>()
              );
            }
          } else if let Some(tool_id) = output.suggested_tool_id.clone() {
            candidate_scores.push(ToolScoreItem {
              tool_id,
              score: output.confidence * 100.0,
            });
          }

          if output.confidence > best_confidence {
            info!(
              "[Sniffer] -> Updating candidate tool from TextSniffer/Model: '{}' -> '{:?}' (confidence: {:.2} > previous {:.2})",
              recommended_tool_id, output.suggested_tool_id, output.confidence, best_confidence
            );
            if let Some(tool_id) = output.suggested_tool_id {
              recommended_tool_id = tool_id;
            }
            if output.tags.contains(&"jev-choice".to_string()) {
              detected_format = "jev-choice".to_string();
            } else {
              detected_format = "text".to_string();
            }
            best_confidence = output.confidence;
          } else {
            debug!(
              "[Sniffer] -> TextSniffer/Model confidence {:.2} <= current best {:.2}, keeping '{}'",
              output.confidence, best_confidence, recommended_tool_id
            );
          }
        } else {
          debug!("[Sniffer] TextSniffer did not match input");
        }
      }
    }

    if !decoding_trace.is_empty() {
      info!("[Sniffer] Appending decoding trace tags: {:?}", decoding_trace);
      for trace in &decoding_trace {
        combined_tags.push(format!("decoded-from-{}", trace));
      }
    }

    info!(
      "[Sniffer] <<< Async sniffing complete: SELECTED TOOL='{}' | format='{}' | confidence={:.2} | candidates={:?} | tags={:?}",
      recommended_tool_id,
      detected_format,
      best_confidence,
      candidate_scores
        .iter()
        .map(|s| format!("{}:{:.1}", s.tool_id, s.score))
        .collect::<Vec<_>>(),
      combined_tags
    );

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
        let mime_type =
          crate::decoder::detect_image_mime(bytes).unwrap_or_else(|| "image/png".to_string());
        let (width, height) = if bytes.len() >= 24 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
          (
            u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
          )
        } else {
          (0, 0)
        };
        let meta = crate::models::ImageMetadata {
          width,
          height,
          mime_type,
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
          recommended_tool_id,
          candidate_tool_scores: candidate_scores,
          created_at: now,
        }
      }
    }
  }
}
