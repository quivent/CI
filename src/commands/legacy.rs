//! Legacy command support for CI compatibility
//!
//! This module provides support for legacy CI commands, ensuring backward
//! compatibility with existing scripts and documentation.

use crate::config::Config;
use crate::helpers::CommandHelpers;
use anyhow::{Result, Context, anyhow};
use std::process::Command;
use std::path::Path;
use std::collections::HashMap;

/// Mapping of legacy commands to their CI equivalents
pub fn get_legacy_command_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    // Legacy CI commands and their CI equivalents
    map.insert("status".to_string(), "status".to_string());
    map.insert("init".to_string(), "init".to_string());
    map.insert("integrate".to_string(), "integrate".to_string());
    map.insert("fix".to_string(), "fix".to_string());
    map.insert("verify".to_string(), "verify".to_string());
    map.insert("agents".to_string(), "agents".to_string());
    map.insert("load".to_string(), "load".to_string());
    map.insert("commit".to_string(), "commit".to_string());
    map.insert("push".to_string(), "deploy".to_string());
    map.insert("evolve".to_string(), "evolve".to_string());
    map.insert("key".to_string(), "key".to_string());
    map.insert("local".to_string(), "local".to_string());
    map.insert("build".to_string(), "build".to_string());
    map.insert("install".to_string(), "install".to_string());
    
    // Legacy batch commands
    map.insert("stage-commit".to_string(), "commit".to_string());
    map.insert("stage-commit-push".to_string(), "deploy".to_string());
    map.insert("update-gitignore".to_string(), "ignore".to_string());
    
    map
}

/// Process a legacy command, mapping it to its CI equivalent
pub async fn process_legacy_command(cmd: &str, args: &[String], _config: &Config) -> Result<()> {
    use colored::Colorize;
    
    let legacy_map = get_legacy_command_map();
    
    if let Some(cir_cmd) = legacy_map.get(cmd) {
        // Print compatibility notice with nicer formatting
        println!("{}", "🔄 Legacy Command Detected".yellow().bold());
        println!("{}", "=======================".yellow());
        println!();
        println!("⚙️  Processing: {} → {}", cmd.cyan().bold(), format!("ci {}", cir_cmd).green().bold());
        println!();
        
        // Build the command string
        let mut command_str = format!("ci {}", cir_cmd);
        for arg in args {
            command_str.push_str(&format!(" {}", arg));
        }
        
        // Execute the command
        let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
        let shell_arg = if cfg!(target_os = "windows") { "/C" } else { "-c" };
        
        println!("📋 Executing: {}", command_str.italic());
        println!();
        
        let output = Command::new(shell)
            .arg(shell_arg)
            .arg(&command_str)
            .output()
            .with_context(|| format!("Failed to execute command: {}", command_str))?;
            
        // Print the command output
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        
        if !output.status.success() {
            println!("{}", "❌ Command execution failed".red().bold());
            return Err(anyhow!("Command failed with status: {}", output.status));
        } else {
            println!("{}", "✅ Command completed successfully".green().bold());
        }
        
        Ok(())
    } else {
        println!("{}", "❌ Unknown Legacy Command".red().bold());
        println!("{}", "=====================".red());
        println!();
        println!("The command '{}' is not recognized as a legacy CI command.", cmd.red().bold());
        println!();
        println!("Run `ci help legacy` to see a list of supported legacy commands.");
        
        Err(anyhow!("Unknown legacy command: {}", cmd))
    }
}

/// Check if a command is a legacy command
pub fn is_legacy_command(cmd: &str) -> bool {
    get_legacy_command_map().contains_key(cmd)
}

