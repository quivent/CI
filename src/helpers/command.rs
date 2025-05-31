//! Command execution and UI helpers for CI
//!
//! This module provides helper functions for command output formatting,
//! command execution, and standard command operations.

use colored::*;
use std::path::Path;
use std::env;
use std::process::Command;
use std::io::{self, Write};
use anyhow::{Result, Context};
use is_terminal::IsTerminal;

/// Helper functions for common CI command operations and UI
pub struct CommandHelpers;

impl CommandHelpers {
    /// Print a standard command header with category and emoji
    pub fn print_command_header(title: &str, emoji: &str, category: &str, color: &str) {
        let border = "┌─────────────────────────────────────────────────────────────────┐";
        let header = format!("│  {}  {}", emoji, category);
        let bottom = "└─────────────────────────────────────────────────────────────────┘";
        
        match color {
            "cyan" => {
                println!("{}", border.cyan());
                println!("{} {}", "│".cyan(), header.cyan().bold());
                println!("{}", bottom.cyan());
            },
            "green" => {
                println!("{}", border.green());
                println!("{} {}", "│".green(), header.green().bold());
                println!("{}", bottom.green());
            },
            "yellow" => {
                println!("{}", border.yellow());
                println!("{} {}", "│".yellow(), header.yellow().bold());
                println!("{}", bottom.yellow());
            },
            "blue" => {
                println!("{}", border.blue());
                println!("{} {}", "│".blue(), header.blue().bold());
                println!("{}", bottom.blue());
            },
            _ => {
                println!("{}", border);
                println!("{} {}", "│", header.bold());
                println!("{}", bottom);
            }
        }
        
        println!();
        
        // Print the title with the same color
        match color {
            "cyan" => println!("{}", title.cyan().bold()),
            "green" => println!("{}", title.green().bold()),
            "yellow" => println!("{}", title.yellow().bold()),
            "blue" => println!("{}", title.blue().bold()),
            _ => println!("{}", title.bold()),
        }
        
        println!();
    }
    
    /// Print a divider line with equals signs
    pub fn print_divider(color: &str) {
        let line = "═".repeat(70);
        match color {
            "cyan" => println!("{}", line.cyan()),
            "green" => println!("{}", line.green()),
            "yellow" => println!("{}", line.yellow()),
            "blue" => println!("{}", line.blue()),
            _ => println!("{}", line),
        }
        println!();
    }
    
    /// Print a success message with checkmark
    pub fn print_success(message: &str) {
        println!("{} {}", "✓".green().bold(), message.green());
    }
    
