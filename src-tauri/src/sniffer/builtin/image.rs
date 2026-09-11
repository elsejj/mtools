use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};

#[derive(Default)]
pub struct ImageSniffer;

impl ContentSniffer for ImageSniffer {
  fn id(&self) -> &'static str {
    "image-sniffer"
  }

  fn name(&self) -> &'static str {
    "Multimodal Image Sniffer"
  }

  fn priority(&self) -> u32 {
    100 // 最高优先级
  }

  fn supports(&self, input: &SniffInput) -> bool {
    matches!(input, SniffInput::Image(_))
  }

  fn sniff(&self, input: &SniffInput) -> SniffOutput {
    let bytes = match input {
      SniffInput::Image(b) => b,
      _ => return SniffOutput::default(),
    };

    let mime = crate::decoder::detect_image_mime(bytes).unwrap_or_else(|| "image/png".to_string());
    let mut metadata = serde_json::Map::new();
    metadata.insert("mimeType".to_string(), mime.clone().into());
    metadata.insert("byteSize".to_string(), bytes.len().into());

    SniffOutput {
      matched: true,
      confidence: 1.0,
      tags: vec!["image".to_string(), "multimodal".to_string()],
      preprocessed_text: None,
      suggested_tool_id: Some("ocr-extractor".to_string()),
      suggested_output_type: Some("image".to_string()),
      metadata,
    }
  }
}
