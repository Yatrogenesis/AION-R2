// src/api/client.rs

use crate::config::Config;
use crate::errors::ServerError;
use anyhow::Result;
use reqwest::{header, Client, Response};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    api_url: String,
}

impl ApiClient {
    pub async fn new(config: &Config) -> Result<Self> {
        let mut headers = header::HeaderMap::new();

        if let Some(api_key) = &config.aion_r_api_key {
            let mut auth_value = header::HeaderValue::from_str(&format!("Bearer {}", api_key))?;
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            api_url: config.aion_r_api_url.clone(),
        })
    }

    async fn handle_response(response: Response) -> Result<Value> {
        let status = response.status();
        if status.is_success() {
            Ok(response.json::<Value>().await?)
        } else {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read error body>".to_string());
            let msg = format!("API request failed with status {}: {}", status, error_body);
            Err(ServerError::ToolError(msg).into())
        }
    }

    pub async fn run_inference(
        &self,
        model: &str,
        prompt: &str,
        params: &Option<Value>,
    ) -> Result<Value> {
        let url = format!("{}/api/v1/infer", self.api_url);
        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "params": params
        });

        let response = self.client.post(&url).json(&body).send().await?;
        Self::handle_response(response).await
    }

    pub async fn data_analysis(&self, data: &Value, ops: &Value) -> Result<Value> {
        let url = format!("{}/api/v1/analyze", self.api_url);
        let body = serde_json::json!({
            "data": data,
            "ops": ops
        });

        let response = self.client.post(&url).json(&body).send().await?;
        Self::handle_response(response).await
    }

    pub async fn list_models(&self) -> Result<Value> {
        // This is an example, the prompt didn't specify a concrete endpoint for this
        let url = format!("{}/api/v1/models", self.api_url);
        let response = self.client.get(&url).send().await?;
        Self::handle_response(response).await
    }
}
