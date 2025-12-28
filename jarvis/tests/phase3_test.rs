use jarvis::orchestration::Manager;
use jarvis::providers::LlmProvider;
use jarvis::agents::planning::{ProductOwner, RequirementsEngineer};
use jarvis::agents::development::SeniorDeveloper;
use jarvis::agents::validation::QATester;
use std::sync::Arc;
use anyhow::Result;

#[tokio::test]
async fn test_phase3_handoff_chain() -> Result<()> {
    struct ChainLlm;
    #[async_trait::async_trait]
    impl LlmProvider for ChainLlm {
        async fn generate(&self, prompt: &str) -> Result<String> {
            if prompt.starts_with("Identity: ProductOwner:") {
                Ok("HANDOFF RequirementsEngineer \"Task understood\" \"Plan the feature\"".to_string())
            } else if prompt.starts_with("Identity: RequirementsEngineer:") {
                Ok("HANDOFF SeniorDeveloper \"Plan ready\" \"Implement the feature\"".to_string())
            } else if prompt.starts_with("Identity: SeniorDeveloper:") {
                Ok("HANDOFF QATester \"Code ready\" \"Verify the feature\"".to_string())
            } else if prompt.starts_with("Identity: QATester:") {
                Ok("SUCCESS Feature verified".to_string())
            } else {
                Ok("I am thinking...".to_string())
            }
        }
        async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> { Ok(vec![]) }
    }

    let llm = Arc::new(ChainLlm);
    let mut manager = Manager::new(3);

    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    let re = Arc::new(RequirementsEngineer::new(llm.clone(), vec![]));
    let dev = Arc::new(SeniorDeveloper::new(llm.clone(), vec![]));
    let qa = Arc::new(QATester::new(llm.clone(), vec![]));

    manager.register_agent("ProductOwner".to_string(), po);
    manager.register_agent("RequirementsEngineer".to_string(), re);
    manager.register_agent("SeniorDeveloper".to_string(), dev);
    manager.register_agent("QATester".to_string(), qa);

    let result = manager.run("ProductOwner", "Build a feature".to_string()).await?;
    assert_eq!(result, "Feature verified");

    Ok(())
}
