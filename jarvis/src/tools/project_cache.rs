use crate::tools::Tool;
use crate::providers::postgres::PostgresProvider;
use crate::project_context::ProjectMetadata;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::path::Path;
use std::fs;

/// Tool to cache project structure in the database for faster subsequent access
pub struct CacheProjectStructureTool {
    pub pg_provider: Arc<PostgresProvider>,
}

#[async_trait]
impl Tool for CacheProjectStructureTool {
    fn name(&self) -> &str {
        "cache_project_structure"
    }

    fn description(&self) -> &str {
        "Cache the current project's directory structure and metadata in the database. This speeds up future agent operations by avoiding redundant filesystem scans."
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let path_str = input["path"].as_str().unwrap_or(".");
        let path = Path::new(path_str);
        
        // Get or create project metadata
        let mut metadata = ProjectMetadata::from_path(path)?;
        
        // Read the directory structure
        let structure = read_dir_recursive(path)?;
        
        // Cache it in the metadata
        metadata.cache_structure(structure.clone());
        
        // Store the metadata in the database
        let metadata_json = json!({
            "project_id": metadata.project_id,
            "project_name": metadata.project_name,
            "project_path": metadata.project_path,
            "project_type": metadata.project_type,
            "key_files": metadata.key_files,
            "cached_structure": metadata.cached_structure,
            "structure_cached_at": metadata.structure_cached_at,
        });
        
        self.pg_provider.store_project_metadata(&metadata.project_id, metadata_json).await?;
        
        Ok(json!({
            "status": "success",
            "project_id": metadata.project_id,
            "project_name": metadata.project_name,
            "cached_at": metadata.structure_cached_at
        }))
    }
}

/// Tool to retrieve cached project structure from the database
pub struct GetCachedProjectStructureTool {
    pub pg_provider: Arc<PostgresProvider>,
}

#[async_trait]
impl Tool for GetCachedProjectStructureTool {
    fn name(&self) -> &str {
        "get_cached_structure"
    }

    fn description(&self) -> &str {
        "Retrieve the cached project structure from the database if available. This is much faster than scanning the filesystem."
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let path_str = input["path"].as_str().unwrap_or(".");
        let path = Path::new(path_str);
        
        // Get project metadata to find the project_id
        let metadata = ProjectMetadata::from_path(path)?;
        
        // Try to retrieve cached metadata from database
        if let Some(cached_meta) = self.pg_provider.get_project_metadata(&metadata.project_id).await? {
            // Check if cache is still valid (within 5 minutes)
            if let Some(cached_at) = cached_meta["structure_cached_at"].as_i64() {
                let now = chrono::Utc::now().timestamp();
                let age_seconds = now - cached_at;
                
                if age_seconds < 300 { // 5 minutes
                    return Ok(json!({
                        "status": "cache_hit",
                        "project_id": metadata.project_id,
                        "project_name": cached_meta["project_name"],
                        "project_type": cached_meta["project_type"],
                        "key_files": cached_meta["key_files"],
                        "structure": cached_meta["cached_structure"],
                        "cached_at": cached_at,
                        "age_seconds": age_seconds
                    }));
                }
            }
        }
        
        // Cache miss or expired
        Ok(json!({
            "status": "cache_miss",
            "project_id": metadata.project_id,
            "message": "No cached structure available or cache expired. Use 'cache_project_structure' to create cache."
        }))
    }
}

fn read_dir_recursive(path: &Path) -> Result<Value> {
    let mut entries = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().map_err(|_| anyhow::anyhow!("Invalid filename"))?;
            let full_path = entry.path();
            
            if full_path.is_dir() {
                // Skip some common directories to avoid bloat
                if file_name == "target" || file_name == ".git" || file_name == "node_modules" || file_name == "dist" || file_name == "build" {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_dir_recursive_structure() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a simple structure
        fs::create_dir(temp_path.join("src")).unwrap();
        fs::write(temp_path.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(temp_path.join("README.md"), "# Test").unwrap();
        
        let structure = read_dir_recursive(temp_path).unwrap();
        let entries = structure.as_array().unwrap();
        
        assert!(entries.len() >= 2); // At least src and README.md
        
        // Find the src directory
        let src_dir = entries.iter().find(|e| e["name"] == "src").unwrap();
        assert_eq!(src_dir["type"], "directory");
        
        let src_contents = src_dir["contents"].as_array().unwrap();
        assert!(src_contents.iter().any(|e| e["name"] == "main.rs"));
    }
}
