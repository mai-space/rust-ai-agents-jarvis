use crate::mcp::types::*;
use crate::tools::Tool;
use anyhow::{Result, anyhow};
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, AsyncRead, AsyncWrite};
use tracing::error;

pub struct McpServer {
    tools: Vec<Arc<dyn Tool>>,
}

impl McpServer {
    pub fn new(tools: Vec<Arc<dyn Tool>>) -> Self {
        Self { tools }
    }

    pub async fn run(&self) -> Result<()> {
        self.run_on_io(tokio::io::stdin(), tokio::io::stdout()).await
    }

    pub async fn run_on_io<R, W>(&self, input: R, mut output: W) -> Result<()>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(input);
        let mut line = String::new();

        loop {
            line.clear();
            if reader.read_line(&mut line).await? == 0 {
                break;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to parse MCP request: {}", e);
                    continue;
                }
            };

            let response = self.handle_request(request).await?;
            let response_str = serde_json::to_string(&response)? + "\n";
            output.write_all(response_str.as_bytes()).await?;
            output.flush().await?;
        }

        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let result = match request.method.as_str() {
            "initialize" => {
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "jarvis-server",
                        "version": "0.1.0"
                    }
                })
            }
            "tools/list" => {
                let tools: Vec<McpTool> = self.tools.iter().map(|t| McpTool {
                    name: t.name().to_string(),
                    description: t.description().to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {} // Simple schema for now, could be improved
                    }),
                }).collect();
                json!({ "tools": tools })
            }
            "tools/call" => {
                let name = request.params["name"].as_str().ok_or_else(|| anyhow!("Missing tool name"))?;
                let arguments = request.params["arguments"].clone();
                
                let tool = self.tools.iter().find(|t| t.name() == name)
                    .ok_or_else(|| anyhow!("Tool not found: {}", name))?;
                
                let tool_result = tool.run(arguments).await?;
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": tool_result.to_string()
                        }
                    ]
                })
            }
            _ => {
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                    id: request.id,
                });
            }
        };

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        })
    }
}
