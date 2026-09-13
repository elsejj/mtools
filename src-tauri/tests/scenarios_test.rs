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

#[test]
fn test_scenario_8_python_float_timestamp_auto_routing() {
  let registry = SnifferRegistry::new();
  let text = "1789264888.123456";

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
    "Python float timestamp '1789264888.123456' must route to timestamp-converter"
  );
  assert!(
    enriched
      .tags
      .contains(&"format-python-timestamp".to_string()),
    "Tags must include format-python-timestamp"
  );
  assert!(
    enriched
      .candidate_tool_scores
      .iter()
      .any(|s| s.tool_id == "timestamp-converter" && s.score >= 85.0),
    "timestamp-converter score must be >= 85"
  );

  let preprocessed = enriched
    .preprocessed_result
    .expect("Must have preprocessed_result");
  assert_eq!(preprocessed.suggested_output_type, "markdown");
  assert!(
    preprocessed
      .formatted_text
      .as_ref()
      .unwrap()
      .contains("| **Python 浮点时间戳 (s)** | `1789264888.123456` |"),
    "Formatted text must contain markdown table with python float timestamp"
  );
}

#[test]
fn test_scenario_9_calculator_unit_expression_routing() {
  let registry = SnifferRegistry::new();
  let text = "1K + 1万+1w";

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
    enriched.recommended_tool_id, "calculator",
    "Expression '1K + 1万+1w' must route to calculator"
  );
  assert!(
    enriched.tags.contains(&"format-calc".to_string()),
    "Tags must include format-calc"
  );
  assert!(
    enriched
      .candidate_tool_scores
      .iter()
      .any(|s| s.tool_id == "calculator" && s.score >= 85.0),
    "calculator score must be >= 85"
  );

  let preprocessed = enriched
    .preprocessed_result
    .expect("Must have preprocessed_result for calculator");
  assert_eq!(preprocessed.suggested_output_type, "markdown");
  let table = preprocessed.formatted_text.as_ref().unwrap();
  assert!(
    table.contains("| **精确值 (Exact)** | 21000 |"),
    "Table must contain exact value 21000"
  );
  assert!(
    table.contains("| **英语习惯 (EN)** | 21K |"),
    "Table must contain English format 21K"
  );
  assert!(
    table.contains("| **中文习惯 (CN)** | 2.1万 |"),
    "Table must contain Chinese format 2.1万"
  );
}

#[test]
fn test_scenario_10_single_number_priority_arbitration() {
  let registry = SnifferRegistry::new();
  let text = "123000000000";

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

  // #1 Must be timestamp-converter
  assert_eq!(
    enriched.recommended_tool_id, "timestamp-converter",
    "Single number '123000000000' must prioritize timestamp-converter"
  );

  // Check scores order: timestamp-converter (>= 90) > calculator (>= 80) > llm-translate (>= 60)
  let time_score = enriched
    .candidate_tool_scores
    .iter()
    .find(|s| s.tool_id == "timestamp-converter")
    .map(|s| s.score)
    .unwrap_or(0.0);
  let calc_score = enriched
    .candidate_tool_scores
    .iter()
    .find(|s| s.tool_id == "calculator")
    .map(|s| s.score)
    .unwrap_or(0.0);
  let translate_score = enriched
    .candidate_tool_scores
    .iter()
    .find(|s| s.tool_id == "llm-translate")
    .map(|s| s.score)
    .unwrap_or(0.0);

  assert!(
    time_score >= 90.0,
    "timestamp-converter score must be >= 90, got {}",
    time_score
  );
  assert!(
    calc_score >= 80.0,
    "calculator score must be >= 80, got {}",
    calc_score
  );
  assert!(
    translate_score >= 60.0 && translate_score < calc_score,
    "translate score must be between 60 and calc_score, got {}",
    translate_score
  );
  assert!(
    time_score > calc_score && calc_score > translate_score,
    "Priority must strictly be timestamp-converter > calculator > llm-translate"
  );

  let preprocessed = enriched
    .preprocessed_result
    .expect("Must have preprocessed_result for timestamp-converter");
  let table = preprocessed.formatted_text.as_ref().unwrap();
  assert!(
    table.contains("| **秒级时间戳 (s)** | `123000000` |"),
    "Table must contain timestamp seconds"
  );
  assert!(
    table.contains("| **英语习惯 (EN)** | 123G |"),
    "Table must contain calculator English format 123G"
  );
  assert!(
    table.contains("| **中文习惯 (CN)** | 1230亿 |"),
    "Table must contain calculator Chinese format 1230亿"
  );
}

