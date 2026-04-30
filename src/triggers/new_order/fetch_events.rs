use crate::client::ApiClient;
use crate::standout::app::types::{
    AppError, ErrorCode, TriggerContext, TriggerEvent, TriggerResponse,
};
use serde_json::Value;

#[allow(dead_code)]
fn client(context: &TriggerContext) -> Result<ApiClient, AppError> {
    let connection_data: Value = serde_json::from_str(&context.connection.serialized_data)
        .map_err(|e| AppError {
            code: ErrorCode::Other,
            message: format!("Invalid connection configuration: {}", e),
        })?;
    ApiClient::new(&connection_data)
}

#[allow(dead_code)]
fn input_data(context: &TriggerContext) -> Result<Value, AppError> {
    serde_json::from_str(&context.serialized_input).map_err(|e| AppError {
        code: ErrorCode::Other,
        message: format!("Invalid input data: {}", e),
    })
}

#[allow(dead_code)]
fn store_data(context: &TriggerContext) -> Result<Value, AppError> {
    if context.store.is_empty() {
        Ok(serde_json::json!({}))
    } else {
        serde_json::from_str(&context.store).map_err(|e| AppError {
            code: ErrorCode::Other,
            message: format!("Failed to parse store: {}", e),
        })
    }
}

pub fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, AppError> {
    let api_client = client(&context)?;
    let store_data = store_data(&context)?;

    let endpoint = "/orders".to_string();
    let query_params = build_query_parameters(&store_data)?;
    let full_endpoint = if query_params.is_empty() {
        endpoint
    } else {
        format!("{}?{}", endpoint, query_params)
    };

    let (status, body) = api_client.get(&full_endpoint)?;

    if status >= 400 {
        return Err(AppError {
            code: ErrorCode::Other,
            message: format!("API error: {} - {}", status, body),
        });
    }

    let response_data: Value = serde_json::from_str(&body).map_err(|e| AppError {
        code: ErrorCode::MalformedResponse,
        message: format!("Invalid JSON response: {}", e),
    })?;

    let items = response_data.as_array().cloned().unwrap_or_default();
    let mut events = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let event_data = serde_json::json!({
            "item": item
        });

        let item_id = item
            .get("id")
            .and_then(|id| {
                id.as_i64()
                    .map(|i| i.to_string())
                    .or_else(|| id.as_str().map(|s| s.to_string()))
            })
            .unwrap_or_else(|| index.to_string());

        let event = TriggerEvent {
            id: item_id,
            serialized_data: serde_json::to_string(&event_data).unwrap_or_default(),
        };
        events.push(event);
    }

    Ok(TriggerResponse {
        events,
        store: context.store,
    })
}

#[allow(dead_code)]
pub fn input_schema(_context: &TriggerContext) -> Result<serde_json::Value, AppError> {
    let base_schema = include_str!("input_schema.json");
    let schema: serde_json::Value = serde_json::from_str(base_schema).unwrap();
    Ok(schema)
}

#[allow(dead_code)]
pub fn output_schema(_context: &TriggerContext) -> Result<serde_json::Value, AppError> {
    let base_schema = include_str!("output_schema.json");
    let schema: serde_json::Value = serde_json::from_str(base_schema).unwrap();
    Ok(schema)
}

#[allow(dead_code)]
fn add_query_parameter(store_data: &Value, param_name: &str, query_parts: &mut Vec<String>) {
    let _ = store_data.get(param_name).map(|value| {
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
fn build_query_parameters(store_data: &Value) -> Result<String, AppError> {
    let mut query_parts: Vec<String> = Vec::new();
    let params = vec![
        "context",
        "page",
        "per_page",
        "search",
        "after",
        "before",
        "modified_after",
        "modified_before",
        "dates_are_gmt",
        "exclude",
        "include",
        "offset",
        "order",
        "orderby",
        "parent",
        "parent_exclude",
        "status",
        "customer",
        "product",
        "dp",
    ];
    for param in params {
        add_query_parameter(store_data, param, &mut query_parts);
    }
    Ok(query_parts.join("&"))
}

#[allow(dead_code)]
fn extract_path_parameters(
    _store_data: &Value,
) -> Result<serde_json::Map<String, Value>, AppError> {
    Ok(serde_json::Map::new())
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
