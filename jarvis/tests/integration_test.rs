use jarvis::orchestration::Manager;
use jarvis::providers::mock::MockLlm;
use jarvis::agents::planning::{ProductOwner, RequirementsEngineer};
use jarvis::agents::development::SeniorDeveloper;
use jarvis::agents::validation::QATester;
use jarvis::agents::security::SecurityExpert;
use jarvis::agents::documentation::Librarian;
use std::sync::Arc;

#[tokio::test]
async fn test_manager_flow() {
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);

    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    let re = Arc::new(RequirementsEngineer::new(llm.clone()));
    let dev = Arc::new(SeniorDeveloper::new(llm.clone(), vec![]));
    let security = Arc::new(SecurityExpert::new(llm.clone(), vec![]));
    let qa = Arc::new(QATester::new(llm.clone(), vec![]));
    let lib = Arc::new(Librarian::new(llm.clone(), vec![]));

    manager.register_agent("ProductOwner".to_string(), po);
    manager.register_agent("RequirementsEngineer".to_string(), re);
    manager.register_agent("SeniorDeveloper".to_string(), dev);
    manager.register_agent("SecurityExpert".to_string(), security);
    manager.register_agent("QATester".to_string(), qa);
    manager.register_agent("Librarian".to_string(), lib);

    let result = manager.run("ProductOwner", "Build a login page".to_string()).await.unwrap();

    assert!(result.contains("Task finalized by Librarian"));
}

#[tokio::test]
async fn test_manager_retry_limit() {
    let llm = Arc::new(MockLlm);
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
