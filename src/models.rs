use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageRequest {
    pub model: String,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: Value,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub provider_name: String,
    pub model: String,
}

pub fn extract_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|part| match part {
                Value::Object(obj) if obj.get("type").and_then(Value::as_str) == Some("text") => {
                    obj.get("text").and_then(Value::as_str).map(str::to_owned)
                }
                Value::Object(obj)
                    if obj.get("type").and_then(Value::as_str) == Some("tool_result") =>
                {
                    Some(obj.get("content").map(extract_text).unwrap_or_default())
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

pub fn estimate_tokens(req: &MessageRequest) -> u64 {
    let mut chars = req.model.len();
    for msg in &req.messages {
        chars += msg.role.len();
        chars += serde_json::to_string(&msg.content)
            .map(|s| s.len())
            .unwrap_or_default();
    }
    if let Some(system) = &req.system {
        chars += serde_json::to_string(system)
            .map(|s| s.len())
            .unwrap_or_default();
    }
    (chars as u64 / 4).max(1)
}

pub fn anthropic_sse(event: &str, data: Value) -> String {
    format!(
        "event: {event}\ndata: {}\n\n",
        serde_json::to_string(&data).unwrap()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn omits_absent_optional_fields_for_upstream_anthropic_wire() {
        let req = MessageRequest {
            model: "umans-coder".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: json!("hello"),
            }],
            system: None,
            max_tokens: Some(32),
            stream: None,
            tools: None,
            tool_choice: None,
            extra: Default::default(),
        };

        let value = serde_json::to_value(req).unwrap();
        assert!(value.get("system").is_none());
        assert!(value.get("stream").is_none());
        assert!(value.get("tools").is_none());
        assert!(value.get("tool_choice").is_none());
        assert_eq!(value["max_tokens"], 32);
    }
}
