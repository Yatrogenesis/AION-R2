// src/util.rs

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};

/// Reads a complete JSON-RPC message from stdio.
/// A message is defined as a block of text terminated by a blank line.
pub async fn read_message(stdin: &mut BufReader<Stdin>) -> Result<Option<String>> {
    let mut buffer = String::new();
    let mut content_length = 0;

    loop {
        buffer.clear();
        if stdin.read_line(&mut buffer).await? == 0 {
            // EOF
            return Ok(None);
        }

        if buffer.trim().is_empty() {
            // End of headers, break to read body
            break;
        }

        let parts: Vec<&str> = buffer.trim().splitn(2, ':').collect();
        if parts.len() == 2 && parts[0].trim().eq_ignore_ascii_case("Content-Length") {
            if let Ok(len) = parts[1].trim().parse::<usize>() {
                content_length = len;
            }
        }
    }

    if content_length > 0 {
        let mut body = vec![0; content_length];
        stdin.read_exact(&mut body).await?;
        let body_str = String::from_utf8(body)?;
        return Ok(Some(body_str));
    }

    Ok(None)
}

/// Writes a complete JSON-RPC message to stdout.
pub async fn write_message(stdout: &mut Stdout, message: &str) -> Result<()> {
    let response = format!("Content-Length: {}\r\n\r\n{}", message.len(), message);
    stdout.write_all(response.as_bytes()).await?;
    stdout.flush().await?;
    Ok(())
}
