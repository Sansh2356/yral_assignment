use std::borrow::Cow;

use ic_http_certification::{HttpResponse,StatusCode};
use serde_json::json;

use crate::{error::TodoApiError, types::HttpRequest};

//Either updating the `POST/PATCH/PUT/DELETE` calls or `GET` calls only
pub fn json_response_get(
    status: u16,
    body: String,
    update_call_or_not: bool,
) -> ic_http_certification::HttpResponse<'static> {
    println!(
        "Creating JSON response with status {} and body: {}",
        status, body
    );
    let byte_body = body.as_bytes();
    HttpResponse::ok(
        Cow::Owned(Vec::from(byte_body)),
        vec![
            ("content-type".to_string(), "application/json".to_string()),
            (
                "strict-transport-security".to_string(),
                "max-age=31536000; includeSubDomains".to_string(),
            ),
            ("x-content-type-options".to_string(), "nosniff".to_string()),
            ("referrer-policy".to_string(), "no-referrer".to_string()),
            (
                "cache-control".to_string(),
                "no-store, max-age=0".to_string(),
            ),
            ("pragma".to_string(), "no-cache".to_string()),
        ],
    )
    .with_status_code(StatusCode::from_u16(status).unwrap())
    .with_upgrade(update_call_or_not)
    .build()
}
pub fn create_not_found_response() -> Result<HttpResponse<'static>, TodoApiError> {
    println!("Creating Not Found response");
    Ok(HttpResponse::not_found(
        Cow::Borrowed(b"Not Found" as &[u8]),
        vec![("Content-Type".to_string(), "text/plain".to_string())],
    )
    .build())
}
//A standard response for an error that has been occurred at the route
pub fn create_error_response(
    request: &HttpRequest,
    error: TodoApiError,
) -> ic_http_certification::HttpResponse<'static> {
    let body = json!({
        "error": error.to_string(),
        "path": request.url,
        "method": request.method,
    });

    let body_str = serde_json::to_string(&body).unwrap_or_else(|_| {
        format!(
            r#"{{"error": "Failed to serialize error response: {}"}}"#,
            error
        )
    });

    json_response_get(500, body_str, false)
}
