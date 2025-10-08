// src/config.rs

use clap::Parser;

/// A production-ready Rust implementation of the AION-R MCP server.
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// The URL of the backend AION-R API.
    #[arg(long, env = "AION_R_API_URL", default_value = "http://localhost:8001")]
    pub aion_r_api_url: String,

    /// An optional API key for the backend AION-R API.
    #[arg(long, env = "AION_R_API_KEY")]
    pub aion_r_api_key: Option<String>,
}
