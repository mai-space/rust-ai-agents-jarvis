use crate::mcp::types::*;
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, AsyncRead, AsyncWrite, AsyncBufRead};
use tokio::process::{Child, Command};

pub struct McpClient {
    reader: Box<dyn AsyncBufRead + Unpin + Send>,
    writer: Box<dyn AsyncWrite + Unpin + Send>,
    next_id: i64,
    _child: Option<Child>,
}

impl McpClient {
    pub async fn spawn(command: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to open stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to open stdout"))?;

        let mut client = Self {
            reader: Box::new(BufReader::new(stdout)),
            writer: Box::new(stdin),
            next_id: 1,
            _child: Some(child),
        };

        client.initialize().await?;

        Ok(client)
    }

    pub async fn new<R, W>(reader: R, writer: W) -> Result<Self>
    where
        R: AsyncRead + Send + Unpin + 'static,
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let mut client = Self {
            reader: Box::new(BufReader::new(reader)),
            writer: Box::new(writer),
            next_id: 1,
            _child: None,
        };
        client.initialize().await?;
        Ok(client)
    }

    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: json!(id),
        };

        let request_str = serde_json::to_string(&request)? + "\n";
        self.writer.write_all(request_str.as_bytes()).await?;
        self.writer.flush().await?;

        let mut line = String::new();
        self.reader.read_line(&mut line).await?;

        if line.is_empty() {
            return Err(anyhow!("MCP server closed connection"));
        }

        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        
        if let Some(error) = response.error {
            return Err(anyhow!("MCP error {}: {}", error.code, error.message));
        }

        response.result.ok_or_else(|| anyhow!("Missing result in MCP response"))
    }

    async fn initialize(&mut self) -> Result<()> {
        self.send_request("initialize", json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "jarvis",
                "version": "0.1.0"
            }
        })).await?;

        // MCP protocol requires a notification after initialization
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let notif_str = serde_json::to_string(&notification)? + "\n";
        self.writer.write_all(notif_str.as_bytes()).await?;
        self.writer.flush().await?;

        Ok(())
    }

    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>> {
        let result = self.send_request("tools/list", json!({})).await?;
        let list_result: McpListToolsResult = serde_json::from_value(result)?;
        Ok(list_result.tools)
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<McpCallToolResult> {
        let result = self.send_request("tools/call", json!({
            "name": name,
            "arguments": arguments
        })).await?;
        let call_result: McpCallToolResult = serde_json::from_value(result)?;
        Ok(call_result)
    }
}
