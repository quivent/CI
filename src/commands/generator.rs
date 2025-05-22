use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result, anyhow};
use colored::Colorize;

use crate::helpers::CommandHelpers;
use crate::helpers::path::PathHelpers;

/// Generator for creating new commands
pub struct CommandGenerator;

impl CommandGenerator {
    /// Create a new command file
    pub fn create_command(name: &str, description: &str, category: &str) -> Result<()> {
        // Determine the project root directory
        let project_root = Self::find_project_root()?;
        
        // Determine proper casing and filenames
        let snake_case = name.to_lowercase().replace('-', "_");
        let command_file = project_root.join("src/commands").join(format!("{}.rs", snake_case));
        
        // Check if command already exists
        if command_file.exists() {
            return Err(anyhow!("Command '{}' already exists at {}", name, command_file.display()));
        }
        
        // Determine category properties
        let (category_name, emoji, color) = Self::get_category_properties(category);
        
        // Create the command content
        let command_content = Self::generate_command_content(&snake_case, description, category_name, emoji, color);
        
        // Create the command file
        PathHelpers::create_file_with_content(&command_file, &command_content)?;
        
        // Update mod.rs to export the new command
        Self::update_mod_rs(&project_root, &snake_case)?;
        
        // Update main.rs to include the command
        Self::update_main_rs(&project_root, name, &snake_case, description, category_name)?;
        
        // Display success message
        CommandHelpers::print_success(&format!("Created command '{}' in category '{}'", name, category_name));
        CommandHelpers::print_info(&format!("File created at: {}", command_file.display()));
        CommandHelpers::print_info("Files updated:");
        CommandHelpers::print_status("src/commands/mod.rs");
        CommandHelpers::print_status("src/main.rs");
        CommandHelpers::print_info("To build and install:");
        CommandHelpers::print_status("cargo build");
        
        Ok(())
    }
    
    /// Find the project root directory
    fn find_project_root() -> Result<PathBuf> {
        // Check current directory and parents for Cargo.toml
        let mut current_dir = std::env::current_dir()
            .with_context(|| "Failed to get current directory")?;
            
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                return Ok(current_dir);
            }
            
