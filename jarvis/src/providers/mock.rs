use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;

pub struct MockLlm;

#[async_trait]
impl LlmProvider for MockLlm {
    async fn generate(&self, prompt: &str) -> Result<String> {
        if prompt.contains("Product Owner") {
            Ok("I have analyzed the request and files.".to_string())
        } else if prompt.contains("Requirements Engineer") {
            Ok("1. Do this. 2. Do that.".to_string())
        } else if prompt.contains("Senior Developer") {
            Ok("I have implemented the plan.".to_string())
        } else if prompt.contains("QA Tester") {
            Ok("Everything looks good.".to_string())
        } else if prompt.contains("Security Expert") {
            Ok("No vulnerabilities found.".to_string())
        } else if prompt.contains("Librarian") {
            Ok("Documentation updated.".to_string())
        } else {
            Ok("Default mock response".to_string())
        }
    }

    async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }
}
