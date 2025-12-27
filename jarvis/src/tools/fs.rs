use crate::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub struct ListFilesTool;

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files in a directory"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let path_str = input["path"].as_str().unwrap_or(".");
        let path = Path::new(path_str);
        
        let mut files = Vec::new();
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let file_name = entry.file_name().into_string().map_err(|_| anyhow::anyhow!("Invalid filename"))?;
                files.push(file_name);
            }
        }
        
        Ok(json!({ "files": files }))
    }
}

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the content of a file"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let path_str = input["path"].as_str().ok_or_else(|| anyhow::anyhow!("Path is required"))?;
        let content = fs::read_to_string(path_str)?;
        Ok(json!({ "content": content }))
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let path_str = input["path"].as_str().ok_or_else(|| anyhow::anyhow!("Path is required"))?;
        let content = input["content"].as_str().ok_or_else(|| anyhow::anyhow!("Content is required"))?;
        
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, content)?;
        Ok(json!({ "status": "success" }))
    }
}
