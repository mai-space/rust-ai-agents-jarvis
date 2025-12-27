use crate::tools::Tool;
use crate::providers::{LlmProvider, VectorDbProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

pub struct StorePreferenceTool {
    pub llm: Arc<dyn LlmProvider>,
    pub vector_db: Arc<dyn VectorDbProvider>,
}

#[async_trait]
impl Tool for StorePreferenceTool {
    fn name(&self) -> &str {
        "store_preference"
    }

    fn description(&self) -> &str {
        "Store a user preference for future reference. Input: { \"preference\": \"the preference description\" }"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let preference = input["preference"].as_str().ok_or_else(|| anyhow::anyhow!("Preference is required"))?;
        
        let embeddings = self.llm.get_embeddings(preference).await?;
        let id = Uuid::new_v4().to_string();
        
        self.vector_db.store(&id, embeddings, json!({ "preference": preference }), "user").await?;
        
        Ok(json!({ "status": "success", "id": id }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::mock::MockLlm;
    use tokio::sync::Mutex;

    struct MockDb {
        stored: Arc<Mutex<Vec<Value>>>,
    }

    #[async_trait]
    impl VectorDbProvider for MockDb {
        async fn store(&self, _id: &str, _v: Vec<f32>, m: Value, _n: &str) -> Result<()> {
            self.stored.lock().await.push(m);
            Ok(())
        }
        async fn search(&self, _v: Vec<f32>, _l: usize, _n: &str) -> Result<Vec<Value>> { Ok(vec![]) }
    }

    #[tokio::test]
    async fn test_store_preference() -> Result<()> {
        let db = Arc::new(MockDb { stored: Arc::new(Mutex::new(vec![])) });
        let tool = StorePreferenceTool {
            llm: Arc::new(MockLlm),
            vector_db: db.clone(),
        };

        let result = tool.run(json!({ "preference": "Use anyhow" })).await?;
        assert_eq!(result["status"], "success");
        
        let stored = db.stored.lock().await;
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0]["preference"], "Use anyhow");

        Ok(())
    }
}
