pub mod fs;
pub mod shell;
pub mod git;
pub mod memory;
pub mod mcp;
pub mod analysis;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn run(&self, input: Value) -> Result<Value>;
}
