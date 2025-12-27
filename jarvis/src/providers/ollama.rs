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
    pub fn new(mut host: String, port: u16, model: String) -> Self {
        if !host.contains("://") {
            host = format!("http://{}", host);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_new() {
        // Should not panic
        let _ = OllamaProvider::new("localhost".to_string(), 11434, "llama3".to_string());
        let _ = OllamaProvider::new("http://localhost".to_string(), 11434, "llama3".to_string());
        let _ = OllamaProvider::new("https://example.com".to_string(), 443, "llama3".to_string());
    }
}
