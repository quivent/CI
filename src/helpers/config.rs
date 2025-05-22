//! Configuration management helpers for CI
//!
//! This module provides helper functions for managing CI configuration,
//! including project configuration, API keys, and environment settings.

use std::path::Path;
use std::fs;
use anyhow::{Context, Result, anyhow};

/// Helper functions for configuration management
pub struct ConfigHelpers;

impl ConfigHelpers {
    /// Create or update .env file with CI configuration
    pub fn create_or_update_env_file(
        project_path: &Path,
        ci_repo_path: &Path,
        explicit_path: Option<&str>
    ) -> Result<()> {
        let env_path = project_path.join(".env");
        
        let env_content = if env_path.exists() {
            fs::read_to_string(&env_path)
                .with_context(|| format!("Failed to read .env file at {}", env_path.display()))?
        } else {
            String::new()
        };
        
        let lines: Vec<&str> = env_content.lines().collect();
        
        // Prepare new content
        let mut new_lines = Vec::new();
        let mut added_cir_path = false;
        
        // Process existing content
        for line in lines {
            if line.starts_with("CI_PATH=") || line.starts_with("CI_REPO_PATH=") {
                // Replace existing line
                if let Some(path) = explicit_path {
                    new_lines.push(format!("CI_PATH={}", path));
                } else {
                    new_lines.push(format!("CI_PATH={}", ci_repo_path.display()));
                }
                added_cir_path = true;
            } else {
                // Keep original line
                new_lines.push(line.to_string());
            }
        }
        
        // Add CI_PATH if not found
        if !added_cir_path {
            if let Some(path) = explicit_path {
                new_lines.push(format!("CI_PATH={}", path));
            } else {
                new_lines.push(format!("CI_PATH={}", ci_repo_path.display()));
            }
        }
        
        // Add empty line at the end
        if !new_lines.is_empty() && !new_lines.last().unwrap().is_empty() {
            new_lines.push(String::new());
        }
        
        // Write updated content
        fs::write(&env_path, new_lines.join("\n"))
            .with_context(|| format!("Failed to write .env file at {}", env_path.display()))?;
            
        Ok(())
    }
    
    /// Create CLAUDE.md configuration file for a project
    pub fn create_claude_config(
        project_path: &Path,
        project_name: &str,
        integration_type: &str,
        agents: &[String]
    ) -> Result<()> {
        let claude_path = project_path.join("CLAUDE.md");
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let agents_list = agents
            .iter()
            .map(|a| format!("- {}", a))
            .collect::<Vec<_>>()
            .join("\n");
            
        let content = format!(
            r#"# Project: {}

## Configuration
Created: {}
Integration: {} integration

## Active Agents
{}

## Project Settings
- Use helpers for common operations
- Maintain consistent code style
- Prioritize documentation and comments
- Add tests for new functionality

## Command Execution
- Always use the CI tool for standard operations
- For custom operations, use the helper functions

## Repository Management
- Follow standard Git commit message format
- Keep commit scope focused and specific
- Tag releases with semantic versioning
"#,
            project_name,
            timestamp,
            integration_type,
            agents_list
        );
        
        fs::write(&claude_path, content)
            .with_context(|| format!("Failed to write CLAUDE.md at {}", claude_path.display()))?;
            
        Ok(())
    }
    
    /// Create CLAUDE.local.md file for local CI configuration
    pub fn create_claude_local_config(
        project_path: &Path,
        ci_repo_path: &Path
    ) -> Result<()> {
        let local_config_path = project_path.join("CLAUDE.local.md");
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let content = format!(
            r#"# Local Configuration

## Local Settings
Created: {}
CI Repository: {}

## User Preferences
- Enable command completion
- Show detailed output
- Auto-create backups

## Local Paths
- Use absolute paths for cross-platform compatibility
- Reference user's home directory when possible

## Connection Settings
- Connect to GitHub via HTTPS
- Use token-based authentication
"#,
            timestamp,
            ci_repo_path.display()
        );
        
        fs::write(&local_config_path, content)
            .with_context(|| format!("Failed to write CLAUDE.local.md at {}", local_config_path.display()))?;
            
        Ok(())
    }
    
    /// Check if CLAUDE.md exists and has required sections
    pub fn check_config_version(path: &Path) -> Result<ConfigStatus> {
        let claude_path = path.join("CLAUDE.md");
        
        if !claude_path.exists() {
            return Ok(ConfigStatus {
                needs_update: true,
                missing_sections: vec!["Configuration", "Active Agents", "Project Settings"],
            });
        }
        
        let content = fs::read_to_string(&claude_path)
            .with_context(|| format!("Failed to read CLAUDE.md at {}", claude_path.display()))?;
            
        let mut status = ConfigStatus {
            needs_update: false,
            missing_sections: Vec::new(),
        };
        
        // Check for required sections
        let required_sections = vec![
            "Configuration",
            "Active Agents",
            "Project Settings",
        ];
        
        for section in required_sections {
            if !content.contains(&format!("## {}", section)) {
                status.needs_update = true;
                status.missing_sections.push(section);
            }
        }
        
        Ok(status)
    }
    
