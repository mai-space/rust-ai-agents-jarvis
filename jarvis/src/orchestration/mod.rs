use crate::agents::{Agent, AgentContext, AgentOutput};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};

pub trait HumanInTheLoop: Send + Sync {
    fn consult(&self, agent_name: &str, task: &str, history: &[String]) -> Result<String>;
}

pub struct Manager {
    agents: HashMap<String, Arc<dyn Agent>>,
    max_retries: usize,
    hitl: Option<Arc<dyn HumanInTheLoop>>,
}

impl Manager {
    pub fn new(max_retries: usize) -> Self {
        Self {
            agents: HashMap::new(),
            max_retries,
            hitl: None,
        }
    }

    pub fn with_hitl(mut self, hitl: Arc<dyn HumanInTheLoop>) -> Self {
        self.hitl = Some(hitl);
        self
    }

    pub fn register_agent(&mut self, name: String, agent: Arc<dyn Agent>) {
        self.agents.insert(name, agent);
    }

    pub async fn run(&self, initial_agent: &str, task: String) -> Result<String> {
        let mut current_agent_name = initial_agent.to_string();
        let mut context = AgentContext {
            task,
            history: Vec::new(),
        };

        let mut retry_counts: HashMap<String, usize> = HashMap::new();

        loop {
            info!("Manager: Calling agent '{}'", current_agent_name);
            let agent = self.agents.get(&current_agent_name)
                .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", current_agent_name))?;

            match agent.process(&mut context).await? {
                AgentOutput::Success(result) => {
                    info!("Manager: Task completed by '{}'", current_agent_name);
                    return Ok(result);
                }
                AgentOutput::Handoff { target, reason, context: new_context } => {
                    info!("Manager: Handoff from '{}' to '{}'. Reason: {}", current_agent_name, target, reason);
                    
                    let retries = retry_counts.entry(target.clone()).or_insert(0);
                    if *retries >= self.max_retries {
                        warn!("Manager: Max retries reached for agent '{}'.", target);
                        if let Some(hitl) = &self.hitl {
                            info!("Manager: Requesting human intervention for agent '{}'", target);
                            let human_input = hitl.consult(&target, &new_context, &context.history)?;
                            info!("Manager: Human intervention received. Continuing.");
                            // Reset retries and update task with human input
                            *retries = 0;
                            context.task = format!("{} (Human Instruction: {})", new_context, human_input);
                        } else {
                            error!("Manager: No Human-in-the-loop provider available. Failing.");
                            return Err(anyhow::anyhow!("Max retries reached for agent '{}'", target));
                        }
                    } else {
                        *retries += 1;
                        context.task = new_context;
                    }

                    context.history.push(format!("Handoff from {} to {}: {}", current_agent_name, target, reason));
                    current_agent_name = target;
                }
                AgentOutput::Error(err) => {
                    error!("Manager: Error from agent '{}': {}", current_agent_name, err);
                    return Err(anyhow::anyhow!("Agent error: {}", err));
                }
            }
        }
    }
}
