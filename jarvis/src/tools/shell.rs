use crate::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;

pub struct RunTestsTool;

#[async_trait]
impl Tool for RunTestsTool {
    fn name(&self) -> &str {
        "run_tests"
    }

    fn description(&self) -> &str {
        "Run cargo tests in the project. Optional arguments can be passed via 'args' array."
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let mut cmd = Command::new("cargo");
        cmd.arg("test");

        if let Some(args) = input.get("args").and_then(|v| v.as_array()) {
            for arg in args {
                if let Some(s) = arg.as_str() {
                    cmd.arg(s);
                }
            }
        }

        let output = cmd.output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        Ok(json!({
            "success": success,
            "stdout": stdout,
            "stderr": stderr
        }))
    }
}

pub struct StaticAnalysisTool;

#[async_trait]
impl Tool for StaticAnalysisTool {
    fn name(&self) -> &str {
        "static_analysis"
    }

    fn description(&self) -> &str {
        "Run cargo clippy for static analysis. Optional arguments can be passed via 'args' array."
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let mut cmd = Command::new("cargo");
        cmd.arg("clippy");
        cmd.arg("--");
        cmd.arg("-D");
        cmd.arg("warnings");

        if let Some(args) = input.get("args").and_then(|v| v.as_array()) {
            for arg in args {
                if let Some(s) = arg.as_str() {
                    cmd.arg(s);
                }
            }
        }

        let output = cmd.output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        Ok(json!({
            "success": success,
            "stdout": stdout,
            "stderr": stderr
        }))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_tests_tool() -> Result<()> {
        let tool = RunTestsTool;
        let result = tool.run(json!({"args": ["--version"]})).await?;
        assert!(result.get("stdout").is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_static_analysis_tool() -> Result<()> {
        let tool = StaticAnalysisTool;
        let result = tool.run(json!({"args": ["--version"]})).await?;
        assert!(result.get("stdout").is_some());
        Ok(())
    }
}
