#[allow(unused_imports)]
use crate::standout::app::types::{AppError, ErrorCode, ActionContext};
use crate::client::ApiClient;
use serde_json::{Value, json};

fn client(context: &ActionContext) -> Result<ApiClient, AppError> {
  let connection_data: serde_json::Value =
    serde_json::from_str(&context.connection.serialized_data).map_err(|e| AppError {
      code: ErrorCode::Other,
      message: format!("Invalid connection configuration: {}", e),
    })?;
  ApiClient::new(&connection_data)
}

fn input_data(context: &ActionContext) -> Result<Value, AppError> {
  serde_json::from_str(&context.serialized_input).map_err(|e| AppError {
    code: ErrorCode::Other,
    message: format!("Invalid input data: {}", e),
  })
}

pub fn execute(context: ActionContext) -> Result<Value, AppError> {
  let client = client(&context)?;
  let input_data = input_data(&context)?;

  let on_not_found = input_data.get("on_not_found")
    .and_then(|v| v.as_str())
    .unwrap_or("fail");

  let endpoint = build_endpoint("/customers/{customerId}", &extract_path_parameters(&input_data)?);
  let (status, body) = client.get(&endpoint)?;

  if status == 404 {
    return match on_not_found {
      "continue" => Ok(Value::Null),
      "exit_level" => Ok(json!({ "status": "exit_level", "data": null })),
      "exit_execution" => Ok(json!({ "status": "exit_execution", "data": null })),
      _ => Err(AppError {
        code: ErrorCode::Other,
        message: format!("Customer not found (404) at {}", endpoint),
      }),
    };
  }

  if status != 200 {
    return Err(AppError {
      code: ErrorCode::Other,
      message: format!("API error: {} - Response: {}", status, body),
    });
  }

  serde_json::from_str(&body).map_err(|e| AppError {
    code: ErrorCode::MalformedResponse,
    message: format!("Invalid JSON response: {}", e),
  })
}

fn extract_path_parameters(input_data: &Value) -> Result<serde_json::Map<String, Value>, AppError> {
  let mut params = serde_json::Map::new();
  let customer_id = input_data.get("customerId")
    .and_then(|v| v.as_str())
    .map(|s| s.to_string())
    .or_else(|| input_data.get("customerId").and_then(|v| v.as_i64()).map(|i| i.to_string()))
    .ok_or_else(|| AppError {
      code: ErrorCode::Misconfigured,
      message: "customerId parameter is required".to_string(),
    })?;
  
  params.insert("customerId".to_string(), Value::String(customer_id));
  Ok(params)
}

fn build_endpoint(path_template: &str, params: &serde_json::Map<String, Value>) -> String {
  let mut endpoint = path_template.to_string();
  for (key, value) in params {
    if let Some(value_str) = value.as_str() {
      // Fix: lade till & framför format! för att lösa Sized-felet
      endpoint = endpoint.replace(&format!("{{{}}}", key), value_str);
    }
  }
  endpoint
}

/// Get the input_schema for this action
#[allow(dead_code)]
pub fn input_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
  let base_schema = include_str!("base_input_schema.json");
  let schema: serde_json::Value = serde_json::from_str(base_schema).map_err(|e| AppError {
    code: ErrorCode::Other,
    message: format!("Failed to parse input schema: {}", e),
  })?;

  Ok(schema)
}

/// Get the output schema for this action
#[allow(dead_code)]
pub fn output_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
  let base_schema = include_str!("base_output_schema.json");
  let schema: serde_json::Value = serde_json::from_str(base_schema).map_err(|e| AppError {
    code: ErrorCode::Other,
    message: format!("Failed to parse output schema: {}", e),
  })?;

  Ok(schema)
}