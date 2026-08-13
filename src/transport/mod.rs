use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};
use tracing::{debug, error};

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn read_request(&mut self) -> Result<Option<JsonRpcRequest>>;
    async fn send_response(&mut self, response: &JsonRpcResponse) -> Result<()>;
}

pub struct StdioTransport {
    reader: BufReader<Stdin>,
    writer: Stdout,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(tokio::io::stdin()),
            writer: tokio::io::stdout(),
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Transport for StdioTransport {
    async fn read_request(&mut self) -> Result<Option<JsonRpcRequest>> {
        let mut line = String::new();
        let bytes_read = self
            .reader
            .read_line(&mut line)
            .await
            .context("Failed to read line from stdin")?;

        if bytes_read == 0 {
            // EOF reached
            return Ok(None);
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        debug!(target: "mcp_turbo::transport", "Received raw message: {}", trimmed);

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(req) => req,
            Err(e) => {
                error!(target: "mcp_turbo::transport", "Deserialization error: {}", e);
                return Err(anyhow::anyhow!("Parse error: {}", e));
            }
        };

        Ok(Some(req))
    }

    async fn send_response(&mut self, response: &JsonRpcResponse) -> Result<()> {
        let json_str = serde_json::to_string(response)
            .context("Failed to serialize JsonRpcResponse")?;

        debug!(target: "mcp_turbo::transport", "Sending raw response: {}", json_str);

        self.writer
            .write_all(json_str.as_bytes())
            .await
            .context("Failed to write to stdout")?;
        self.writer
            .write_all(b"\n")
            .await
            .context("Failed to write newline to stdout")?;
        self.writer
            .flush()
            .await
            .context("Failed to flush stdout")?;

        Ok(())
    }
}
