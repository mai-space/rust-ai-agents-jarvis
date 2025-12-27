use crate::agents::{Agent, AgentContext, AgentOutput};
use crate::tools::Tool;
use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ProductOwner {
    llm: Arc<dyn LlmProvider>,
    tools: Vec<Arc<dyn Tool>>,
}

impl ProductOwner {
    pub fn new(llm: Arc<dyn LlmProvider>, tools: Vec<Arc<dyn Tool>>) -> Self {
        Self { llm, tools }
    }
}

#[async_trait]
impl Agent for ProductOwner {
    fn identity(&self) -> String {
        "Product Owner: You orchestrate the feature. Read the user request, scan the current codebase structure to identify relevant files, and delegate to the Requirements Engineer.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        let prompt = format!(
            "Identity: {}\nTask: {}\nHistory: {:?}\n\nAnalyze the task and codebase. If you have enough info, hand off to Requirements Engineer with technical details.",
            self.identity(),
            context.task,
            context.history
        );

        let response = self.llm.generate(&prompt).await?;
        
        // Simple logic for handoff for now
        Ok(AgentOutput::Handoff {
            target: "RequirementsEngineer".to_string(),
            reason: "Initial planning complete".to_string(),
            context: format!("PO context: {}", response),
        })
    }
}

pub struct RequirementsEngineer {
    llm: Arc<dyn LlmProvider>,
}

impl RequirementsEngineer {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Agent for RequirementsEngineer {
    fn identity(&self) -> String {
        "Requirements Engineer: Create a technical plan. Analyze the PO's context. Output a step-by-step implementation plan for the Developer.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        vec![]
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        let prompt = format!(
            "Identity: {}\nPO Context: {}\n\nGenerate a step-by-step technical plan.",
            self.identity(),
            context.task
        );

        let response = self.llm.generate(&prompt).await?;
        
        Ok(AgentOutput::Handoff {
            target: "SeniorDeveloper".to_string(),
            reason: "Technical plan generated".to_string(),
            context: response,
        })
    }
}
