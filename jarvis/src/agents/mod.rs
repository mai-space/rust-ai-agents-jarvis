pub mod planning;
pub mod development;
pub mod validation;
pub mod security;
pub mod documentation;

use anyhow::Result;
use async_trait::async_trait;
use crate::tools::Tool;
use std::sync::Arc;

pub struct AgentContext {
    pub task: String,
    pub history: Vec<String>,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn identity(&self) -> String;
    fn capabilities(&self) -> Vec<Arc<dyn Tool>>;
    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput>;
}

pub enum AgentOutput {
    Success(String),
    Handoff {
        target: String,
        reason: String,
        context: String,
    },
    Error(String),
}
