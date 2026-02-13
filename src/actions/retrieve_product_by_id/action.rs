#[allow(unused_imports)]
use crate::standout::app::types::{AppError, ErrorCode, ActionContext};
use crate::client::ApiClient;
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

  // Make HTTP request
  let endpoint = build_endpoint("/products/{productId}", &extract_path_parameters(&input_data)?);

  let (status, response_body) = client.get(&endpoint)?;

  if status >= 400 {
    return Err(AppError {
      code: ErrorCode::Other,
      message: format!("WooCommerce returnerade felkod {}: {}", status, response_body),
    });
  }

  let response_json: Value = serde_json::from_str(&response_body).map_err(|e| AppError {
    code: ErrorCode::MalformedResponse,
    message: format!("Misslyckades att tolka JSON-svar för produkt: {}", e),
  })?;

  Ok(response_json)
}

/// Get the input_schema for this action
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



/// Extract path parameters from input data
#[allow(dead_code)]
fn extract_path_parameters(input_data: &Value) -> Result<serde_json::Map<String, Value>, AppError> {
    let mut params = serde_json::Map::new();

    let product_id = input_data.get("productId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| input_data.get("productId")
            .and_then(|v| v.as_i64())
            .map(|i| i.to_string()))
        .ok_or_else(|| AppError {
            code: ErrorCode::Misconfigured,
            message: "productId parameter is required".to_string(),
        })?;
    params.insert("productId".to_string(), serde_json::Value::String(product_id));


    Ok(params)
}

/// Build endpoint URL with path parameters
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

