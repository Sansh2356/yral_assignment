use std::{fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TodoApiError {
    RequestParamNotParsed(String, String, String),
    ErrorText(String),
    ResponseBodyNotConvertedToJsonStr(String, String),
    InvalidPage(String, u64),
    InvalidLimit(String, u64),
}

impl Display for TodoApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoApiError::InvalidLimit(request, limit) => {
                write!(
                    f,
                    "Invalid limit {:?} provided by the user in {:?}",
                    limit, request
                )
            }
            TodoApiError::InvalidPage(request, page_param) => {
                write!(
                    f,
                    "Invalid page {:?} requested in {:?}",
                    page_param, request
                )
            }
            TodoApiError::ResponseBodyNotConvertedToJsonStr(request_type, error) => {
                write!(
                    f,
                    "{:?} occurred while converting response body to json string in request - {:?}",
                    error, request_type
                )
            }
            TodoApiError::RequestParamNotParsed(request_type, error, param_not_parsed) => {
                write!(
                    f,
                    "An error occurred - {:?} while parsing query param {:?} in the request - {:?}",
                    error, param_not_parsed, request_type
                )
            }
            _ => Ok(()),
        }
    }
}

impl From<serde_json::Error> for TodoApiError {
    fn from(value: serde_json::Error) -> Self {
        TodoApiError::ErrorText(value.to_string())
    }
}
