use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};

#[derive(Default)]
pub struct CalculatorSniffer;

/// 格式化精确值：大于等于1最多保留3位小数，小于1最多保留7位小数，均去除末尾的0
fn format_exact(val: f64) -> String {
  if val.is_nan() || val.is_infinite() {
    return val.to_string();
  }
  let abs = val.abs();
  let s = if abs >= 1.0 {
    format!("{:.3}", val)
  } else {
    format!("{:.7}", val)
  };
  s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// 格式化英语习惯：以 K M G T P 等为单位，最多保留3位小数，去尾0
pub fn format_english(val: f64) -> String {
  if val.is_nan() || val.is_infinite() {
    return val.to_string();
  }
  let sign = if val < 0.0 { "-" } else { "" };
  let abs = val.abs();
  let fmt = |n: f64, u: &str| -> String {
    let s = format!("{:.3}", n);
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    format!("{}{}{}", sign, trimmed, u)
  };

  if abs >= 1e15 {
    fmt(abs / 1e15, "P")
  } else if abs >= 1e12 {
    fmt(abs / 1e12, "T")
  } else if abs >= 1e9 {
    fmt(abs / 1e9, "G")
  } else if abs >= 1e6 {
    fmt(abs / 1e6, "M")
  } else if abs >= 1e3 {
    fmt(abs / 1e3, "K")
  } else {
    format_exact(val)
  }
}

/// 格式化中文习惯：以 千/万/亿/万亿 等为单位，最多保留3位小数，去尾0
pub fn format_chinese(val: f64) -> String {
  if val.is_nan() || val.is_infinite() {
    return val.to_string();
  }
  let sign = if val < 0.0 { "-" } else { "" };
  let abs = val.abs();
  let fmt = |n: f64, u: &str| -> String {
    let s = format!("{:.3}", n);
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    format!("{}{}{}", sign, trimmed, u)
  };

  if abs >= 1e12 {
    fmt(abs / 1e12, "万亿")
  } else if abs >= 1e8 {
    fmt(abs / 1e8, "亿")
  } else if abs >= 1e4 {
    fmt(abs / 1e4, "万")
  } else if abs >= 1e3 {
    fmt(abs / 1e3, "千")
  } else {
    format_exact(val)
  }
}

pub fn generate_calc_markdown(input: &str, result: f64) -> String {
  let exact = format_exact(result);
  let en = format_english(result);
  let cn = format_chinese(result);

  format!(
    "| 格式 / 维度 | 结算结果 |\n\
     | :--- | :--- |\n\
     | **输入表达式 (Input)** | `{}` |\n\
     | **精确值 (Exact)** | {} |\n\
     | **英语习惯 (EN)** | {} |\n\
     | **中文习惯 (CN)** | {} |\n",
    input, exact, en, cn
  )
}

/// 替换单位为数值乘数
fn normalize_units(expr: &str) -> String {
  let mut s = expr.replace('×', "*").replace('÷', "/").replace('^', "**");

  let units = [
    ("万亿", "* 1e12"),
    ("兆", "* 1e12"),
    ("千万", "* 1e7"),
    ("百万", "* 1e6"),
    ("万", "* 1e4"),
    ("亿", "* 1e8"),
    ("千", "* 1e3"),
    ("百", "* 1e2"),
  ];

  for (u, rep) in units {
    s = s.replace(u, rep);
  }

  // Handle single ASCII letter units attached to numbers
  let mut result = String::new();
  let chars: Vec<char> = s.chars().collect();
  let len = chars.len();
  let mut i = 0;

  while i < len {
    let c = chars[i];
    if c.is_ascii_digit() {
      result.push(c);
      i += 1;
      continue;
    }

    let prev = if i > 0 { Some(chars[i - 1]) } else { None };
    let prev_is_num = prev
      .map(|p| p.is_ascii_digit() || p == '.')
      .unwrap_or(false);

    if prev_is_num {
      match c {
        'K' | 'k' | 'q' | 'Q' => {
          result.push_str(" * 1e3");
          i += 1;
          continue;
        }
        'w' | 'W' => {
          result.push_str(" * 1e4");
          i += 1;
          continue;
        }
        'M' | 'm' => {
          result.push_str(" * 1e6");
          i += 1;
          continue;
        }
        'y' | 'Y' => {
          result.push_str(" * 1e8");
          i += 1;
          continue;
        }
        'G' | 'g' | 'B' => {
          result.push_str(" * 1e9");
          i += 1;
          continue;
        }
        'b' => {
          // lowercase b is 百 (100)
          result.push_str(" * 1e2");
          i += 1;
          continue;
        }
        'T' | 't' | 'z' | 'Z' => {
          result.push_str(" * 1e12");
          i += 1;
          continue;
        }
        'P' | 'p' => {
          result.push_str(" * 1e15");
          i += 1;
          continue;
        }
        _ => {}
      }
    }

    result.push(c);
    i += 1;
  }

  result
}

/// 简易算式求值 (支持 + - * / 加减乘除及括号)
fn simple_eval(expr: &str) -> Option<f64> {
  let normalized = normalize_units(expr);
  // Tokenize numbers and operators
  let mut tokens: Vec<String> = Vec::new();
  let mut cur = String::new();

  for c in normalized.chars() {
    if c.is_whitespace() {
      if !cur.is_empty() {
        tokens.push(cur.clone());
        cur.clear();
      }
    } else if "+-*/()".contains(c) {
      if !cur.is_empty() {
        tokens.push(cur.clone());
        cur.clear();
      }
      tokens.push(c.to_string());
    } else {
      cur.push(c);
    }
  }
  if !cur.is_empty() {
    tokens.push(cur);
  }

  // Parse using basic expression grammar
  let mut pos = 0;
  parse_add_sub(&tokens, &mut pos)
}

fn parse_primary(tokens: &[String], pos: &mut usize) -> Option<f64> {
  if *pos >= tokens.len() {
    return None;
  }
  let tok = &tokens[*pos];
  if tok == "(" {
    *pos += 1;
    let val = parse_add_sub(tokens, pos)?;
    if *pos < tokens.len() && tokens[*pos] == ")" {
      *pos += 1;
      Some(val)
    } else {
      None
    }
  } else if tok == "-" {
    *pos += 1;
    let val = parse_primary(tokens, pos)?;
    Some(-val)
  } else if tok == "+" {
    *pos += 1;
    parse_primary(tokens, pos)
  } else {
    *pos += 1;
    tok.parse::<f64>().ok()
  }
}

fn parse_mul_div(tokens: &[String], pos: &mut usize) -> Option<f64> {
  let mut left = parse_primary(tokens, pos)?;
  while *pos < tokens.len() {
    let op = &tokens[*pos];
    if op == "*" {
      *pos += 1;
      let right = parse_primary(tokens, pos)?;
      left *= right;
    } else if op == "/" {
      *pos += 1;
      let right = parse_primary(tokens, pos)?;
      if right == 0.0 {
        return None;
      }
      left /= right;
    } else {
      break;
    }
  }
  Some(left)
}

fn parse_add_sub(tokens: &[String], pos: &mut usize) -> Option<f64> {
  let mut left = parse_mul_div(tokens, pos)?;
  while *pos < tokens.len() {
    let op = &tokens[*pos];
    if op == "+" {
      *pos += 1;
      let right = parse_mul_div(tokens, pos)?;
      left += right;
    } else if op == "-" {
      *pos += 1;
      let right = parse_mul_div(tokens, pos)?;
      left -= right;
    } else {
      break;
    }
  }
  Some(left)
}

impl ContentSniffer for CalculatorSniffer {
  fn id(&self) -> &'static str {
    "calculator-sniffer"
  }

  fn name(&self) -> &'static str {
    "Smart Calculator Sniffer"
  }

  fn priority(&self) -> u32 {
    88
  }

  fn supports(&self, input: &SniffInput) -> bool {
    matches!(input, SniffInput::Text(_))
  }

  fn sniff(&self, input: &SniffInput) -> SniffOutput {
    let text = match input {
      SniffInput::Text(t) => t.trim(),
      _ => return SniffOutput::default(),
    };

    if text.len() < 2 || text.len() > 150 {
      return SniffOutput::default();
    }

    // Exclude JSON, URL, base64 data, XML
    if text.starts_with('{')
      || text.starts_with('[')
      || text.starts_with('<')
      || text.starts_with("http://")
      || text.starts_with("https://")
      || text.starts_with("data:")
    {
      return SniffOutput::default();
    }

    // Must contain at least one digit
    if !text.chars().any(|c| c.is_ascii_digit()) {
      return SniffOutput::default();
    }

    // Check for math operators or unit abbreviations attached to digits
    let has_math_op = text.contains('+')
      || text.contains('-')
      || text.contains('*')
      || text.contains('/')
      || text.contains('^')
      || text.contains('%')
      || text.contains('×')
      || text.contains('÷');

    let has_unit = text.contains('万')
      || text.contains('亿')
      || text.contains('千')
      || text.contains('百')
      || text.contains('兆')
      || text.contains("万亿")
      || text.contains('K')
      || text.contains('k')
      || text.contains('M')
      || text.contains('m')
      || text.contains('G')
      || text.contains('g')
      || text.contains('T')
      || text.contains('t')
      || text.contains('P')
      || text.contains('p')
      || text.contains('w')
      || text.contains('W')
      || text.contains('q')
      || text.contains('Q')
      || text.contains('y')
      || text.contains('Y')
      || text.contains('z')
      || text.contains('Z')
      || text.contains('b')
      || text.contains('B');

    let is_pure_num =
      text.chars().all(|c| c.is_ascii_digit() || c == '.') && text.parse::<f64>().is_ok();

    if !has_math_op && !has_unit && !is_pure_num {
      return SniffOutput::default();
    }

    // Exclude date formats like 2024-1-2 or 2026-09-13
    if text.len() <= 12 && text.contains('-') && !has_unit {
      let parts: Vec<&str> = text.split('-').collect();
      if parts.len() == 3
        && parts[0].len() == 4
        && parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[1].chars().all(|c| c.is_ascii_digit())
        && parts[2].chars().all(|c| c.is_ascii_digit())
      {
        return SniffOutput::default();
      }
    }

    // Try evaluating in Rust
    let evaluated = simple_eval(text);
    let (formatted, metadata) = if let Some(val) = evaluated {
      let table = generate_calc_markdown(text, val);
      let mut map = serde_json::Map::new();
      map.insert("result".to_string(), val.into());
      map.insert("exact".to_string(), format_exact(val).into());
      map.insert("en".to_string(), format_english(val).into());
      map.insert("cn".to_string(), format_chinese(val).into());
      (Some(table), map)
    } else {
      (None, serde_json::Map::new())
    };

    let confidence = if has_math_op || has_unit { 0.93 } else { 0.80 };

    SniffOutput {
      matched: true,
      confidence,
      tags: vec!["format-calc".to_string(), "format-math".to_string()],
      preprocessed_text: formatted,
      suggested_tool_id: Some("calculator".to_string()),
      suggested_output_type: Some("markdown".to_string()),
      metadata,
    }
  }
}
