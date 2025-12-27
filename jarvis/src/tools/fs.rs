use crate::tools::Tool;
use crate::providers::{LlmProvider, VectorDbProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::sync::Arc;

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
        let cwd = input["cwd"].as_str();
        
        use std::process::Command;
        use std::io::Write;
        
        // Try dry-run first
        let mut dry_run_cmd = Command::new("patch");
        dry_run_cmd.arg("-p1").arg("--dry-run");
        if let Some(c) = cwd {
            dry_run_cmd.current_dir(c);
        }
        
        let mut dry_run_child = dry_run_cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
            
        {
            let stdin = dry_run_child.stdin.as_mut().ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;
            stdin.write_all(patch_content.as_bytes())?;
        }
        
        let dry_run_output = dry_run_child.wait_with_output()?;
        
        if !dry_run_output.status.success() {
            return Ok(json!({
                "status": "conflict",
                "message": "Patch would not apply cleanly",
                "stderr": String::from_utf8_lossy(&dry_run_output.stderr)
            }));
        }

        let mut cmd = Command::new("patch");
        cmd.arg("-p1");
        if let Some(c) = cwd {
            cmd.current_dir(c);
        }

        let mut child = cmd
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

pub struct SearchCodebaseTool {
    pub llm: Arc<dyn LlmProvider>,
    pub vector_db: Arc<dyn VectorDbProvider>,
}

#[async_trait]
impl Tool for SearchCodebaseTool {
    fn name(&self) -> &str {
        "search_codebase"
    }

    fn description(&self) -> &str {
        "Search the codebase using embeddings. Input: { \"query\": \"search query\" }"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let query = input["query"].as_str().ok_or_else(|| anyhow::anyhow!("Query is required"))?;
        
        let embeddings = self.llm.get_embeddings(query).await?;
        let results = self.vector_db.search(embeddings, 5, "project").await?;
        
        Ok(json!({ "results": results }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_dir(name: &str) -> PathBuf {
        let mut path = std::env::current_dir().unwrap();
        path.push("target");
        path.push("test_data");
        path.push(name);
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[tokio::test]
    async fn test_list_files() -> Result<()> {
        let test_dir = setup_test_dir("test_list_files");
        fs::write(test_dir.join("file1.txt"), "hello")?;
        fs::write(test_dir.join("file2.txt"), "world")?;

        let tool = ListFilesTool;
        let result = tool.run(json!({ "path": test_dir.to_str().unwrap() })).await?;
        
        let files = result["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);
        let mut file_names: Vec<_> = files.iter().map(|f| f.as_str().unwrap()).collect();
        file_names.sort();
        assert_eq!(file_names, vec!["file1.txt", "file2.txt"]);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_read_write_file() -> Result<()> {
        let test_dir = setup_test_dir("test_read_write");
        let file_path = test_dir.join("test.txt");
        let content = "Hello, Jarvis!";

        let write_tool = WriteFileTool;
        let write_res = write_tool.run(json!({
            "path": file_path.to_str().unwrap(),
            "content": content
        })).await?;
        assert_eq!(write_res["status"], "success");

        let read_tool = ReadFileTool;
        let read_res = read_tool.run(json!({
            "path": file_path.to_str().unwrap()
        })).await?;
        assert_eq!(read_res["content"], content);

        Ok(())
    }

    #[tokio::test]
    async fn test_read_structure() -> Result<()> {
        let test_dir = setup_test_dir("test_structure");
        fs::create_dir(test_dir.join("src"))?;
        fs::write(test_dir.join("src/main.rs"), "fn main() {}")?;
        fs::write(test_dir.join("README.md"), "# Test")?;

        let tool = ReadStructureTool;
        let result = tool.run(json!({ "path": test_dir.to_str().unwrap() })).await?;
        
        let structure = result["structure"].as_array().unwrap();
        // Should have src (directory) and README.md (file)
        assert_eq!(structure.len(), 2);
        
        let src_entry = structure.iter().find(|e| e["name"] == "src").unwrap();
        assert_eq!(src_entry["type"], "directory");
        assert!(src_entry["contents"].is_array());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_apply_patch() -> Result<()> {
        let test_dir = setup_test_dir("test_apply_patch");
        let file_path = test_dir.join("hello.txt");
        fs::write(&file_path, "Hello World\n")?;

        // Patch with -p1 expects a/filename and b/filename
        // If we set cwd to test_dir, then -p1 will strip 'a' and look for 'filename' in test_dir
        let patch = 
"--- a/hello.txt
+++ b/hello.txt
@@ -1 +1 @@
-Hello World
+Hello Jarvis
";

        let tool = ApplyPatchTool;
        let result = tool.run(json!({ 
            "patch": patch,
            "cwd": test_dir.to_str().unwrap()
        })).await?;
        
        assert_eq!(result["status"], "success", "Patch failed: {}", result["stderr"]);
        
        let new_content = fs::read_to_string(&file_path)?;
        assert_eq!(new_content, "Hello Jarvis\n");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_search_codebase() -> Result<()> {
        use crate::providers::mock::MockLlm;
        
        struct MockDb;
        #[async_trait]
        impl VectorDbProvider for MockDb {
            async fn store(&self, _id: &str, _v: Vec<f32>, _m: Value, _n: &str) -> Result<()> { Ok(()) }
            async fn search(&self, _v: Vec<f32>, _l: usize, _n: &str) -> Result<Vec<Value>> {
                Ok(vec![json!({ "path": "src/main.rs", "content": "fn main() {}" })])
            }
        }

        let tool = SearchCodebaseTool {
            llm: Arc::new(MockLlm),
            vector_db: Arc::new(MockDb),
        };

        let result = tool.run(json!({ "query": "main function" })).await?;
        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["path"], "src/main.rs");

        Ok(())
    }
}
