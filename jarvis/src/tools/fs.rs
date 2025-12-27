use crate::tools::Tool;
use crate::providers::{LlmProvider, VectorDbProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
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

// Helper function to apply a unified diff patch in a cross-platform way
fn apply_unified_patch(patch_content: &str, base_dir: &Path) -> Result<Vec<String>> {
    // Try to parse as multiple patches first, fall back to single if that fails
    let patches = match patch::Patch::from_multiple(patch_content) {
        Ok(patches) => patches,
        Err(multi_err) => {
            // If multi-patch parsing fails, try single patch
            match patch::Patch::from_single(patch_content) {
                Ok(single) => vec![single],
                Err(single_err) => {
                    return Err(anyhow::anyhow!(
                        "Failed to parse patch. Multi-patch error: {}, Single-patch error: {}",
                        multi_err, single_err
                    ));
                }
            }
        }
    };
    
    let mut applied_files = Vec::new();
    
    for parsed_patch in patches {
        // Get the old and new filenames, stripping the a/ and b/ prefixes (p1 behavior)
        let old_file = parsed_patch.old.path.strip_prefix("a/")
            .or_else(|| parsed_patch.old.path.strip_prefix("b/"))
            .unwrap_or(&parsed_patch.old.path);
        let new_file = parsed_patch.new.path.strip_prefix("b/")
            .or_else(|| parsed_patch.new.path.strip_prefix("a/"))
            .unwrap_or(&parsed_patch.new.path);
        
        // Use the new file path (or old file if new is /dev/null for deletions)
        let file_path = if new_file == "/dev/null" {
            base_dir.join(old_file)
        } else {
            base_dir.join(new_file)
        };
        
        // Handle file creation, deletion, or modification
        if old_file == "/dev/null" {
            // New file creation
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Collect all added lines
            let mut new_content = String::new();
            for hunk in &parsed_patch.hunks {
                for line in &hunk.lines {
                    match line {
                        patch::Line::Add(content) => {
                            new_content.push_str(content);
                            new_content.push('\n');
                        }
                        patch::Line::Context(content) => {
                            new_content.push_str(content);
                            new_content.push('\n');
                        }
                        _ => {}
                    }
                }
            }
            fs::write(&file_path, new_content)?;
            applied_files.push(file_path.display().to_string());
        } else if new_file == "/dev/null" {
            // File deletion
            if file_path.exists() {
                fs::remove_file(&file_path)?;
                applied_files.push(file_path.display().to_string());
            }
        } else {
            // File modification
            let original_content = fs::read_to_string(&file_path)
                .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path.display(), e))?;
            let mut lines: Vec<&str> = original_content.lines().collect();
            
            // Apply each hunk
            for hunk in &parsed_patch.hunks {
                // Validate hunk start position
                if hunk.old_range.start == 0 {
                    return Err(anyhow::anyhow!(
                        "Invalid hunk in file {}: start position cannot be 0",
                        file_path.display()
                    ));
                }
                
                let old_start = (hunk.old_range.start - 1) as usize;
                
                // Collect new lines from the hunk
                let mut new_lines: Vec<&str> = Vec::new();
                let mut expected_old_lines: Vec<&str> = Vec::new();
                
                for line in &hunk.lines {
                    match line {
                        patch::Line::Add(content) => {
                            new_lines.push(content.as_ref());
                        }
                        patch::Line::Remove(content) => {
                            expected_old_lines.push(content.as_ref());
                        }
                        patch::Line::Context(content) => {
                            expected_old_lines.push(content.as_ref());
                            new_lines.push(content.as_ref());
                        }
                    }
                }
                
                // Verify we have enough lines in the file
                if old_start + expected_old_lines.len() > lines.len() {
                    return Err(anyhow::anyhow!(
                        "Patch conflict in file {}: hunk at line {} extends beyond file end (file has {} lines)",
                        file_path.display(),
                        old_start + 1,
                        lines.len()
                    ));
                }
                
                // Verify the old lines match (basic conflict detection)
                let actual_old_lines: Vec<&str> = lines.iter()
                    .skip(old_start)
                    .take(expected_old_lines.len())
                    .copied()
                    .collect();
                
                if actual_old_lines != expected_old_lines {
                    return Err(anyhow::anyhow!(
                        "Patch conflict in file {}: expected lines don't match at line {}",
                        file_path.display(),
                        old_start + 1
                    ));
                }
                
                // Apply the change using the number of expected old lines for consistency
                lines.splice(old_start..old_start + expected_old_lines.len(), new_lines.iter().copied());
            }
            
            // Write the modified content back
            let new_content = lines.join("\n");
            let new_content = if original_content.ends_with('\n') {
                format!("{}\n", new_content)
            } else {
                new_content
            };
            
            fs::write(&file_path, new_content)?;
            applied_files.push(file_path.display().to_string());
        }
    }
    
    Ok(applied_files)
}

