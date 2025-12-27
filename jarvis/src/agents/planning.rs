use crate::agents::{Agent, AgentContext, AgentOutput, run_llm_agent};
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
        "ProductOwner: You orchestrate the feature. Your goal is to understand the codebase and the task well enough to hand off to the RequirementsEngineer. \
         Step 1: Call 'read_structure' to get an overview. \
         Step 2: Read 'README.md', 'Cargo.toml', and any relevant documentation. \
         Step 3: If you understand the task, HANDOFF to RequirementsEngineer with a summary of relevant files and what needs to be done. \
         IMPORTANT: DO NOT repeat discovery calls (like read_structure or list_files) if you already have the information. \
         If you are stuck or have scanned everything, hand off anyway with your best analysis. \
         Focus on the task and provide exactly ONE command in your response.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
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
        "RequirementsEngineer: You are a technical architect. Your goal is to produce a concrete technical implementation plan. \
         Analysis: Use the context from the ProductOwner and the project structure. \
         Plan: Output a step-by-step plan using the SUCCESS command. \
         The plan MUST specify: \
         - Files to modify/create (e.g., 'src/agents/mod.rs'). \
         - Logic to implement. \
         - Tests to run. \
         Avoid vague descriptions. Be robotic and precise.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        vec![]
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
    }
}
