// tests/integration_test.rs

use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Helper function to write a JSON-RPC message to the child process
async fn write_rpc_message(
    stdin: &mut (impl AsyncWriteExt + Unpin),
    message: &Value,
) -> Result<()> {
    let msg_str = serde_json::to_string(message)?;
    let msg_len = msg_str.len();
    let rpc_frame = format!("Content-Length: {}\r\n\r\n{}", msg_len, msg_str);
    stdin.write_all(rpc_frame.as_bytes()).await?;
    stdin.flush().await?;
    Ok(())
}

// Helper function to read a JSON-RPC message from the child process
async fn read_rpc_message(stdout: &mut (impl AsyncBufReadExt + Unpin)) -> Result<Option<Value>> {
    let mut buffer = String::new();
    let mut content_length = 0;

    loop {
        buffer.clear();
        if stdout.read_line(&mut buffer).await? == 0 {
            return Ok(None); // EOF
        }
        if buffer.trim().is_empty() {
            break; // End of headers
        }
        if let Some(len_str) = buffer.strip_prefix("Content-Length:") {
            content_length = len_str.trim().parse::<usize>()?;
        }
    }

    if content_length > 0 {
        let mut body_buf = vec![0; content_length];
        stdout.read_exact(&mut body_buf).await?;
        let body_str = String::from_utf8(body_buf)?;
        let json_val: Value = serde_json::from_str(&body_str)?;
        return Ok(Some(json_val));
    }

    Ok(None)
}

// Helper to spawn the server process for testing
async fn spawn_server(mock_server: &MockServer) -> Child {
    // Ensure the binary is built for the integration test
    // Note: `cargo test` builds the main binary in the debug profile.
    let build_status = Command::new("cargo")
        .args(["build", "--bin", "aionr2"])
        .status()
        .await
        .expect("Failed to build aionr2 binary for testing");
    assert!(build_status.success(), "Cargo build failed for testing");

    let binary_name = if cfg!(windows) { "aionr2.exe" } else { "aionr2" };
    let binary_path = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("debug")
        .join(binary_name);

    Command::new(binary_path)
        .env("AION_R_API_URL", &mock_server.uri())
        .env("RUST_LOG", "warn") // Keep test logs clean
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Pipe stderr to check for errors if needed
        .spawn()
        .expect("Failed to spawn aionr2 process")
}

#[tokio::test]
async fn test_initialize_and_tools_list() -> Result<()> {
    let mock_server = MockServer::start().await;
    let mut child = spawn_server(&mock_server).await;
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let mut stdout = BufReader::new(child.stdout.as_mut().expect("Failed to open stdout"));

    // 1. Test initialize
    let init_req = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": { "protocolVersion": "2024-11-05" },
        "id": 1
    });
    write_rpc_message(stdin, &init_req).await?;

    let init_resp = read_rpc_message(&mut stdout).await?.unwrap();
    assert_eq!(init_resp["id"], 1);
    assert_eq!(init_resp["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(init_resp["result"]["server"]["name"], "aionr2");

    // 2. Test tools/list
    let list_req = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 2
    });
    write_rpc_message(stdin, &list_req).await?;

    let list_resp = read_rpc_message(&mut stdout).await?.unwrap();
    assert_eq!(list_resp["id"], 2);
    let tools = list_resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t["name"] == "run_inference"));
    assert!(tools.iter().any(|t| t["name"] == "data_analysis"));

    // Shutdown
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_tool_call_inference() -> Result<()> {
    let mock_server = MockServer::start().await;
    let mut child = spawn_server(&mock_server).await;
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let mut stdout = BufReader::new(child.stdout.as_mut().expect("Failed to open stdout"));

    // Mock the API endpoint
    Mock::given(method("POST"))
        .and(path("/api/v1/infer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "success",
            "inference_id": "inf_123",
            "output": "The meaning of life is 42."
        })))
        .mount(&mock_server)
        .await;

    let call_req = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "run_inference",
            "inputs": {
                "model": "universe-brain-v2",
                "prompt": "What is the meaning of life?"
            }
        },
        "id": 3
    });
    write_rpc_message(stdin, &call_req).await?;

    let call_resp = read_rpc_message(&mut stdout).await?.unwrap();
    assert_eq!(call_resp["id"], 3);
    assert!(
        call_resp["error"].is_null(),
        "RPC call failed: {}",
        call_resp["error"]
    );
    assert_eq!(call_resp["result"]["status"], "success");
    assert_eq!(call_resp["result"]["output"], "The meaning of life is 42.");

    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_tool_call_data_analysis() -> Result<()> {
    let mock_server = MockServer::start().await;
    let mut child = spawn_server(&mock_server).await;
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let mut stdout = BufReader::new(child.stdout.as_mut().expect("Failed to open stdout"));

    // Mock the API endpoint
    Mock::given(method("POST"))
        .and(path("/api/v1/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "completed",
            "analysis_id": "ana_456",
            "results": [
                {"metric": "mean", "value": 15.5},
                {"metric": "std_dev", "value": 3.2}
            ]
        })))
        .mount(&mock_server)
        .await;

    let call_req = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "data_analysis",
            "inputs": {
                "data": [10, 12, 15, 18, 22],
                "ops": ["mean", "std_dev"]
            }
        },
        "id": 4
    });
    write_rpc_message(stdin, &call_req).await?;

    let call_resp = read_rpc_message(&mut stdout).await?.unwrap();
    assert_eq!(call_resp["id"], 4);
    assert!(
        call_resp["error"].is_null(),
        "RPC call failed: {}",
        call_resp["error"]
    );
    assert_eq!(call_resp["result"]["status"], "completed");
    assert_eq!(call_resp["result"]["results"][0]["metric"], "mean");

    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_resources_list_models() -> Result<()> {
    let mock_server = MockServer::start().await;
    let mut child = spawn_server(&mock_server).await;
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let mut stdout = BufReader::new(child.stdout.as_mut().expect("Failed to open stdout"));

    // Mock the API endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": "model-1", "name": "Universe Brain v1"},
            {"id": "model-2", "name": "Universe Brain v2"},
        ])))
        .mount(&mock_server)
        .await;

    let list_req = json!({
        "jsonrpc": "2.0",
        "method": "resources/list",
        "params": {
            "uri": "aion-r://models/catalog"
        },
        "id": 5
    });
    write_rpc_message(stdin, &list_req).await?;

    let list_resp = read_rpc_message(&mut stdout).await?.unwrap();
    assert_eq!(list_resp["id"], 5);
    assert!(
        list_resp["error"].is_null(),
        "RPC call failed: {}",
        list_resp["error"]
    );
    let models = list_resp["result"].as_array().unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[1]["name"], "Universe Brain v2");

    child.kill().await?;
    Ok(())
}
