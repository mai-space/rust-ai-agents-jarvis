use crate::mcp::types::*;
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

pub struct McpClient {
    child: Child,
    next_id: i64,
}

impl McpClient {
    pub async fn spawn(command: &str, args: &[&str]) -> Result<Self> {
        let child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut client = Self {
            child,
            next_id: 1,
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

        let stdin = self.child.stdin.as_mut().ok_or_else(|| anyhow!("Failed to open stdin"))?;
        let request_str = serde_json::to_string(&request)? + "\n";
        stdin.write_all(request_str.as_bytes()).await?;
        stdin.flush().await?;

        let stdout = self.child.stdout.as_mut().ok_or_else(|| anyhow!("Failed to open stdout"))?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

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
        let stdin = self.child.stdin.as_mut().ok_or_else(|| anyhow!("Failed to open stdin"))?;
        let notif_str = serde_json::to_string(&notification)? + "\n";
        stdin.write_all(notif_str.as_bytes()).await?;
        stdin.flush().await?;

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
