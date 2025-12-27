use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use serde_json::Value;
use sha2::{Sha256, Digest};

/// Represents metadata about a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Unique identifier for the project (hash of project path)
    pub project_id: String,
    /// Absolute path to the project
    pub project_path: String,
    /// Project name (derived from directory name)
    pub project_name: String,
    /// Project type (e.g., rust, python, javascript)
    pub project_type: Option<String>,
    /// Cached project structure
    pub cached_structure: Option<Value>,
    /// Timestamp when structure was last cached
    pub structure_cached_at: Option<i64>,
    /// Common file paths (README, config files, etc.)
    pub key_files: Vec<String>,
}

impl ProjectMetadata {
    /// Create a new ProjectMetadata for the given path
    pub fn from_path(path: &Path) -> Result<Self> {
        let absolute_path = fs::canonicalize(path)?;
        let path_str = absolute_path.to_string_lossy().to_string();
        
        // Generate a stable project ID from the path
        let mut hasher = Sha256::new();
        hasher.update(path_str.as_bytes());
        let project_id = format!("{:x}", hasher.finalize());
        
        let project_name = absolute_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let project_type = Self::detect_project_type(&absolute_path);
        let key_files = Self::find_key_files(&absolute_path)?;
        
        Ok(Self {
            project_id,
            project_path: path_str,
            project_name,
            project_type,
            cached_structure: None,
            structure_cached_at: None,
            key_files,
        })
    }
    
    /// Detect the type of project based on files present
    fn detect_project_type(path: &Path) -> Option<String> {
        if path.join("Cargo.toml").exists() {
            Some("rust".to_string())
        } else if path.join("package.json").exists() {
            Some("javascript".to_string())
        } else if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
            Some("python".to_string())
        } else if path.join("go.mod").exists() {
            Some("go".to_string())
        } else if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            Some("java".to_string())
        } else {
            None
        }
    }
    
    /// Find key files in the project
    fn find_key_files(path: &Path) -> Result<Vec<String>> {
        let mut key_files = Vec::new();
        
        let common_files = vec![
            "README.md",
            "README",
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "setup.py",
            "go.mod",
            "pom.xml",
            "build.gradle",
            ".gitignore",
            "LICENSE",
            "Makefile",
        ];
        
        for file in common_files {
            let file_path = path.join(file);
            if file_path.exists() {
                key_files.push(file.to_string());
            }
        }
        
        Ok(key_files)
    }
    
    /// Cache the project structure
    pub fn cache_structure(&mut self, structure: Value) {
        self.cached_structure = Some(structure);
        self.structure_cached_at = Some(chrono::Utc::now().timestamp());
    }
    
    /// Check if cached structure is still valid (less than 5 minutes old)
    pub fn is_structure_cache_valid(&self) -> bool {
        if let (Some(_), Some(cached_at)) = (&self.cached_structure, self.structure_cached_at) {
            let now = chrono::Utc::now().timestamp();
            let age_seconds = now - cached_at;
            age_seconds < 300 // 5 minutes
        } else {
            false
        }
    }
    
    /// Get a summary of the project for context injection
    pub fn get_summary(&self) -> String {
        let mut summary = format!(
            "Project: {} (ID: {})\nPath: {}\n",
            self.project_name, self.project_id, self.project_path
        );
        
        if let Some(ptype) = &self.project_type {
            summary.push_str(&format!("Type: {}\n", ptype));
        }
        
        if !self.key_files.is_empty() {
            summary.push_str(&format!("Key Files: {}\n", self.key_files.join(", ")));
        }
        
        summary
    }
}

/// Manager for project contexts
pub struct ProjectContextManager {
    current_project: Option<ProjectMetadata>,
}

impl ProjectContextManager {
    pub fn new() -> Self {
        Self {
            current_project: None,
        }
    }
    
    /// Initialize project context from current working directory
    pub fn init_from_cwd(&mut self) -> Result<&ProjectMetadata> {
        let cwd = std::env::current_dir()?;
        self.init_from_path(&cwd)
    }
    
    /// Initialize project context from a specific path
    pub fn init_from_path(&mut self, path: &Path) -> Result<&ProjectMetadata> {
        let metadata = ProjectMetadata::from_path(path)?;
        self.current_project = Some(metadata);
        Ok(self.current_project.as_ref().unwrap())
    }
    
    /// Get the current project metadata
    pub fn get_current(&self) -> Option<&ProjectMetadata> {
        self.current_project.as_ref()
    }
    
    /// Get a mutable reference to current project metadata
    pub fn get_current_mut(&mut self) -> Option<&mut ProjectMetadata> {
        self.current_project.as_mut()
    }
}

impl Default for ProjectContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use serde_json::json;
    
    #[test]
    fn test_project_metadata_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create a Cargo.toml to make it a Rust project
        fs::write(project_path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::write(project_path.join("README.md"), "# Test Project").unwrap();
        
        let metadata = ProjectMetadata::from_path(project_path).unwrap();
        
        assert_eq!(metadata.project_type, Some("rust".to_string()));
        assert!(metadata.key_files.contains(&"Cargo.toml".to_string()));
        assert!(metadata.key_files.contains(&"README.md".to_string()));
        assert!(!metadata.project_id.is_empty());
    }
    
    #[test]
    fn test_structure_cache_validity() {
        let temp_dir = TempDir::new().unwrap();
        let mut metadata = ProjectMetadata::from_path(temp_dir.path()).unwrap();
        
        assert!(!metadata.is_structure_cache_valid());
        
        metadata.cache_structure(json!({"files": ["test.rs"]}));
        assert!(metadata.is_structure_cache_valid());
    }
    
    #[test]
    fn test_project_context_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProjectContextManager::new();
        
        assert!(manager.get_current().is_none());
        
        manager.init_from_path(temp_dir.path()).unwrap();
        assert!(manager.get_current().is_some());
        
        let current = manager.get_current().unwrap();
        assert!(!current.project_id.is_empty());
    }
}
