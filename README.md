# aionr2 - AION-R MCP Server

`aionr2` is a production-ready, high-performance server that implements the Mission Control Protocol (MCP) for interacting with an AION-R backend. It exposes AI inference and data analysis capabilities through a standard JSON-RPC 2.0 interface over stdio.

This server is designed to be run as a background process by a controlling application, which communicates with it by writing JSON-RPC requests to its standard input and reading responses from its standard output.

## Features

- **JSON-RPC 2.0 Interface:** Communication via stdio using the JSON-RPC 2.0 protocol.
- **Dynamic Tool Discovery:** Implements `tools/list` to announce available capabilities.
- **Extensible Tools:** Currently supports:
  - `run_inference`: Execute AI model inference.
  - `data_analysis`: Perform data analysis operations.
- **Resource Discovery:** Implements `resources/list` to discover available resources, such as ML models.
- **Configuration via Environment:** All configuration is managed through environment variables for easy deployment.

## Prerequisites

- Rust programming language (version 1.70 or newer).
- Cargo, the Rust package manager.

## Building

To build the server for production, run the following command from the root of the project directory:

```sh
cargo build --release
```

The optimized binary will be located at `target/release/aionr2`.

## Running the Server

The server is configured through environment variables. The following variables are required:

- `AION_R_API_URL`: The full URL of the backend AION-R API.
- `AION_R_API_KEY`: (Optional) A bearer token for authenticating with the AION-R API.
- `RUST_LOG`: The logging level. Set to `info` for normal operation or `debug` for detailed logs. (e.g., `RUST_LOG=info`)

To run the server, execute the binary:

```sh
AION_R_API_URL=http://localhost:8001 RUST_LOG=info ./target/release/aionr2
```

The server will start and wait for JSON-RPC messages on its standard input.

## Example Usage

Communication with the server happens via framed JSON-RPC messages. Each message is prefixed with a `Content-Length` header.

### 1. Initialize Request

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": { "protocolVersion": "2024-11-05" },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "server": {
      "name": "aionr2",
      "version": "0.1.0"
    }
  },
  "id": 1
}
```

### 2. List Tools

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 2
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "name": "run_inference",
        "description": "Runs AI inference by calling the backend AION-R API.",
        "inputs": {
          "type": "object",
          "properties": {
            "model": { "type": "string" },
            "prompt": { "type": "string" },
            "params": { "type": "object" }
          },
          "required": ["model", "prompt"]
        }
      },
      {
        "name": "data_analysis",
        "description": "Runs data analysis by calling the backend AION-R API.",
        "inputs": {
          "type": "object",
          "properties": {
            "data": {},
            "ops": { "type": "array" }
          },
          "required": ["data", "ops"]
        }
      }
    ]
  },
  "id": 2
}
```

### 3. Call a Tool (run_inference)

**Request:**
```json
{
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
}
```

**Response (from the backend):**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "success",
    "inference_id": "inf_123",
    "output": "The meaning of life is 42."
  },
  "id": 3
}
```
