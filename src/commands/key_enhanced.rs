use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::helpers::CommandHelpers;
use crate::helpers::config::ConfigHelpers;

/// Enhanced API key management with secure storage and configuration
pub async fn key_enhanced(
    command: &Option<KeyCommands>,
    service: Option<&str>,
    key: Option<&str>,
    __config: &Config
) -> Result<()> {
    CommandHelpers::print_command_header(
        "Enhanced API Key Management", 
        "⚙️", 
        "System Management", 
        "cyan"
    );
    
    match command {
        Some(KeyCommands::List) => {
            list_keys().await?;
        },
        Some(KeyCommands::Add) => {
            if let (Some(svc), Some(k)) = (service, key) {
                add_key(svc, k).await?;
            } else {
                // Interactive mode
                let svc = CommandHelpers::prompt_input("Service name", None)?;
                let k = CommandHelpers::prompt_input("API key", None)?;
                add_key(&svc, &k).await?;
            }
        },
        Some(KeyCommands::Remove) => {
            if let Some(svc) = service {
                remove_key(svc).await?;
            } else {
                // Interactive mode
                let svc = CommandHelpers::prompt_input("Service name to remove", None)?;
                remove_key(&svc).await?;
            }
        },
        Some(KeyCommands::Get) => {
            if let Some(svc) = service {
                get_key(svc).await?;
            } else {
                // Interactive mode
                let svc = CommandHelpers::prompt_input("Service name to retrieve", None)?;
                get_key(&svc).await?;
            }
        },
        None => {
            // Default to list
            list_keys().await?;
        }
    }
    
    Ok(())
}

/// List all stored API keys
async fn list_keys() -> Result<()> {
    let keys = ConfigHelpers::list_api_keys()?;
    
    if keys.is_empty() {
        CommandHelpers::print_info("No API keys stored");
        CommandHelpers::print_info("To add a key, use:");
        CommandHelpers::print_status("ci key add <service> <key>");
        return Ok(());
    }
    
    CommandHelpers::print_info("Stored API keys:");
    for key in keys {
        CommandHelpers::print_status(&key);
    }
    
    CommandHelpers::print_info("To retrieve a key, use:");
    CommandHelpers::print_status("ci key get <service>");
    
    Ok(())
}

/// Add a new API key
async fn add_key(service: &str, key: &str) -> Result<()> {
    // Check if key already exists
    if let Ok(Some(_)) = ConfigHelpers::get_api_key(service) {
        CommandHelpers::print_warning(&format!("Key for '{}' already exists", service));
        if !CommandHelpers::prompt_confirmation("Overwrite?") {
            CommandHelpers::print_info("Operation cancelled");
            return Ok(());
        }
    }
    
    // Add the key
    CommandHelpers::with_progress(&format!("Adding key for '{}'", service), || {
        ConfigHelpers::add_api_key(service, key)
    })?;
    
    CommandHelpers::print_success(&format!("API key for '{}' stored successfully", service));
    
    // Provide usage example
    CommandHelpers::print_info("To retrieve this key in your code:");
    CommandHelpers::print_status(&format!("ci key get {}", service));
    
    // Add service-specific instructions
    match service.to_lowercase().as_str() {
        "anthropic" => {
            CommandHelpers::print_info("Anthropic API key usage:");
            CommandHelpers::print_status("Set ANTHROPIC_API_KEY environment variable");
            CommandHelpers::print_status("Use with claude-cpp, Python SDK, or API directly");
        },
        "openai" => {
            CommandHelpers::print_info("OpenAI API key usage:");
            CommandHelpers::print_status("Set OPENAI_API_KEY environment variable");
            CommandHelpers::print_status("Use with Node.js, Python, or other official SDKs");
        },
        "github" => {
            CommandHelpers::print_info("GitHub API key usage:");
            CommandHelpers::print_status("Use with gh CLI: gh auth login --with-token");
            CommandHelpers::print_status("Or set GITHUB_TOKEN environment variable");
        },
        _ => {
            // General instructions
            CommandHelpers::print_info("To use this key in environment variables:");
            CommandHelpers::print_status(&format!("export {}_API_KEY=$(ci key get {})", 
                                       service.to_uppercase(), service));
        }
    }
    
    Ok(())
}

/// Remove an API key
async fn remove_key(service: &str) -> Result<()> {
    // Check if key exists
    if let Ok(None) = ConfigHelpers::get_api_key(service) {
        CommandHelpers::print_error(&format!("No key found for '{}'", service));
        return Ok(());
    }
    
    // Confirm removal
    CommandHelpers::print_warning(&format!("This will permanently remove the API key for '{}'", service));
    if !CommandHelpers::prompt_confirmation("Continue?") {
        CommandHelpers::print_info("Operation cancelled");
        return Ok(());
    }
    
    // Remove the key
    let removed = ConfigHelpers::remove_api_key(service)?;
    
    if removed {
        CommandHelpers::print_success(&format!("API key for '{}' removed successfully", service));
    } else {
        CommandHelpers::print_warning(&format!("No key found for '{}'", service));
    }
    
    Ok(())
}

/// Get an API key
async fn get_key(service: &str) -> Result<()> {
    match ConfigHelpers::get_api_key(service)? {
        Some(key) => {
            // Just print the key without any formatting for easy use in scripts
            println!("{}", key);
        },
        None => {
            CommandHelpers::print_error(&format!("No key found for '{}'", service));
        }
    }
    
    Ok(())
}

/// Key management subcommands
#[derive(Debug, Clone)]
pub enum KeyCommands {
    /// List all stored API keys
    List,
    /// Add a new API key
    Add,
    /// Remove an API key
    Remove,
    /// Get a specific API key
    Get,
}