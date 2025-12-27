use crate::agents::{Agent, AgentContext, AgentOutput};
use crate::tools::Tool;
use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct QATester {
    llm: Arc<dyn LlmProvider>,
    tools: Vec<Arc<dyn Tool>>,
}

impl QATester {
    pub fn new(llm: Arc<dyn LlmProvider>, tools: Vec<Arc<dyn Tool>>) -> Self {
        Self { llm, tools }
    }
}

#[async_trait]
impl Agent for QATester {
    fn identity(&self) -> String {
        "QA Tester: You are a quality assurance engineer. Your task is to verify that the implementation meets the requirements and does not introduce bugs. You can run tests and write new tests.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        let prompt = format!(
            "Identity: {}\nImplementation Context: {}\nHistory: {:?}\n\nVerify the implementation. Run tests using available tools. If everything is correct, hand off to Librarian. If there are issues, hand off back to Senior Developer with details.",
            self.identity(),
            context.task,
            context.history
        );

        let response = self.llm.generate(&prompt).await?;
        
        if response.contains("FAIL") || response.contains("RETRY") || response.contains("SeniorDeveloper") {
            Ok(AgentOutput::Handoff {
                target: "SeniorDeveloper".to_string(),
                reason: "QA verification failed, needs fixes".to_string(),
                context: format!("QA Feedback: {}", response),
            })
        } else {
            Ok(AgentOutput::Handoff {
                target: "Librarian".to_string(),
                reason: "QA verification passed".to_string(),
                context: format!("QA Feedback: {}", response),
            })
        }
    }
}
