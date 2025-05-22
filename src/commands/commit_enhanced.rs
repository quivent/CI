use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::{self, Write};

use crate::config::Config;
use crate::helpers::CommandHelpers;
use crate::helpers::repository::RepositoryHelpers;
use crate::helpers::path::PathHelpers;

/// Enhanced commit with intelligent message generation
pub async fn commit_enhanced(
    message: Option<&str>,
    analyze: bool,
    edit: bool,
    __config: &Config
) -> Result<()> {
    CommandHelpers::print_command_header(
        "Create enhanced commit", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    // Get current directory
    let current_dir = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
    
    // Check if it's a git repository
    if !RepositoryHelpers::is_inside_git_repo(&current_dir) {
        CommandHelpers::print_error("Not in a git repository");
        return Err(anyhow::anyhow!("Not in a git repository"));
    }
    
    // Check for uncommitted changes
    let status = RepositoryHelpers::get_repository_status(&current_dir)?;
    if !status.has_uncommitted_changes {
        CommandHelpers::print_warning("No changes to commit");
        return Ok(());
    }
    
    // Run gitignore update if needed
    CommandHelpers::with_progress("Updating .gitignore", || {
        let patterns = [
            "CLAUDE.local.md",
            ".ci/",
            "*.log",
            ".DS_Store",
            "Thumbs.db"
        ];
        let updated = RepositoryHelpers::update_gitignore(&current_dir, &patterns)?;
        if updated {
            CommandHelpers::print_info("Updated .gitignore with CI patterns");
        }
        Ok(())
    })?;
    
    // Stage changes if requested (default behavior)
    CommandHelpers::with_progress("Staging files", || {
        // Stage all files to include in this commit
        let output = Command::new("git")
            .args(["add", "."])
            .current_dir(&current_dir)
            .output()
            .with_context(|| "Failed to run git add")?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to stage files: {}", stderr));
        }
        
        Ok(())
    })?;
    
    // Get list of staged files
    let staged_files = RepositoryHelpers::get_staged_files(&current_dir)?;
    
    if staged_files.is_empty() {
        CommandHelpers::print_warning("No files staged for commit");
        return Ok(());
    }
    
    CommandHelpers::print_info("Staged files:");
    for file in &staged_files {
        CommandHelpers::print_status(file);
    }
    
    // Get changes to be committed
    CommandHelpers::print_info("Changes to be committed:");
    let output = Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .current_dir(&current_dir)
        .output()
        .with_context(|| "Failed to run git diff")?;
        
    println!("{}", String::from_utf8_lossy(&output.stdout));
    
    // Generate or use commit message
    let commit_msg = if analyze {
        // Generate commit message by analyzing changes
        CommandHelpers::with_progress("Analyzing changes for commit message", || {
            generate_commit_message(&current_dir)
        })?
    } else if let Some(msg) = message {
        msg.to_string()
    } else {
        // Prompt for a commit message
        CommandHelpers::print_info("Enter commit message:");
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .with_context(|| "Failed to read commit message")?;
        input.trim().to_string()
    };
    
    // Check if the commit message is empty
    if commit_msg.trim().is_empty() {
        CommandHelpers::print_error("Commit message cannot be empty");
        return Err(anyhow::anyhow!("Commit message cannot be empty"));
    }
    
    // Allow editing the commit message if requested
    let final_message = if edit {
        edit_commit_message(&commit_msg)?
    } else {
        commit_msg
    };
    
    // Show the final commit message
    CommandHelpers::print_info("Commit message:");
    println!("{}", final_message);
    
    // Confirm commit
    if !CommandHelpers::prompt_confirmation("Create commit with this message?") {
        CommandHelpers::print_info("Commit cancelled");
        return Ok(());
    }
    
    // Create the commit
    CommandHelpers::with_progress("Creating commit", || {
        RepositoryHelpers::create_commit(&current_dir, &final_message)?;
        Ok(())
    })?;
    
    // Show success message
    CommandHelpers::print_success("Commit created successfully");
    
    // Show current branch status
    let status = RepositoryHelpers::get_repository_status(&current_dir)?;
    if let Some(branch) = &status.current_branch {
        CommandHelpers::print_info(&format!("Branch: {}", branch));
        
        if status.has_remote && status.commits_ahead > 0 {
            CommandHelpers::print_info(&format!("Your branch is ahead by {} commit(s)", status.commits_ahead));
            CommandHelpers::print_info("To push these changes, run:");
            CommandHelpers::print_status(&format!("git push origin {}", branch));
            // Or use our deploy command
            CommandHelpers::print_status("ci deploy");
        }
    }
    
    Ok(())
}

