use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

pub struct OllamaProvider {
    client: Ollama,
    model: String,
}

impl OllamaProvider {
    pub fn new(host: String, port: u16, model: String) -> Self {
        Self {
            client: Ollama::new(host, port),
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let res = self.client.generate(GenerationRequest::new(self.model.clone(), prompt.to_string())).await?;
        Ok(res.response)
    }

    async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        use ollama_rs::generation::embeddings::request::{GenerateEmbeddingsRequest, EmbeddingsInput};
        let res = self.client.generate_embeddings(GenerateEmbeddingsRequest::new(self.model.clone(), EmbeddingsInput::Single(text.to_string()))).await?;
        Ok(res.embeddings.into_iter().next().unwrap_or_default())
    }
}
