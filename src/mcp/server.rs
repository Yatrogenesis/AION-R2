// src/mcp/server.rs

use crate::{
    api::client::ApiClient,
    config::Config,
    errors::ServerError,
    mcp::types::{
        InitializeResult, JsonRpcRequest, JsonRpcResponse, ResourcesListParams, ServerInfo,
        ToolDefinition, ToolsCallParams, ToolsListResult,
    },
    tools, util,
};
use anyhow::Result;
use serde_json::{json, Value};
use tokio::io::BufReader;

const MCP_VERSION: &str = "2024-11-05";

pub struct McpServer {
    api_client: ApiClient,
}

impl McpServer {
    pub async fn new(config: &Config) -> Result<Self> {
        let api_client = ApiClient::new(config).await?;
        Ok(Self { api_client })
    }

    pub async fn run(&self) -> Result<()> {
        let mut stdin = BufReader::new(tokio::io::stdin());
        let mut stdout = tokio::io::stdout();

        loop {
            match util::read_message(&mut stdin).await? {
                Some(message_str) => {
                    let request: JsonRpcRequest = match serde_json::from_str(&message_str) {
                        Ok(req) => req,
                        Err(e) => {
                            let err_resp = self.create_error_response(
                                None,
                                -32700,
                                format!("Parse error: {}", e),
                            );
                            util::write_message(&mut stdout, &err_resp).await?;
                            continue;
                        }
                    };

                    let request_id = request.id.clone();
                    let response = self.dispatch(request).await;

                    if let Some(_id) = request_id {
                        // It's a request, not a notification
                        util::write_message(&mut stdout, &serde_json::to_string(&response)?)
                            .await?;
                    } // else it was a notification, no response needed
                }
                None => {
                    // Stdin closed, exit loop
                    tracing::info!("Stdin closed, shutting down.");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn dispatch(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let request_id = request.id.clone().unwrap_or(Value::Null);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params).await,
            "resources/list" => self.handle_resources_list(request.params).await,
            _ => Err(ServerError::MethodNotFound(request.method).into()),
        };

        match result {
            Ok(res_val) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(res_val),
                error: None,
                id: request_id,
            },
            Err(e) => {
                // Convert anyhow::Error back to our ServerError to get a specific error code
                let server_error = e.downcast_ref::<ServerError>();
                let (code, message) = match server_error {
                    Some(ServerError::InvalidJsonRpcRequest(s)) => (-32600, s.clone()),
                    Some(ServerError::MethodNotFound(s)) => {
                        (-32601, format!("Method not found: {}", s))
                    }
                    Some(ServerError::InvalidParameters { .. }) => (-32602, e.to_string()),
                    Some(ServerError::ToolError(s)) => (-32000, s.clone()),
                    _ => (-32603, e.to_string()), // Generic internal error
                };

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(crate::mcp::types::JsonRpcError {
                        code,
                        message,
                        data: None, // Optionally, you could add more error data here
                    }),
                    id: request_id,
                }
            }
        }
    }

    fn create_error_response(&self, id: Option<Value>, code: i32, message: String) -> String {
        let error_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(crate::mcp::types::JsonRpcError {
                code,
                message,
                data: None,
            }),
            id: id.unwrap_or(Value::Null),
        };
        serde_json::to_string(&error_response).unwrap_or_else(|_| "{\"jsonrpc\": \"2.0\", \"error\": {\"code\": -32603, \"message\": \"Internal error during error serialization\"}, \"id\": null}".to_string())
    }

    async fn handle_initialize(&self, _params: Option<Value>) -> Result<Value> {
        // Note: The current MCP spec for 'initialize' doesn't use any parameters,
        // but we accept them for forward compatibility.
        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            server: ServerInfo {
                name: "aionr2".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        Ok(serde_json::to_value(result)?)
    }

    async fn handle_tools_list(&self) -> Result<Value> {
        let tools = vec![
            ToolDefinition {
                name: "run_inference".to_string(),
                description: "Runs AI inference by calling the backend AION-R API.".to_string(),
                inputs: json!({
                    "type": "object",
                    "properties": {
                        "model": { "type": "string" },
                        "prompt": { "type": "string" },
                        "params": { "type": "object" }
                    },
                    "required": ["model", "prompt"]
                }),
            },
            ToolDefinition {
                name: "data_analysis".to_string(),
                description: "Runs data analysis by calling the backend AION-R API.".to_string(),
                inputs: json!({
                    "type": "object",
                    "properties": {
                        "data": {},
                        "ops": { "type": "array" }
                    },
                    "required": ["data", "ops"]
                }),
            },
        ];
        let result = ToolsListResult { tools };
        Ok(serde_json::to_value(result)?)
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> Result<Value> {
        let params: ToolsCallParams = serde_json::from_value(params.unwrap_or(Value::Null))?;

        match params.name.as_str() {
            "run_inference" => {
                tools::inference::run_inference(&self.api_client, &params.inputs).await
            }
            "data_analysis" => {
                tools::analytics::data_analysis(&self.api_client, &params.inputs).await
            }
            _ => {
                Err(ServerError::MethodNotFound(format!("Tool '{}' not found", params.name)).into())
            }
        }
    }

    async fn handle_resources_list(&self, params: Option<Value>) -> Result<Value> {
        let params: ResourcesListParams = serde_json::from_value(params.unwrap_or(Value::Null))?;
        match params.uri.as_str() {
            "aion-r://models/catalog" => {
                // Example implementation: forward to an API endpoint
                self.api_client.list_models().await
            }
            _ => Ok(json!([])), // Return empty list for unknown resources
        }
    }
}
