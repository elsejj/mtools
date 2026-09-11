use crate::sniffer::{ContentSniffer, SniffInput, SniffOutput};
use regex::Regex;

pub struct DynamicRegexSniffer {
    id: String,
    name: String,
    regex: Regex,
    target_tool_id: String,
    priority_val: u32,
}

impl DynamicRegexSniffer {
    pub fn new(id: String, name: String, pattern: &str, target_tool_id: String, priority_val: u32) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        Ok(Self {
            id,
            name,
            regex,
            target_tool_id,
            priority_val,
        })
    }
}

impl ContentSniffer for DynamicRegexSniffer {
    fn id(&self) -> &'static str {
        // 使用泄露的静态生命周期字符串或统一标识
        "dynamic-regex-sniffer"
    }

    fn name(&self) -> &'static str {
        "User Defined Dynamic Regex Sniffer"
    }

    fn priority(&self) -> u32 {
        self.priority_val
    }

    fn supports(&self, input: &SniffInput) -> bool {
        matches!(input, SniffInput::Text(_))
    }

    fn sniff(&self, input: &SniffInput) -> SniffOutput {
        let text = match input {
            SniffInput::Text(t) => t.trim(),
            _ => return SniffOutput::default(),
        };

        if self.regex.is_match(text) {
            let mut metadata = serde_json::Map::new();
            metadata.insert("matchedRuleId".to_string(), self.id.clone().into());
            metadata.insert("matchedRuleName".to_string(), self.name.clone().into());

            SniffOutput {
                matched: true,
                confidence: 0.85,
                tags: vec![format!("custom-rule-{}", self.id)],
                preprocessed_text: None,
                suggested_tool_id: Some(self.target_tool_id.clone()),
                suggested_output_type: Some("text".to_string()),
                metadata,
            }
        } else {
            SniffOutput::default()
        }
    }
}

