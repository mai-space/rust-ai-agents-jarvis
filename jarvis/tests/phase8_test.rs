use jarvis::providers::mock::MockLlm;
use jarvis::providers::VectorDbProvider;
use jarvis::agents::{AgentContext, run_llm_agent};
use jarvis::agents::documentation::Librarian;
use jarvis::tools::{Tool, memory::StorePreferenceTool};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::{json, Value};
use tokio::sync::Mutex;

struct MockVectorDb {
    project_data: Arc<Mutex<Vec<Value>>>,
    user_data: Arc<Mutex<Vec<Value>>>,
}

#[async_trait]
impl VectorDbProvider for MockVectorDb {
    async fn store(&self, _id: &str, _vector: Vec<f32>, metadata: Value, namespace: &str) -> Result<()> {
        if namespace == "user" {
            self.user_data.lock().await.push(metadata);
        } else {
            self.project_data.lock().await.push(metadata);
        }
        Ok(())
    }
    async fn search(&self, _vector: Vec<f32>, _limit: usize, namespace: &str) -> Result<Vec<Value>> {
        if namespace == "user" {
            Ok(self.user_data.lock().await.clone())
        } else {
            Ok(self.project_data.lock().await.clone())
        }
    }
    async fn store_with_project(&self, _id: &str, _vector: Vec<f32>, metadata: Value, namespace: &str, _project_id: &str) -> Result<()> {
        if namespace == "user" {
            self.user_data.lock().await.push(metadata);
        } else {
            self.project_data.lock().await.push(metadata);
        }
        Ok(())
    }
    async fn search_with_project(&self, _vector: Vec<f32>, _limit: usize, namespace: &str, _project_id: &str) -> Result<Vec<Value>> {
        if namespace == "user" {
            Ok(self.user_data.lock().await.clone())
        } else {
            Ok(self.project_data.lock().await.clone())
        }
    }
}

#[tokio::test]
async fn test_dual_stream_rag() -> Result<()> {
    let llm = Arc::new(MockLlm);
    let vector_db = Arc::new(MockVectorDb { 
        project_data: Arc::new(Mutex::new(vec![json!("Project Architecture Specs")])),
        user_data: Arc::new(Mutex::new(vec![json!("User prefers anyhow for errors")])),
    });
    
    let mut context = AgentContext {
        task: "Implement error handling".to_string(),
        history: Vec::new(),
        vector_db: Some(vector_db.clone()),
        available_agents: vec!["Librarian".to_string()],
        project_metadata: None,
        handoff_count: std::collections::HashMap::new(),
    };

    let lib = Librarian::new(llm.clone(), vec![]);
    
    // We want to verify that run_llm_agent includes both contexts.
    // Since run_llm_agent is a loop that calls LLM, we'd need to mock LLM to inspect the prompt.
    // For now, let's just ensure it runs without error.
    
    let output = run_llm_agent(&lib, llm, &mut context).await?;
    match output {
        jarvis::agents::AgentOutput::Success(_) => {},
        _ => panic!("Expected Success"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_store_preference_tool() -> Result<()> {
    let llm = Arc::new(MockLlm);
    let vector_db = Arc::new(MockVectorDb { 
        project_data: Arc::new(Mutex::new(vec![])),
        user_data: Arc::new(Mutex::new(vec![])),
    });
    
    let tool = StorePreferenceTool {
        llm: llm.clone(),
        vector_db: vector_db.clone(),
    };
    
    let result = tool.run(json!({ "preference": "Use functional style" })).await?;
    assert_eq!(result["status"], "success");
    
    let user_prefs = vector_db.user_data.lock().await;
    assert_eq!(user_prefs.len(), 1);
    assert_eq!(user_prefs[0]["preference"], "Use functional style");
    
    Ok(())
}
