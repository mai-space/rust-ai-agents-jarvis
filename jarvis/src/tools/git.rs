use crate::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;

pub struct ReadDiffTool;

#[async_trait]
impl Tool for ReadDiffTool {
    fn name(&self) -> &str {
        "read_diff"
    }

    fn description(&self) -> &str {
        "Read the current git diff of the project"
    }

    async fn run(&self, _input: Value) -> Result<Value> {
        let output = Command::new("git")
            .arg("diff")
            .output()?;
            
        let diff = String::from_utf8_lossy(&output.stdout).to_string();
        
        Ok(json!({ "diff": diff }))
    }
}

pub struct GitCommitTool;

#[async_trait]
impl Tool for GitCommitTool {
    fn name(&self) -> &str {
        "git_commit"
    }

    fn description(&self) -> &str {
        "Commit changes to git. Input: { \"message\": \"commit message\" }"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let message = input["message"].as_str().ok_or_else(|| anyhow::anyhow!("Message is required"))?;
        
        // Add all changes
        Command::new("git").arg("add").arg(".").output()?;
        
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()?;
            
        if output.status.success() {
            Ok(json!({ "status": "success", "stdout": String::from_utf8_lossy(&output.stdout).to_string() }))
        } else {
            Ok(json!({ "status": "error", "stderr": String::from_utf8_lossy(&output.stderr).to_string() }))
        }
    }
}

pub struct GitCheckoutTool;

#[async_trait]
impl Tool for GitCheckoutTool {
    fn name(&self) -> &str {
        "git_checkout"
    }

    fn description(&self) -> &str {
        "Checkout a branch or file. Input: { \"target\": \"branch_or_file\" }"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let target = input["target"].as_str().ok_or_else(|| anyhow::anyhow!("Target is required"))?;
        
        let output = Command::new("git")
            .arg("checkout")
            .arg(target)
            .output()?;
            
        if output.status.success() {
            Ok(json!({ "status": "success", "stdout": String::from_utf8_lossy(&output.stdout).to_string() }))
        } else {
            Ok(json!({ "status": "error", "stderr": String::from_utf8_lossy(&output.stderr).to_string() }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_git_repo(name: &str) -> PathBuf {
        let mut path = std::env::current_dir().unwrap();
        path.push("target");
        path.push("test_git");
        path.push(name);
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }
        fs::create_dir_all(&path).unwrap();
        
        Command::new("git").arg("init").current_dir(&path).output().unwrap();
        Command::new("git").arg("config").arg("user.email").arg("test@example.com").current_dir(&path).output().unwrap();
        Command::new("git").arg("config").arg("user.name").arg("Test User").current_dir(&path).output().unwrap();
        
        path
    }

    #[tokio::test]
    async fn test_read_diff() -> Result<()> {
        let tool = ReadDiffTool;
        let result = tool.run(json!({})).await?;
        assert!(result.get("diff").is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_git_commit_checkout() -> Result<()> {
        let test_dir = setup_test_git_repo("test_git_commit");
        let original_dir = std::env::current_dir()?;
        
        // Change to test dir
        std::env::set_current_dir(&test_dir)?;
        
        fs::write(test_dir.join("test.txt"), "hello git")?;
        
        let commit_tool = GitCommitTool;
        let result = commit_tool.run(json!({ "message": "initial commit" })).await?;
        assert_eq!(result["status"], "success");
        
        // Create a branch
        Command::new("git").arg("checkout").arg("-b").arg("feature").current_dir(&test_dir).output()?;
        fs::write(test_dir.join("feature.txt"), "new feature")?;
        commit_tool.run(json!({ "message": "feature commit" })).await?;
        
        let checkout_tool = GitCheckoutTool;
        let result = checkout_tool.run(json!({ "target": "master" })).await?;
        // Some systems use 'main' instead of 'master' by default now, let's check
        if result["status"] == "error" {
             let result = checkout_tool.run(json!({ "target": "main" })).await?;
             assert_eq!(result["status"], "success");
        } else {
             assert_eq!(result["status"], "success");
        }
        
        // Switch back to original dir
        std::env::set_current_dir(original_dir)?;
        Ok(())
    }
}
