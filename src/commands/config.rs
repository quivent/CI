//! Config command for managing CI configuration
//!
//! This module provides functionality for managing CI configuration,
//! including creating, reading, and updating .ci-config.json files.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use colored::Colorize;
use serde_json::Value;

use crate::config::{CIConfig, find_nearest_config};
use crate::helpers::CommandHelpers;

/// Initialize a new CI configuration file
pub async fn init(
    path: &Path,
    project_name: Option<&str>,
    agents: Option<&str>,
    fast_activation: bool,
) -> Result<()> {
    // Get absolute path to target directory
    let target_path = PathBuf::from(path);
    if !target_path.exists() {
        return Err(anyhow!("Error: Target directory '{}' does not exist", path.display()));
    }
    
    // Get project name from directory if not specified
    let project_name = project_name.unwrap_or_else(|| {
        target_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project")
    });
    
    CommandHelpers::print_command_header(
        &format!("Initializing CI configuration for: {}", project_name), 
        "⚙️", 
        "Configuration", 
        "cyan"
    );
    
    // Check if config already exists
    let config_path = target_path.join(".ci-config.json");
    if config_path.exists() {
        return Err(anyhow!("Configuration file already exists at: {}", config_path.display()));
    }
    
    // Parse agents if specified
    let active_agents = if let Some(agents_str) = agents {
        agents_str.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        vec!["Athena".to_string(), "ProjectArchitect".to_string()]
    };
    
    // Create config
    let mut config = CIConfig::with_options(
        project_name,
        active_agents.clone(),
        fast_activation,
    );
    
    // Save config
    config.to_directory(&target_path)?;
    
    CommandHelpers::print_success(&format!("Created CI configuration at: {}", config_path.display()));
    CommandHelpers::print_info(&format!("Project name: {}", project_name));
    CommandHelpers::print_info(&format!("Active agents: {}", active_agents.join(", ")));
    CommandHelpers::print_info(&format!("Fast activation: {}", fast_activation));
    
    Ok(())
}

/// Get a configuration value
pub async fn get(
    path: &Path,
    key: &str,
) -> Result<()> {
    // Try to find config in current or parent directories
    let (config_path, config) = find_nearest_config(path)
        .ok_or_else(|| anyhow!("No CI configuration found in {} or any parent directory", path.display()))?;
    
    CommandHelpers::print_command_header(
        &format!("Get CI configuration value: {}", key), 
        "⚙️", 
        "Configuration", 
        "cyan"
    );
    
    CommandHelpers::print_info(&format!("Using configuration from: {}", config_path.display()));
    
    // Handle special keys
    match key {
        "project_name" => println!("{}", config.project_name),
        "ci_version" => println!("{}", config.ci_version),
        "created_at" => println!("{}", config.created_at),
        "updated_at" => println!("{}", config.updated_at),
        "active_agents" => println!("{}", config.active_agents.join(", ")),
        "fast_activation" => println!("{}", config.fast_activation),
        _ => {
            // Try to get value from metadata
            if let Some(value) = config.get_metadata(key) {
                println!("{}", serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()));
            } else {
                return Err(anyhow!("Configuration key not found: {}", key));
            }
        }
    }
    
    Ok(())
}

