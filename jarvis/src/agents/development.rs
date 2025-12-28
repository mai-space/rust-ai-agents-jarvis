use crate::agents::{Agent, AgentContext, AgentOutput, run_llm_agent};
use crate::tools::Tool;
use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct SeniorDeveloper {
    llm: Arc<dyn LlmProvider>,
    tools: Vec<Arc<dyn Tool>>,
}

impl SeniorDeveloper {
    pub fn new(llm: Arc<dyn LlmProvider>, tools: Vec<Arc<dyn Tool>>) -> Self {
        Self { llm, tools }
    }
}

#[async_trait]
impl Agent for SeniorDeveloper {
    fn identity(&self) -> String {
        "SeniorDeveloper: You are an expert developer. Your task is to implement the concrete plan provided to you. \
         DO NOT spend time planning; focus on coding, reading relevant files, and writing the implementation. \
         Write clean, modular, and well-documented code. \
         Once implemented, HANDOFF to QATester to verify your changes.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
    }
}
