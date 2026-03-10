use crate::standout::app::types::{AppError, ErrorCode, ActionContext};
use crate::client::ApiClient;
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
    eprintln!("DEBUG: lib.rs received action_id: {}", context.action_id);

    eprintln!("DEBUG: Entering execute_action");

    let client = client(&context)?;

    // 1. Laga felet med input_data: Vi måste parsa serialized_input till en Value::Object
    let input_value: Value = serde_json::from_str(&context.serialized_input).map_err(|e| AppError {
        code: ErrorCode::Other,
        message: format!("Kunde inte parsa input JSON: {}", e),
    })?;

    let input_map = input_value.as_object().ok_or_else(|| AppError {
        code: ErrorCode::Other,
        message: "Input data must be a JSON object".to_string(),
    })?;

    eprintln!("DEBUG: Input keys: {:?}", input_map.keys());

    // 2. Laga "type annotations needed": Vi hjälper kompilatorn med i64-typerna
    let product_id = input_map.get("productId").and_then(|v| {
        v.as_i64()
            .or_else(|| {
                v.as_str().and_then(|s| s.parse::<i64>().ok())
            })
    });

    // 3. Skapa body för API-anropet
    let mut body_map = input_map.clone();
    body_map.remove("productId");
    body_map.remove("id");
    body_map.remove("on_not_found");
    let request_body = Value::Object(body_map);

    // 4. Utför anropet
    let (status, response_body) = if let Some(id) = product_id {
        eprintln!("DEBUG: Performing PUT to /products/{}", id);
        client.put(&format!("/products/{}", id), &request_body)
    } else {
        eprintln!("DEBUG: Performing POST to /products");
        client.post("/products", &request_body)
    }.map_err(|e| AppError {
        code: ErrorCode::Other,
        message: format!("API request failed: {}", e.message),
    })?;

    eprintln!("DEBUG: API Response: status={}, body={}", status, response_body);

    if status >= 400 {
        return Err(AppError {
            code: ErrorCode::Other,
            message: format!("WooCommerce error {}: {}", status, response_body),
        });
    }

    let response_json: Value = serde_json::from_str(&response_body).map_err(|e| AppError {
        code: ErrorCode::MalformedResponse,
        message: format!("Failed to parse response: {}", e),
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
