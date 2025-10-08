// src/mcp/types.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;

// JSON-RPC 2.0 Structures

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// MCP Method-specific Parameters and Results

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)] // Not constructed by the server, but part of the spec
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub server: ServerInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToolsListResult {
    pub tools: Vec<ToolDefinition>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub inputs: Value, // JSON Schema for inputs
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToolsCallParams {
    pub name: String,
    pub inputs: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)] // Not constructed by the server
pub struct ToolsCallResult {
    pub outputs: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourcesListParams {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)] // Not constructed by the server
pub struct ResourcesListResult {
    pub resources: Vec<Value>,
}
