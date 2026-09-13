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

    // 1. 纯数字时间戳探测 (10位秒 或 13位毫秒 或 16位微秒)
    if text.chars().all(|c| c.is_ascii_digit()) {
      if let Ok(num) = text.parse::<i64>() {
        let dt: Option<DateTime<Utc>> = if text.len() == 10 {
          Utc.timestamp_opt(num, 0).single()
        } else if text.len() == 13 {
          Utc.timestamp_millis_opt(num).single()
        } else if text.len() == 16 {
          Some(Utc.timestamp_nanos(num))
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
            confidence: 0.90,
            tags: vec!["format-timestamp".to_string()],
            preprocessed_text: Some(formatted),
            suggested_tool_id: Some("timestamp-converter".to_string()),
            suggested_output_type: Some("text".to_string()),
            metadata,
          };
        }
      }
    }

    // 2. 日期与时间字符串探测 (如 2024-1-2, 2024-01-02 15:30:00, 2024/1/2, ISO8601 等)
    if text.len() >= 8 && text.len() <= 35 {
      // 尝试 RFC3339 / ISO8601
      if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        let utc_dt = dt.with_timezone(&Utc);
        let local_dt = dt.with_timezone(&Local);
        let ts_sec = utc_dt.timestamp();
        let formatted = format!(
          "Local Time: {}\nUTC Time:   {}\nTimestamp:  {}",
          local_dt.format("%Y-%m-%d %H:%M:%S %Z"),
          utc_dt.format("%Y-%m-%d %H:%M:%S UTC"),
          ts_sec
        );
        let mut metadata = serde_json::Map::new();
        metadata.insert("timestamp".to_string(), ts_sec.into());
        metadata.insert("local".to_string(), local_dt.to_rfc3339().into());
        metadata.insert("utc".to_string(), utc_dt.to_rfc3339().into());

        return SniffOutput {
          matched: true,
          confidence: 0.92,
          tags: vec!["format-timestamp".to_string(), "format-date".to_string()],
          preprocessed_text: Some(formatted),
          suggested_tool_id: Some("timestamp-converter".to_string()),
          suggested_output_type: Some("text".to_string()),
          metadata,
        };
      }

      // 尝试常规 YYYY-M-D / YYYY/M/D / YYYY-M-D H:M:S
      let clean_text = text.replace('/', "-").replace('.', "-");
      let parts: Vec<&str> = clean_text.split_whitespace().collect();
      if (parts.len() == 1 || parts.len() == 2) && parts[0].contains('-') {
        let date_tokens: Vec<&str> = parts[0].split('-').collect();
        if date_tokens.len() == 3 {
          if let (Ok(y), Ok(m), Ok(d)) = (
            date_tokens[0].parse::<i32>(),
            date_tokens[1].parse::<u32>(),
            date_tokens[2].parse::<u32>(),
          ) {
            if y >= 1970 && y <= 2100 && (1..=12).contains(&m) && (1..=31).contains(&d) {
              if let Some(naive_date) = chrono::NaiveDate::from_ymd_opt(y, m, d) {
                let naive_time = if parts.len() == 2 {
                  let time_tokens: Vec<&str> = parts[1].split(':').collect();
                  if time_tokens.len() == 2 {
                    if let (Ok(h), Ok(min)) =
                      (time_tokens[0].parse::<u32>(), time_tokens[1].parse::<u32>())
                    {
                      chrono::NaiveTime::from_hms_opt(h, min, 0)
                    } else {
                      None
                    }
                  } else if time_tokens.len() == 3 {
                    if let (Ok(h), Ok(min), Ok(s)) = (
                      time_tokens[0].parse::<u32>(),
                      time_tokens[1].parse::<u32>(),
                      time_tokens[2].parse::<u32>(),
                    ) {
                      chrono::NaiveTime::from_hms_opt(h, min, s)
                    } else {
                      None
                    }
                  } else {
                    None
                  }
                } else {
                  Some(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                };

                if let Some(time) = naive_time {
                  let naive_dt = naive_date.and_time(time);
                  if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                    let utc_dt: DateTime<Utc> = DateTime::from(local_dt);
                    let ts_sec = local_dt.timestamp();
                    let formatted = format!(
                      "Local Time: {}\nUTC Time:   {}\nTimestamp:  {}",
                      local_dt.format("%Y-%m-%d %H:%M:%S %Z"),
                      utc_dt.format("%Y-%m-%d %H:%M:%S UTC"),
                      ts_sec
                    );

                    let mut metadata = serde_json::Map::new();
                    metadata.insert("timestamp".to_string(), ts_sec.into());
                    metadata.insert("local".to_string(), local_dt.to_rfc3339().into());
                    metadata.insert("utc".to_string(), utc_dt.to_rfc3339().into());

                    return SniffOutput {
                      matched: true,
                      confidence: 0.90,
                      tags: vec!["format-timestamp".to_string(), "format-date".to_string()],
                      preprocessed_text: Some(formatted),
                      suggested_tool_id: Some("timestamp-converter".to_string()),
                      suggested_output_type: Some("text".to_string()),
                      metadata,
                    };
                  }
                }
              }
            }
          }
        }
      }
    }

    SniffOutput::default()
  }
}