/// Generate a commit message by analyzing the changes
fn generate_commit_message(repo_dir: &Path) -> Result<String> {
    // Get the diff of the staged changes
    let diffoutput = Command::new("git")
        .args(["diff", "--cached"])
        .current_dir(repo_dir)
        .output()
        .with_context(|| "Failed to run git diff")?;
        
    let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();
    
    // Get list of files changed
    let filesoutput = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo_dir)
        .output()
        .with_context(|| "Failed to get changed files")?;
        
    let files = String::from_utf8_lossy(&files_output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    
    // Analyze the changes to generate a commit message
    let mut message = String::new();
    
    // Check for common patterns
    let mut add_count = 0;
    let mut modify_count = 0;
    let mut delete_count = 0;
    let mut doc_count = 0;
    let mut code_count = 0;
    let mut test_count = 0;
    
    // Count file types and operations
    for file in &files {
        // Check file extension
        if file.ends_with(".md") || file.ends_with(".txt") || file.ends_with(".rst") {
            doc_count += 1;
        } else if file.ends_with(".test.js") || file.ends_with("_test.go") || 
                  file.contains("test") || file.contains("spec") {
            test_count += 1;
        } else {
            code_count += 1;
        }
        
        // Check if file is new, modified, or deleted
        let file_status = Command::new("git")
            .args(["status", "--porcelain", file])
            .current_dir(repo_dir)
            .output()
            .with_context(|| format!("Failed to get status for file: {}", file))?;
            
        let status_line = String::from_utf8_lossy(&file_status.stdout);
        if status_line.starts_with("A ") || status_line.starts_with("?? ") {
            add_count += 1;
        } else if status_line.starts_with("M ") {
            modify_count += 1;
        } else if status_line.starts_with("D ") {
            delete_count += 1;
        }
    }
    
    // Check for patterns in changes
    if diff.contains("fix") || diff.contains("bug") || diff.contains("issue") || diff.contains("error") {
        message.push_str("Fix: ");
    } else if add_count > 0 && modify_count == 0 && delete_count == 0 {
        message.push_str("Add: ");
    } else if add_count == 0 && delete_count > 0 && modify_count == 0 {
        message.push_str("Remove: ");
    } else if modify_count > 0 {
        message.push_str("Update: ");
    } else if test_count > 0 && test_count > code_count {
        message.push_str("Test: ");
    } else if doc_count > 0 && doc_count > code_count {
        message.push_str("Docs: ");
    } else {
        message.push_str("Change: ");
    }
    
    // Add description based on files changed
    if files.len() == 1 {
        // Single file change
        message.push_str(&format!("{}", Path::new(&files[0]).file_name().unwrap().to_string_lossy()));
    } else if files.len() <= 3 {
        // Few files changed - list them
        let file_names: Vec<_> = files.iter()
            .map(|f| Path::new(f).file_name().unwrap().to_string_lossy().to_string())
            .collect();
        message.push_str(&file_names.join(", "));
    } else {
        // Many files changed - group by directory
        let mut dirs = std::collections::HashMap::new();
        for file in &files {
            let parent = Path::new(file).parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            *dirs.entry(parent).or_insert(0) += 1;
        }
        
        // Get directories with the most files
        let mut dir_counts: Vec<_> = dirs.into_iter().collect();
        dir_counts.sort_by(|a, b| b.1.cmp(&a.1));
        
        if dir_counts.len() == 1 {
            // All files in the same directory
            message.push_str(&format!("files in {}", dir_counts[0].0));
        } else {
            // Multiple directories
            message.push_str(&format!("files in {} directories", dir_counts.len()));
        }
    }
    
    Ok(message)
}

/// Edit the commit message in the user's editor
fn edit_commit_message(initial_message: &str) -> Result<String> {
    // Create a temporary file for editing
    let mut temp_file = NamedTempFile::new()
        .with_context(|| "Failed to create temporary file for commit message")?;
        
    // Write the initial message to the file
    writeln!(temp_file, "{}", initial_message)
        .with_context(|| "Failed to write to temporary file")?;
        
    // Get the editor from git config or environment
    let editor = Command::new("git")
        .args(["config", "--get", "core.editor"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .or_else(|| std::env::var("EDITOR").ok())
        .or_else(|| std::env::var("VISUAL").ok())
        .unwrap_or_else(|| {
            if PathHelpers::command_exists("vim") {
                "vim".to_string()
            } else if PathHelpers::command_exists("nano") {
                "nano".to_string()
            } else {
                "vi".to_string()
            }
        });
    
    // Open the editor
    let status = Command::new(&editor)
        .arg(temp_file.path())
        .status()
        .with_context(|| format!("Failed to open editor: {}", editor))?;
        
    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }
    
    // Read the edited message
    let edited_message = std::fs::read_to_string(temp_file.path())
        .with_context(|| "Failed to read edited commit message")?;
        
    Ok(edited_message.trim().to_string())
}