use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};
use base64::prelude::*;
use serde_json::Value;

#[derive(Default)]
pub struct JwtSniffer;

impl ContentSniffer for JwtSniffer {
    fn id(&self) -> &'static str {
        "jwt-sniffer"
    }

    fn name(&self) -> &'static str {
        "JWT Token Parser & Sniffer"
    }

    fn priority(&self) -> u32 {
        85
    }

    fn supports(&self, input: &SniffInput) -> bool {
        matches!(input, SniffInput::Text(_))
    }

    fn sniff(&self, input: &SniffInput) -> SniffOutput {
        let text = match input {
            SniffInput::Text(t) => t.trim(),
            _ => return SniffOutput::default(),
        };

        // 必须为 3 段以点分隔
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() != 3 {
            return SniffOutput::default();
        }

        // 尝试解码 Header 和 Payload (Base64Url without padding)
        let decode_part = |s: &str| -> Option<Value> {
            let bytes = BASE64_URL_SAFE_NO_PAD.decode(s).or_else(|_| BASE64_STANDARD_NO_PAD.decode(s)).ok()?;
            serde_json::from_slice::<Value>(&bytes).ok()
        };

        let header = decode_part(parts[0]);
        let payload = decode_part(parts[1]);

        if let (Some(h), Some(p)) = (header, payload) {
            let mut metadata = serde_json::Map::new();
            metadata.insert("header".to_string(), h.clone());
            metadata.insert("payload".to_string(), p.clone());

            let formatted = format!(
                "// --- Header ---\n{}\n\n// --- Payload ---\n{}",
                serde_json::to_string_pretty(&h).unwrap_or_default(),
                serde_json::to_string_pretty(&p).unwrap_or_default()
            );

            SniffOutput {
                matched: true,
                confidence: 0.92,
                tags: vec!["format-jwt".to_string(), "auth-token".to_string()],
                preprocessed_text: Some(formatted),
                suggested_tool_id: Some("jwt-viewer".to_string()),
                suggested_output_type: Some("json".to_string()),
                metadata,
            }
        } else {
            SniffOutput::default()
        }
    }
}