            if !current_dir.pop() {
                break;
            }
        }
        
        Err(anyhow!("Could not find project root (no Cargo.toml found in current directory or parents)"))
    }
    
    /// Get category properties based on category name
    fn get_category_properties(category: &str) -> (&'static str, &'static str, &'static str) {
        match category.to_lowercase().as_str() {
            "intelligence" | "discovery" | "intelligence & discovery" => 
                ("Intelligence & Discovery", "🧠", "blue"),
            "source" | "git" | "source control" => 
                ("Source Control", "📊", "green"),
            "project" | "lifecycle" | "project lifecycle" => 
                ("Project Lifecycle", "🚀", "yellow"),
            "system" | "management" | "system management" => 
                ("System Management", "⚙️", "cyan"),
            _ => 
                ("System Management", "⚙️", "cyan"), // Default
        }
    }
    
    /// Categorize command based on description keywords
    pub fn categorize_command(description: &str) -> &'static str {
        let description = description.to_lowercase();
        
        // Intelligence & Discovery
        if description.contains("agent") || description.contains("memory") || 
           description.contains("intelligence") || description.contains("knowledge") ||
           description.contains("load") || description.contains("list") {
            return "Intelligence & Discovery";
        }
        
        // Source Control
        if description.contains("git") || description.contains("commit") || 
           description.contains("branch") || description.contains("merge") ||
           description.contains("repo") || description.contains("push") ||
           description.contains("pull") {
            return "Source Control";
        }
        
        // Project Lifecycle
        if description.contains("project") || description.contains("init") || 
           description.contains("config") || description.contains("setup") ||
           description.contains("create") || description.contains("new") {
            return "Project Lifecycle";
        }
        
        // Default: System Management
        "System Management"
    }
    
    /// Generate command file content
    fn generate_command_content(name: &str, description: &str, category: &str, emoji: &str, color: &str) -> String {
        format!(
r#"use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::config::Config;
use crate::helpers::CommandHelpers;

pub async fn {name}(__config: &Config) -> Result<()> {{
    CommandHelpers::print_command_header(
        "{description}", 
        "{emoji}", 
        "{category}", 
        "{color}"
    );
    
    // TODO: Implement command logic
    println!("Implementing {name} command...");
    
    CommandHelpers::print_success("{name} command executed successfully");
    
    Ok(())
}}
"#,
            name = name,
            description = description,
            emoji = emoji,
            category = category,
            color = color
        )
    }
    
    /// Update commands/mod.rs to export the new command
    fn update_mod_rs(project_root: &Path, name: &str) -> Result<()> {
        let mod_rs_path = project_root.join("src/commands/mod.rs");
        
        if !mod_rs_path.exists() {
            return Err(anyhow!("Could not find src/commands/mod.rs"));
        }
        
        let mod_rs_content = fs::read_to_string(&mod_rs_path)
            .with_context(|| format!("Failed to read {}", mod_rs_path.display()))?;
            
        // Check if the command is already exported
        if mod_rs_content.contains(&format!("pub mod {};", name)) {
            return Ok(()); // Already exported
        }
        
        // Add the command to mod.rs
        let updated_content = format!("{}\npub mod {};\n", mod_rs_content.trim_end(), name);
        
        fs::write(&mod_rs_path, updated_content)
            .with_context(|| format!("Failed to update {}", mod_rs_path.display()))?;
            
        Ok(())
    }
    
    /// Update main.rs to include the new command
    fn update_main_rs(project_root: &Path, display_name: &str, func_name: &str, description: &str, category: &str) -> Result<()> {
        let main_rs_path = project_root.join("src/main.rs");
        
        if !main_rs_path.exists() {
            return Err(anyhow!("Could not find src/main.rs"));
        }
        
        let main_rs_content = fs::read_to_string(&main_rs_path)
            .with_context(|| format!("Failed to read {}", main_rs_path.display()))?;
            
        // 1. Add command to Commands enum
        let mut lines: Vec<String> = main_rs_content.lines().map(|s| s.to_string()).collect();
        let mut enum_end_index = 0;
        let mut commands_match_start = 0;
        let mut enum_category_comment_index = 0;
        
        // Find the right category comment in the enum
        let category_comment = match category {
            "Intelligence & Discovery" => "// Intelligence & Discovery Commands",
            "Source Control" => "// Source Control Commands",
            "Project Lifecycle" => "// Project Lifecycle Commands",
            "System Management" => "// System Management Commands",
            _ => "// System Management Commands",
        };
        
        // Find the enum Commands { ... } block
        for (i, line) in lines.iter().enumerate() {
            if line.contains("enum Commands {") {
                // Find the right category comment
                for j in i..lines.len() {
                    if lines[j].contains(category_comment) {
                        enum_category_comment_index = j;
                        break;
                    }
                }
                
                // Find the end of the enum block
                for j in i..lines.len() {
                    if line.contains("}") && !line.contains("{") && !line.contains("=>") {
                        enum_end_index = j;
                        break;
                    }
                }
            }
            
            if line.contains("match cli.command.unwrap() {") {
                commands_match_start = i;
            }
        }
        
        // If category comment found, find where to insert the new command
        if enum_category_comment_index > 0 {
            let mut insert_index = enum_category_comment_index;
            // Find the end of the category block (next category comment or end of enum)
            for i in enum_category_comment_index + 1..enum_end_index {
                if lines[i].contains("//") && lines[i].contains("Commands") {
                    insert_index = i - 1;
                    break;
                }
            }
            
            // Insert the new command
            let command_entry = format!("    /// {}\n    {},", description, display_name);
            lines.insert(insert_index + 1, command_entry);
        }
        
        // 2. Add command to match statement
        if commands_match_start > 0 {
            let mut match_insert_index = 0;
            let category_pattern = match category {
                "Intelligence & Discovery" => "Commands::Intent",
                "Source Control" => "Commands::Status",
                "Project Lifecycle" => "Commands::Init",
                "System Management" => "Commands::Evolve",
                _ => "Commands::Version",
            };
            
            // Find a suitable position to insert the command in the match statement
            for i in commands_match_start..lines.len() {
                if lines[i].contains(category_pattern) {
                    // Find the end of this category's match blocks
                    for j in i..lines.len() {
                        if lines[j].contains("},") && !lines[j-1].contains("=>") {
                            match_insert_index = j + 1;
                            break;
                        }
                    }
                    break;
                }
            }
            
            if match_insert_index > 0 {
                let match_entry = format!("        Commands::{} => {{\n            commands::{}::{}(&config).await\n        }},", 
                    display_name, func_name, func_name);
                lines.insert(match_insert_index, match_entry);
            }
        }
        
        // Write updated content
        let updated_content = lines.join("\n");
        fs::write(&main_rs_path, updated_content)
            .with_context(|| format!("Failed to update {}", main_rs_path.display()))?;
            
        Ok(())
    }
    
    /// Build the project after adding a new command
    pub fn build_project(project_root: &Path) -> Result<()> {
        CommandHelpers::print_info("Building project...");
        
        let output = Command::new("cargo")
            .arg("build")
            .current_dir(project_root)
            .output()
            .with_context(|| "Failed to run cargo build")?;
            
        if output.status.success() {
            CommandHelpers::print_success("Build successful");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            CommandHelpers::print_error(&format!("Build failed: {}", stderr));
            Err(anyhow!("Build failed"))
        }
    }
}