use mtools_lib::decoder::{DecodedOutput, DecoderPipeline};
use mtools_lib::models::{SaveFileRequest, WindowGeometry};
use mtools_lib::post_action::save_content;
use mtools_lib::sniffer::{SniffInput, SnifferRegistry};
use mtools_lib::storage::window::{get_window_geometry, save_window_geometry};
use mtools_lib::storage::AppStorage;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_scenario_1_disordered_json_auto_routing() {
  let registry = SnifferRegistry::new();
  let raw_json = r#"{"z":10,"a":"hello","b":[1,2,3]}"#;

  // 1. Decoder pass
  let decoded = DecoderPipeline::decode(raw_json);
  let (actual_text, trace) = match decoded {
    DecodedOutput::Text { text, trace } => (text, trace),
    DecodedOutput::PassThrough => (raw_json.to_string(), Vec::new()),
    _ => panic!("Expected text or passthrough decoded output"),
  };
  assert!(
    trace.is_empty(),
    "Plain JSON should not have decoding trace"
  );

  // 2. Sniffer pass via registry execute
  let enriched = registry.execute(
    &SniffInput::Text(&actual_text),
    raw_json.to_string(),
    actual_text.clone(),
    trace,
    None,
  );

  assert_eq!(
    enriched.recommended_tool_id, "json-formatter",
    "Should recommend json-formatter"
  );
  assert!(
    enriched.tags.contains(&"format-json".to_string()),
    "Tags must include format-json"
  );

  // 3. Preprocessed pretty text check
  let prep = enriched
    .preprocessed_result
    .expect("Should produce preprocessedResult");
  let pretty = prep
    .formatted_text
    .expect("Should produce pretty JSON text");
  assert!(pretty.contains("  \"a\": \"hello\""));
  assert!(pretty.contains("  \"z\": 10"));
}

#[test]
fn test_scenario_1_base64_encoded_json_auto_routing() {
  let registry = SnifferRegistry::new();
  // {"action":"query","limit":50} in base64
  let b64_json = "eyJhY3Rpb24iOiJxdWVyeSIsImxpbWl0Ijo1MH0=";

  // 1. Decoder pass
  let decoded = DecoderPipeline::decode(b64_json);
  let (actual_text, trace) = match decoded {
    DecodedOutput::Text { text, trace } => (text, trace),
    _ => panic!("Expected text decoded output from Base64"),
  };
  assert_eq!(trace, vec!["base64".to_string()]);
  assert_eq!(actual_text, "{\"action\":\"query\",\"limit\":50}");

  // 2. Sniffer pass
  let enriched = registry.execute(
    &SniffInput::Text(&actual_text),
    b64_json.to_string(),
    actual_text.clone(),
    trace.clone(),
    None,
  );

  assert_eq!(enriched.recommended_tool_id, "json-formatter");
  assert_eq!(
    enriched.metadata["decodingTrace"],
    serde_json::json!(vec!["base64".to_string()])
  );
}

#[test]
fn test_scenario_2_base64_image_magic_upgrade() {
  // 1x1 transparent PNG encoded in Base64
  let png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

  let decoded = DecoderPipeline::decode(png_base64);
  match decoded {
    DecodedOutput::Image {
      bytes,
      mime_type,
      trace,
    } => {
      assert_eq!(mime_type, "image/png");
      assert_eq!(trace, vec!["base64".to_string()]);
      assert!(!bytes.is_empty());
      // Check PNG magic bytes: 0x89, 'P', 'N', 'G'
      assert_eq!(&bytes[0..4], &[0x89, b'P', b'N', b'G']);
    }
    _ => panic!("Expected DecodedOutput::Image upgraded from base64 magic bytes"),
  }
}

