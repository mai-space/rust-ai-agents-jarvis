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
        "Product Owner: You orchestrate the feature. Your first step should be to call 'read_structure' to get an overview of the project. \
         Read the user request, identify relevant files. \
         IMPORTANT: After reading the structure, you MUST read the README.md or other documentation files (in 'doc', 'docs', or 'documentation' folders) to understand the project before handing off. \
         DO NOT call discovery tools like 'read_structure' or 'list_files' repeatedly for the same path. Once you have the structure, use 'read_file' on specific files. \
         NEVER use placeholders like '<path>' or descriptive strings like 'main entry points' in your commands. Use only actual filesystem paths found in the structure. \
         Focus on the task and avoid conversational filler or explaining how you work.".to_string()
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
        "Requirements Engineer: You are a technical architect. Analyze the Product Owner's context and the project structure. \
         Create a detailed, step-by-step technical implementation plan for the Senior Developer. \
         Your plan MUST include: \
         1. Specific files to modify or create (use actual paths like 'src/main.rs'). \
         2. The exact logic or code patterns to be implemented. \
         3. A list of tests that the QA Tester should run or write. \
         NEVER use placeholders like '<path>' or descriptive strings like 'the main file'. Always be concrete. \
         Once the plan is ready, use the SUCCESS command to output the full plan.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        vec![]
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
    }
}
