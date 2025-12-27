use jarvis::orchestration::Manager;
use jarvis::providers::mock::MockLlm;
use jarvis::agents::refinement::{AccessibilityExpert, SEOExpert};
use jarvis::tools::git::ReadDiffTool;
use jarvis::tools::fs::ApplyPatchTool;
use std::sync::Arc;
use anyhow::Result;

#[tokio::test]
async fn test_phase5_refinement_agents() -> Result<()> {
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);

    // Mock tools
    let diff_tool = Arc::new(ReadDiffTool);
    let patch_tool = Arc::new(ApplyPatchTool);
    let tools = vec![diff_tool as Arc<dyn jarvis::tools::Tool>, patch_tool as Arc<dyn jarvis::tools::Tool>];

    let accessibility = Arc::new(AccessibilityExpert::new(llm.clone(), tools.clone()));
    let seo = Arc::new(SEOExpert::new(llm.clone(), tools.clone()));

    manager.register_agent("AccessibilityExpert".to_string(), accessibility);
    manager.register_agent("SEOExpert".to_string(), seo);

    // Test AccessibilityExpert
    let result = manager.run("AccessibilityExpert", "Check index.html for accessibility".to_string()).await?;
    assert!(result.contains("Default mock response"), "Expected default mock response from AccessibilityExpert");

    // Test SEOExpert
    let result = manager.run("SEOExpert", "Check index.html for SEO".to_string()).await?;
    assert!(result.contains("Default mock response"), "Expected default mock response from SEOExpert");

    Ok(())
}

#[tokio::test]
async fn test_phase5_git_tools_registration() -> Result<()> {
    // This test ensures that the tools can be instantiated and used in agents
    let _llm = Arc::new(MockLlm);
    
    struct ToolTester;
    #[async_trait::async_trait]
    impl jarvis::agents::Agent for ToolTester {
        fn identity(&self) -> String { "Tester".to_string() }
        fn capabilities(&self) -> Vec<Arc<dyn jarvis::tools::Tool>> {
            vec![
                Arc::new(ReadDiffTool),
                Arc::new(ApplyPatchTool),
            ]
        }
        async fn process(&self, context: &mut jarvis::agents::AgentContext) -> Result<jarvis::agents::AgentOutput> {
            jarvis::agents::run_llm_agent(self, Arc::new(MockLlm), context).await
        }
    }

    let mut manager = Manager::new(1);
    manager.register_agent("Tester".to_string(), Arc::new(ToolTester));
    
    let result = manager.run("Tester", "Use git tools".to_string()).await?;
    assert!(result.contains("Default mock response"));
    
    Ok(())
}
