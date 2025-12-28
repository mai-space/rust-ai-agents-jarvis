use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;
use anyhow::{Result, anyhow};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelConfig {
    /// Model to use for planning tasks (ProductOwner, RequirementsEngineer)
    pub planning_model: String,
    /// Model to use for code analysis tasks
    pub analysis_model: String,
    /// Model to use for development/coding tasks (SeniorDeveloper)
    pub coding_model: String,
    /// Model to use for documentation/writing tasks (Librarian)
    pub writing_model: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            planning_model: "llama3".to_string(),
            analysis_model: "llama3".to_string(),
            coding_model: "llama3".to_string(),
            writing_model: "llama3".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ollama_host: String,
    pub ollama_port: u16,
    pub model: String,  // Default/fallback model
    pub database_url: Option<String>,
    pub mcp_config: Option<String>,
    pub model_config: Option<ModelConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama_host: "localhost".to_string(),
            ollama_port: 11434,
            model: "llama3".to_string(),
            database_url: None,
            mcp_config: None,
            model_config: Some(ModelConfig::default()),
        }
    }
}

impl Config {
    pub fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "jarvis", "jarvis")
            .ok_or_else(|| anyhow!("Could not find config directory"))?;
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }
        Ok(config_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config {
            ollama_host: "localhost".to_string(),
            ollama_port: 11434,
            model: "llama3".to_string(),
            database_url: Some("postgres://...".to_string()),
            mcp_config: None,
            model_config: Some(ModelConfig::default()),
        };
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.ollama_host, deserialized.ollama_host);
        assert_eq!(config.ollama_port, deserialized.ollama_port);
    }

    #[test]
    fn test_config_path() {
        let path = Config::get_config_path().unwrap();
        assert!(path.to_str().unwrap().contains("jarvis"));
    }
}
