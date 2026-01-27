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

  // 2. Make the request - We ignore 'status' here to avoid unused variable warning
  // but we still check for general 404/errors inside the client.get Result
  let (_status, body) = client.get(&full_endpoint)?;

  // 3. Parse the body string into a Value
  let parsed_val: Value = serde_json::from_str(&body).map_err(|e| AppError {
    code: ErrorCode::MalformedResponse,
    message: format!("Failed to parse JSON response: {}", e),
  })?;

  // 4. Extract array - removed 'mut' as it's not modified (Feedback fix)
  let products = parsed_val.as_array().cloned().unwrap_or_default();

  // 5. Handle SKU filtering logic
  if let Some(target_sku) = input_data.get("sku").and_then(|v| v.as_str()) {
    // Search for exact SKU match in the list
    let exact_match = products.iter().find(|p| {
      p.get("sku").and_then(|s| s.as_str()) == Some(target_sku)
    });

    match exact_match {
      Some(product) => {
        // Return wrapped in "items" to match schema (Feedback point 1)
        return Ok(json!({ "items": [product] }));
      },
      None => {
        // Trigger handle_not_found if list is empty or no SKU match (Feedback point 3)
        return handle_not_found(on_not_found);
      }
    }
  }

  // 6. Default: Return full list wrapped in "items" object
  Ok(json!({ "items": products }))
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
