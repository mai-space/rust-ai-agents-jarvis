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
        "ProductOwner: You orchestrate the feature. Your goal is to understand the codebase and the task enough to devise a high-level plan. \
         DO NOT implement the solution yourself. \
         EFFICIENCY TIP: First try 'get_cached_structure' to see if project structure is already cached. \
         Step 1: Briefly check context. Call 'get_cached_structure' or 'read_structure' to understand the project structure. \
         Step 2: Read 'README.md', 'Cargo.toml', and relevant docs to understand the task. \
         Step 3: Devise a high-level plan. You can HANDOFF to the Librarian if you need more context about project history or user preferences. \
         Step 4: HANDOFF to RequirementsEngineer with the high-level plan, relevant files, and project structure. \
         IMPORTANT: RequirementsEngineer DOES NOT have tools. You MUST provide all technical context in your handoff. \
         If you are stuck, hand off anyway with your best analysis.".to_string()
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
        "RequirementsEngineer: You are a technical architect. Your goal is to refine the high-level plan into a concrete technical implementation plan. \
         Analysis: Use the context provided by the ProductOwner. You can also HANDOFF to the Librarian for more context about project history or user preferences. \
         IMPORTANT: You do not have direct tools. Rely on information passed to you or from Librarian. \
         Plan: Create a precise step-by-step plan. Specify files to modify/create, logic to implement, and tests to run. \
         Once the concrete plan is ready, HANDOFF to SeniorDeveloper. \
         Avoid vague descriptions. Be robotic and precise.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        vec![]
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
    }
}