/// Get help information for legacy commands
pub fn get_legacy_commands_help() -> String {
    use colored::Colorize;
    
    let mut help = String::new();
    help.push_str(&format!("{}\n", "🔄 Legacy CI Command Support".yellow().bold()));
    help.push_str(&format!("{}\n", "=========================".yellow()));
    help.push_str("\n");
    help.push_str("CI supports the following legacy CI commands for backward compatibility:\n\n");
    
    // Group the commands by category for better organization
    let mut basic_commands = Vec::new();
    let mut lifecycle_commands = Vec::new();
    let mut git_commands = Vec::new();
    let mut system_commands = Vec::new();
    
    for (legacy, ci) in get_legacy_command_map() {
        match legacy.as_str() {
            "status" | "agents" | "local" | "load" => basic_commands.push((legacy, ci)),
            "init" | "integrate" | "verify" | "fix" => lifecycle_commands.push((legacy, ci)),
            "commit" | "push" | "stage-commit" | "stage-commit-push" | "update-gitignore" => 
                git_commands.push((legacy, ci)),
            _ => system_commands.push((legacy, ci)),
        }
    }
    
    help.push_str(&format!("{}\n", "📋 Basic Commands:".cyan().bold()));
    for (legacy, ci) in basic_commands {
        help.push_str(&format!("  {:<15} → {}\n", legacy.cyan(), format!("ci {}", ci).green()));
    }
    
    help.push_str(&format!("\n{}\n", "🔄 Lifecycle Commands:".cyan().bold()));
    for (legacy, ci) in lifecycle_commands {
        help.push_str(&format!("  {:<15} → {}\n", legacy.cyan(), format!("ci {}", ci).green()));
    }
    
    help.push_str(&format!("\n{}\n", "🌿 Git Commands:".cyan().bold()));
    for (legacy, ci) in git_commands {
        help.push_str(&format!("  {:<15} → {}\n", legacy.cyan(), format!("ci {}", ci).green()));
    }
    
    help.push_str(&format!("\n{}\n", "⚙️ System Commands:".cyan().bold()));
    for (legacy, ci) in system_commands {
        help.push_str(&format!("  {:<15} → {}\n", legacy.cyan(), format!("ci {}", ci).green()));
    }
    
    help.push_str("\n");
    help.push_str(&format!("{}\n", "💡 Usage:".green().bold()));
    help.push_str("  Legacy commands can be used directly and will be automatically mapped to their CI equivalents.\n");
    help.push_str("  For example, 'status' will be executed as 'ci status'.\n");
    
    help
}

/// Create symlinks for legacy commands in the specified directory
pub fn create_legacy_command_symlinks(bin_dir: &Path) -> Result<usize> {
    let ci_path = bin_dir.join("ci");
    if !ci_path.exists() {
        return Err(anyhow!("CI binary not found in {}", bin_dir.display()));
    }
    
    let mut count = 0;
    
    for legacy_cmd in get_legacy_command_map().keys() {
        let link_path = bin_dir.join(legacy_cmd);
        
        // Skip if the symlink already exists
        if link_path.exists() {
            continue;
        }
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            match symlink(&ci_path, &link_path) {
                Ok(_) => {
                    count += 1;
                    CommandHelpers::print_info(&format!("Created symlink for legacy command '{}'", legacy_cmd));
                },
                Err(e) => {
                    CommandHelpers::print_warning(&format!("Failed to create symlink for '{}': {}", legacy_cmd, e));
                }
            }
        }
        
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            match symlink_file(&ci_path, &link_path) {
                Ok(_) => {
                    count += 1;
                    CommandHelpers::print_info(&format!("Created symlink for legacy command '{}'", legacy_cmd));
                },
                Err(e) => {
                    CommandHelpers::print_warning(&format!("Failed to create symlink for '{}': {}", legacy_cmd, e));
                }
            }
        }
    }
    
    Ok(count)
}

/// Remove legacy command symlinks from the specified directory
pub fn remove_legacy_command_symlinks(bin_dir: &Path) -> Result<usize> {
    let mut count = 0;
    
    for legacy_cmd in get_legacy_command_map().keys() {
        let link_path = bin_dir.join(legacy_cmd);
        
        // Skip if the symlink doesn't exist
        if !link_path.exists() {
            continue;
        }
        
        // Check if it's a symlink pointing to the CI binary
        #[cfg(unix)]
        let is_cir_symlink = {
            
            match std::fs::metadata(&link_path) {
                Ok(metadata) => metadata.is_file() && metadata.is_symlink(),
                Err(_) => false,
            }
        };
        
        #[cfg(windows)]
        let is_cir_symlink = {
            use std::os::windows::fs::MetadataExt;
            match std::fs::metadata(&link_path) {
                Ok(metadata) => metadata.is_file() && metadata.file_attributes() & 0x400 != 0, // Check for symlink attribute
                Err(_) => false,
            }
        };
        
        if is_cir_symlink {
            match std::fs::remove_file(&link_path) {
                Ok(_) => {
                    count += 1;
                    CommandHelpers::print_info(&format!("Removed symlink for legacy command '{}'", legacy_cmd));
                },
                Err(e) => {
                    CommandHelpers::print_warning(&format!("Failed to remove symlink for '{}': {}", legacy_cmd, e));
                }
            }
        }
    }
    
    Ok(count)
}