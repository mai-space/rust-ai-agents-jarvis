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