/// Set a configuration value
pub async fn set(
    path: &Path,
    key: &str,
    value: &str,
) -> Result<()> {
    // Try to find config in current or parent directories
    let (config_path, mut config) = find_nearest_config(path)
        .ok_or_else(|| anyhow!("No CI configuration found in {} or any parent directory", path.display()))?;
    
    CommandHelpers::print_command_header(
        &format!("Set CI configuration value: {} = {}", key, value), 
        "⚙️", 
        "Configuration", 
        "cyan"
    );
    
    CommandHelpers::print_info(&format!("Using configuration from: {}", config_path.display()));
    
    // Handle special keys
    match key {
        "project_name" => config.project_name = value.to_string(),
        "active_agents" => {
            config.active_agents = value.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        },
        "fast_activation" => {
            config.fast_activation = match value.to_lowercase().as_str() {
                "true" | "yes" | "1" | "on" => true,
                "false" | "no" | "0" | "off" => false,
                _ => return Err(anyhow!("Invalid boolean value for fast_activation: {}", value)),
            };
        },
        "integration_type" => {
            // Validate integration type
            let integration_type = match value.to_lowercase().as_str() {
                "standalone" => "standalone",
                "override" => "override",
                _ => return Err(anyhow!("Invalid integration type: {}. Valid options: standalone, override", value)),
            };
            
            // Set as metadata and also in primary config if applicable
            let json_value = Value::String(integration_type.to_string());
            config.set_metadata(key, json_value);
            
            CommandHelpers::print_info(&format!("Integration type set to: {}", integration_type));
            
            // Update CI method in CLAUDE.md if needed
            let claude_md_path = Path::new(path).join("CLAUDE.md");
            if claude_md_path.exists() {
                CommandHelpers::print_info("Note: Changing integration type in config doesn't automatically update CLAUDE.md");
                CommandHelpers::print_info("To fully change integration method, use 'ci integrate --integration <type>'.");
            }
        },
        _ => {
            // Try to parse value as JSON
            let json_value = match serde_json::from_str::<Value>(value) {
                Ok(v) => v,
                Err(_) => {
                    // If not valid JSON, treat as string
                    Value::String(value.to_string())
                }
            };
            
            // Set value in metadata
            config.set_metadata(key, json_value);
        }
    }
    
    // Save updated config
    config.to_file(&config_path)?;
    
    CommandHelpers::print_success(&format!("Updated configuration value: {} = {}", key, value));
    
    Ok(())
}

/// Show all configuration values
pub async fn show(
    path: &Path,
    format: &str,
) -> Result<()> {
    // Try to find config in current or parent directories
    let (config_path, config) = find_nearest_config(path)
        .ok_or_else(|| anyhow!("No CI configuration found in {} or any parent directory", path.display()))?;
    
    CommandHelpers::print_command_header(
        "Show CI configuration", 
        "⚙️", 
        "Configuration", 
        "cyan"
    );
    
    CommandHelpers::print_info(&format!("Using configuration from: {}", config_path.display()));
    
    // Format output based on requested format
    match format.to_lowercase().as_str() {
        "json" => {
            // Output as JSON
            println!("{}", serde_json::to_string_pretty(&config)?);
        },
        "text" | _ => {
            // Output as human-readable text
            println!("{}: {}", "Project name".bold(), config.project_name);
            println!("{}: {}", "CI version".bold(), config.ci_version);
            println!("{}: {}", "Created at".bold(), config.created_at);
            println!("{}: {}", "Updated at".bold(), config.updated_at);
            println!("{}: {}", "Active agents".bold(), config.active_agents.join(", "));
            println!("{}: {}", "Fast activation".bold(), config.fast_activation);
            
            // Show metadata if any
            if let Value::Object(map) = &config.metadata {
                if !map.is_empty() {
                    println!("\n{}", "Metadata:".bold());
                    for (key, value) in map {
                        println!("  {}: {}", key.bold(), match value {
                            Value::String(s) => s.clone(),
                            _ => serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()),
                        });
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Handle the config command and its subcommands
pub async fn config(
    subcommand: &str,
    path: &Path,
    project_name: Option<&str>,
    agents: Option<&str>,
    fast: Option<bool>,
    key: Option<&str>,
    value: Option<&str>,
    format: Option<&str>,
) -> Result<()> {
    match subcommand {
        "init" => {
            init(path, project_name, agents, fast.unwrap_or(true)).await
        },
        "get" => {
            if let Some(key) = key {
                get(path, key).await
            } else {
                Err(anyhow!("Key parameter is required for get command"))
            }
        },
        "set" => {
            if let (Some(key), Some(value)) = (key, value) {
                set(path, key, value).await
            } else {
                Err(anyhow!("Key and value parameters are required for set command"))
            }
        },
        "show" => {
            show(path, format.unwrap_or("text")).await
        },
        "integration" => {
            // Process integration type configuration
            if let Some(value) = value {
                set(path, "integration_type", value).await
            } else {
                get(path, "integration_type").await
            }
        },
        "agents" => {
            // Process agents configuration
            if let Some(value) = value {
                set(path, "active_agents", value).await
            } else {
                get(path, "active_agents").await
            }
        },
        "project" => {
            // Process project name configuration
            if let Some(value) = value {
                set(path, "project_name", value).await
            } else {
                get(path, "project_name").await
            }
        },
        _ => {
            Err(anyhow!("Unknown config subcommand: {}. Valid options: init, get, set, show, integration, agents, project", subcommand))
        }
    }
}