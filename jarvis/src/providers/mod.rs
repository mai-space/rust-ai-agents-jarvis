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
    async fn store(&self, id: &str, vector: Vec<f32>, metadata: serde_json::Value) -> Result<()>;
    async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<serde_json::Value>>;
}
