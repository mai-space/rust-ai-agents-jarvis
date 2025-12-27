use crate::agents::{Agent, AgentContext, AgentOutput, run_llm_agent};
use crate::tools::Tool;
use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct Librarian {
    llm: Arc<dyn LlmProvider>,
    tools: Vec<Arc<dyn Tool>>,
}

impl Librarian {
    pub fn new(llm: Arc<dyn LlmProvider>, tools: Vec<Arc<dyn Tool>>) -> Self {
        Self { llm, tools }
    }
}

#[async_trait]
impl Agent for Librarian {
    fn identity(&self) -> String {
        "Librarian: You finalize the task. Your job is to update documentation, and ensure the task is complete and well-documented. \
         You are also responsible for identifying and storing user preferences and style choices observed during the session using the store_preference tool. \
         Look for patterns in how the user wants things done (e.g., specific libraries, coding styles, or preferred ways of explaining things).".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
    }
}
