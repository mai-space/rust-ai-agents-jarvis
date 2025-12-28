/// Tests for context files functionality
/// 
/// Context files allow users to provide specific files as context to agents,
/// enabling more focused and efficient task execution. These tests verify:
/// - Context files are properly passed to agents
/// - Agents can access context file content
/// - Empty context file handling works correctly

use jarvis::orchestration::Manager;
use jarvis::agents::{Agent, AgentContext, AgentOutput, ContextFile};
use jarvis::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Test agent that verifies context files are accessible
struct TestAgent;

#[async_trait]
impl Agent for TestAgent {
    fn identity(&self) -> String { 
        "TestAgent: A test agent that checks for context files".to_string() 
    }
    
    fn capabilities(&self) -> Vec<Arc<dyn Tool>> { 
        vec![] 
    }
    
    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        // Check if context files were provided
        if context.context_files.is_empty() {
            return Ok(AgentOutput::Error("No context files provided".to_string()));
        }
        
        // Verify context files are accessible
        let mut file_info = Vec::new();
        for cf in &context.context_files {
            file_info.push(format!("{}:{}", cf.path, cf.content.len()));
        }
        
        Ok(AgentOutput::Success(format!("Found {} context files: {}", 
            context.context_files.len(), 
            file_info.join(", "))))
    }
}

/// Test that context files are correctly passed to agents
/// 
/// Verifies that:
/// - Multiple context files can be provided
/// - File paths are preserved
/// - File content is accessible to agents
/// - Agent receives correct number of files
#[tokio::test]
async fn test_context_files_passed_to_agent() -> Result<()> {
    let mut manager = Manager::new(3);
    let agent = Arc::new(TestAgent);
    manager.register_agent("TestAgent".to_string(), agent);

    let context_files = vec![
        ContextFile {
            path: "test1.txt".to_string(),
            content: "This is test content 1".to_string(),
        },
        ContextFile {
            path: "test2.txt".to_string(),
            content: "This is test content 2 with more text".to_string(),
        },
    ];

    let (result, _session_id) = manager.run_with_session(
        "TestAgent", 
        "Check context files".to_string(), 
        None,
        context_files,
    ).await?;
    
    assert!(result.contains("Found 2 context files"));
    assert!(result.contains("test1.txt:22"));
    assert!(result.contains("test2.txt:37"));
    
    Ok(())
}

/// Test agent behavior with empty context files
/// 
/// Ensures that agents can handle cases where no context
/// files are provided, returning appropriate errors.
#[tokio::test]
async fn test_empty_context_files() -> Result<()> {
    let mut manager = Manager::new(3);
    let agent = Arc::new(TestAgent);
    manager.register_agent("TestAgent".to_string(), agent);

    let result = manager.run_with_session(
        "TestAgent", 
        "Check context files".to_string(), 
        None,
        vec![],
    ).await;
    
    // Should fail because no context files provided
    assert!(result.is_err() || result.unwrap().0.contains("No context files"));
    
    Ok(())
}

/// Test ContextFile struct creation and field access
/// 
/// Verifies that ContextFile instances can be created
/// and their fields accessed correctly.
#[test]
fn test_context_file_creation() {
    let cf = ContextFile {
        path: "example.rs".to_string(),
        content: "fn main() {}".to_string(),
    };
    
    assert_eq!(cf.path, "example.rs");
    assert_eq!(cf.content, "fn main() {}");
}
