use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PayloadType {
  Text,
  Image,
  Files,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextMetadata {
  pub char_count: usize,
  pub line_count: usize,
  pub detected_format: String, // "json", "url", "jwt", "xml", "sql", "cron", "plain"
  pub decoding_trace: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
  pub width: u32,
  pub height: u32,
  pub mime_type: String,
  pub byte_size: usize,
  pub local_cache_path: String,
  pub decoding_trace: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreprocessedResult {
  pub formatted_text: Option<String>,
  pub diff_source: Option<String>,
  pub suggested_output_type: String, // "json", "text", "markdown", "image"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolScoreItem {
  pub tool_id: String,
  pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedPayload {
  pub id: String,
  pub payload_type: PayloadType,
  pub raw_original: String,
  pub actual_content: String,
  pub metadata: serde_json::Value,
  pub tags: Vec<String>,
  pub preprocessed_result: Option<PreprocessedResult>,
  pub recommended_tool_id: String,
  pub candidate_tool_scores: Vec<ToolScoreItem>,
  pub created_at: i64,
}

// ----------------- CLI Request / Response -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliExecuteRequest {
  pub command: String,
  pub args: Vec<String>,
  pub working_dir: Option<String>,
  pub stdin_content: Option<String>,
  pub env_vars: Option<std::collections::HashMap<String, String>>,
  pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliExecuteResponse {
  pub exit_code: Option<i32>,
  pub stdout: String,
  pub stderr: String,
  pub duration_ms: u64,
}

// ----------------- Save File (Post Action) -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileRequest {
  pub target_directory: String,
  pub extension: String,
  pub content: String,
  pub custom_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileResponse {
  pub full_path: String,
  pub file_name: String,
  pub byte_size: u64,
}

// ----------------- History Records -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewHistoryRecord {
  pub tool_id: String,
  pub tool_name: String,
  pub payload_type: String,
  pub input_summary: String,
  pub input_text: Option<String>,
  pub input_image_path: Option<String>,
  pub output_content: Option<String>,
  pub post_action_type: String,
  pub output_file_path: Option<String>,
  pub status: String,
  pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecordItem {
  pub id: String,
  pub tool_id: String,
  pub tool_name: String,
  pub payload_type: String,
  pub input_summary: String,
  pub input_text: Option<String>,
  pub input_image_path: Option<String>,
  pub output_content: Option<String>,
  pub post_action_type: String,
  pub output_file_path: Option<String>,
  pub status: String,
  pub duration_ms: u64,
  pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
  pub tool_id: Option<String>,
  pub keyword: Option<String>,
  pub limit: u32,
  pub offset: u32,
}

// ----------------- Image Cache Stats -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageCacheStats {
  pub total_bytes: u64,
  pub file_count: usize,
  pub directory_path: String,
}

// ----------------- Window Geometry -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowGeometry {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
  pub is_maximized: bool,
}

// ----------------- Evaluation Model Config -----------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationModelConfig {
  #[serde(default)]
  pub base_url: String,
  #[serde(default)]
  pub api_key: String,
  #[serde(default = "default_jev_model")]
  pub model: String,
}

fn default_jev_model() -> String {
  "jev-latest".to_string()
}

impl Default for EvaluationModelConfig {
  fn default() -> Self {
    Self {
      base_url: "https://api.typesafe.ai/v1/systemone".to_string(),
      api_key: String::new(),
      model: default_jev_model(),
    }
  }
}

impl EvaluationModelConfig {
  pub fn is_enabled(&self) -> bool {
    !self.base_url.trim().is_empty() && !self.api_key.trim().is_empty()
  }
}
