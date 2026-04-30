use crate::actions::utils::request_body_without_empty_values;
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
    let input_value = input_data(&context)?;

    let mut body_map = input_value
        .as_object()
        .ok_or_else(|| AppError {
            code: ErrorCode::Other,
            message: "Input data must be a JSON object".to_string(),
        })?
        .clone();

    let product_id = body_map
        .remove("productId")
        .and_then(|v| {
            v.as_str()
                .map(|s| s.to_string())
                .or_else(|| v.as_i64().map(|n| n.to_string()))
        })
        .ok_or_else(|| AppError {
            code: ErrorCode::Other,
            message: "Missing required parameter: productId".to_string(),
        })?;

    let request_body = request_body_without_empty_values(&Value::Object(body_map), &[])?;
    let endpoint = format!("/products/{}", product_id);

    let (status, response_body) = client.put(&endpoint, &request_body)?;

    if status >= 400 {
        return Err(AppError {
            code: ErrorCode::Other,
            message: format!(
                "WooCommerce returnerade felkod {}: {}",
                status, response_body
            ),
        });
    }

    let response_json: Value = serde_json::from_str(&response_body).map_err(|e| AppError {
        code: ErrorCode::MalformedResponse,
        message: format!("Kunde inte tolka svar från API: {}", e),
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
