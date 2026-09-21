use crate::models::EvaluationModelConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceQuestion {
  #[serde(rename = "type")]
  pub question_type: String,
  pub instructions: String,
  pub criteria: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceRequest {
  pub state: String,
  pub model: String,
  pub questions: HashMap<String, ChoiceQuestion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceAnswer {
  #[serde(rename = "type")]
  pub answer_type: Option<String>,
  pub choice: String,
  #[serde(default)]
  pub probabilities: HashMap<String, f32>,
  #[serde(default)]
  pub confidence: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceResponse {
  #[serde(default)]
  pub model: String,
  pub answers: HashMap<String, ChoiceAnswer>,
}

#[derive(Debug, Clone)]
pub struct JevChoiceResult {
  pub choice: String,
  pub confidence: f32,
  pub probabilities: HashMap<String, f32>,
}

pub struct JevSniffer {
  client: reqwest::Client,
}

impl Default for JevSniffer {
  fn default() -> Self {
    let client = reqwest::Client::builder()
      .timeout(Duration::from_secs(5))
      .build()
      .unwrap_or_else(|_| reqwest::Client::new());
    Self { client }
  }
}

impl JevSniffer {
  pub fn new() -> Self {
    Self::default()
  }

  /// 规范化 endpoint 地址，若未包含 /systemone 则补齐
  pub fn resolve_endpoint(base_url: &str) -> String {
    let mut url = base_url.trim().to_string();
    while url.ends_with('/') {
      url.pop();
    }
    if !url.ends_with("/systemone") {
      url.push_str("/systemone");
    }
    url
  }

  /// 构建并发送 Choice 请求
  pub async fn evaluate(
    &self,
    text: &str,
    config: &EvaluationModelConfig,
    tools: &[(String, String)],
  ) -> Result<JevChoiceResult, String> {
    if !config.is_enabled() {
      return Err("Evaluation model is not configured or disabled".to_string());
    }

    if tools.is_empty() {
      return Err("No candidate tools provided".to_string());
    }

    let mut criteria = HashMap::new();
    for (id, desc) in tools {
      criteria.insert(id.clone(), desc.clone());
    }

    let mut questions = HashMap::new();
    questions.insert(
      "tool_choice".to_string(),
      ChoiceQuestion {
        question_type: "choice".to_string(),
        instructions: "根据输入文本的内容和意图，从候选工具列表中选择最适合处理该输入的工具。"
          .to_string(),
        criteria,
      },
    );

    let request_body = ChoiceRequest {
      state: text.to_string(),
      model: if config.model.trim().is_empty() {
        "jev-latest".to_string()
      } else {
        config.model.trim().to_string()
      },
      questions,
    };

    let endpoint = Self::resolve_endpoint(&config.base_url);

    let response = self
      .client
      .post(&endpoint)
      .header("Authorization", format!("Bearer {}", config.api_key.trim()))
      .header("Content-Type", "application/json")
      .json(&request_body)
      .send()
      .await
      .map_err(|e| format!("Jev request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
      let err_text = response
        .text()
        .await
        .unwrap_or_else(|_| "Failed to read response body".to_string());
      return Err(format!("Jev API HTTP {} error: {}", status, err_text));
    }

    let res_data = response
      .json::<ChoiceResponse>()
      .await
      .map_err(|e| format!("Failed to parse Jev response: {}", e))?;

    let answer = res_data
      .answers
      .get("tool_choice")
      .ok_or_else(|| "Missing 'tool_choice' answer in response".to_string())?;

    // 验证 choice 是否属于候选工具
    if !tools.iter().any(|(id, _)| id == &answer.choice) {
      return Err(format!(
        "Returned choice '{}' is not in candidate list",
        answer.choice
      ));
    }

    Ok(JevChoiceResult {
      choice: answer.choice.clone(),
      confidence: answer.confidence,
      probabilities: answer.probabilities.clone(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_resolve_endpoint() {
    assert_eq!(
      JevSniffer::resolve_endpoint("https://api.typesafe.ai/v1"),
      "https://api.typesafe.ai/v1/systemone"
    );
    assert_eq!(
      JevSniffer::resolve_endpoint("https://api.typesafe.ai/v1/"),
      "https://api.typesafe.ai/v1/systemone"
    );
    assert_eq!(
      JevSniffer::resolve_endpoint("https://api.typesafe.ai/v1/systemone"),
      "https://api.typesafe.ai/v1/systemone"
    );
    assert_eq!(
      JevSniffer::resolve_endpoint("https://api.typesafe.ai/v1/systemone/"),
      "https://api.typesafe.ai/v1/systemone"
    );
  }

  #[test]
  fn test_choice_request_serialization() {
    let mut criteria = HashMap::new();
    criteria.insert("json-formatter".to_string(), "Format JSON".to_string());
    criteria.insert("calculator".to_string(), "Calc expressions".to_string());

    let mut questions = HashMap::new();
    questions.insert(
      "tool_choice".to_string(),
      ChoiceQuestion {
        question_type: "choice".to_string(),
        instructions: "Pick one".to_string(),
        criteria,
      },
    );

    let req = ChoiceRequest {
      state: "1 + 2".to_string(),
      model: "jev-latest".to_string(),
      questions,
    };

    let json_val = serde_json::to_value(&req).unwrap();
    assert_eq!(json_val["model"], "jev-latest");
    assert_eq!(json_val["state"], "1 + 2");
    assert_eq!(json_val["questions"]["tool_choice"]["type"], "choice");
    assert_eq!(
      json_val["questions"]["tool_choice"]["criteria"]["calculator"],
      "Calc expressions"
    );
  }

  #[test]
  fn test_choice_response_deserialization() {
    let raw = r#"{
      "model": "jev-1.13.0",
      "answers": {
        "tool_choice": {
          "type": "choice",
          "choice": "calculator",
          "probabilities": {
            "calculator": 0.92,
            "json-formatter": 0.08
          },
          "confidence": 0.89
        }
      },
      "usage": { "input_tokens": 300, "output_tokens": 25 }
    }"#;

    let res: ChoiceResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(res.model, "jev-1.13.0");
    let ans = res.answers.get("tool_choice").unwrap();
    assert_eq!(ans.choice, "calculator");
    assert_eq!(ans.confidence, 0.89);
    assert_eq!(ans.probabilities.get("calculator").copied(), Some(0.92));
  }
}
