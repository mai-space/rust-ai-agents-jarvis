use crate::tools::Tool;
use crate::mcp::McpClient;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct McpTool {
    pub name: String,
    pub description: String,
    pub client: Arc<Mutex<McpClient>>,
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let mut client = self.client.lock().await;
        let result = client.call_tool(&self.name, input).await?;
        
        if result.is_error {
            return Ok(json!({
                "error": true,
                "content": result.content
            }));
        }

        Ok(json!({
            "success": true,
            "content": result.content
        }))
    }
}
