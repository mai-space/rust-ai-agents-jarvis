use jarvis::orchestration::Manager;
use jarvis::providers::mock::MockLlm;
use jarvis::providers::{VectorDbProvider, PersistenceProvider};
use jarvis::agents::{Agent, AgentContext, AgentOutput};
use jarvis::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::{json, Value};
use tokio::sync::Mutex;

struct MockVectorDb {
    data: Arc<Mutex<Vec<Value>>>,
}

#[async_trait]
impl VectorDbProvider for MockVectorDb {
    async fn store(&self, _id: &str, _vector: Vec<f32>, metadata: Value, _namespace: &str) -> Result<()> {
        self.data.lock().await.push(metadata);
        Ok(())
    }
    async fn search(&self, _vector: Vec<f32>, _limit: usize, _namespace: &str) -> Result<Vec<Value>> {
        Ok(self.data.lock().await.clone())
    }
    async fn store_with_project(&self, _id: &str, _vector: Vec<f32>, metadata: Value, _namespace: &str, _project_id: &str) -> Result<()> {
        self.data.lock().await.push(metadata);
        Ok(())
    }
    async fn search_with_project(&self, _vector: Vec<f32>, _limit: usize, _namespace: &str, _project_id: &str) -> Result<Vec<Value>> {
        Ok(self.data.lock().await.clone())
    }
}

struct MockPersistence {
    state: Arc<Mutex<Option<Value>>>,
}

#[async_trait]
impl PersistenceProvider for MockPersistence {
    async fn save_state(&self, _session_id: &str, state: Value) -> Result<()> {
        *self.state.lock().await = Some(state);
        Ok(())
    }
    async fn load_state(&self, _session_id: &str) -> Result<Option<Value>> {
        Ok(self.state.lock().await.clone())
    }
}

struct SimpleAgent;

#[async_trait]
impl Agent for SimpleAgent {
    fn identity(&self) -> String { "SimpleAgent".to_string() }
    fn capabilities(&self) -> Vec<Arc<dyn Tool>> { vec![] }
    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        if context.task.contains("SUCCESS") {
            Ok(AgentOutput::Success("Done".to_string()))
        } else {
            Ok(AgentOutput::Handoff {
                target: "SimpleAgent".to_string(),
                reason: "Not done yet".to_string(),
                context: "SUCCESS".to_string(),
            })
        }
    }
}

#[tokio::test]
async fn test_persistence() -> Result<()> {
    let persistence = Arc::new(MockPersistence { state: Arc::new(Mutex::new(None)) });
    let mut manager = Manager::new(3).with_persistence(persistence.clone());
    
    let agent = Arc::new(SimpleAgent);
    manager.register_agent("SimpleAgent".to_string(), agent);

    // Run first time - should save state
    let (_result, _session_id) = manager.run_with_session("SimpleAgent", "START".to_string(), Some("session1".to_string()), vec![]).await?;
    
    let saved_state = persistence.load_state("session1").await?.expect("State should be saved");
    assert_eq!(saved_state["task"], "SUCCESS");
    
    Ok(())
}

#[tokio::test]
async fn test_rag_logic() -> Result<()> {
    let llm = Arc::new(MockLlm);
    let vector_db = Arc::new(MockVectorDb { data: Arc::new(Mutex::new(vec![json!("Important Context")])) });
    
    let mut context = AgentContext {
        task: "TEST TASK".to_string(),
        history: Vec::new(),
        vector_db: Some(vector_db),
        available_agents: vec!["SimpleAgent".to_string()],
        project_metadata: None,
        handoff_count: std::collections::HashMap::new(),
        context_files: vec![],
    };

    // We can't easily check the prompt sent to LLM here without more mocking,
    // but we can at least ensure it doesn't crash and returns success.
    // In a real scenario, run_llm_agent would be called.
    
    let agent = SimpleAgent;
    let output = jarvis::agents::run_llm_agent(&agent, llm, &mut context).await?;
    
    match output {
        AgentOutput::Success(_) => {},
        _ => panic!("Expected Success"),
    }

    Ok(())
}