    /// Print an error message with cross
    pub fn print_error(message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message.red());
    }
    
    /// Print a warning message with warning symbol
    pub fn print_warning(message: &str) {
        println!("{} {}", "!".yellow().bold(), message.yellow());
    }
    
    /// Print an info message with info symbol
    pub fn print_info(message: &str) {
        println!("{} {}", "ℹ".blue().bold(), message.blue());
    }
    
    /// Print a step message with number and total
    pub fn print_step(step: usize, total: usize, message: &str) {
        // Update window title with step progress FIRST
        if let Ok(command) = env::var("CI_CURRENT_COMMAND") {
            let status = format!("Step {}/{}: {}", step, total, message);
            Self::update_window_title_with_status(&command, &status);
        }
        
        println!("{} {} {}", format!("[{}/{}]", step, total).blue(), "•".yellow(), message);
        println!("{}", "=".repeat(60).blue());
        
        // Add a small delay to make title changes visible
        if env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
    
    /// Print a section header with title and underline
    pub fn print_section(title: &str) {
        println!("\n{}", title.bold());
        println!("{}", "-".repeat(title.len()));
    }
    
    /// Print a status message with bullet point
    pub fn print_status(message: &str) {
        println!("  • {}", message);
    }
    
    /// Print a status check with green checkmark
    pub fn print_status_check(message: &str) {
        println!("{} {}", "✓".green().bold(), message.green());
    }
    
    /// Print a status warning with yellow warning symbol
    pub fn print_status_warning(message: &str) {
        println!("{} {}", "⚠".yellow().bold(), message.yellow());
    }
    
    /// Print a status error with red cross symbol
    pub fn print_status_error(message: &str) {
        println!("{} {}", "✘".red().bold(), message.red());
    }
    
    /// Print a step message with arrow
    pub fn print_arrow_step(message: &str) {
        println!("→ {}", message.cyan());
    }
    
    /// Print a box around text
    pub fn print_box(message: &str, color: &str) {
        let width = message.len() + 4;
        let top = format!("┌{}┐", "─".repeat(width));
        let middle = format!("│  {}  │", message);
        let bottom = format!("└{}┘", "─".repeat(width));
        
        match color {
            "cyan" => {
                println!("{}", top.cyan());
                println!("{}", middle.cyan());
                println!("{}", bottom.cyan());
            },
            "green" => {
                println!("{}", top.green());
                println!("{}", middle.green());
                println!("{}", bottom.green());
            },
            "yellow" => {
                println!("{}", top.yellow());
                println!("{}", middle.yellow());
                println!("{}", bottom.yellow());
            },
            "blue" => {
                println!("{}", top.blue());
                println!("{}", middle.blue());
                println!("{}", bottom.blue());
            },
            _ => {
                println!("{}", top);
                println!("{}", middle);
                println!("{}", bottom);
            }
        }
    }
    
    /// Print a list item with status
    pub fn print_list_item(message: &str, status: Option<&str>) {
        print!("• {}", message);
        
        if let Some(status) = status {
            match status {
                "success" => println!(" {}", "[OK]".green()),
                "failure" => println!(" {}", "[FAILED]".red()),
                "warning" => println!(" {}", "[WARNING]".yellow()),
                "skipped" => println!(" {}", "[SKIPPED]".magenta()),
                _ => println!(" {}", format!("[{}]", status).cyan()),
            }
        } else {
            println!();
        }
    }
    
    /// Execute function with spinner/progress indicator
    pub fn with_progress<F, R>(message: &str, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>
    {
        print!("{}... ", message);
        io::stdout().flush().unwrap();
        
        match f() {
            Ok(result) => {
                println!("{}", "✓".green());
                Ok(result)
            },
            Err(e) => {
                println!("{}", "✗".red());
                Err(e)
            }
        }
    }
    
    /// Run a command and capture output
    pub fn run_command_with_output(command: &str, args: &[&str], dir: Option<&Path>) -> Result<(bool, String, String)> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        
        if let Some(path) = dir {
            cmd.current_dir(path);
        }
        
        let output = cmd.output()
            .with_context(|| format!("Failed to run {}: {}", command, args.join(" ")))?;
            
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        Ok((output.status.success(), stdout, stderr))
    }
    
    /// Run a command with progress display and capture output
    pub async fn run_command_with_progress(
        command: &str, 
        args: &[&str], 
        working_dir: &Path, 
        message: &str
    ) -> Result<String> {
        print!("{}... ", message);
        io::stdout().flush().unwrap();
        
        let output = tokio::process::Command::new(command)
            .args(args)
            .current_dir(working_dir)
            .output()
            .await
            .with_context(|| format!("Failed to run command: {} {}", command, args.join(" ")))?;
        
        if output.status.success() {
            println!("{}", "✓".green());
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            println!("{}", "✗".red());
            let error_message = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow::anyhow!("Command failed: {}", error_message))
        }
    }
    
    /// Prompt user for confirmation
    pub fn prompt_confirmation(message: &str) -> bool {
        print!("{} (y/n): ", message);
        io::stdout().flush().unwrap();
        
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        
        response.trim().to_lowercase() == "y" || response.trim().to_lowercase() == "yes"
    }
    
    /// Prompt user for input with optional default
    pub fn prompt_input(message: &str, default: Option<&str>) -> Result<String> {
        if let Some(def) = default {
            print!("{} [{}]: ", message, def);
        } else {
            print!("{}: ", message);
        }
        io::stdout().flush().unwrap();
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)
            .with_context(|| "Failed to read user input")?;
        
        let trimmed = response.trim();
        if trimmed.is_empty() && default.is_some() {
            Ok(default.unwrap().to_string())
        } else {
            Ok(trimmed.to_string())
        }
    }
    
    /// Format file list for display
    pub fn format_file_list(files: &[String]) -> String {
        files.iter()
            .map(|f| format!("  • {}", f))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Display command help with enhanced formatting
    pub fn display_enhanced_help(command: &str, description: &str, usage: &str, examples: &[&str]) {
        println!("{}", command.bold());
        println!("{}", "=".repeat(command.len()));
        println!();
        println!("{}", description);
        println!();
        println!("{}", "Usage:".bold());
        println!("  {}", usage);
        println!();
        
        if !examples.is_empty() {
            println!("{}", "Examples:".bold());
            for example in examples {
                println!("  {}", example);
            }
            println!();
        }
    }
    
    /// Get current timestamp in human-readable format
    pub fn get_timestamp() -> String {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    /// Check if running in verbose mode
    pub fn is_verbose() -> bool {
        env::var("CI_VERBOSE").unwrap_or_default() == "true"
    }
    
    /// Check if running in debug mode
    pub fn is_debug() -> bool {
        env::var("CI_DEBUG").unwrap_or_default() == "true"
    }
    
    /// Set terminal window title with CI command info
    pub fn set_window_title(command: &str) {
        let is_tty_atty = atty::is(atty::Stream::Stdout);
        let is_tty_is_terminal = io::stdout().is_terminal();
        let force_title = env::var("CI_FORCE_WINDOW_TITLE").unwrap_or_default() == "true";
        
        // Check for common terminal environments that support OSC sequences
        let has_term_env = env::var("TERM").is_ok();
        let in_known_terminal = env::var("TERM_PROGRAM").is_ok() || 
                               env::var("ITERM_SESSION_ID").is_ok() ||
                               env::var("TERMINAL_EMULATOR").is_ok();
        
        // Debug output (only when specifically debugging window titles)
        if Self::is_debug() && env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
            eprintln!("DEBUG: Setting window title to 'CI: {}'", command);
            eprintln!("DEBUG: Is interactive terminal (atty): {}", is_tty_atty);
            eprintln!("DEBUG: Is interactive terminal (is-terminal): {}", is_tty_is_terminal);
            eprintln!("DEBUG: Has TERM env: {}", has_term_env);
            eprintln!("DEBUG: In known terminal: {}", in_known_terminal);
            eprintln!("DEBUG: Force window title: {}", force_title);
        }
        
        // Set title if:
        // 1. We're in an interactive terminal OR
        // 2. We have terminal environment variables OR  
        // 3. It's forced via env var
        if is_tty_atty || is_tty_is_terminal || has_term_env || in_known_terminal || force_title {
            // OSC sequence to set window title: \x1b]0;title\x07
            print!("\x1b]0;CI: {}\x07", command);
            let _ = io::stdout().flush();
            
            if Self::is_debug() && env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
                eprintln!("DEBUG: Window title escape sequence sent");
            }
        } else if Self::is_debug() && env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
            eprintln!("DEBUG: Skipping window title (no terminal detection)");
        }
    }
    
    /// Update window title with status information (state changes)
    pub fn update_window_title_with_status(command: &str, status: &str) {
        let is_tty_atty = atty::is(atty::Stream::Stdout);
        let is_tty_is_terminal = io::stdout().is_terminal();
        let force_title = env::var("CI_FORCE_WINDOW_TITLE").unwrap_or_default() == "true";
        
        // Check for common terminal environments that support OSC sequences
        let has_term_env = env::var("TERM").is_ok();
        let in_known_terminal = env::var("TERM_PROGRAM").is_ok() || 
                               env::var("ITERM_SESSION_ID").is_ok() ||
                               env::var("TERMINAL_EMULATOR").is_ok();
        
        // Set title if terminal supports it
        if is_tty_atty || is_tty_is_terminal || has_term_env || in_known_terminal || force_title {
            // OSC sequence to set window title: \x1b]0;title\x07
            print!("\x1b]0;CI: {} - {}\x07", command, status);
            let _ = io::stdout().flush();
            
            if Self::is_debug() && env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
                eprintln!("DEBUG: Window title updated to 'CI: {} - {}'", command, status);
            }
        }
    }
    
    /// Restore original terminal window title
    pub fn restore_window_title() {
        let is_tty_atty = atty::is(atty::Stream::Stdout);
        let is_tty_is_terminal = io::stdout().is_terminal();
        let force_title = env::var("CI_FORCE_WINDOW_TITLE").unwrap_or_default() == "true";
        
        // Check for common terminal environments that support OSC sequences
        let has_term_env = env::var("TERM").is_ok();
        let in_known_terminal = env::var("TERM_PROGRAM").is_ok() || 
                               env::var("ITERM_SESSION_ID").is_ok() ||
                               env::var("TERMINAL_EMULATOR").is_ok();
        
        // Check if Claude Code or other Node processes might override the title
        let claude_detected = env::var("CLAUDE_SESSION_ID").is_ok() ||
                              env::var("CLAUDE_CLI").is_ok() ||
                              std::process::Command::new("pgrep")
                                  .args(&["-f", "claude.*code"])
                                  .output()
                                  .map(|output| !output.stdout.is_empty())
                                  .unwrap_or(false);
        
        // Debug output (only when specifically debugging window titles)
        if Self::is_debug() && env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
            eprintln!("DEBUG: Restoring window title");
            eprintln!("DEBUG: Is interactive terminal (atty): {}", is_tty_atty);
            eprintln!("DEBUG: Is interactive terminal (is-terminal): {}", is_tty_is_terminal);
            eprintln!("DEBUG: Has TERM env: {}", has_term_env);
            eprintln!("DEBUG: In known terminal: {}", in_known_terminal);
            eprintln!("DEBUG: Force window title: {}", force_title);
            eprintln!("DEBUG: Claude Code detected: {}", claude_detected);
        }
        
        // Restore title using same logic as set_window_title
        if is_tty_atty || is_tty_is_terminal || has_term_env || in_known_terminal || force_title {
            if claude_detected {
                // Don't restore to blank when Claude Code is running - set a meaningful title instead
                print!("\x1b]0;CI - Command Complete\x07");
                if Self::is_debug() && env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
                    eprintln!("DEBUG: Window title set to 'CI - Command Complete' (Claude Code detected)");
                }
            } else {
                // OSC sequence to restore original title
                print!("\x1b]0;\x07");
                if Self::is_debug() && env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
                    eprintln!("DEBUG: Window title restore sequence sent");
                }
            }
            let _ = io::stdout().flush();
        } else if Self::is_debug() && env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
            eprintln!("DEBUG: Skipping window title restore (no terminal detection)");
        }
    }
    
    /// Execute a command with window title override
    pub fn with_window_title<F, R>(command: &str, f: F) -> R
    where
        F: FnOnce() -> R
    {
        Self::set_window_title(command);
        let result = f();
        Self::restore_window_title();
        result
    }
    
    /// Update window title for common status messages
    pub fn update_title_for_status(status: &str) {
        if let Ok(command) = env::var("CI_CURRENT_COMMAND") {
            Self::update_window_title_with_status(&command, status);
        }
    }
    
    /// Print status message and update window title
    pub fn print_status_with_title(message: &str) {
        Self::print_status(message);
        Self::update_title_for_status(message);
    }
    
    /// Print success message and update window title
    pub fn print_success_with_title(message: &str) {
        Self::print_success(message);
        Self::update_title_for_status(&format!("✓ {}", message));
    }
    
    /// Print error message and update window title
    pub fn print_error_with_title(message: &str) {
        Self::print_error(message);
        Self::update_title_for_status(&format!("✗ {}", message));
    }
    
    /// Test window title functionality with visible progress
    pub fn test_window_title_progress() -> anyhow::Result<()> {
        let steps = vec![
            "Initializing system",
            "Loading configuration", 
            "Connecting to services",
            "Processing data",
            "Finalizing operations"
        ];
        
        println!("{}", "Testing Window Title Updates".bold().cyan());
        println!("{}", "Watch your terminal window title!".yellow());
        println!();
        
        for (i, step) in steps.iter().enumerate() {
            Self::print_step(i + 1, steps.len(), step);
            std::thread::sleep(std::time::Duration::from_millis(2000));
        }
        
        Self::print_success_with_title("Window title test completed!");
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_file_list() {
        let files = vec![
            "file1.rs".to_string(),
            "file2.rs".to_string(),
            "file3.rs".to_string(),
        ];
        
        let formatted = CommandHelpers::format_file_list(&files);
        assert_eq!(formatted, "  • file1.rs\n  • file2.rs\n  • file3.rs");
    }
}