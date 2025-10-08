// src/errors.rs

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)] // Allow variants that are not yet constructed
pub enum ServerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API client error: {0}")]
    ApiClient(#[from] reqwest::Error),

    #[error("Invalid JSON-RPC request: {0}")]
    InvalidJsonRpcRequest(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid parameters for method '{method}': {details}")]
    InvalidParameters { method: String, details: String },

    #[error("Internal tool error: {0}")]
    ToolError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
