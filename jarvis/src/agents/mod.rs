pub mod planning;
pub mod development;
pub mod validation;
pub mod security;
pub mod documentation;
pub mod refinement;

use anyhow::Result;
use async_trait::async_trait;
use crate::tools::Tool;
use crate::providers::LlmProvider;
use std::sync::Arc;
use serde_json::Value;

pub struct AgentContext {
    pub task: String,
    pub history: Vec<String>,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn identity(&self) -> String;
    fn capabilities(&self) -> Vec<Arc<dyn Tool>>;
    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput>;
}

pub enum AgentOutput {
    Success(String),
    Handoff {
        target: String,
        reason: String,
        context: String,
    },
    Error(String),
}

pub async fn run_llm_agent(
    agent: &dyn Agent,
    llm: Arc<dyn LlmProvider>,
    context: &mut AgentContext,
) -> Result<AgentOutput> {
    let mut session_history = Vec::new();
    let capabilities = agent.capabilities();
    let identity = agent.identity();

    let tools_desc = capabilities.iter()
        .map(|t| format!("{}: {}", t.name(), t.description()))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = format!(
        "Identity: {}\n\nAvailable Tools:\n{}\n\nCommands:\n- CALL <tool_name> {{ \"arg\": \"val\" }}\n- HANDOFF <target_agent> <reason> <context_for_next_agent>\n- SUCCESS <final_result>\n- ERROR <error_message>\n\nAlways use one of these commands. Provide only the command in your response.",
        identity, tools_desc
    );

    loop {
        let mut full_prompt = system_prompt.clone();
        full_prompt.push_str(&format!("\n\nTask: {}\n", context.task));
        full_prompt.push_str(&format!("Global History: {:?}\n", context.history));
        full_prompt.push_str(&format!("Current Session Trace:\n{:?}\n", session_history));

        let response = llm.generate(&full_prompt).await?;
        session_history.push(format!("Assistant: {}", response));

        if response.starts_with("CALL") {
            let parts: Vec<&str> = response.splitn(3, ' ').collect();
            if parts.len() < 3 {
                session_history.push("System: Invalid CALL format. Use CALL <tool_name> <json_input>".to_string());
                continue;
            }
            let tool_name = parts[1];
            let tool_input_str = parts[2];
            
            let tool = capabilities.iter().find(|t| t.name() == tool_name);
            match tool {
                Some(t) => {
                    let input: Value = match serde_json::from_str(tool_input_str) {
                        Ok(v) => v,
                        Err(e) => {
                            session_history.push(format!("System: Error parsing JSON input: {}", e));
                            continue;
                        }
                    };
                    match t.run(input).await {
                        Ok(res) => session_history.push(format!("System: Tool '{}' result: {}", tool_name, res)),
                        Err(e) => session_history.push(format!("System: Tool '{}' error: {}", tool_name, e)),
                    }
                }
                None => session_history.push(format!("System: Tool '{}' not found", tool_name)),
            }
        } else if response.starts_with("HANDOFF") {
            let parts: Vec<&str> = response.splitn(4, ' ').collect();
            if parts.len() < 4 {
                session_history.push("System: Invalid HANDOFF format. Use HANDOFF <target> <reason> <context>".to_string());
                continue;
            }
            return Ok(AgentOutput::Handoff {
                target: parts[1].to_string(),
                reason: parts[2].to_string(),
                context: parts[3].to_string(),
            });
        } else if response.starts_with("SUCCESS") {
            let result = response.strip_prefix("SUCCESS ").unwrap_or(&response);
            return Ok(AgentOutput::Success(result.to_string()));
        } else if response.starts_with("ERROR") {
            let err = response.strip_prefix("ERROR ").unwrap_or(&response);
            return Ok(AgentOutput::Error(err.to_string()));
        } else {
            session_history.push("System: Unknown command format. Please use CALL, HANDOFF, SUCCESS, or ERROR.".to_string());
        }

        // Prevent infinite loops in one agent process
        if session_history.len() > 20 {
            return Ok(AgentOutput::Error("Agent exceeded maximum interaction steps".to_string()));
        }
    }
}
