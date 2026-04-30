use crate::client::ApiClient;
use crate::standout::app::types::{ActionContext, AppError, ErrorCode};
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

    let mut endpoint = build_endpoint(
        "/products/{productId}",
        &extract_path_parameters(&input_data)?,
    );

    let force = input_data
        .get("force")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    endpoint = format!("{}?force={}", endpoint, force);

    client.delete(&endpoint)
}

#[allow(dead_code)]
pub fn input_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
    let base_schema = include_str!("base_input_schema.json");
    let schema: serde_json::Value = serde_json::from_str(base_schema).unwrap();

    Ok(schema)
}

#[allow(dead_code)]
pub fn output_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
    let base_schema = include_str!("../../schemas/shared/product_base_output_schema.json");
    let schema: serde_json::Value = serde_json::from_str(base_schema).unwrap();

    Ok(schema)
}

#[allow(dead_code)]
fn extract_path_parameters(input_data: &Value) -> Result<serde_json::Map<String, Value>, AppError> {
    let mut params = serde_json::Map::new();

    let product_id = input_data
        .get("productId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            input_data
                .get("productId")
                .and_then(|v| v.as_i64())
                .map(|i| i.to_string())
        })
        .ok_or_else(|| AppError {
            code: ErrorCode::Misconfigured,
            message: "productId parameter is required".to_string(),
        })?;

    params.insert(
        "productId".to_string(),
        serde_json::Value::String(product_id),
    );

    Ok(params)
}

#[allow(dead_code)]
fn build_endpoint(path_template: &str, params: &serde_json::Map<String, Value>) -> String {
    let mut endpoint = path_template.to_string();

    for (key, value) in params {
        let _ = value.as_str().map(|value_str| {
            endpoint = endpoint.replace(&format!("{{{}}}", key), value_str);
        });
    }

    endpoint
}
