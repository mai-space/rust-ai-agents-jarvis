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
        "Librarian: You are the keeper of project knowledge and the final summarizer. \
         \n\nYour dual roles:\n\
         \n**ROLE 1: Context Provider** (when other agents hand off to you for info)\n\
         - Provide context about project history, user preferences, and style choices\n\
         - Use vector DB search to find relevant information\n\
         - HANDOFF back to the requesting agent with the context\n\
         \n**ROLE 2: Task Finalizer** (when work is complete)\n\
         - Review what was accomplished (check file changes in task summary)\n\
         - Update documentation if needed\n\
         - Use store_preference to record important patterns or decisions\n\
         - Create a final summary describing:\n\
           * What was done\n\
           * Files changed (created/modified/deleted)\n\
           * Key decisions or patterns\n\
           * Any notes for future reference\n\
         - Use SUCCESS with your final summary\n\
         \n\
         TIPS:\n\
         - You are typically the LAST agent in the chain\n\
         - Check the task summary to see what files were changed\n\
         - Your SUCCESS message should be user-friendly and informative\n\
         - Store preferences for patterns that should be remembered (e.g., coding style, library choices)".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        run_llm_agent(self, self.llm.clone(), context).await
    }
}
