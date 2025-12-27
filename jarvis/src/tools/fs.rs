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

pub struct ReadStructureTool;

#[async_trait]
impl Tool for ReadStructureTool {
    fn name(&self) -> &str {
        "read_structure"
    }

    fn description(&self) -> &str {
        "Recursively list the directory structure"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let path_str = input["path"].as_str().unwrap_or(".");
        let path = Path::new(path_str);
        
        fn read_dir_recursive(path: &Path) -> Result<Value> {
            let mut entries = Vec::new();
            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let file_name = entry.file_name().into_string().map_err(|_| anyhow::anyhow!("Invalid filename"))?;
                    let full_path = entry.path();
                    
                    if full_path.is_dir() {
                        // Skip some common directories to avoid bloat
                        if file_name == "target" || file_name == ".git" || file_name == "node_modules" {
                            continue;
                        }
                        entries.push(json!({
                            "name": file_name,
                            "type": "directory",
                            "contents": read_dir_recursive(&full_path)?
                        }));
                    } else {
                        entries.push(json!({
                            "name": file_name,
                            "type": "file"
                        }));
                    }
                }
            }
            Ok(json!(entries))
        }
        
        let structure = read_dir_recursive(path)?;
        Ok(json!({ "structure": structure }))
    }
}

pub struct ApplyPatchTool;

#[async_trait]
impl Tool for ApplyPatchTool {
    fn name(&self) -> &str {
        "apply_patch"
    }

    fn description(&self) -> &str {
        "Apply a unified diff patch to the codebase"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let patch_content = input["patch"].as_str().ok_or_else(|| anyhow::anyhow!("Patch content is required"))?;
        
        use std::process::Command;
        use std::io::Write;
        
        let mut child = Command::new("patch")
            .arg("-p1")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
            
        {
            let stdin = child.stdin.as_mut().ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;
            stdin.write_all(patch_content.as_bytes())?;
        }
        
        let output = child.wait_with_output()?;
        
        if output.status.success() {
            Ok(json!({ "status": "success", "stdout": String::from_utf8_lossy(&output.stdout) }))
        } else {
            Ok(json!({ "status": "error", "stderr": String::from_utf8_lossy(&output.stderr) }))
        }
    }
}
