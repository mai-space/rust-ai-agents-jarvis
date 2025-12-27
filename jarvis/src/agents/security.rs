use crate::agents::{Agent, AgentContext, AgentOutput};
use crate::tools::Tool;
use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct SecurityExpert {
    llm: Arc<dyn LlmProvider>,
    tools: Vec<Arc<dyn Tool>>,
}

impl SecurityExpert {
    pub fn new(llm: Arc<dyn LlmProvider>, tools: Vec<Arc<dyn Tool>>) -> Self {
        Self { llm, tools }
    }
}

#[async_trait]
impl Agent for SecurityExpert {
    fn identity(&self) -> String {
        "Security Expert: You scan for SQL injection, XSS, and weak dependencies. Ensure the code follows security best practices.".to_string()
    }

    fn capabilities(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        let prompt = format!(
            "Identity: {}\nImplementation Context: {}\nHistory: {:?}\n\nScan the implementation for security vulnerabilities. If safe, hand off to QA Tester. If vulnerabilities are found, hand off back to Senior Developer.",
            self.identity(),
            context.task,
            context.history
        );

        let response = self.llm.generate(&prompt).await?;
        
        if response.contains("FAIL") || response.contains("VULNERABILITY") || response.contains("SeniorDeveloper") {
            Ok(AgentOutput::Handoff {
                target: "SeniorDeveloper".to_string(),
                reason: "Security vulnerabilities found".to_string(),
                context: format!("Security Feedback: {}", response),
            })
        } else {
            Ok(AgentOutput::Handoff {
                target: "QATester".to_string(),
                reason: "Security check passed".to_string(),
                context: format!("Security check passed: {}", response),
            })
        }
    }
}
