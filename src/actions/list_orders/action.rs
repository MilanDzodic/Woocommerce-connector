use crate::client::ApiClient;
use crate::standout::app::types::{ActionContext, AppError, ErrorCode};
use serde_json::Value;
use urlencoding::encode;

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
    let input = input_data(&context)?;
    let mut endpoint = String::from("/orders");

    if let Some(obj) = input.as_object() {
        let mut query_params = Vec::new();
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                query_params.push(format!("{}={}", k, encode(s)));
            } else if let Some(n) = v.as_number() {
                query_params.push(format!("{}={}", k, n));
            } else if let Some(b) = v.as_bool() {
                query_params.push(format!("{}={}", k, b));
            }
        }
        if !query_params.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&query_params.join("&"));
        }
    }

    let (status, response_body) = client.get(&endpoint)?;

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
