use colored::Colorize;
use std::process::Command;

/// Helper struct for common command functionality
pub struct CommandHelpers;

impl CommandHelpers {
    /// Print a command header with category
    pub fn print_command_header(title: &str, emoji: &str, category: &str, color: &str) {
        let header_line = "=".repeat(title.len());
        
        match color {
            "blue" => {
                println!("{} {}", emoji, title.blue().bold());
                println!("{}", header_line.blue());
            },
            "green" => {
                println!("{} {}", emoji, title.green().bold());
                println!("{}", header_line.green());
            },
            "yellow" => {
                println!("{} {}", emoji, title.yellow().bold());
                println!("{}", header_line.yellow());
            },
            "cyan" => {
                println!("{} {}", emoji, title.cyan().bold());
                println!("{}", header_line.cyan());
            },
            _ => {
                println!("{} {}", emoji, title.bold());
                println!("{}", header_line);
            }
        }
        
        println!("Category: {}", category);
        println!();
    }
    
    /// Print a success message (green with checkmark)
    pub fn print_success(message: &str) {
        println!("{} {}", "✓".green().bold(), message.green());
    }
    
    /// Print an error message (red with cross)
    pub fn print_error(message: &str) {
        println!("{} {}", "✗".red().bold(), message.red());
    }
    
    /// Print a warning message (yellow with exclamation)
    pub fn print_warning(message: &str) {
        println!("{} {}", "!".yellow().bold(), message.yellow());
    }
    
    /// Print an info message (cyan with arrow)
    pub fn print_info(message: &str) {
        println!("{} {}", "→".cyan(), message);
    }
    
    /// Print a status message (no special formatting)
    pub fn print_status(message: &str) {
        println!("{}", message);
    }
    
    /// Run a command and return the output as a String
    pub fn run_command(cmd: &str, args: &[&str]) -> Result<String, std::io::Error> {
        let output = Command::new(cmd)
            .args(args)
            .output()?;
            
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        Ok(stdout)
    }
    
    /// Check if a command exists in the system PATH
    pub fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}