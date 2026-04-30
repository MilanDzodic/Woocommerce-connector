use crate::standout::app::types::AppError;
use serde_json::{Value, json};

#[allow(dead_code)]
pub fn clean_empty_values(value: &Value) -> Value {
    match value {
        Value::Null => Value::Null,
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => Value::Number(n.clone()),
        Value::String(s) => {
            if s.is_empty() {
                Value::Null
            } else {
                Value::String(s.clone())
            }
        }
        Value::Array(arr) => {
            let cleaned: Vec<Value> = arr
                .iter()
                .map(clean_empty_values)
                .filter(|v| !v.is_null())
                .collect();

            if cleaned.is_empty() {
                Value::Null
            } else {
                Value::Array(cleaned)
            }
        }
        Value::Object(obj) => {
            let cleaned: serde_json::Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), clean_empty_values(v)))
                .filter(|(_, v)| !v.is_null())
                .collect();

            if cleaned.is_empty() {
                Value::Null
            } else {
                Value::Object(cleaned)
            }
        }
    }
}

#[allow(dead_code)]
pub fn request_body_without_empty_values(
    input_data: &Value,
    path_parameters: &[&str],
) -> Result<Value, AppError> {
    let mut body = serde_json::Map::new();

    for (key, value) in input_data.as_object().unwrap_or(&serde_json::Map::new()) {
        if !path_parameters.contains(&key.as_str()) {
            let cleaned_value = clean_empty_values(value);

            if !cleaned_value.is_null() {
                body.insert(key.clone(), cleaned_value);
            }
        }
    }

    Ok(json!(body))
}
