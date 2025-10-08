// src/tools/analytics.rs

use crate::api::client::ApiClient;
use crate::errors::ServerError;
use anyhow::Result;
use serde_json::Value;

/// Tool: data_analysis
/// Runs data analysis by calling the backend AION-R API.
pub async fn data_analysis(api_client: &ApiClient, inputs: &Value) -> Result<Value> {
    let data = inputs
        .get("data")
        .ok_or_else(|| ServerError::InvalidParameters {
            method: "data_analysis".to_string(),
            details: "Missing 'data' field".to_string(),
        })?;

    let ops = inputs
        .get("ops")
        .ok_or_else(|| ServerError::InvalidParameters {
            method: "data_analysis".to_string(),
            details: "Missing 'ops' field".to_string(),
        })?;

    tracing::info!("Executing data_analysis tool");

    let result = api_client.data_analysis(data, ops).await?;

    Ok(result)
}
