use jarvis::orchestration::Manager;
use jarvis::providers::LlmProvider;
use jarvis::agents::{Agent, AgentContext, AgentOutput, run_llm_agent};
use jarvis::tools::fs::{ReadStructureTool, ListFilesTool};
use jarvis::tools::Tool;
use std::sync::Arc;
use anyhow::Result;

#[tokio::test]
async fn test_phase6_autonomous_multi_step_loop() -> Result<()> {
    struct MultiStepLlm {
        step: std::sync::atomic::AtomicUsize,
    }
    
    #[async_trait::async_trait]
    impl LlmProvider for MultiStepLlm {
        async fn generate(&self, prompt: &str) -> Result<String> {
            let s = self.step.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            match s {
                0 => Ok("CALL read_structure {\"path\": \".\"}".to_string()),
                1 => {
                    assert!(prompt.contains("Tool 'read_structure' result"));
                    Ok("CALL list_files {\"path\": \"src\"}".to_string())
                },
                2 => {
                    assert!(prompt.contains("Tool 'list_files' result"));
                    Ok("SUCCESS All steps completed autonomously".to_string())
                },
                _ => Ok("ERROR Too many steps".to_string()),
            }
        }
        async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> { Ok(vec![]) }
    }

    let llm = Arc::new(MultiStepLlm { step: std::sync::atomic::AtomicUsize::new(0) });
    let mut manager = Manager::new(3);

    struct AutonomousAgent {
        llm: Arc<dyn LlmProvider>,
        tools: Vec<Arc<dyn Tool>>,
    }

    #[async_trait::async_trait]
    impl Agent for AutonomousAgent {
        fn identity(&self) -> String { "AutonomousAgent".to_string() }
        fn capabilities(&self) -> Vec<Arc<dyn Tool>> { self.tools.clone() }
        async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
            run_llm_agent(self, self.llm.clone(), context).await
        }
    }

    let tools: Vec<Arc<dyn Tool>> = vec![
        Arc::new(ReadStructureTool),
        Arc::new(ListFilesTool),
    ];
    let agent = Arc::new(AutonomousAgent { llm: llm.clone(), tools });
    manager.register_agent("AutonomousAgent".to_string(), agent);

    let result = manager.run("AutonomousAgent", "Analyze project structure".to_string()).await?;
    assert_eq!(result, "All steps completed autonomously");

    Ok(())
}
