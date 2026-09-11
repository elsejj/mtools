use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};
use url::Url;

#[derive(Default)]
pub struct UrlSniffer;

impl ContentSniffer for UrlSniffer {
    fn id(&self) -> &'static str {
        "url-sniffer"
    }

    fn name(&self) -> &'static str {
        "URL Protocol & Query Sniffer"
    }

    fn priority(&self) -> u32 {
        80
    }

    fn supports(&self, input: &SniffInput) -> bool {
        matches!(input, SniffInput::Text(_))
    }

    fn sniff(&self, input: &SniffInput) -> SniffOutput {
        let text = match input {
            SniffInput::Text(t) => t.trim(),
            _ => return SniffOutput::default(),
        };

        if !text.starts_with("http://") && !text.starts_with("https://") && !text.starts_with("ftp://") {
            return SniffOutput::default();
        }

        if let Ok(parsed) = Url::parse(text) {
            let mut metadata = serde_json::Map::new();
            metadata.insert("scheme".to_string(), parsed.scheme().into());
            if let Some(host) = parsed.host_str() {
                metadata.insert("host".to_string(), host.into());
            }
            metadata.insert("path".to_string(), parsed.path().into());

            let mut query_map = serde_json::Map::new();
            for (k, v) in parsed.query_pairs() {
                query_map.insert(k.to_string(), v.to_string().into());
            }
            let has_query = !query_map.is_empty();
            metadata.insert("query".to_string(), serde_json::Value::Object(query_map));

            let preprocessed = if has_query {
                format!(
                    "Scheme: {}\nHost: {}\nPath: {}\nQuery:\n{}",
                    parsed.scheme(),
                    parsed.host_str().unwrap_or("-"),
                    parsed.path(),
                    serde_json::to_string_pretty(&metadata.get("query").unwrap()).unwrap_or_default()
                )
            } else {
                format!(
                    "Scheme: {}\nHost: {}\nPath: {}",
                    parsed.scheme(),
                    parsed.host_str().unwrap_or("-"),
                    parsed.path()
                )
            };

            SniffOutput {
                matched: true,
                confidence: 0.90,
                tags: vec!["format-url".to_string()],
                preprocessed_text: Some(preprocessed),
                suggested_tool_id: Some("url-decoder".to_string()),
                suggested_output_type: Some("text".to_string()),
                metadata,
            }
        } else {
            SniffOutput::default()
        }
    }
}

