//! CI Configuration module
//!
//! This module provides structures and functions for managing CI configuration,
//! stored in .ci-config.json files in project directories.

use anyhow::{anyhow, Context, Result};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::fs;
use chrono;

/// Represents the configuration for a CI project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIConfig {
    /// Project name
    pub project_name: String,
    
    /// Version of CI that created this config
    pub ci_version: String,
    
    /// Timestamp when the config was created
    pub created_at: String,
    
    /// Timestamp when the config was last updated
    pub updated_at: String,
    
    /// List of active agents
    pub active_agents: Vec<String>,
    
    /// Whether fast activation is enabled
    pub fast_activation: bool,
    
    /// Custom project metadata (for extensibility)
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl CIConfig {
    /// Create a new CIConfig with default values
    #[allow(dead_code)]
    pub fn new(project_name: &str) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        
        CIConfig {
            project_name: project_name.to_string(),
            ci_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now.clone(),
            updated_at: now,
            active_agents: vec!["Athena".to_string(), "ProjectArchitect".to_string()],
            fast_activation: true,
            metadata: serde_json::json!({}),
        }
    }
    
    /// Create a new CIConfig with specified values
    pub fn with_options(
        project_name: &str,
        active_agents: Vec<String>,
        fast_activation: bool,
    ) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        
        CIConfig {
            project_name: project_name.to_string(),
            ci_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now.clone(),
            updated_at: now,
            active_agents,
            fast_activation,
            metadata: serde_json::json!({}),
        }
    }
    
    /// Load CIConfig from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
            
        let config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
            
        Ok(config)
    }
    
    /// Load CIConfig from a project directory
    #[allow(dead_code)]
    pub fn from_directory(dir: &Path) -> Result<Self> {
        let config_path = dir.join(".ci-config.json");
        
        if !config_path.exists() {
            return Err(anyhow!("No .ci-config.json found in {}", dir.display()));
        }
        
        Self::from_file(&config_path)
    }
    
    /// Save CIConfig to a file
    pub fn to_file(&mut self, path: &Path) -> Result<()> {
        // Update the updated_at timestamp
        self.updated_at = chrono::Local::now().to_rfc3339();
        
        // Serialize to JSON
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize config to JSON")?;
            
        // Write to file
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
            
        Ok(())
    }
    
    /// Save CIConfig to a project directory
    pub fn to_directory(&mut self, dir: &Path) -> Result<()> {
        let config_path = dir.join(".ci-config.json");
        self.to_file(&config_path)
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), value);
        } else {
            let mut map = serde_json::Map::new();
            map.insert(key.to_string(), value);
            self.metadata = serde_json::Value::Object(map);
        }
    }
    
    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        if let serde_json::Value::Object(ref map) = self.metadata {
            map.get(key)
        } else {
            None
        }
    }
    
    /// Merge configuration with another config (for updates)
    #[allow(dead_code)]
    pub fn merge(&mut self, other: &CIConfig) {
        // Keep the original created_at timestamp
        let created_at = self.created_at.clone();
        
        // Update fields from the other config
        self.project_name = other.project_name.clone();
        self.ci_version = other.ci_version.clone();
        self.updated_at = chrono::Local::now().to_rfc3339();
        self.active_agents = other.active_agents.clone();
        self.fast_activation = other.fast_activation;
        
        // Merge metadata
        if let (serde_json::Value::Object(ref mut self_map), serde_json::Value::Object(ref other_map)) = 
            (&mut self.metadata, &other.metadata) {
            for (key, value) in other_map {
                self_map.insert(key.clone(), value.clone());
            }
        } else {
            self.metadata = other.metadata.clone();
        }
        
        // Restore the original created_at timestamp
        self.created_at = created_at;
    }
}

/// Find the nearest CIConfig file by walking up the directory tree
pub fn find_nearest_config(start_dir: &Path) -> Option<(PathBuf, CIConfig)> {
    let mut current_dir = start_dir.to_path_buf();
    
    loop {
        let config_path = current_dir.join(".ci-config.json");
        
        if config_path.exists() {
            if let Ok(config) = CIConfig::from_file(&config_path) {
                return Some((config_path, config));
            }
        }
        
        // Move up one directory
        if !current_dir.pop() {
            break;
        }
    }
    
    None
}

/// Create a default CIConfig file in a directory
#[allow(dead_code)]
pub fn create_default_config(dir: &Path, project_name: &str) -> Result<CIConfig> {
    let config = CIConfig::new(project_name);
    let config_path = dir.join(".ci-config.json");
    
    let content = serde_json::to_string_pretty(&config)
        .with_context(|| "Failed to serialize config to JSON")?;
        
    fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
        
    Ok(config)
}