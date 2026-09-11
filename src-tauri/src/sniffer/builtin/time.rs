use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};
use chrono::{DateTime, Local, TimeZone, Utc};

#[derive(Default)]
pub struct TimestampSniffer;

impl ContentSniffer for TimestampSniffer {
    fn id(&self) -> &'static str {
        "timestamp-sniffer"
    }

    fn name(&self) -> &'static str {
        "Timestamp & DateTime Sniffer"
    }

    fn priority(&self) -> u32 {
        75
    }

    fn supports(&self, input: &SniffInput) -> bool {
        matches!(input, SniffInput::Text(_))
    }

    fn sniff(&self, input: &SniffInput) -> SniffOutput {
        let text = match input {
            SniffInput::Text(t) => t.trim(),
            _ => return SniffOutput::default(),
        };

        // 1. 纯数字时间戳探测 (10位秒 或 13位毫秒)
        if text.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(num) = text.parse::<i64>() {
                let dt: Option<DateTime<Utc>> = if text.len() == 10 {
                    Utc.timestamp_opt(num, 0).single()
                } else if text.len() == 13 {
                    Utc.timestamp_millis_opt(num).single()
                } else {
                    None
                };

                if let Some(utc_dt) = dt {
                    let local_dt: DateTime<Local> = DateTime::from(utc_dt);
                    let formatted = format!(
                        "Local Time: {}\nUTC Time:   {}\nTimestamp:  {}",
                        local_dt.format("%Y-%m-%d %H:%M:%S %Z"),
                        utc_dt.format("%Y-%m-%d %H:%M:%S UTC"),
                        num
                    );

                    let mut metadata = serde_json::Map::new();
                    metadata.insert("timestamp".to_string(), num.into());
                    metadata.insert("local".to_string(), local_dt.to_rfc3339().into());
                    metadata.insert("utc".to_string(), utc_dt.to_rfc3339().into());

                    return SniffOutput {
                        matched: true,
                        confidence: 0.88,
                        tags: vec!["format-timestamp".to_string()],
                        preprocessed_text: Some(formatted),
                        suggested_tool_id: Some("time-converter".to_string()),
                        suggested_output_type: Some("text".to_string()),
                        metadata,
                    };
                }
            }
        }

        SniffOutput::default()
    }
}