#[async_trait]
impl Tool for ApplyPatchTool {
    fn name(&self) -> &str {
        "apply_patch"
    }

    fn description(&self) -> &str {
        "Apply a unified diff patch to the codebase (cross-platform)"
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let patch_content = input["patch"].as_str().ok_or_else(|| anyhow::anyhow!("Patch content is required"))?;
        let cwd = input["cwd"].as_str().unwrap_or(".");
        let base_dir = PathBuf::from(cwd);
        
        match apply_unified_patch(patch_content, &base_dir) {
            Ok(files) => Ok(json!({ 
                "status": "success", 
                "applied_files": files,
                "message": format!("Successfully applied patch to {} file(s)", files.len())
            })),
            Err(e) => Ok(json!({ 
                "status": "error", 
                "message": format!("Failed to apply patch: {}", e)
            }))
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
        
        assert_eq!(result["status"], "success", "Patch failed: {}", result["message"]);
        
        let new_content = fs::read_to_string(&file_path)?;
        assert_eq!(new_content, "Hello Jarvis\n");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_apply_patch_multiline() -> Result<()> {
        let test_dir = setup_test_dir("test_apply_patch_multiline");
        let file_path = test_dir.join("code.rs");
        fs::write(&file_path, "fn main() {\n    println!(\"Hello\");\n}\n")?;

        let patch = 
"--- a/code.rs
+++ b/code.rs
@@ -1,3 +1,4 @@
 fn main() {
-    println!(\"Hello\");
+    println!(\"Hello, World!\");
+    println!(\"Welcome to Jarvis\");
 }
";

        let tool = ApplyPatchTool;
        let result = tool.run(json!({ 
            "patch": patch,
            "cwd": test_dir.to_str().unwrap()
        })).await?;
        
        assert_eq!(result["status"], "success", "Patch failed: {}", result["message"]);
        
        let new_content = fs::read_to_string(&file_path)?;
        assert_eq!(new_content, "fn main() {\n    println!(\"Hello, World!\");\n    println!(\"Welcome to Jarvis\");\n}\n");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_apply_patch_new_file() -> Result<()> {
        let test_dir = setup_test_dir("test_apply_patch_new_file");

        let patch = 
"--- /dev/null
+++ b/newfile.txt
@@ -0,0 +1,2 @@
+This is a new file
+Created by patch
";

        let tool = ApplyPatchTool;
        let result = tool.run(json!({ 
            "patch": patch,
            "cwd": test_dir.to_str().unwrap()
        })).await?;
        
        assert_eq!(result["status"], "success", "Patch failed: {}", result["message"]);
        
        let file_path = test_dir.join("newfile.txt");
        assert!(file_path.exists(), "New file should be created");
        let content = fs::read_to_string(&file_path)?;
        assert_eq!(content, "This is a new file\nCreated by patch\n");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_apply_patch_delete_file() -> Result<()> {
        let test_dir = setup_test_dir("test_apply_patch_delete_file");
        let file_path = test_dir.join("todelete.txt");
        fs::write(&file_path, "This file will be deleted\n")?;

        let patch = 
"--- a/todelete.txt
+++ /dev/null
@@ -1 +0,0 @@
-This file will be deleted
";

        let tool = ApplyPatchTool;
        let result = tool.run(json!({ 
            "patch": patch,
            "cwd": test_dir.to_str().unwrap()
        })).await?;
        
        assert_eq!(result["status"], "success", "Patch failed: {}", result["message"]);
        assert!(!file_path.exists(), "File should be deleted");
        
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
