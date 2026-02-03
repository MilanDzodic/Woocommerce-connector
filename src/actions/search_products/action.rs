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

    // Determine the strategy to use if no products are found
    let on_not_found = input_data.get("on_not_found")
        .and_then(|v| v.as_str())
        .unwrap_or("fail");

    // 1. Build query parameters from input data
    // Note: 'page' and 'per_page' are no longer taken from input_data to keep the UI clean
    let query_params = build_query_parameters(&input_data)?;

    // 2. Construct the full endpoint with internal pagination
    // We hardcode page=1 and per_page=100 to handle pagination in the background
    let mut full_endpoint = "/products?page=1&per_page=100".to_string();

    if !query_params.is_empty() {
        // If query_params already starts with '&', don't add another one
        if !query_params.starts_with('&') {
            full_endpoint.push('&');
        }
        full_endpoint.push_str(&query_params);
    }

    // 3. Make the HTTP request to WooCommerce API
    let (status, body) = client.get(&full_endpoint)?;

    // Handle explicit 404 errors if the API returns them for empty searches
    if status == 404 {
        return handle_not_found(on_not_found);
    }

    // 4. Parse the response body string into a JSON Value
    let parsed_val: Value = serde_json::from_str(&body).map_err(|e| AppError {
        code: ErrorCode::MalformedResponse,
        message: format!("Failed to parse JSON response: {}", e),
    })?;

    // 5. Extract products array from the parsed response
    let products = parsed_val.as_array().cloned().unwrap_or_default();

    // 6. Handle "Not Found" logic for specific searches
    // If the list is empty and the user was looking for something specific (like a SKU),
    // we trigger the selected 'on_not_found' strategy.
    if products.is_empty() && (input_data.get("sku").is_some() || input_data.get("search").is_some()) {
        return handle_not_found(on_not_found);
    }

    // 7. Return the product list wrapped in an "items" object for schema consistency
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
