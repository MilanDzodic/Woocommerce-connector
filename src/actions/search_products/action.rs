#[allow(unused_imports)]
use crate::standout::app::types::{AppError, ErrorCode, ActionContext};
use crate::client::ApiClient;
use serde_json::{json, Value};

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

  let query_params = build_query_parameters(&input_data)?;

  let mut all_products = Vec::new();
  let mut current_page = 1;
  let per_page = 100;

  loop {
    let mut endpoint = format!("/products?page={}&per_page={}", current_page, per_page);
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

    let page_products: Vec<Value> = match serde_json::from_str(&body) {
      Ok(products) => products,
      Err(_) => break,
    };

    let fetched_count = page_products.len();
    all_products.extend(page_products);

    if fetched_count < per_page as usize {
      break;
    }

    current_page += 1;
  }

  Ok(json!({ "items": all_products }))
}

fn build_query_parameters(input_data: &Value) -> Result<String, AppError> {
  let mut query_parts = Vec::new();

  let params = vec![
    "context", "search", "after", "before", "exclude", "include",
    "offset", "order", "orderby", "parent", "parent_exclude", "slug",
    "status", "type", "sku", "featured", "category", "tag",
    "shipping_class", "attribute", "attribute_term", "tax_class",
    "on_sale", "min_price", "max_price", "stock_status"
  ];

  for param in params {
    add_query_parameter(input_data, param, &mut query_parts);
  }

  Ok(query_parts.join("&"))
}

fn add_query_parameter(input_data: &Value, param_name: &str, query_parts: &mut Vec<String>) {
  if let Some(value) = input_data.get(param_name) {
    match value {
      Value::String(s) if !s.is_empty() => {
        query_parts.push(format!("{}={}", param_name, urlencoding::encode(s)));
      }
      Value::Array(arr) if !arr.is_empty() => {
        let values: Vec<String> = arr.iter()
          .filter_map(|v| v.as_str())
          .filter(|s| !s.is_empty())
          .map(|s| s.to_string())
          .collect();

        if !values.is_empty() {
          let joined = values.join(",");
          query_parts.push(format!("{}={}", param_name, urlencoding::encode(&joined)));
        }
      }
      Value::Number(n) => {
        query_parts.push(format!("{}={}", param_name, n));
      }
      Value::Bool(b) => {
        query_parts.push(format!("{}={}", param_name, b));
      }
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
