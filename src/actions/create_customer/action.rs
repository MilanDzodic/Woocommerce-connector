use crate::standout::app::types::{AppError, ErrorCode, ActionContext};
use crate::client::ApiClient;
use crate::actions::utils::request_body_without_empty_values;
use serde_json::Value;

/// Get the ApiClient from context
#[allow(dead_code)]
fn client(context: &ActionContext) -> Result<ApiClient, AppError> {
    let connection_data: serde_json::Value =
        serde_json::from_str(&context.connection.serialized_data).map_err(|e| AppError {
            code: ErrorCode::Other,
            message: format!("Invalid connection configuration: {}", e),
        })?;
    ApiClient::new(&connection_data)
}

/// Get the input data from context
#[allow(dead_code)]
fn input_data(context: &ActionContext) -> Result<Value, AppError> {
    serde_json::from_str(&context.serialized_input).map_err(|e| AppError {
        code: ErrorCode::Other,
        message: format!("Invalid input data: {}", e),
    })
}

/// Execute the action
#[allow(dead_code)]
pub fn execute(context: ActionContext) -> Result<Value, AppError> {
  let client = client(&context)?;
  let input_data = input_data(&context)?;

  let id = input_data.get("id").and_then(|v| v.as_i64());

  let request_body = request_body_without_empty_values(&input_data, &["id"])?;

  let (status, response_body) = if let Some(customer_id) = id {
    let endpoint = format!("/customers/{}", customer_id);
    client.put(&endpoint, &request_body)
  } else {
    let endpoint = "/customers";
    client.post(endpoint, &request_body)
  }.map_err(|e| AppError {
    code: ErrorCode::Other,
    message: format!("API-anrop misslyckades: {}", e.message),
  })?;

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

/// Get the input schema for this action
#[allow(dead_code)]
pub fn input_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
    let base_schema = include_str!("base_input_schema.json");
    let schema: serde_json::Value = serde_json::from_str(base_schema).unwrap();

    Ok(schema)
}

/// Get the output schema for this action
#[allow(dead_code)]
pub fn output_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
    let base_schema = include_str!("base_output_schema.json");
    let schema: serde_json::Value = serde_json::from_str(base_schema).unwrap();

    Ok(schema)
}


