use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};
use chrono::{DateTime, Local, TimeZone, Utc};

#[derive(Default)]
pub struct TimestampSniffer;

fn generate_markdown_table(
  input: &str,
  local_dt: &DateTime<Local>,
  utc_dt: &DateTime<Utc>,
  sec: i64,
  ms: i64,
  python_float: &str,
  subsecond_str: Option<&str>,
) -> String {
  let local_str = match subsecond_str {
    Some(sub) => format!("{}.{}", local_dt.format("%Y-%m-%d %H:%M:%S"), sub),
    None => local_dt.format("%Y-%m-%d %H:%M:%S").to_string(),
  };
  let utc_str = match subsecond_str {
    Some(sub) => format!("{}.{} UTC", utc_dt.format("%Y-%m-%d %H:%M:%S"), sub),
    None => utc_dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
  };
  let iso_str = utc_dt.to_rfc3339();

  format!(
    "| 格式 / 属性 | 数值 / 结果 |\n\
     | :--- | :--- |\n\
     | **原始输入 (Input)** | `{}` |\n\
     | **本地时间 (Local Time)** | {} |\n\
     | **UTC 时间 (UTC Time)** | {} |\n\
     | **ISO 8601** | `{}` |\n\
     | **秒级时间戳 (s)** | `{}` |\n\
     | **毫秒级时间戳 (ms)** | `{}` |\n\
     | **Python 浮点时间戳 (s)** | `{}` |\n",
    input, local_str, utc_str, iso_str, sec, ms, python_float
  )
}

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
        let (dt, sec, ms, python_float, sub_str): (
          Option<DateTime<Utc>>,
          i64,
          i64,
          String,
          Option<String>,
        ) = if text.len() == 10 {
          (
            Utc.timestamp_opt(num, 0).single(),
            num,
            num * 1000,
            format!("{}.0", num),
            None,
          )
        } else if text.len() == 13 {
          let sec = num / 1000;
          let sub = (num % 1000) as u32;
          let py = if sub == 0 {
            format!("{}.0", sec)
          } else {
            format!("{}.{:03}", sec, sub)
          };
          let sub_s = if sub != 0 {
            Some(format!("{:03}", sub))
          } else {
            None
          };
          (Utc.timestamp_millis_opt(num).single(), sec, num, py, sub_s)
        } else if text.len() == 16 {
          let sec = num / 1_000_000;
          let sub = (num % 1_000_000) as u32;
          let py = if sub == 0 {
            format!("{}.0", sec)
          } else {
            format!("{}.{:06}", sec, sub)
          };
          let sub_s = if sub != 0 {
            Some(format!("{:06}", sub))
          } else {
            None
          };
          (
            Utc.timestamp_micros(num).single(),
            sec,
            num / 1000,
            py,
            sub_s,
          )
        } else {
          (None, 0, 0, String::new(), None)
        };

        if let Some(utc_dt) = dt {
          let local_dt: DateTime<Local> = DateTime::from(utc_dt);
          let formatted = generate_markdown_table(
            text,
            &local_dt,
            &utc_dt,
            sec,
            ms,
            &python_float,
            sub_str.as_deref(),
          );

          let mut metadata = serde_json::Map::new();
          metadata.insert("timestamp".to_string(), sec.into());
          metadata.insert("timestamp_ms".to_string(), ms.into());
          metadata.insert("python_float".to_string(), python_float.into());
          metadata.insert("local".to_string(), local_dt.to_rfc3339().into());
          metadata.insert("utc".to_string(), utc_dt.to_rfc3339().into());

          return SniffOutput {
            matched: true,
            confidence: 0.90,
            tags: vec!["format-timestamp".to_string()],
            preprocessed_text: Some(formatted),
            suggested_tool_id: Some("timestamp-converter".to_string()),
            suggested_output_type: Some("markdown".to_string()),
            metadata,
          };
        }
      }
    }

    // 2. Python 浮点数时间戳探测 (如 1789264888.123456, 1789264888.0)
    let dot_parts: Vec<&str> = text.split('.').collect();
    if dot_parts.len() == 2
      && dot_parts[0].len() >= 9
      && dot_parts[0].len() <= 11
      && dot_parts[0].chars().all(|c| c.is_ascii_digit())
      && dot_parts[1].chars().all(|c| c.is_ascii_digit())
    {
      if let Ok(sec) = dot_parts[0].parse::<i64>() {
        let frac_str = dot_parts[1];
        let nanos = if frac_str.len() <= 9 {
          let padded = format!("{:0<9}", frac_str);
          padded.parse::<u32>().unwrap_or(0)
        } else {
          frac_str[..9].parse::<u32>().unwrap_or(0)
        };

        if let Some(utc_dt) = Utc.timestamp_opt(sec, nanos).single() {
          let local_dt: DateTime<Local> = DateTime::from(utc_dt);
          let ms = utc_dt.timestamp_millis();
          let formatted =
            generate_markdown_table(text, &local_dt, &utc_dt, sec, ms, text, Some(frac_str));

          let mut metadata = serde_json::Map::new();
          metadata.insert("timestamp".to_string(), sec.into());
          metadata.insert("timestamp_ms".to_string(), ms.into());
          metadata.insert("python_float".to_string(), text.into());
          metadata.insert("local".to_string(), local_dt.to_rfc3339().into());
          metadata.insert("utc".to_string(), utc_dt.to_rfc3339().into());

          return SniffOutput {
            matched: true,
            confidence: 0.92,
            tags: vec![
              "format-timestamp".to_string(),
              "format-python-timestamp".to_string(),
            ],
            preprocessed_text: Some(formatted),
            suggested_tool_id: Some("timestamp-converter".to_string()),
            suggested_output_type: Some("markdown".to_string()),
            metadata,
          };
        }
      }
    }

    // 3. 日期与时间字符串探测 (如 2024-1-2, 2024-01-02 15:30:00, 2024/1/2, ISO8601 等)
    if text.len() >= 8 && text.len() <= 35 {
      // 尝试 RFC3339 / ISO8601
      if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        let utc_dt = dt.with_timezone(&Utc);
        let local_dt = dt.with_timezone(&Local);
        let sec = utc_dt.timestamp();
        let ms = utc_dt.timestamp_millis();
        let sub = (ms % 1000).abs();
        let py = if sub == 0 {
          format!("{}.0", sec)
        } else {
          format!("{}.{:03}", sec, sub)
        };
        let sub_s = if sub != 0 {
          Some(format!("{:03}", sub))
        } else {
          None
        };

        let formatted =
          generate_markdown_table(text, &local_dt, &utc_dt, sec, ms, &py, sub_s.as_deref());

        let mut metadata = serde_json::Map::new();
        metadata.insert("timestamp".to_string(), sec.into());
        metadata.insert("timestamp_ms".to_string(), ms.into());
        metadata.insert("python_float".to_string(), py.into());
        metadata.insert("local".to_string(), local_dt.to_rfc3339().into());
        metadata.insert("utc".to_string(), utc_dt.to_rfc3339().into());

        return SniffOutput {
          matched: true,
          confidence: 0.92,
          tags: vec!["format-timestamp".to_string(), "format-date".to_string()],
          preprocessed_text: Some(formatted),
          suggested_tool_id: Some("timestamp-converter".to_string()),
          suggested_output_type: Some("markdown".to_string()),
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
                    let sec = local_dt.timestamp();
                    let ms = local_dt.timestamp_millis();
                    let py = format!("{}.0", sec);
                    let formatted =
                      generate_markdown_table(text, &local_dt, &utc_dt, sec, ms, &py, None);

                    let mut metadata = serde_json::Map::new();
                    metadata.insert("timestamp".to_string(), sec.into());
                    metadata.insert("timestamp_ms".to_string(), ms.into());
                    metadata.insert("python_float".to_string(), py.into());
                    metadata.insert("local".to_string(), local_dt.to_rfc3339().into());
                    metadata.insert("utc".to_string(), utc_dt.to_rfc3339().into());

                    return SniffOutput {
                      matched: true,
                      confidence: 0.90,
                      tags: vec!["format-timestamp".to_string(), "format-date".to_string()],
                      preprocessed_text: Some(formatted),
                      suggested_tool_id: Some("timestamp-converter".to_string()),
                      suggested_output_type: Some("markdown".to_string()),
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
