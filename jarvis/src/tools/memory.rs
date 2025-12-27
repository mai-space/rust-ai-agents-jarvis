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
