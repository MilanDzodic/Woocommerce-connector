use crate::standout::app::types::{AppError, ErrorCode, ActionContext};
use crate::client::ApiClient;
use crate::actions::utils::request_body_without_empty_values;
use serde_json::Value;

#[allow(dead_code)]
fn client(context: &ActionContext) -> Result<ApiClient, AppError> {
  let connection_data: serde_json::Value =
    serde_json::from_str(&context.connection.serialized_data).map_err(|e| AppError {
      code: ErrorCode::Other,
      message: format!("Invalid connection configuration: {}", e),
    })?;
  ApiClient::new(&connection_data)
}

#[allow(dead_code)]
fn input_data(context: &ActionContext) -> Result<Value, AppError> {
  serde_json::from_str(&context.serialized_input).map_err(|e| AppError {
    code: ErrorCode::Other,
    message: format!("Invalid input data: {}", e),
  })
}

#[allow(dead_code)]
pub fn execute(context: ActionContext) -> Result<Value, AppError> {
  let client = client(&context)?;
  let input_data = input_data(&context)?;

  let on_not_found = input_data.get("on_not_found")
    .and_then(|v| v.as_str())
    .unwrap_or("fail");

  let request_body = request_body_without_empty_values(&input_data, &["customerId", "on_not_found"])?;

  let endpoint = build_endpoint("/customers/{customerId}", &extract_path_parameters(&input_data)?);

  let (status, response_body) = client.put(&endpoint, &request_body).map_err(|e| AppError {
    code: ErrorCode::Other,
    message: format!(
      "PUT request failed - URL: {}, Error: {}",
      endpoint,
      e.message
    ),
  })?;

  if status == 404 {
    return match on_not_found {
      "continue" => Ok(serde_json::json!({})),
      "exit_level" => Err(AppError {
        code: ErrorCode::CompleteParent,
        message: "Customer not found".to_string(),
      }),
      "exit_execution" => Err(AppError {
        code: ErrorCode::CompleteWorkflow,
        message: "Customer not found".to_string(),
      }),
      _ => Err(AppError {
        code: ErrorCode::Other,
        message: format!("WooCommerce returnerade felkod 404: {}", response_body),
      }),
    };
  }

  if status >= 400 {
    return Err(AppError {
      code: ErrorCode::Other,
      message: format!("WooCommerce returnerade felkod {}: {}", status, response_body),
    });
  }

  let response_json: Value = serde_json::from_str(&response_body).map_err(|e| AppError {
    code: ErrorCode::MalformedResponse,
    message: format!("Misslyckades att tolka JSON-svar: {}", e),
  })?;

  Ok(response_json)
}

#[allow(dead_code)]
pub fn input_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
  let base_schema = include_str!("base_input_schema.json");
  let schema: serde_json::Value = serde_json::from_str(base_schema).unwrap();
  Ok(schema)
}

#[allow(dead_code)]
pub fn output_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
  let base_schema = include_str!("base_output_schema.json");
  let schema: serde_json::Value = serde_json::from_str(base_schema).unwrap();
  Ok(schema)
}

#[allow(dead_code)]
fn extract_path_parameters(input_data: &Value) -> Result<serde_json::Map<String, Value>, AppError> {
  let mut params = serde_json::Map::new();

  let customer_id = input_data.get("customerId")
    .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|i| i.to_string())))
    .ok_or_else(|| AppError {
      code: ErrorCode::Misconfigured,
      message: "customerId parameter is required".to_string(),
    })?;

  params.insert("customerId".to_string(), serde_json::Value::String(customer_id));
  Ok(params)
}

#[allow(dead_code)]
fn build_endpoint(path_template: &str, params: &serde_json::Map<String, Value>) -> String {
  let mut endpoint = path_template.to_string();
  for (key, value) in params {
    if let Some(value_str) = value.as_str() {
      endpoint = endpoint.replace(&format!("{{{}}}", key), value_str);
    }
  }
  endpoint
}
