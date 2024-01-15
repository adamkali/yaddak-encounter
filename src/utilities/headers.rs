use axum::http;
use hyper::StatusCode;

use crate::models::errors::{YaddakError, SResult};

pub fn auth_handler(headers: http::HeaderMap) -> SResult<String> {
    let authorization: String;
    if let Some(header_value) = headers.get("Authorization") {
        authorization = header_value.to_str().unwrap_or("").to_string();
        match authorization.as_str().rsplit(' ').next() {
            Some(header) => return Ok(header.to_string()),
            None => return Err(YaddakError::authorize_error("Authorization Header is empty".to_string()))
        }

    } else {
        Err(YaddakError::authorize_error("No Authorization Header".to_string()))
    }
}
