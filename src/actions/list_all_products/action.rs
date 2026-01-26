#[allow(unused_imports)]
use crate::standout::app::types::{AppError, ErrorCode, ActionContext};
use crate::client::ApiClient;
use serde_json::{json, Value};

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

#[allow(dead_code)]
pub fn execute(context: ActionContext) -> Result<Value, AppError> {
  let client = client(&context)?;
  let input_data = input_data(&context)?;

  let on_not_found = input_data.get("on_not_found")
    .and_then(|v| v.as_str())
    .unwrap_or("fail");

  // 1. Build endpoint and query parameters
  let endpoint = "/products".to_string();
  let query_params = build_query_parameters(&input_data)?;
  let full_endpoint = if query_params.is_empty() {
    endpoint
  } else {
    format!("{}?{}", endpoint, query_params)
  };

  // 2. Make the HTTP request
  // In this project, client.get returns Result<(u16, String), AppError>
  let response_result = client.get(&full_endpoint);

  // Handle 404 specifically for SKU searches before unpacking
  if let Err(ref e) = response_result {
    if e.message.contains("404") && input_data.get("sku").is_some() {
      return handle_not_found(on_not_found);
    }
  }

  // Unpack the tuple: (status_code, body_string)
  let (status, body) = response_result?;

  // Additional check if the status is not success
  if status != 200 {
    // If it's a 404 that wasn't caught by the error check above
    if status == 404 && input_data.get("sku").is_some() {
      return handle_not_found(on_not_found);
    }

    return Err(AppError {
      code: ErrorCode::Other,
      message: format!("WooCommerce API error: {} - {}", status, body),
    });
  }

  // 3. Parse the body string into a serde_json::Value
  let parsed_response: Value = serde_json::from_str(&body).map_err(|e| AppError {
    code: ErrorCode::MalformedResponse,
    message: format!("Failed to parse JSON response: {}", e),
  })?;

  // 4. Handle SKU filtering logic
  if let Some(target_sku) = input_data.get("sku").and_then(|v| v.as_str()) {
    // Now .as_array() will work because parsed_response is a Value
    let products = parsed_response.as_array().ok_or_else(|| AppError {
      code: ErrorCode::MalformedResponse,
      message: "Expected an array of products from WooCommerce".to_string(),
    })?;

    // Manual filtering to find the exact SKU match
    let exact_match = products.iter().find(|p| {
      p.get("sku").and_then(|s| s.as_str()) == Some(target_sku)
    });

    match exact_match {
      Some(product) => return Ok(product.clone()),
      None => return handle_not_found(on_not_found),
    }
  }

  // 5. Default: Return the parsed response (the list)
  Ok(parsed_response)
}

/// Helper to handle "Not Found" scenarios
fn handle_not_found(strategy: &str) -> Result<Value, AppError> {
  match strategy {
    "continue" => Ok(json!({})),
    "exit_level" => Err(AppError {
      code: ErrorCode::CompleteParent,
      message: "Product with specified SKU not found.".to_string(),
    }),
    "exit_execution" => Err(AppError {
      code: ErrorCode::CompleteWorkflow,
      message: "Product with specified SKU not found.".to_string(),
    }),
    _ => Err(AppError {
      code: ErrorCode::Other,
      message: "No product found matching the provided SKU".to_string(),
    }),
  }
}

/// Build query parameters from input data
fn build_query_parameters(input_data: &Value) -> Result<String, AppError> {
  let mut query_parts = Vec::new();

  let params = vec![
    "context", "page", "per_page", "search", "after", "before",
    "modified_after", "modified_before", "dates_are_gmt", "exclude",
    "include", "offset", "order", "orderby", "parent",
    "parent_exclude", "slug", "status", "include_status",
    "exclude_status", "type", "include_types", "exclude_types",
    "sku", "featured", "category", "tag", "shipping_class",
    "attribute", "attribute_term", "tax_class", "on_sale",
    "min_price", "max_price", "stock_status", "virtual", "downloadable"
  ];

  for param in params {
    add_query_parameter(input_data, param, &mut query_parts);
  }

  Ok(query_parts.join("&"))
}

/// Add a query parameter if it exists in input_data
fn add_query_parameter(input_data: &Value, param_name: &str, query_parts: &mut Vec<String>) {
  if let Some(value) = input_data.get(param_name) {
    if let Some(str_val) = value.as_str() {
      if !str_val.is_empty() {
        query_parts.push(format!("{}={}", param_name, urlencoding::encode(str_val)));
      }
    } else if let Some(int_val) = value.as_i64() {
      query_parts.push(format!("{}={}", param_name, int_val));
    } else if let Some(bool_val) = value.as_bool() {
      query_parts.push(format!("{}={}", param_name, bool_val));
    }
  }
}

#[allow(dead_code)]
pub fn input_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
  let base_schema = include_str!("base_input_schema.json");
  Ok(serde_json::from_str(base_schema).unwrap())
}

#[allow(dead_code)]
pub fn output_schema(_context: &ActionContext) -> Result<serde_json::Value, AppError> {
  let base_schema = include_str!("base_output_schema.json");
  Ok(serde_json::from_str(base_schema).unwrap())
}
