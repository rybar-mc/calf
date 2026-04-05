use crate::models::ErrorResponse;
use worker::{Response, Result};

pub fn json_error(message: &str, status: u16) -> Result<Response> {
    let body = ErrorResponse {
        error: message.to_string(),
        status,
    };

    let response = Response::from_json(&body)?.with_status(status);
    Ok(response)
}
