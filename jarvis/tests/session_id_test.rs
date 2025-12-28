use jarvis::orchestration::Manager;
use jarvis::agents::{Agent, AgentContext, AgentOutput};
use jarvis::providers::PersistenceProvider;
use jarvis::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;

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
    async fn process(&self, _context: &mut AgentContext) -> Result<AgentOutput> {
        Ok(AgentOutput::Success("Task completed".to_string()))
    }
}

#[tokio::test]
async fn test_auto_generated_session_id() -> Result<()> {
    let persistence = Arc::new(MockPersistence { 
        state: Arc::new(Mutex::new(None)) 
    });
    let mut manager = Manager::new(3).with_persistence(persistence.clone());
    
    let agent = Arc::new(SimpleAgent);
    manager.register_agent("SimpleAgent".to_string(), agent);

    // Run without providing a session ID - should auto-generate one
    let (_result, session_id) = manager.run_with_session(
        "SimpleAgent", 
        "Test task".to_string(), 
        None, 
        vec![]
    ).await?;
    
    // Verify session ID was generated
    assert!(session_id.is_some());
    let generated_id = session_id.unwrap();
    assert!(!generated_id.is_empty());
    
    // Verify it's a valid UUID format (36 characters with hyphens)
    assert_eq!(generated_id.len(), 36);
    assert_eq!(generated_id.chars().filter(|c| *c == '-').count(), 4);
    
    Ok(())
}

#[tokio::test]
async fn test_provided_session_id_preserved() -> Result<()> {
    let persistence = Arc::new(MockPersistence { 
        state: Arc::new(Mutex::new(None)) 
    });
    let mut manager = Manager::new(3).with_persistence(persistence.clone());
    
    let agent = Arc::new(SimpleAgent);
    manager.register_agent("SimpleAgent".to_string(), agent);

    let provided_id = "my-custom-session-123".to_string();
    
    // Run with a provided session ID
    let (_result, session_id) = manager.run_with_session(
        "SimpleAgent", 
        "Test task".to_string(), 
        Some(provided_id.clone()), 
        vec![]
    ).await?;
    
    // Verify the provided session ID was preserved
    assert_eq!(session_id, Some(provided_id));
    
    Ok(())
}

#[tokio::test]
async fn test_no_session_id_without_persistence() -> Result<()> {
    let mut manager = Manager::new(3);
    
    let agent = Arc::new(SimpleAgent);
    manager.register_agent("SimpleAgent".to_string(), agent);

    // Run without persistence and without session ID
    let (_result, session_id) = manager.run_with_session(
        "SimpleAgent", 
        "Test task".to_string(), 
        None, 
        vec![]
    ).await?;
    
    // Verify no session ID was generated (no persistence)
    assert!(session_id.is_none());
    
    Ok(())
}
