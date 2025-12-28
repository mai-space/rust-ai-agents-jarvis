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
        "ProductOwner: You orchestrate the feature and create the high-level strategy. \
         DO NOT implement the solution yourself. \
         \n\nYour workflow:\n\
         1. ANALYZE: Use 'get_cached_structure' or 'read_structure' to understand project layout. Read README.md and key files.\n\
         2. CREATE PLAN: Use the PLAN command to create a structured markdown plan with clear sections:\n\
            - Overview: What needs to be done\n\
            - Key Files: List relevant files\n\
            - Approach: High-level strategy\n\
            - Success Criteria: How we know it's done\n\
         3. HANDOFF: Once you have created the plan, HANDOFF to RequirementsEngineer with a clear summary.\n\
         \n\
         TIPS:\n\
         - Be decisive and efficient. Gather essential info (2-4 tool calls max), create plan, then handoff.\n\
         - RequirementsEngineer has read-only tools and can look at files you mention.\n\
         - Use PLAN command to make your plan visible to the UI and other agents.\n\
         - If confused or stuck for more than 3 attempts, create a basic plan and hand off anyway.".to_string()
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
    tools: Vec<Arc<dyn Tool>>,
}

impl RequirementsEngineer {
    pub fn new(llm: Arc<dyn LlmProvider>, tools: Vec<Arc<dyn Tool>>) -> Self {
        Self { llm, tools }
    }
}

#[async_trait]
impl Agent for RequirementsEngineer {
    fn identity(&self) -> String {
        "RequirementsEngineer: You are a technical architect who creates detailed implementation plans. \
         \n\nYour workflow:\n\
         1. REVIEW: Read the ProductOwner's plan and any files mentioned. You have read-only tools (read_file, list_files).\n\
         2. CREATE DETAILED PLAN: Use the PLAN command to create a specific technical plan with:\n\
            - Exact files to create/modify\n\
            - Code structure and logic\n\
            - Step-by-step implementation sequence\n\
            - Testing approach\n\
         3. HANDOFF: Once plan is ready, HANDOFF to SeniorDeveloper with clear instructions.\n\
         \n\
         TIPS:\n\
         - Be specific: mention exact file paths, function names, and logic.\n\
         - You can read files to understand current implementation.\n\
         - Use PLAN command to make your detailed plan visible.\n\
         - Work quickly: 3-5 tool calls max, then create plan and handoff.\n\
         - If stuck, create the best plan you can with available info and handoff.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
    }
}
