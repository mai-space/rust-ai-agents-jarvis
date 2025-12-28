/// Event system for real-time agent feedback to GUI and other consumers
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// Agent started processing
    AgentStarted {
        agent_name: String,
        timestamp: i64,
    },
    /// Agent thinking/reasoning
    AgentThought {
        agent_name: String,
        thought: String,
        timestamp: i64,
    },
    /// Agent calling a tool
    ToolCall {
        agent_name: String,
        tool_name: String,
        input_summary: String,
        timestamp: i64,
    },
    /// Tool execution completed
    ToolResult {
        agent_name: String,
        tool_name: String,
        output_summary: String,
        success: bool,
        timestamp: i64,
    },
    /// Agent produced a structured plan
    PlanCreated {
        agent_name: String,
        plan: String,
        timestamp: i64,
    },
    /// Agent handing off to another agent
    Handoff {
        from_agent: String,
        to_agent: String,
        reason: String,
        timestamp: i64,
    },
    /// Task completed successfully
    TaskCompleted {
        agent_name: String,
        result: String,
        summary: Option<TaskSummary>,
        timestamp: i64,
    },
    /// Task failed
    TaskFailed {
        agent_name: String,
        error: String,
        timestamp: i64,
    },
    /// File operation occurred
    FileOperation {
        operation: FileOpType,
        path: String,
        timestamp: i64,
    },
    /// Loop detected
    LoopDetected {
        agents: Vec<String>,
        timestamp: i64,
    },
    /// Human intervention requested
    HumanInterventionRequested {
        agent_name: String,
        reason: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOpType {
    Created,
    Modified,
    Deleted,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub files_created: Vec<String>,
    pub files_modified: Vec<String>,
    pub files_deleted: Vec<String>,
    pub agents_involved: Vec<String>,
    pub total_duration_ms: Option<u64>,
    pub description: String,
}

impl TaskSummary {
    pub fn new() -> Self {
        Self {
            files_created: Vec::new(),
            files_modified: Vec::new(),
            files_deleted: Vec::new(),
            agents_involved: Vec::new(),
            total_duration_ms: None,
            description: String::new(),
        }
    }

    pub fn add_file_operation(&mut self, op: &FileOpType, path: String) {
        match op {
            FileOpType::Created => {
                if !self.files_created.contains(&path) {
                    self.files_created.push(path);
                }
            }
            FileOpType::Modified => {
                if !self.files_modified.contains(&path) {
                    self.files_modified.push(path);
                }
            }
            FileOpType::Deleted => {
                if !self.files_deleted.contains(&path) {
                    self.files_deleted.push(path);
                }
            }
            FileOpType::Read => {}
        }
    }

    pub fn add_agent(&mut self, agent_name: String) {
        if !self.agents_involved.contains(&agent_name) {
            self.agents_involved.push(agent_name);
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# Task Summary\n\n");
        
        if !self.description.is_empty() {
            md.push_str(&format!("{}\n\n", self.description));
        }

        md.push_str("## Agents Involved\n");
        if self.agents_involved.is_empty() {
            md.push_str("- None\n\n");
        } else {
            for agent in &self.agents_involved {
                md.push_str(&format!("- {}\n", agent));
            }
            md.push('\n');
        }

        md.push_str("## Files Changed\n\n");
        
        if !self.files_created.is_empty() {
            md.push_str("### Created\n");
            for file in &self.files_created {
                md.push_str(&format!("- `{}`\n", file));
            }
            md.push('\n');
        }

        if !self.files_modified.is_empty() {
            md.push_str("### Modified\n");
            for file in &self.files_modified {
                md.push_str(&format!("- `{}`\n", file));
            }
            md.push('\n');
        }

        if !self.files_deleted.is_empty() {
            md.push_str("### Deleted\n");
            for file in &self.files_deleted {
                md.push_str(&format!("- `{}`\n", file));
            }
            md.push('\n');
        }

        if self.files_created.is_empty() && self.files_modified.is_empty() && self.files_deleted.is_empty() {
            md.push_str("No files were changed.\n\n");
        }

        if let Some(duration) = self.total_duration_ms {
            md.push_str(&format!("**Duration:** {}ms\n", duration));
        }

        md
    }
}

impl Default for TaskSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Event broadcaster for sending events to multiple listeners
pub struct EventBroadcaster {
    senders: Arc<tokio::sync::RwLock<Vec<mpsc::UnboundedSender<AgentEvent>>>>,
}

impl EventBroadcaster {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Subscribe to events, returns a receiver
    pub async fn subscribe(&self) -> mpsc::UnboundedReceiver<AgentEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut senders = self.senders.write().await;
        senders.push(tx);
        rx
    }

    /// Broadcast an event to all subscribers
    pub async fn broadcast(&self, event: AgentEvent) {
        let mut senders = self.senders.write().await;
        // Remove closed channels and send to active ones
        senders.retain(|tx| tx.send(event.clone()).is_ok());
    }

    /// Emit agent started event
    pub async fn agent_started(&self, agent_name: String) {
        self.broadcast(AgentEvent::AgentStarted {
            agent_name,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit agent thought event
    pub async fn agent_thought(&self, agent_name: String, thought: String) {
        self.broadcast(AgentEvent::AgentThought {
            agent_name,
            thought,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit tool call event
    pub async fn tool_call(&self, agent_name: String, tool_name: String, input_summary: String) {
        self.broadcast(AgentEvent::ToolCall {
            agent_name,
            tool_name,
            input_summary,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit tool result event
    pub async fn tool_result(
        &self,
        agent_name: String,
        tool_name: String,
        output_summary: String,
        success: bool,
    ) {
        self.broadcast(AgentEvent::ToolResult {
            agent_name,
            tool_name,
            output_summary,
            success,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit plan created event
    pub async fn plan_created(&self, agent_name: String, plan: String) {
        self.broadcast(AgentEvent::PlanCreated {
            agent_name,
            plan,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit handoff event
    pub async fn handoff(&self, from_agent: String, to_agent: String, reason: String) {
        self.broadcast(AgentEvent::Handoff {
            from_agent,
            to_agent,
            reason,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit task completed event
    pub async fn task_completed(
        &self,
        agent_name: String,
        result: String,
        summary: Option<TaskSummary>,
    ) {
        self.broadcast(AgentEvent::TaskCompleted {
            agent_name,
            result,
            summary,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit task failed event
    pub async fn task_failed(&self, agent_name: String, error: String) {
        self.broadcast(AgentEvent::TaskFailed {
            agent_name,
            error,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit file operation event
    pub async fn file_operation(&self, operation: FileOpType, path: String) {
        self.broadcast(AgentEvent::FileOperation {
            operation,
            path,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit loop detected event
    pub async fn loop_detected(&self, agents: Vec<String>) {
        self.broadcast(AgentEvent::LoopDetected {
            agents,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }

    /// Emit human intervention requested event
    pub async fn human_intervention_requested(&self, agent_name: String, reason: String) {
        self.broadcast(AgentEvent::HumanInterventionRequested {
            agent_name,
            reason,
            timestamp: chrono::Utc::now().timestamp(),
        })
        .await;
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}
