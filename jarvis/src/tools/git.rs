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
