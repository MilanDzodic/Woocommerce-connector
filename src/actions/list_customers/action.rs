use crate::client::ApiClient;
use crate::standout::app::types::{ActionContext, AppError, ErrorCode};
use serde_json::{Value, json};

#[allow(dead_code)]
fn client(context: &ActionContext) -> Result<ApiClient, AppError> {
    let connection_data: Value = serde_json::from_str(&context.connection.serialized_data)
        .map_err(|e| AppError {
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

    let query_params = build_query_parameters(&input_data)?;

    let mut all_customers = Vec::new();
    let mut current_page = 1;
    let per_page = 100;

    loop {
        let mut endpoint = format!("/customers?page={}&per_page={}", current_page, per_page);
        if !query_params.is_empty() {
            endpoint.push('&');
            endpoint.push_str(&query_params);
        }

        let (status, body) = client.get(&endpoint)?;

        if status >= 400 {
            return Err(AppError {
                code: ErrorCode::Other,
                message: format!("WooCommerce returnerade felkod {}: {}", status, body),
            });
        }

        let page_customers: Vec<Value> = match serde_json::from_str(&body) {
            Ok(customers) => customers,
            Err(_) => break,
        };

        let fetched_count = page_customers.len();
        all_customers.extend(page_customers);

        if fetched_count < per_page as usize {
            break;
        }

        current_page += 1;
    }

    Ok(json!({ "items": all_customers }))
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
fn add_query_parameter(input_data: &Value, param_name: &str, query_parts: &mut Vec<String>) {
    let _ = input_data.get(param_name).map(|value| {
        if let Some(str_val) = value.as_str().filter(|s| !s.is_empty()) {
            query_parts.push(format!("{}={}", param_name, urlencoding::encode(str_val)));
        } else if let Some(int_val) = value.as_i64() {
            query_parts.push(format!("{}={}", param_name, int_val));
        } else if let Some(bool_val) = value.as_bool() {
            query_parts.push(format!("{}={}", param_name, bool_val));
        }
    });
}

#[allow(dead_code)]
fn build_query_parameters(input_data: &Value) -> Result<String, AppError> {
    let mut query_parts = Vec::new();
    let params = vec![
        "context", "search", "exclude", "include", "offset", "order", "orderby", "email", "role",
    ];
    for param in params {
        add_query_parameter(input_data, param, &mut query_parts);
    }
    Ok(query_parts.join("&"))
}