#[test]
fn test_scenario_3_post_action_timestamp_file_save() {
  let temp_dir = std::env::temp_dir().join(format!("mtools_test_save_{}", uuid::Uuid::new_v4()));
  fs::create_dir_all(&temp_dir).unwrap();

  let target_dir_str = temp_dir.to_str().unwrap().to_string();
  let content_str = "OCR 识别提取出来的排版文本样例内容";

  let req = SaveFileRequest {
    target_directory: target_dir_str,
    extension: "txt".to_string(),
    content: content_str.to_string(),
    custom_filename: None,
  };

  let res = save_content(req).unwrap();

  let saved_path = PathBuf::from(&res.full_path);
  assert!(
    saved_path.exists(),
    "Saved file must physically exist on disk"
  );
  assert!(
    res.file_name.ends_with(".txt"),
    "Filename must have .txt extension"
  );
  assert!(
    res.file_name.contains('_'),
    "Filename should follow YYYY-MM-DD_HH-mm-ss naming pattern"
  );

  let read_back = fs::read_to_string(&saved_path).unwrap();
  assert_eq!(read_back, content_str, "Content written must match exactly");

  // Clean up
  let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_scenario_4_window_geometry_persistence() {
  let temp_db_dir = std::env::temp_dir().join(format!("mtools_test_db_{}", uuid::Uuid::new_v4()));
  fs::create_dir_all(&temp_db_dir).unwrap();

  let storage = AppStorage::init(temp_db_dir.clone()).unwrap();

  let original_geom = WindowGeometry {
    x: 120,
    y: 240,
    width: 1080,
    height: 720,
    is_maximized: false,
  };

  // 1. Save window state
  {
    let conn = storage.db.lock().unwrap();
    save_window_geometry(&conn, &original_geom).unwrap();
  }

  // 2. Restore window state
  let loaded_geom = {
    let conn = storage.db.lock().unwrap();
    get_window_geometry(&conn)
      .unwrap()
      .expect("Window geometry should exist in storage")
  };

  assert_eq!(loaded_geom.x, 120);
  assert_eq!(loaded_geom.y, 240);
  assert_eq!(loaded_geom.width, 1080);
  assert_eq!(loaded_geom.height, 720);
  assert_eq!(loaded_geom.is_maximized, false);

  // Clean up
  let _ = fs::remove_dir_all(&temp_db_dir);
}

#[test]
fn test_scenario_5_natural_language_auto_routing_to_llm_translate() {
  let registry = SnifferRegistry::new();
  let text = "Native Bigint was added to JS recently, so we added an option to leverage it instead of bignumber.js. However, the parsing with native BigInt is kept an option for backward compability.";

  let decoded = DecoderPipeline::decode(text);
  let (actual_text, trace) = match decoded {
    DecodedOutput::Text { text, trace } => (text, trace),
    DecodedOutput::PassThrough => (text.to_string(), Vec::new()),
    _ => panic!("Expected text or passthrough decoded output"),
  };

  let enriched = registry.execute(
    &SniffInput::Text(&actual_text),
    text.to_string(),
    actual_text.clone(),
    trace,
    None,
  );

  assert_eq!(
    enriched.recommended_tool_id, "llm-translate",
    "Natural language English text should recommend llm-translate"
  );
  assert!(
    enriched.tags.contains(&"natural-language".to_string()),
    "Tags must include natural-language"
  );
  assert!(
    enriched
      .candidate_tool_scores
      .iter()
      .any(|s| s.tool_id == "llm-translate" && s.score >= 60.0),
    "llm-translate candidate score should be >= 60"
  );
}

#[test]
fn test_scenario_6_date_string_auto_routing_to_timestamp_converter() {
  let registry = SnifferRegistry::new();
  let text = "2024-1-2";

  let decoded = DecoderPipeline::decode(text);
  let (actual_text, trace) = match decoded {
    DecodedOutput::Text { text, trace } => (text, trace),
    DecodedOutput::PassThrough => (text.to_string(), Vec::new()),
    _ => panic!("Expected text or passthrough decoded output"),
  };

  let enriched = registry.execute(
    &SniffInput::Text(&actual_text),
    text.to_string(),
    actual_text.clone(),
    trace,
    None,
  );

  assert_eq!(
    enriched.recommended_tool_id, "timestamp-converter",
    "Date string '2024-1-2' must prioritize timestamp-converter over translation"
  );
  assert!(
    enriched.tags.contains(&"format-date".to_string()),
    "Tags must include format-date"
  );
  assert!(
    enriched
      .candidate_tool_scores
      .iter()
      .any(|s| s.tool_id == "timestamp-converter" && s.score >= 85.0),
    "timestamp-converter score must be >= 85"
  );
}
