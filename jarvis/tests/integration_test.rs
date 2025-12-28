use jarvis::orchestration::Manager;
use jarvis::providers::mock::MockLlm;
use jarvis::agents::planning::{ProductOwner, RequirementsEngineer};
use jarvis::agents::development::SeniorDeveloper;
use jarvis::agents::refinement::{AccessibilityExpert, SEOExpert};
use jarvis::agents::validation::QATester;
use jarvis::agents::security::SecurityExpert;
use jarvis::agents::documentation::Librarian;
use std::sync::Arc;

#[tokio::test]
async fn test_manager_flow() {
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);

    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    let re = Arc::new(RequirementsEngineer::new(llm.clone(), vec![]));
    let dev = Arc::new(SeniorDeveloper::new(llm.clone(), vec![]));
    let accessibility = Arc::new(AccessibilityExpert::new(llm.clone(), vec![]));
    let seo = Arc::new(SEOExpert::new(llm.clone(), vec![]));
    let security = Arc::new(SecurityExpert::new(llm.clone(), vec![]));
    let qa = Arc::new(QATester::new(llm.clone(), vec![]));
    let lib = Arc::new(Librarian::new(llm.clone(), vec![]));

    manager.register_agent("ProductOwner".to_string(), po);
    manager.register_agent("RequirementsEngineer".to_string(), re);
    manager.register_agent("SeniorDeveloper".to_string(), dev);
    manager.register_agent("AccessibilityExpert".to_string(), accessibility);
    manager.register_agent("SEOExpert".to_string(), seo);
    manager.register_agent("SecurityExpert".to_string(), security);
    manager.register_agent("QATester".to_string(), qa);
    manager.register_agent("Librarian".to_string(), lib);

    let result = manager.run("ProductOwner", "Build a login page".to_string()).await.unwrap();

    assert!(result.contains("Task finalized by Librarian"));
}

#[tokio::test]
async fn test_manager_retry_limit() {
    let _llm = Arc::new(MockLlm);
    let mut manager = Manager::new(1); // Set low retry limit

    // Mock agent that always hands off to itself (loop)
    struct LoopAgent;
    #[async_trait::async_trait]
    impl jarvis::agents::Agent for LoopAgent {
        fn identity(&self) -> String { "Loop".to_string() }
        fn capabilities(&self) -> Vec<Arc<dyn jarvis::tools::Tool>> { vec![] }
        async fn process(&self, _context: &mut jarvis::agents::AgentContext) -> anyhow::Result<jarvis::agents::AgentOutput> {
            Ok(jarvis::agents::AgentOutput::Handoff {
                target: "Loop".to_string(),
                reason: "Looping".to_string(),
                context: "Looping".to_string(),
            })
        }
    }

    manager.register_agent("Loop".to_string(), Arc::new(LoopAgent));

    let result = manager.run("Loop", "Start".to_string()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Max retries reached"));
}

#[tokio::test]
async fn test_agent_tool_calling() {
    struct ToolLlm;
    #[async_trait::async_trait]
    impl jarvis::providers::LlmProvider for ToolLlm {
        async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
            if prompt.contains("Tool 'list_files' result") {
                Ok("SUCCESS Tool worked".to_string())
            } else {
                Ok("CALL list_files {\"path\": \".\"}".to_string())
            }
        }
        async fn get_embeddings(&self, _text: &str) -> anyhow::Result<Vec<f32>> { Ok(vec![]) }
    }

    let llm = Arc::new(ToolLlm);
    let mut manager = Manager::new(3);

    let tools = vec![Arc::new(jarvis::tools::fs::ListFilesTool) as Arc<dyn jarvis::tools::Tool>];
    let agent = Arc::new(ProductOwner::new(llm.clone(), tools));
    manager.register_agent("PO".to_string(), agent);

    let result = manager.run("PO", "list files".to_string()).await.unwrap();

    assert_eq!(result, "Tool worked");
}

#[tokio::test]
async fn test_manager_hitl() {
    let _llm = Arc::new(MockLlm);
    
    struct MockHitl;
    impl jarvis::orchestration::HumanInTheLoop for MockHitl {
        fn consult(&self, _agent: &str, _task: &str, _history: &[String]) -> anyhow::Result<String> {
            Ok("human instruction".to_string())
        }
    }

    struct LoopAgent;
    #[async_trait::async_trait]
    impl jarvis::agents::Agent for LoopAgent {
        fn identity(&self) -> String { "Loop".to_string() }
        fn capabilities(&self) -> Vec<Arc<dyn jarvis::tools::Tool>> { vec![] }
        async fn process(&self, context: &mut jarvis::agents::AgentContext) -> anyhow::Result<jarvis::agents::AgentOutput> {
            if context.task.contains("human instruction") {
                Ok(jarvis::agents::AgentOutput::Success("Recovered".to_string()))
            } else {
                Ok(jarvis::agents::AgentOutput::Handoff {
                    target: "Loop".to_string(),
                    reason: "Looping".to_string(),
                    context: "Looping".to_string(),
                })
            }
        }
    }

    let mut manager = Manager::new(1).with_hitl(Arc::new(MockHitl));
    manager.register_agent("Loop".to_string(), Arc::new(LoopAgent));

    let result = manager.run("Loop", "Start".to_string()).await.unwrap();

    assert_eq!(result, "Recovered");
}
