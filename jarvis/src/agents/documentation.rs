use crate::agents::{Agent, AgentContext, AgentOutput};
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
        "Librarian: You finalize the task. Your job is to update documentation, KDocs, and ensure the task is complete and well-documented.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        let prompt = format!(
            "Identity: {}\nTask Context: {}\nHistory: {:?}\n\nFinalize the task by updating documentation. Use available tools to read or write files. Once done, provide a final summary of the work.",
            self.identity(),
            context.task,
            context.history
        );

        let response = self.llm.generate(&prompt).await?;
        
        // In a real scenario, the agent would use WriteFileTool here.
        // For the first release, we'll assume it has done its job if the LLM says so.

        Ok(AgentOutput::Success(format!("Task finalized by Librarian: {}", response)))
    }
}
