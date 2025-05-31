use anyhow::{Context, Result};
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Command standardization protocol for CI implementations
pub struct CommandStandardization;

impl CommandStandardization {
    /// Standard command file template
    pub const STANDARD_COMMAND_TEMPLATE: &'static str = r#"use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::errors::CIError;
use crate::helpers::CommandHelpers;

/// Create the {command_name} command
pub fn create_command() -> Command {{
    Command::new("{command_name}")
        .about("{command_description}")
        // Add command arguments here
}}

/// Execute the {command_name} command
pub fn execute(matches: &ArgMatches) -> Result<()> {{
    CommandHelpers::print_command_header(
        "{command_description}",
        "{command_icon}",
        "{command_category}",
        "{command_color}"
    );
    
    // Command implementation here
    
    Ok(())
}}

/// Execute the {command_name} command with config (async version)
pub async fn execute_async(config: &Config) -> Result<()> {{
    CommandHelpers::print_command_header(
        "{command_description}",
        "{command_icon}",
        "{command_category}",
        "{command_color}"
    );
    
    // Async command implementation here
    
    Ok(())
}}
"#;

    /// Standard helper function patterns
    pub fn get_standard_helper_patterns() -> HashMap<String, String> {
        let mut patterns = HashMap::new();
        
        patterns.insert("file_operations".to_string(), r#"
/// Standard file operation with error handling
fn perform_file_operation(path: &Path, operation: &str) -> Result<()> {
    // Implementation with CIError handling
    Ok(())
}
"#.to_string());

        patterns.insert("command_header".to_string(), r#"
/// Print standardized command header
CommandHelpers::print_command_header(
    "Command Description",
    "🔧",
    "Category",
    "blue"
);
"#.to_string());

        patterns.insert("progress_indication".to_string(), r#"
/// Standard progress indication
use crate::helpers::{with_spinner, SpinnerResult};

let result = with_spinner("Operation description", || {
    // Operation implementation
    Ok(())
})?;
"#.to_string());

        patterns
    }
    
    /// Analyze command file for standardization violations
    pub fn analyze_command_file(file_path: &Path, content: &str) -> Vec<CommandViolation> {
        let mut violations = Vec::new();
        
        // Check for standard imports
        violations.extend(Self::check_import_patterns(file_path, content));
        
        // Check for standard function signatures
        violations.extend(Self::check_function_patterns(file_path, content));
        
        // Check for error handling patterns
        violations.extend(Self::check_error_handling(file_path, content));
        
        // Check for output formatting
        violations.extend(Self::check_output_formatting(file_path, content));
        
        violations
    }
    
    fn check_import_patterns(file_path: &Path, content: &str) -> Vec<CommandViolation> {
        let mut violations = Vec::new();
        
        // Check for consistent colored import
        if content.contains("use colored::*") {
            violations.push(CommandViolation {
                file: file_path.display().to_string(),
                line: Self::find_line_number(content, "use colored::*"),
                violation_type: CommandViolationType::InconsistentImport,
                description: "Use 'colored::Colorize' instead of 'colored::*' for consistency".to_string(),
            });
        }
        
        // Check for missing CommandHelpers import in command files
        if file_path.to_string_lossy().contains("/commands/") && 
           !content.contains("use crate::helpers::CommandHelpers") {
            violations.push(CommandViolation {
                file: file_path.display().to_string(),
                line: 1,
                violation_type: CommandViolationType::MissingStandardImport,
                description: "Command files should import CommandHelpers for consistent output".to_string(),
            });
        }
        
        violations
    }
    
    fn check_function_patterns(file_path: &Path, content: &str) -> Vec<CommandViolation> {
        let mut violations = Vec::new();
        
        // Check for standard create_command function
        if file_path.to_string_lossy().contains("/commands/") {
            if !content.contains("pub fn create_command() -> Command") {
                violations.push(CommandViolation {
                    file: file_path.display().to_string(),
                    line: 1,
                    violation_type: CommandViolationType::MissingStandardFunction,
                    description: "Command files should have 'pub fn create_command() -> Command'".to_string(),
                });
            }
            
            if !content.contains("pub fn execute(matches: &ArgMatches) -> Result<()>") &&
               !content.contains("pub async fn") {
                violations.push(CommandViolation {
                    file: file_path.display().to_string(),
                    line: 1,
                    violation_type: CommandViolationType::MissingStandardFunction,
                    description: "Command files should have execute function".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_error_handling(file_path: &Path, content: &str) -> Vec<CommandViolation> {
        let mut violations = Vec::new();
        
        // Check for anyhow usage instead of CIError
        if content.contains("anyhow::anyhow!") && file_path.to_string_lossy().contains("/commands/") {
            violations.push(CommandViolation {
                file: file_path.display().to_string(),
                line: Self::find_line_number(content, "anyhow::anyhow!"),
                violation_type: CommandViolationType::InconsistentErrorHandling,
                description: "Use CIError instead of anyhow::anyhow! in command files".to_string(),
            });
        }
        
        violations
    }
    
    fn check_output_formatting(file_path: &Path, content: &str) -> Vec<CommandViolation> {
        let mut violations = Vec::new();
        
        // Check for direct println! usage instead of CommandHelpers
        if content.contains("println!(") && file_path.to_string_lossy().contains("/commands/") {
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if line.contains("println!(") && !line.contains("// Direct output OK") {
                    violations.push(CommandViolation {
                        file: file_path.display().to_string(),
                        line: i + 1,
                        violation_type: CommandViolationType::InconsistentOutput,
                        description: "Consider using CommandHelpers for consistent output formatting".to_string(),
                    });
                    break; // Only report first occurrence
                }
            }
        }
        
        violations
    }
    
    fn find_line_number(content: &str, search_text: &str) -> usize {
        content.lines()
            .enumerate()
            .find(|(_, line)| line.contains(search_text))
            .map(|(i, _)| i + 1)
            .unwrap_or(1)
    }
    
    /// Generate standardized command file
    pub fn generate_standard_command_file(
        command_name: &str,
        command_description: &str,
        command_category: &str,
        command_icon: &str,
        command_color: &str,
    ) -> String {
        Self::STANDARD_COMMAND_TEMPLATE
            .replace("{command_name}", command_name)
            .replace("{command_description}", command_description)
            .replace("{command_category}", command_category)
            .replace("{command_icon}", command_icon)
            .replace("{command_color}", command_color)
    }
    
    /// Apply command standardization to a file
    pub fn standardize_command_file(file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;
        
        let violations = Self::analyze_command_file(file_path, &content);
        
        if violations.is_empty() {
            return Ok(());
        }
        
        println!("{} Standardizing command file: {}", "🔧".cyan(), file_path.display());
        
        for violation in &violations {
            println!("  {} Line {}: {}", "•".yellow(), violation.line, violation.description);
        }
        
        // TODO: Implement automatic fixes for common violations
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandViolation {
    pub file: String,
    pub line: usize,
    pub violation_type: CommandViolationType,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum CommandViolationType {
    InconsistentImport,
    MissingStandardImport,
    MissingStandardFunction,
    InconsistentErrorHandling,
    InconsistentOutput,
    NonStandardPattern,
}

/// Run command standardization across all command files
pub fn standardize_all_commands() -> Result<()> {
    println!("{}", "🔍 Analyzing command patterns across CI codebase...".cyan().bold());
    
    let commands_dir = std::env::current_dir()?.join("src/commands");
    let mut total_violations = 0;
    
    if let Ok(entries) = fs::read_dir(&commands_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(&path)?;
                let violations = CommandStandardization::analyze_command_file(&path, &content);
                
                if !violations.is_empty() {
                    total_violations += violations.len();
                    CommandStandardization::standardize_command_file(&path)?;
                }
            }
        }
    }
    
    // Also check subdirectories
    let visualize_dir = commands_dir.join("visualize");
    if visualize_dir.exists() {
        if let Ok(entries) = fs::read_dir(&visualize_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    let content = fs::read_to_string(&path)?;
                    let violations = CommandStandardization::analyze_command_file(&path, &content);
                    
                    if !violations.is_empty() {
                        total_violations += violations.len();
                        CommandStandardization::standardize_command_file(&path)?;
                    }
                }
            }
        }
    }
    
    if total_violations > 0 {
        println!("{} Found {} command pattern violations across codebase", "⚠".yellow().bold(), total_violations);
    } else {
        println!("{} All command patterns are standardized", "✓".green().bold());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_command_analysis() {
        let content = r#"use colored::*;
use std::fs;

pub fn create_command() -> Command {
    Command::new("test")
}

fn main() {
    println!("Hello");
}
"#;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("commands").join("test.rs");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, content).unwrap();
        
        let violations = CommandStandardization::analyze_command_file(&file_path, content);
        
        // Should detect inconsistent import and missing CommandHelpers
        assert!(!violations.is_empty());
    }
}