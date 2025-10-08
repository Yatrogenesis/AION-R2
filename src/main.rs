// src/main.rs

// Declare modules
mod api;
mod config;
mod errors;
mod mcp;
mod tools;
mod util;

use crate::config::Config;
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Parse command-line arguments, which will also read from environment variables
    let config = Config::parse();

    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting aionr2 MCP server..."
    );

    // Create and run the MCP server
    let mcp_server = mcp::server::McpServer::new(&config).await?;

    // Run the server and handle graceful shutdown
    if let Err(e) = mcp_server.run().await {
        tracing::error!(error = %e, "MCP server exited with an error");
        return Err(e);
    }

    tracing::info!("aionr2 MCP server shut down gracefully.");
    Ok(())
}
