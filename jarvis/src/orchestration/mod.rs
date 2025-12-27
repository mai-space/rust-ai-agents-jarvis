pub mod acp;

use crate::agents::{Agent, AgentContext, AgentOutput};
use crate::providers::{VectorDbProvider, PersistenceProvider};
use crate::project_context::ProjectContextManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};
use serde_json::json;

pub trait HumanInTheLoop: Send + Sync {
    fn consult(&self, agent_name: &str, task: &str, history: &[String]) -> Result<String>;
}

pub struct Manager {
    agents: HashMap<String, Arc<dyn Agent>>,
    max_retries: usize,
    hitl: Option<Arc<dyn HumanInTheLoop>>,
    vector_db: Option<Arc<dyn VectorDbProvider>>,
    persistence: Option<Arc<dyn PersistenceProvider>>,
}

impl Manager {
    pub fn new(max_retries: usize) -> Self {
        Self {
            agents: HashMap::new(),
            max_retries,
            hitl: None,
            vector_db: None,
            persistence: None,
        }
    }

    pub fn with_hitl(mut self, hitl: Arc<dyn HumanInTheLoop>) -> Self {
        self.hitl = Some(hitl);
        self
    }

    pub fn with_vector_db(mut self, vector_db: Arc<dyn VectorDbProvider>) -> Self {
        self.vector_db = Some(vector_db);
        self
    }

    pub fn with_persistence(mut self, persistence: Arc<dyn PersistenceProvider>) -> Self {
        self.persistence = Some(persistence);
        self
    }

    pub fn register_agent(&mut self, name: String, agent: Arc<dyn Agent>) {
        self.agents.insert(name, agent);
    }

    pub async fn run(&self, initial_agent: &str, task: String) -> Result<String> {
        self.run_with_session(initial_agent, task, None).await
    }

    pub async fn run_with_session(&self, initial_agent: &str, task: String, session_id: Option<String>) -> Result<String> {
        let mut current_agent_name = initial_agent.to_string();
        
        // Initialize project context
        let mut project_ctx_manager = ProjectContextManager::new();
        let project_metadata = project_ctx_manager.init_from_cwd().ok().cloned();
        
        // Log project context
        if let Some(ref meta) = project_metadata {
            info!("Manager: Initialized project context: {}", meta.project_name);
            info!("Manager: Project ID: {}", meta.project_id);
        }
        
        let mut context = if let (Some(persistence), Some(sid)) = (&self.persistence, &session_id) {
            if let Some(state) = persistence.load_state(sid).await? {
                info!("Manager: Resuming session '{}'", sid);
                AgentContext {
                    task: state["task"].as_str().unwrap_or(&task).to_string(),
                    history: state["history"].as_array()
                        .map(|a| a.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
                        .unwrap_or_default(),
                    vector_db: self.vector_db.clone(),
                    available_agents: self.agents.keys().cloned().collect(),
                    project_metadata: project_metadata.clone(),
                    handoff_count: HashMap::new(),
                }
            } else {
                AgentContext {
                    task,
                    history: Vec::new(),
                    vector_db: self.vector_db.clone(),
                    available_agents: self.agents.keys().cloned().collect(),
                    project_metadata: project_metadata.clone(),
                    handoff_count: HashMap::new(),
                }
            }
        } else {
            AgentContext {
                task,
                history: Vec::new(),
                vector_db: self.vector_db.clone(),
                available_agents: self.agents.keys().cloned().collect(),
                project_metadata: project_metadata.clone(),
                handoff_count: HashMap::new(),
            }
        };

        let mut retry_counts: HashMap<String, usize> = HashMap::new();
        let mut agent_call_sequence: Vec<String> = Vec::new();

        loop {
            if let (Some(persistence), Some(sid)) = (&self.persistence, &session_id) {
                persistence.save_state(sid, json!({
                    "task": context.task,
                    "history": context.history,
                    "current_agent": current_agent_name
                })).await?;
            }

            info!("Manager: Calling agent '{}'", current_agent_name);
            
            // Track agent call sequence for loop detection
            agent_call_sequence.push(current_agent_name.clone());
            
            // Detect immediate loops (A -> B -> A -> B pattern)
            if agent_call_sequence.len() >= 4 {
                let len = agent_call_sequence.len();
                let last_four = &agent_call_sequence[len-4..len];
                if last_four[0] == last_four[2] && last_four[1] == last_four[3] {
                    warn!("Manager: Detected immediate handoff loop: {} <-> {}", last_four[0], last_four[1]);
                    if let Some(hitl) = &self.hitl {
                        info!("Manager: Requesting human intervention to break the loop");
                        let human_input = hitl.consult(&current_agent_name, "Loop detected between agents", &context.history)?;
                        info!("Manager: Human intervention received. Continuing.");
                        context.task = format!("{} (Human Instruction to break loop: {})", context.task, human_input);
                        agent_call_sequence.clear(); // Reset sequence after intervention
                    } else {
                        error!("Manager: Loop detected but no Human-in-the-loop provider available.");
                        return Err(anyhow::anyhow!("Agent handoff loop detected: {} <-> {}", last_four[0], last_four[1]));
                    }
                }
            }
            
            // Prevent excessive total iterations
            if agent_call_sequence.len() > 30 {
                error!("Manager: Exceeded maximum total agent iterations (30)");
                return Err(anyhow::anyhow!("Task exceeded maximum agent iterations"));
            }
            
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
