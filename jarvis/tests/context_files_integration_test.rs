use jarvis::orchestration::Manager;
use jarvis::agents::{Agent, AgentContext, AgentOutput, ContextFile};
use jarvis::providers::mock::MockLlm;
use jarvis::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::fs;
use tempfile::TempDir;

/// Example agent that can analyze context files
struct CodeReviewAgent {
    #[allow(dead_code)]
    llm: Arc<dyn jarvis::providers::LlmProvider>,
}

#[async_trait]
impl Agent for CodeReviewAgent {
    fn identity(&self) -> String { 
        "CodeReviewAgent: I review code from context files and provide feedback".to_string() 
    }
    
    fn capabilities(&self) -> Vec<Arc<dyn Tool>> { 
        vec![] 
    }
    
    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput> {
        // In a real scenario, this would use the LLM to analyze the files
        // For this test, we'll just verify we have access to the files
        
        if context.context_files.is_empty() {
            return Ok(AgentOutput::Error("No files to review".to_string()));
        }
        
        let mut review = String::from("Code Review Summary:\n");
        for file in &context.context_files {
            review.push_str(&format!("- {}: {} lines\n", 
                file.path, 
                file.content.lines().count()));
        }
        
        Ok(AgentOutput::Success(review))
    }
}

#[tokio::test]
async fn test_context_files_integration() -> Result<()> {
    // Create temporary directory with test files
    let temp_dir = TempDir::new()?;
    let file1_path = temp_dir.path().join("auth.rs");
    let file2_path = temp_dir.path().join("models.rs");
    
    fs::write(&file1_path, "pub fn authenticate(token: &str) -> bool {\n    true\n}")?;
    fs::write(&file2_path, "pub struct User {\n    pub id: u32,\n    pub name: String,\n}")?;
    
    // Create context files
    let context_files = vec![
        ContextFile {
            path: file1_path.to_string_lossy().to_string(),
            content: fs::read_to_string(&file1_path)?,
        },
        ContextFile {
            path: file2_path.to_string_lossy().to_string(),
            content: fs::read_to_string(&file2_path)?,
        },
    ];
    
    // Set up manager with agent
    let mut manager = Manager::new(3);
    let llm = Arc::new(MockLlm);
    let agent = Arc::new(CodeReviewAgent { llm });
    manager.register_agent("CodeReviewAgent".to_string(), agent);
    
    // Run with context files
    let result = manager.run_with_session(
        "CodeReviewAgent",
        "Review the authentication and model code".to_string(),
        None,
        context_files,
    ).await?;
    
    // Verify results
    assert!(result.contains("Code Review Summary"));
    assert!(result.contains("auth.rs"));
    assert!(result.contains("models.rs"));
    assert!(result.contains("3 lines") || result.contains("2 lines")); // At least one file should have line count
    
    Ok(())
}

#[test]
fn test_context_file_struct() {
    let cf = ContextFile {
        path: "/tmp/test.rs".to_string(),
        content: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
    };
    
    assert_eq!(cf.path, "/tmp/test.rs");
    assert!(cf.content.contains("println"));
    assert_eq!(cf.content.lines().count(), 3);
}

#[tokio::test]
async fn test_multiple_context_files_with_different_sizes() -> Result<()> {
    let context_files = vec![
        ContextFile {
            path: "small.rs".to_string(),
            content: "let x = 1;".to_string(),
        },
        ContextFile {
            path: "medium.rs".to_string(),
            content: "fn main() {\n    let x = 1;\n    println!(\"{}\", x);\n}".to_string(),
        },
        ContextFile {
            path: "large.rs".to_string(),
            content: (0..100).map(|i| format!("let x{} = {};\n", i, i)).collect::<String>(),
        },
    ];
    
    let mut manager = Manager::new(3);
    let llm = Arc::new(MockLlm);
    let agent = Arc::new(CodeReviewAgent { llm });
    manager.register_agent("CodeReviewAgent".to_string(), agent);
    
    let result = manager.run_with_session(
        "CodeReviewAgent",
        "Review all files".to_string(),
        None,
        context_files,
    ).await?;
    
    assert!(result.contains("small.rs"));
    assert!(result.contains("medium.rs"));
    assert!(result.contains("large.rs"));
    
    Ok(())
}
