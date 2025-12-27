pub mod ollama;
pub mod postgres;
pub mod mock;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;
    async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
pub trait VectorDbProvider: Send + Sync {
    async fn store(&self, id: &str, vector: Vec<f32>, metadata: serde_json::Value, namespace: &str) -> Result<()>;
    async fn search(&self, vector: Vec<f32>, limit: usize, namespace: &str) -> Result<Vec<serde_json::Value>>;
    async fn store_with_project(&self, id: &str, vector: Vec<f32>, metadata: serde_json::Value, namespace: &str, project_id: &str) -> Result<()>;
    async fn search_with_project(&self, vector: Vec<f32>, limit: usize, namespace: &str, project_id: &str) -> Result<Vec<serde_json::Value>>;
}

#[async_trait]
pub trait PersistenceProvider: Send + Sync {
    async fn save_state(&self, session_id: &str, state: serde_json::Value) -> Result<()>;
    async fn load_state(&self, session_id: &str) -> Result<Option<serde_json::Value>>;
}
