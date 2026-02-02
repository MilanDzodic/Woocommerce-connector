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
  // We receive the status and body.
  // Note: For full pagination support, we might need to look at response headers
  // like 'X-WP-Total' and 'X-WP-TotalPages' in the future.
  let (_status, body) = client.get(&full_endpoint)?;

  // 3. Parse the body string into a serde_json::Value
  let parsed_val: Value = serde_json::from_str(&body).map_err(|e| AppError {
    code: ErrorCode::MalformedResponse,
    message: format!("Failed to parse JSON response: {}", e),
  })?;

  // 4. Extract products array
  let products = parsed_val.as_array().cloned().unwrap_or_default();

  // 5. Handle SKU specific logic (Unique lookup)
  // If a SKU is provided, we expect a result. If empty, we trigger the chosen strategy.
  if input_data.get("sku").and_then(|v| v.as_str()).is_some() {
    if products.is_empty() {
      return handle_not_found(on_not_found);
    }

    // Even if SKU is unique, WooCommerce returns an array.
    // We wrap it in "items" to keep the output schema consistent.
    return Ok(json!({ "items": products }));
  }

  // 6. Default: Return the product list wrapped in "items"
  // For large datasets, the user should provide 'page' and 'per_page' in the input.
  Ok(json!({ "items": products }))
}

/// Helper to handle "Not Found" scenarios
fn handle_not_found(strategy: &str) -> Result<Value, AppError> {
  match strategy {
    "continue" => Ok(json!({ "items": [] })),
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

/// Build query parameters. Uses input values for pagination if provided,
/// otherwise falls back to defaults (Feedback fix).
fn build_query_parameters(input_data: &Value) -> Result<String, AppError> {
  let mut query_parts = Vec::new();

  // 1. Handle Pagination with defaults
  let page = input_data.get("page")
    .and_then(|v| v.as_i64())
    .unwrap_or(1);

  let per_page = input_data.get("per_page")
    .and_then(|v| v.as_i64())
    .unwrap_or(100);

  query_parts.push(format!("page={}", page));
  query_parts.push(format!("per_page={}", per_page));

  // 2. Define other allowed filters
  let params = vec![
    "context", "search", "after", "before", "exclude", "include",
    "offset", "order", "orderby", "parent", "parent_exclude", "slug",
    "status", "type", "sku", "featured", "category", "tag",
    "shipping_class", "attribute", "attribute_term", "tax_class",
    "on_sale", "min_price", "max_price", "stock_status"
  ];

  for param in params {
    // Our updated add_query_parameter already skips empty/null values
    add_query_parameter(input_data, param, &mut query_parts);
  }

  Ok(query_parts.join("&"))
}

/// Add a query parameter if it exists in input_data
fn add_query_parameter(input_data: &Value, param_name: &str, query_parts: &mut Vec<String>) {
  if let Some(value) = input_data.get(param_name) {
    match value {
      // Only add string parameters if they are not an empty string
      Value::String(s) if !s.is_empty() => {
        query_parts.push(format!("{}={}", param_name, urlencoding::encode(s)));
      }
      // Handle arrays (e.g., 'include' or 'exclude' lists)
      // Only add if the array contains at least one non-empty element
      Value::Array(arr) if !arr.is_empty() => {
        let values: Vec<String> = arr.iter()
          .filter_map(|v| v.as_str())
          .filter(|s| !s.is_empty())
          .map(|s| s.to_string())
          .collect();

        if !values.is_empty() {
          // WooCommerce expects comma-separated values for array filters
          let joined = values.join(",");
          query_parts.push(format!("{}={}", param_name, urlencoding::encode(&joined)));
        }
      }
      // Numbers and booleans are always considered valid values
      Value::Number(n) => {
        query_parts.push(format!("{}={}", param_name, n));
      }
      Value::Bool(b) => {
        query_parts.push(format!("{}={}", param_name, b));
      }
      // Ignore Null, empty strings, empty arrays, or objects
      _ => {}
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