    /// Update or create a section in a markdown file
    pub fn update_markdown_section(path: &Path, section_name: &str, content: &str) -> Result<()> {
        let existing_content = if path.exists() {
            fs::read_to_string(path)
                .with_context(|| format!("Failed to read file at {}", path.display()))?
        } else {
            String::new()
        };
        
        let section_header = format!("## {}", section_name);
        let updated_content = if existing_content.contains(&section_header) {
            // Replace existing section
            let lines: Vec<&str> = existing_content.lines().collect();
            let mut new_lines = Vec::new();
            let mut in_section = false;
            let mut _section_replaced = false;
            
            for line in lines {
                if line == section_header {
                    in_section = true;
                    new_lines.push(line);
                    new_lines.push("");
                    new_lines.push(content);
                    new_lines.push("");
                    _section_replaced = true;
                } else if in_section && line.starts_with("##") {
                    in_section = false;
                    new_lines.push(line);
                } else if !in_section {
                    new_lines.push(line);
                }
            }
            
            new_lines.join("\n")
        } else {
            // Append new section
            format!("{}\n\n{}\n{}\n", existing_content.trim_end(), section_header, content)
        };
        
        fs::write(path, &updated_content)
            .with_context(|| format!("Failed to write file at {}", path.display()))?;
            
        Ok(())
    }
    
    /// Add API key to configuration
    pub fn add_api_key(service: &str, key: &str) -> Result<()> {
        // Get config directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("ci");
            
        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory at {}", config_dir.display()))?;
        }
        
        // Create keys file if it doesn't exist
        let keys_file = config_dir.join("keys.json");
        let keys_content = if keys_file.exists() {
            fs::read_to_string(&keys_file)
                .with_context(|| "Failed to read keys file")?
        } else {
            "{}".to_string()
        };
        
        // Parse current keys
        let mut keys: serde_json::Value = serde_json::from_str(&keys_content)
            .with_context(|| "Failed to parse keys file")?;
            
        // Add or update the key
        if let Some(obj) = keys.as_object_mut() {
            obj.insert(service.to_string(), serde_json::Value::String(key.to_string()));
        }
        
        // Write updated keys
        let updated_content = serde_json::to_string_pretty(&keys)
            .with_context(|| "Failed to serialize keys")?;
            
        fs::write(&keys_file, updated_content)
            .with_context(|| "Failed to write keys file")?;
            