#[test]
fn test_scenario_11_quoted_json_string_decoding_and_routing() {
  let registry = SnifferRegistry::new();

  // 1. 双引号转义的 JSON 字符串
  let raw_quoted_json = r#""{\"name\":\"mtools\",\"tags\":[\"tauri\",\"vue\"]}""#;
  let decoded = DecoderPipeline::decode(raw_quoted_json);
  let (actual_text, trace) = match decoded {
    DecodedOutput::Text { text, trace } => (text, trace),
    _ => panic!("Expected DecodedOutput::Text"),
  };
  assert_eq!(trace, vec!["unquote"]);
  assert_eq!(actual_text, r#"{"name":"mtools","tags":["tauri","vue"]}"#);

  let enriched = registry.execute(
    &SniffInput::Text(&actual_text),
    raw_quoted_json.to_string(),
    actual_text.clone(),
    trace,
    None,
  );
  assert_eq!(enriched.recommended_tool_id, "json-formatter");
  assert!(enriched.tags.contains(&"format-json".to_string()));
  assert!(enriched.tags.contains(&"decoded-from-unquote".to_string()));
  let formatted = enriched
    .preprocessed_result
    .expect("Must have preprocessed_result")
    .formatted_text
    .expect("Must have formatted_text");
  assert!(formatted.contains(r#""name": "mtools""#));

  // 2. 单引号包裹的 JSON 字符串
  let raw_single_quoted = r#"'{"user":"alice","active":true}'"#;
  let decoded2 = DecoderPipeline::decode(raw_single_quoted);
  let (actual_text2, trace2) = match decoded2 {
    DecodedOutput::Text { text, trace } => (text, trace),
    _ => panic!("Expected DecodedOutput::Text"),
  };
  assert_eq!(trace2, vec!["unquote"]);
  assert_eq!(actual_text2, r#"{"user":"alice","active":true}"#);

  let enriched2 = registry.execute(
    &SniffInput::Text(&actual_text2),
    raw_single_quoted.to_string(),
    actual_text2.clone(),
    trace2,
    None,
  );
  assert_eq!(enriched2.recommended_tool_id, "json-formatter");

  // 3. 引号包裹的 Base64 字符串复合解码
  let raw_quoted_b64 = r#""eyJhY3Rpb24iOiJxdWVyeSIsImxpbWl0Ijo1MH0=""#;
  let decoded3 = DecoderPipeline::decode(raw_quoted_b64);
  let (actual_text3, trace3) = match decoded3 {
    DecodedOutput::Text { text, trace } => (text, trace),
    _ => panic!("Expected DecodedOutput::Text"),
  };
  assert_eq!(trace3, vec!["unquote", "base64"]);
  assert_eq!(actual_text3, r#"{"action":"query","limit":50}"#);

  let enriched3 = registry.execute(
    &SniffInput::Text(&actual_text3),
    raw_quoted_b64.to_string(),
    actual_text3.clone(),
    trace3,
    None,
  );
  assert_eq!(enriched3.recommended_tool_id, "json-formatter");
  assert!(enriched3.tags.contains(&"decoded-from-unquote".to_string()));
  assert!(enriched3.tags.contains(&"decoded-from-base64".to_string()));
}

#[test]
fn test_scenario_12_multiline_file_paths_auto_reading_and_routing() {
  use std::io::Write;
  let registry = SnifferRegistry::new();
  let temp_dir = std::env::temp_dir();
  let json_path = temp_dir.join("mtools_scenario_12.json");
  let other_path = temp_dir.join("mtools_scenario_12.txt");

  let mut f1 = std::fs::File::create(&json_path).unwrap();
  write!(f1, "{{\"user\":\"bob\",\"score\":99}}").unwrap();

  let mut f2 = std::fs::File::create(&other_path).unwrap();
  write!(f2, "other text").unwrap();

  // 1. 多行文件路径输入：第1行目录，第2行JSON文件，第3行TXT文件
  let multiline_input = format!(
    "{}\n{}\n{}",
    temp_dir.to_str().unwrap(),
    json_path.to_str().unwrap(),
    other_path.to_str().unwrap()
  );

  let decoded = DecoderPipeline::decode(&multiline_input);
  let (actual_text, trace) = match decoded {
    DecodedOutput::Text { text, trace } => (text, trace),
    _ => panic!("Expected DecodedOutput::Text"),
  };
  assert_eq!(trace, vec!["file"]);
  assert_eq!(actual_text, "{\"user\":\"bob\",\"score\":99}");

  let enriched = registry.execute(
    &SniffInput::Text(&actual_text),
    multiline_input.clone(),
    actual_text.clone(),
    trace,
    None,
  );
  assert_eq!(enriched.recommended_tool_id, "json-formatter");
  assert!(enriched.tags.contains(&"decoded-from-file".to_string()));
  assert!(enriched.tags.contains(&"format-json".to_string()));

  let _ = std::fs::remove_file(json_path);
  let _ = std::fs::remove_file(other_path);
}

#[test]
fn test_scenario_13_single_image_file_path_routes_to_ocr_not_calculator() {
  use std::io::Write;
  let registry = SnifferRegistry::new();
  let temp_dir = std::env::temp_dir();
  let image_path = temp_dir.join("f4400f9cb4e12eee8bfd5162962203f4.jpg");

  // 写入合法的 JPEG 图片文件
  let mut f = std::fs::File::create(&image_path).unwrap();
  f.write_all(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46])
    .unwrap();

  // 模拟带首尾空格的单个图片路径输入
  let input = format!("  {}  ", image_path.to_str().unwrap());

  // 1. DecoderPass: 自动识别存在的文件并升格为 Image
  let decoded = DecoderPipeline::decode(&input);
  let (bytes, mime_type, trace) = match decoded {
    DecodedOutput::Image {
      bytes,
      mime_type,
      trace,
    } => (bytes, mime_type, trace),
    _ => panic!("Expected DecodedOutput::Image for single image file path"),
  };
  assert_eq!(mime_type, "image/jpeg");
  assert_eq!(trace, vec!["file"]);

  // 2. SnifferPass: 输入为 Image，直接命中 ImageSniffer，推荐 ocr-extractor，绝不命中计算器
  let sniff_input = SniffInput::Image(&bytes);
  let enriched = registry.execute(&sniff_input, input.clone(), input.clone(), trace, None);
  assert_eq!(
    enriched.recommended_tool_id, "ocr-extractor",
    "Should recommend ocr-extractor"
  );
  assert!(
    enriched.tags.contains(&"image".to_string()),
    "Tags must contain image"
  );
  assert!(
    !enriched
      .candidate_tool_scores
      .iter()
      .any(|s| s.tool_id == "calculator"),
    "Calculator must NOT be a candidate for image inputs"
  );

  let _ = std::fs::remove_file(image_path);
}
