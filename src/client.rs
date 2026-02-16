use crate::standout::app::{
  http::{Method, RequestBuilder},
  types::{AppError, ErrorCode},
};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ApiClient {
  pub base_url: String,
  pub headers: HashMap<String, String>,
}

impl ApiClient {
  pub fn new(connection_data: &Value) -> Result<Self, AppError> {
    let base_url = connection_data
      .get("base_url")
      .and_then(|v| v.as_str())
      .ok_or_else(|| AppError {
        code: ErrorCode::Misconfigured,
        message: "base_url not found in connection data".to_string(),
      })?
      .to_string();

    let headers_obj = connection_data
      .get("headers")
      .and_then(|v| v.as_object())
      .ok_or_else(|| AppError {
        code: ErrorCode::Misconfigured,
        message: "Headers not found in connection data".to_string(),
      })?;

    let mut headers = HashMap::new();
    for (key, value) in headers_obj {
      if let Some(header_value) = value.as_str() {
        headers.insert(key.clone(), header_value.to_string());
      }
    }

    Ok(ApiClient { base_url, headers })
  }

  pub fn get(&self, endpoint: &str) -> Result<(u16, String), AppError> {
    let url = format!("{}{}", self.base_url, endpoint);
    let mut request_builder = RequestBuilder::new().method(Method::Get).url(&url);

    for (key, value) in &self.headers {
      request_builder = request_builder.header(key, value);
    }

    let response = request_builder.send().map_err(|_err| AppError {
      code: ErrorCode::Other,
      message: "Request failed".to_string(),
    })?;

    Ok((response.status, response.body))
  }

pub fn post(&self, endpoint: &str, body: &serde_json::Value) -> Result<(u16, String), AppError> {
  let clean_base = self.base_url.trim_end_matches('/');
  let clean_endpoint = endpoint.trim_start_matches('/');
  let url = format!("{}/{}", clean_base, clean_endpoint);

  let mut request_builder = RequestBuilder::new()
    .method(Method::Post)
    .url(&url);

  for (key, value) in &self.headers {
    request_builder = request_builder.header(key, value);
  }

  let body_str = serde_json::to_string(body).map_err(|e| AppError {
    code: ErrorCode::Other,
    message: format!("Kunde inte serialisera request body: {}", e),
  })?;

  request_builder = request_builder.body(&body_str);

  let response = request_builder.send().map_err(|_err| AppError {
    code: ErrorCode::Other,
    message: format!("POST-anrop misslyckades till URL: {}", url),
  })?;

  Ok((response.status, response.body))
}

  pub fn put(&self, endpoint: &str, body: &serde_json::Value) -> Result<(u16, String), AppError> {
    let url = format!("{}{}", self.base_url, endpoint);
    let mut request_builder = RequestBuilder::new().method(Method::Put).url(&url);

    for (key, value) in &self.headers {
      request_builder = request_builder.header(key, value);
    }

    let body_str = serde_json::to_string(body).map_err(|e| AppError {
      code: ErrorCode::Other,
      message: format!("Kunde inte serialisera request body: {}", e),
    })?;

    request_builder = request_builder.body(&body_str);

    let response = request_builder.send().map_err(|_err| AppError {
      code: ErrorCode::Other,
      message: "PUT-anrop misslyckades".to_string(),
    })?;

    Ok((response.status, response.body))
  }
}