        Ok(())
    }
    
    /// Get API key from configuration
    pub fn get_api_key(service: &str) -> Result<Option<String>> {
        // Get config directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("ci");
            
        // Check if keys file exists
        let keys_file = config_dir.join("keys.json");
        if !keys_file.exists() {
            return Ok(None);
        }
        
        // Read keys file
        let keys_content = fs::read_to_string(&keys_file)
            .with_context(|| "Failed to read keys file")?;
            
        // Parse keys
        let keys: serde_json::Value = serde_json::from_str(&keys_content)
            .with_context(|| "Failed to parse keys file")?;
            
        // Get the requested key
        if let Some(obj) = keys.as_object() {
            if let Some(key) = obj.get(service) {
                if let Some(key_str) = key.as_str() {
                    return Ok(Some(key_str.to_string()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// List all stored API keys
    pub fn list_api_keys() -> Result<Vec<String>> {
        // Get config directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("ci");
            
        // Check if keys file exists
        let keys_file = config_dir.join("keys.json");
        if !keys_file.exists() {
            return Ok(Vec::new());
        }
        
        // Read keys file
        let keys_content = fs::read_to_string(&keys_file)
            .with_context(|| "Failed to read keys file")?;
            
        // Parse keys
        let keys: serde_json::Value = serde_json::from_str(&keys_content)
            .with_context(|| "Failed to parse keys file")?;
            
        // Get all key names
        let mut key_names = Vec::new();
        if let Some(obj) = keys.as_object() {
            for key in obj.keys() {
                key_names.push(key.clone());
            }
        }
        
        Ok(key_names)
    }
    
    /// Remove API key from configuration
    pub fn remove_api_key(service: &str) -> Result<bool> {
        // Get config directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("ci");
            
        // Check if keys file exists
        let keys_file = config_dir.join("keys.json");
        if !keys_file.exists() {
            return Ok(false);
        }
        
        // Read keys file
        let keys_content = fs::read_to_string(&keys_file)
            .with_context(|| "Failed to read keys file")?;
            
        // Parse keys
        let mut keys: serde_json::Value = serde_json::from_str(&keys_content)
            .with_context(|| "Failed to parse keys file")?;
            
        // Remove the key
        let mut removed = false;
        if let Some(obj) = keys.as_object_mut() {
            removed = obj.remove(service).is_some();
        }
        
        if removed {
            // Write updated keys
            let updated_content = serde_json::to_string_pretty(&keys)
                .with_context(|| "Failed to serialize keys")?;
                
            fs::write(&keys_file, updated_content)
                .with_context(|| "Failed to write keys file")?;
        }
        
        Ok(removed)
    }
    
    /// Export API keys as shell environment variables
    pub fn export_api_keys() -> Result<String> {
        // Get config directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("ci");
            
        // Check if keys file exists
        let keys_file = config_dir.join("keys.json");
        if !keys_file.exists() {
            return Ok(String::new());
        }
        
        // Read keys file
        let keys_content = fs::read_to_string(&keys_file)
            .with_context(|| "Failed to read keys file")?;
            
        // Parse keys
        let keys: serde_json::Value = serde_json::from_str(&keys_content)
            .with_context(|| "Failed to parse keys file")?;
            
        // Build export statements
        let mut exports = Vec::new();
        if let Some(obj) = keys.as_object() {
            for (service, value) in obj {
                if let Some(key) = value.as_str() {
                    let service_upper = service.to_uppercase();
                    exports.push(format!("export {}_API_KEY=\"{}\"", service_upper, key));
                }
            }
        }
        
        Ok(exports.join("\n"))
    }
    
    /// Load and merge configurations from multiple sources with precedence
    pub fn load_merged_config() -> Result<serde_json::Value> {
        // Start with empty config object
        let mut config = serde_json::json!({});
        
        // 1. Load from system config
        let system_config_path = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("ci/config.json");
            
        if system_config_path.exists() {
            let system_config = fs::read_to_string(&system_config_path)
                .with_context(|| format!("Failed to read system config from {}", system_config_path.display()))?;
                
            let parsed: serde_json::Value = serde_json::from_str(&system_config)
                .with_context(|| "Failed to parse system config")?;
                
            Self::merge_configs(&mut config, &parsed);
        }
        
        // 2. Load from current directory
        let local_config_path = std::env::current_dir()?
            .join(".ci/config.json");
            
        if local_config_path.exists() {
            let local_config = fs::read_to_string(&local_config_path)
                .with_context(|| format!("Failed to read local config from {}", local_config_path.display()))?;
                
            let parsed: serde_json::Value = serde_json::from_str(&local_config)
                .with_context(|| "Failed to parse local config")?;
                
            Self::merge_configs(&mut config, &parsed);
        }
        
        // 3. Load from environment
        // Look for CI_* environment variables
        for (key, value) in std::env::vars() {
            if key.starts_with("CI_") {
                let config_key = key[4..].to_lowercase(); // Remove CI_ prefix
                let parts: Vec<_> = config_key.split('_').collect();
                
                // Build nested structure based on underscore separation
                let mut current = &mut config;
                for (i, part) in parts.iter().enumerate() {
                    if i == parts.len() - 1 {
                        // Last part, set the value
                        current[part] = serde_json::Value::String(value.clone());
                    } else {
                        // Ensure nested object exists
                        if !current.get(part).map_or(false, |v| v.is_object()) {
                            current[part] = serde_json::json!({});
                        }
                        current = current.get_mut(part).unwrap();
                    }
                }
            }
        }
        
        Ok(config)
    }
    
    /// Merge two JSON objects, with the second taking precedence
    fn merge_configs(target: &mut serde_json::Value, source: &serde_json::Value) {
        if let (Some(target_obj), Some(source_obj)) = (target.as_object_mut(), source.as_object()) {
            for (key, value) in source_obj {
                if !target_obj.contains_key(key) {
                    target_obj.insert(key.clone(), value.clone());
                } else if value.is_object() {
                    // Recursively merge objects
                    let mut existing = target_obj.get(key).unwrap().clone();
                    Self::merge_configs(&mut existing, value);
                    target_obj.insert(key.clone(), existing);
                } else {
                    // For non-objects, source overwrites target
                    target_obj.insert(key.clone(), value.clone());
                }
            }
        }
    }
}

/// Structure to hold configuration status information
#[derive(Default)]
pub struct ConfigStatus {
    pub needs_update: bool,
    pub missing_sections: Vec<&'static str>,
}