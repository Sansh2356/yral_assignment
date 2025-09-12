use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TodoError {
    NotFound(u64),

    InvalidInput(String),

    Unexpected(String),
}

impl Display for TodoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoError::InvalidInput(input) => {
                write!(f, "")
            }
            TodoError::NotFound(id) => {
                write!(f, "")
            }
            TodoError::Unexpected(input) => {
                write!(f, "")
            }
            _ => Ok(()),
        }
    }
}
