// src/tools/inference.rs

use crate::api::client::ApiClient;
use crate::errors::ServerError;
use anyhow::Result;
use serde_json::Value;

/// Tool: run_inference
/// Runs AI inference by calling the backend AION-R API.
pub async fn run_inference(api_client: &ApiClient, inputs: &Value) -> Result<Value> {
    let model = inputs["model"]
        .as_str()
        .ok_or_else(|| ServerError::InvalidParameters {
            method: "run_inference".to_string(),
            details: "Missing or invalid 'model' field".to_string(),
        })?;

    let prompt = inputs["prompt"]
        .as_str()
        .ok_or_else(|| ServerError::InvalidParameters {
            method: "run_inference".to_string(),
            details: "Missing or invalid 'prompt' field".to_string(),
        })?;

    let params = inputs.get("params");

    tracing::info!(model = model, "Executing run_inference tool");

    let result = api_client
        .run_inference(model, prompt, &params.cloned())
        .await?;

    Ok(result)
}
