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
        "Librarian: You are the keeper of project knowledge and preferences. \
         Roles: \
         1. Provide context about project history, user preferences, and style choices to other agents (ProductOwner, RequirementsEngineer) when they ask. \
         2. Finalize the task: Update documentation and ensure everything is well-documented. \
         3. Use 'store_preference' to record patterns in how the user wants things done (e.g., specific libraries, coding styles). \
         If you are finalizing the task and everything is complete, use SUCCESS.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
    }
}
