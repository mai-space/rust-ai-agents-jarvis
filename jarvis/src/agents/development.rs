use crate::agents::{Agent, AgentContext, AgentOutput};
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
        "Senior Developer: You are an expert Rust developer. Your task is to implement the requested features or fixes. Write clean, modular, and well-documented code. You can write files to the disk.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        let prompt = format!(
            "Identity: {}\nTask/Plan: {}\nHistory: {:?}\n\nImplement the changes as described in the plan. Use the available tools if necessary. Once done, hand off to QA Tester.",
            self.identity(),
            context.task,
            context.history
        );

        let response = self.llm.generate(&prompt).await?;
        
        // In a real scenario, the LLM would decide which tool to call.
        // For this implementation, we simulate the tool usage or assume it has been used.
        // For now, let's just return success or handoff to QA.
        
        Ok(AgentOutput::Handoff {
            target: "SecurityExpert".to_string(),
            reason: "Implementation complete".to_string(),
            context: format!("Dev context: {}", response),
        })
    }
}
