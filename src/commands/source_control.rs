//! Source Control commands for CI
//!
//! This module provides commands for managing source control operations,
//! including git repository management and commit handling.

use crate::config::Config;
use crate::helpers::{CommandHelpers, RepositoryHelpers, CommitAnalyzer};
use crate::RepoCommands;
use anyhow::{Result, Context, anyhow};
use std::path::Path;
use std::process::Command;
use colored::*;

/// Display detailed status of the git repository, working tree, and CI integration
pub async fn status(__config: &Config) -> Result<()> {
    // Match exact CI formatting
    println!("{}", "📊 CI Integration Status".green().bold());
    println!("{}", "====================".green());
    println!();
    
    let target_path = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
        
    println!("Project path: {}", target_path.display().to_string().cyan());
    println!();
    
    // Check for CLAUDE.md file
    let claude_md_path = target_path.join("CLAUDE.md");
    if claude_md_path.exists() {
        println!("{} CLAUDE.md file exists", "✓".green());
        
        // Read content and check for basic CI integration
        if let Ok(content) = std::fs::read_to_string(&claude_md_path) {
            if content.contains("Project:") && (content.contains("Integration") || content.contains("Integrated")) {
                println!("{} CI integration confirmed in CLAUDE.md", "✓".green());
            } else {
                println!("{} CLAUDE.md exists but may not be properly configured", "!".yellow());
            }
        } else {
            println!("{} Failed to read CLAUDE.md", "✗".red());
        }
    } else {
        println!("{} CLAUDE.md file not found", "✗".red());
        println!("Run {} to integrate CI into this project", "ci integrate".cyan());
    }
    
    // Check for local CI overrides
    let claude_local_path = target_path.join("CLAUDE.local.md");
    if claude_local_path.exists() {
        println!("{} CLAUDE.local.md file exists (local overrides)", "✓".green());
    } else {
        println!("{} No local CI configuration overrides", "!".yellow());
    }
    
    // Check for git integration
    let git_dir = target_path.join(".git");
    if git_dir.exists() {
        println!("{} Git repository found", "✓".green());
        
        // Check .gitignore for CLAUDE patterns
        let gitignore_path = target_path.join(".gitignore");
        if gitignore_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&gitignore_path) {
                if content.contains("CLAUDE.local.md") {
                    println!("{} .gitignore properly configured for CI", "✓".green());
                } else {
                    println!("{} .gitignore should exclude CLAUDE.local.md", "!".yellow());
                    println!("Run {} to fix .gitignore configuration", "ci ignore".cyan());
                }
            }
        } else {
            println!("{} No .gitignore file found", "!".yellow());
            println!("Run {} to create proper .gitignore for CI", "ci ignore".cyan());
        }
    } else {
        println!("{} Not a git repository", "!".yellow());
    }
    
    Ok(())
}

/// Manage GitHub repositories using gh CLI
pub async fn repo(command: &Option<RepoCommands>, _config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Manage GitHub repositories", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    // Check if GitHub CLI is installed
    if !RepositoryHelpers::check_gh_installed() {
        CommandHelpers::print_error("GitHub CLI (gh) is not installed");
        CommandHelpers::print_info("Please install GitHub CLI from https://cli.github.com/");
        return Ok(());
    }
    
    // Process subcommands
    match command {
        Some(RepoCommands::List) => list_repositories().await,
        Some(RepoCommands::Create { name, description, private }) => {
            create_repository(name, description.as_deref(), *private).await
        },
        Some(RepoCommands::Clone { repo, dir }) => {
            clone_repository(repo, dir.as_deref()).await
        },
        Some(RepoCommands::View { repo }) => {
            view_repository(repo).await
        },
        None => {
            // Default to listing repositories if no subcommand is provided
            list_repositories().await
        },
    }
}

/// Clean build artifacts from the project
pub async fn clean(_config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Clean build artifacts", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    CommandHelpers::print_info("Cleaning not yet implemented");
    CommandHelpers::print_success("Command completed");
    
    Ok(())
}

/// Update .gitignore with appropriate patterns for CI
pub async fn ignore(__config: &Config) -> Result<()> {
    // Get target path
    let target_path = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
    
    println!("Updating .gitignore in: {}", target_path.display().to_string().cyan());
    
    // Check if git is initialized
    let is_repo = Command::new("git")
        .args(&["rev-parse", "--is-inside-work-tree"])
        .current_dir(&target_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    
    if !is_repo {
        eprintln!("{}", "✗ Directory is not a git repository. Initialize a git repository first:".red());
        eprintln!("  {}", "ci status --init".cyan());
        return Err(anyhow!("Directory is not a git repository"));
    }
    
    // Standard patterns for Collaborative Intelligence projects
    let ci_patterns = vec![
        // CI-specific files
        ".ci/",
        ".ci-config.json",
        "CLAUDE.local.md",
        
        // Common environment variable files
        ".env",
        ".env.local",
        ".env.development.local",
        ".env.test.local",
        ".env.production.local",
        
        // Secrets and API keys
        "*.pem",
        "*.key",
        "*.crt",
        
        // Logs
        "logs/",
        "*.log",
        "npm-debug.log*",
        "yarn-debug.log*",
        "yarn-error.log*",
        
        // Build folders
        "dist/",
        "build/",
        "out/",
        
        // Dependency directories
        "node_modules/",
        "__pycache__/",
        "target/",
        "vendor/",
        
        // Package files
        "package-lock.json",
        "yarn.lock",
        "Cargo.lock",
        
        // OS specific
        ".DS_Store",
        "Thumbs.db",
        
        // Editor directories and files
        ".idea/",
        ".vscode/",
        "*.swp",
        "*.swo",
        ".vim/",
        "*.sublime-workspace",
        
        // Cache directories
        ".cache/",
        ".pytest_cache/",
        ".eslintcache",
        ".parcel-cache/",
        
        // Python specific
        "*.py[cod]",
        "*$py.class",
        ".Python",
        "env/",
        "venv/",
        "ENV/",
        "*.egg-info/",
        "*.egg",
    ];
    
    // Path to .gitignore file
    let gitignore_path = target_path.join(".gitignore");
    
    // Check if .gitignore exists and read it
    let mut existing_patterns = Vec::new();
    let mut gitignore_content = String::new();
    
    if gitignore_path.exists() {
        // Read the existing .gitignore file
        match std::fs::read_to_string(&gitignore_path) {
            Ok(content) => {
                gitignore_content = content.clone();
                
                // Parse the patterns
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        existing_patterns.push(trimmed.to_string());
                    }
                }
                
                println!("{}", "✓ Found existing .gitignore file".green());
            },
            Err(e) => {
                eprintln!("{}", format!("Error reading .gitignore: {}", e).red());
                eprintln!("Creating a new .gitignore file");
            }
        }
    } else {
        println!("{}", "Creating new .gitignore file".yellow());
    }
    
    // Determine which patterns to add (only add new patterns)
    let patterns_to_add: Vec<String> = ci_patterns.iter()
        .filter(|p| !existing_patterns.contains(&p.to_string()))
        .map(|s| s.to_string())
        .collect();
    
    if patterns_to_add.is_empty() {
        println!("{}", "✓ .gitignore already contains all recommended patterns".green());
        return Ok(());
    }
    
    // Build the new content
    let mut new_content = if gitignore_path.exists() {
        // Make sure the file ends with a newline
        if !gitignore_content.ends_with('\n') {
            gitignore_content.push('\n');
        }
        
        // Add a separator comment
        gitignore_content.push_str("\n# Added by Collaborative Intelligence in Rust\n");
        gitignore_content
    } else {
        // Create a new file with a header
        "# Collaborative Intelligence in Rust - Generated .gitignore\n\n".to_string()
    };
    
    // Add the new patterns
    for pattern in &patterns_to_add {
        new_content.push_str(&format!("{}\n", pattern));
    }
    
    // Write the updated content
    match std::fs::write(&gitignore_path, new_content) {
        Ok(_) => {
            println!("{}", format!("✓ Added {} new patterns to .gitignore", patterns_to_add.len()).green());
            
            // Print the added patterns
            println!("{}", "Added patterns:".cyan());
            for pattern in &patterns_to_add {
                println!("  + {}", pattern);
            }
        },
        Err(e) => {
            eprintln!("{}", format!("Error writing .gitignore: {}", e).red());
            return Err(anyhow!(e));
        }
    }
    
    Ok(())
}

/// Run ignore and then stage all untracked and unstaged files
pub async fn stage(config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Stage files for commit", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    let repo_path = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
    
    if !RepositoryHelpers::is_inside_git_repo(&repo_path) {
        return Err(anyhow!("Not in a git repository"));
    }
    
    // First run ignore to ensure .gitignore is up to date
    ignore(config).await?;
    
    // Stage all changes
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .with_context(|| "Failed to stage files")?;
        
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to stage files: {}", error));
    }
    
    CommandHelpers::print_success("Files staged successfully");
    
    Ok(())
}

/// Configure git remotes for personal and organizational repositories
pub async fn remotes(_config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Configure git remotes", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    let repo_path = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
    
    if !RepositoryHelpers::is_inside_git_repo(&repo_path) {
        return Err(anyhow!("Not in a git repository"));
    }
    
    // Display current remotes
    let output = Command::new("git")
        .args(["remote", "-v"])
        .current_dir(&repo_path)
        .output()
        .with_context(|| "Failed to list remotes")?;
        
    let remotes = String::from_utf8_lossy(&output.stdout);
    
    if !remotes.trim().is_empty() {
        CommandHelpers::print_info("Current remotes:");
        println!("{}", remotes);
    } else {
        CommandHelpers::print_info("No remotes configured");
    }
    
    // Prompt for personal remote
    if CommandHelpers::prompt_confirmation("Would you like to configure a personal remote?") {
        let personal_remote = CommandHelpers::prompt_input("Enter personal remote URL (e.g., https://github.com/username/repo.git)", None)?;
        
        if !personal_remote.trim().is_empty() {
            // First check if origin remote exists
            let output = Command::new("git")
                .args(["remote", "get-url", "origin"])
                .current_dir(&repo_path)
                .output();
                
            let args = if output.is_ok() && output.unwrap().status.success() {
                // Remote exists, set URL
                vec!["remote", "set-url", "origin", &personal_remote]
            } else {
                // Remote does not exist, add it
                vec!["remote", "add", "origin", &personal_remote]
            };
            
            let output = Command::new("git")
                .args(&args)
                .current_dir(&repo_path)
                .output()
                .with_context(|| "Failed to configure personal remote")?;
                
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("Failed to configure personal remote: {}", error));
            }
            
            CommandHelpers::print_success("Personal remote configured as 'origin'");
        }
    }
    
    // Prompt for organization remote
    if CommandHelpers::prompt_confirmation("Would you like to configure an organization remote?") {
        let org_remote = CommandHelpers::prompt_input("Enter organization remote URL (e.g., https://github.com/orgname/repo.git)", None)?;
        
        if !org_remote.trim().is_empty() {
            // First check if upstream remote exists
            let output = Command::new("git")
                .args(["remote", "get-url", "upstream"])
                .current_dir(&repo_path)
                .output();
                
            let args = if output.is_ok() && output.unwrap().status.success() {
                // Remote exists, set URL
                vec!["remote", "set-url", "upstream", &org_remote]
            } else {
                // Remote does not exist, add it
                vec!["remote", "add", "upstream", &org_remote]
            };
            
            let output = Command::new("git")
                .args(&args)
                .current_dir(&repo_path)
                .output()
                .with_context(|| "Failed to configure organization remote")?;
                
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("Failed to configure organization remote: {}", error));
            }
            
            CommandHelpers::print_success("Organization remote configured as 'upstream'");
        }
    }
    
    // Display updated remotes
    let output = Command::new("git")
        .args(["remote", "-v"])
        .current_dir(&repo_path)
        .output()
        .with_context(|| "Failed to list remotes")?;
        
    let remotes = String::from_utf8_lossy(&output.stdout);
    
    if !remotes.trim().is_empty() {
        CommandHelpers::print_info("Updated remotes:");
        println!("{}", remotes);
    }
    
    CommandHelpers::print_success("Remote configuration completed");
    
    Ok(())
}

/// Run ignore, stage files, analyze changes, and commit with a detailed message
pub async fn commit(message: Option<&str>, _config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Create a commit with staged changes", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    let repo_path = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
    
    if !RepositoryHelpers::is_inside_git_repo(&repo_path) {
        return Err(anyhow!("Not in a git repository"));
    }
    
    // Check if there are staged changes
    let output = Command::new("git")
        .args(["diff", "--staged", "--quiet"])
        .current_dir(&repo_path)
        .output()
        .with_context(|| "Failed to check for staged changes")?;
        
    if output.status.success() {
        return Err(anyhow!("No staged changes to commit"));
    }
    
    // If no message was provided, generate one
    let commit_message = if let Some(msg) = message {
        msg.to_string()
    } else {
        // Analyze the changes to generate a commit message
        let analysis = CommitAnalyzer::analyze_staged_changes(&repo_path).await?;
        
        // Display the analysis
        CommitAnalyzer::display_analysis(&analysis);
        
        // Ask if the user wants to use the generated message
        if CommandHelpers::prompt_confirmation("Use the suggested commit message?") {
            analysis.suggested_message
        } else {
            // Prompt for a custom message
            CommandHelpers::prompt_input("Enter commit message", None)?
        }
    };
    
    // Create the commit
    let output = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .current_dir(&repo_path)
        .output()
        .with_context(|| "Failed to create commit")?;
        
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create commit: {}", error));
    }
    
    CommandHelpers::print_success(&format!("Commit created: {}", commit_message));
    
    Ok(())
}

/// Enhanced commit with detailed analysis
#[allow(dead_code)]
pub async fn commit_enhanced(
    message: Option<&str>,
    path: &Path,
    no_ignore: bool,
    all: bool,
    push: bool,
    sign: bool,
    config: &Config
) -> Result<()> {
    CommandHelpers::print_command_header(
        "Create commit with enhanced analysis", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    if !RepositoryHelpers::is_inside_git_repo(path) {
        return Err(anyhow!("Not in a git repository"));
    }
    
    // Run ignore command if not disabled
    if !no_ignore {
        ignore(config).await?;
    }
    
    // Stage files if needed (all=true will stage all files)
    if all {
        let output = Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to stage files")?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to stage files: {}", error));
        }
    }
    
    // Check if there are staged changes
    let output = Command::new("git")
        .args(["diff", "--staged", "--quiet"])
        .current_dir(path)
        .output()
        .with_context(|| "Failed to check for staged changes")?;
        
    if output.status.success() {
        return Err(anyhow!("No staged changes to commit"));
    }
    
    // Generate commit message if not provided
    let commit_message = if let Some(msg) = message {
        msg.to_string()
    } else {
        // Analyze the changes to generate a commit message
        let analysis = CommitAnalyzer::analyze_staged_changes(path).await?;
        
        // Display the analysis
        CommitAnalyzer::display_analysis(&analysis);
        
        // Ask if the user wants to use the generated message
        if CommandHelpers::prompt_confirmation("Use the suggested commit message?") {
            analysis.suggested_message
        } else {
            // Prompt for a custom message
            CommandHelpers::prompt_input("Enter commit message", None)?
        }
    };
    
    // Create the commit
    let mut args = vec!["commit", "-m", &commit_message];
    if sign {
        args.push("-S");
    }
    
    let output = Command::new("git")
        .args(&args)
        .current_dir(path)
        .output()
        .with_context(|| "Failed to create commit")?;
        
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create commit: {}", error));
    }
    
    CommandHelpers::print_success(&format!("Commit created: {}", commit_message));
    
    // Push if requested
    if push {
        let output = Command::new("git")
            .args(["push"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to push to remote")?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to push to remote: {}", error));
        }
        
        CommandHelpers::print_success("Pushed to remote successfully");
    }
    
    Ok(())
}

/// Run ignore, stage, commit, and push in one operation
pub async fn deploy(config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Deploy changes: stage, commit, and push", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    let repo_path = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
    
    // Update .gitignore
    ignore(config).await?;
    
    // Stage all changes
    stage(config).await?;
    
    // Get detailed analysis for the commit
    let analysis = CommitAnalyzer::analyze_staged_changes(&repo_path).await?;
    
    // Display the analysis
    CommitAnalyzer::display_analysis(&analysis);
    
    // Ask if the user wants to use the generated message
    let commit_message = if CommandHelpers::prompt_confirmation("Use the suggested commit message?") {
        analysis.suggested_message
    } else {
        // Prompt for a custom message
        CommandHelpers::prompt_input("Enter commit message", None)?
    };
    
    // Create the commit
    let output = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .current_dir(&repo_path)
        .output()
        .with_context(|| "Failed to create commit")?;
        
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create commit: {}", error));
    }
    
    CommandHelpers::print_success(&format!("Commit created: {}", commit_message));
    
    // Push to remote
    let output = Command::new("git")
        .args(["push"])
        .current_dir(&repo_path)
        .output()
        .with_context(|| "Failed to push to remote")?;
        
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        CommandHelpers::print_warning(&format!("Failed to push to remote: {}", error));
        CommandHelpers::print_info("You may need to configure a remote first with 'git remote add origin <url>'");
        return Ok(());
    }
    
    CommandHelpers::print_success("Changes deployed successfully");
    
    Ok(())
}

/// Display extremely detailed status with CI integration diagnostics
pub async fn status_detailed(format: &str, system: bool, agents: bool, _config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Display comprehensive status report", 
        "📊", 
        "Source Control", 
        "green"
    );
    
    let repo_path = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;
    
    // Generate comprehensive status report
    let report = crate::helpers::StatusReporter::generate_status_report(&repo_path)?;
    
    // Customize report based on command line options
    let mut filtered_report = report.clone();
    
    // Optionally remove system information
    if !system {
        // Set default values instead of real system metrics
        filtered_report.system_resources.cpu_usage = 0.0;
        filtered_report.system_resources.memory_usage = 0.0;
        filtered_report.system_resources.disk_usage = 0.0;
    }
    
    // Optionally remove agent details
    if !agents {
        filtered_report.agent_status = Vec::new();
    }
    
    // Handle different output formats
    match format.to_lowercase().as_str() {
        "json" => {
            // Output as JSON
            let json = serde_json::to_string_pretty(&filtered_report)
                .with_context(|| "Failed to serialize status report to JSON")?;
            println!("{}", json);
        },
        "compact" => {
            // Compact text output
            crate::helpers::StatusReporter::display_compact_status(&filtered_report);
        },
        _ => {
            // Default to full text output
            crate::helpers::StatusReporter::display_status_report(&filtered_report);
        }
    }
    
    CommandHelpers::print_success("Status report displayed successfully");
    
    Ok(())
}

/// List GitHub repositories
async fn list_repositories() -> Result<()> {
    CommandHelpers::print_info("Fetching repositories...");
    
    // Fetch repositories using the helper
    let repos = RepositoryHelpers::list_github_repos()?;
    
    if repos.is_empty() {
        CommandHelpers::print_info("No repositories found");
        return Ok(());
    }
    
    // Display repositories in a formatted table
    println!("\n{:<30} {:<50} {:<15}", "Name", "Description", "Visibility");
    println!("{:<30} {:<50} {:<15}", "----", "-----------", "----------");
    
    let repo_count = repos.len();
    
    for repo in repos {
        let description = repo.description
            .as_ref()
            .map_or_else(|| "", |s| s.as_str())
            .chars()
            .take(47)
            .collect::<String>();
            
        let desc_display = if repo.description.as_ref().map_or(0, |d| d.len()) > 47 {
            format!("{}...", description)
        } else {
            description
        };
        
        println!("{:<30} {:<50} {:<15}", 
            repo.name, 
            desc_display,
            repo.visibility
        );
    }
    
    CommandHelpers::print_success(&format!("Listed {} repositories", repo_count));
    
    Ok(())
}

/// Create a new GitHub repository
async fn create_repository(name: &str, description: Option<&str>, private: bool) -> Result<()> {
    CommandHelpers::print_info(&format!("Creating repository '{}'...", name));
    
    // Create repository
    let repo = RepositoryHelpers::create_github_repo(name, description, private)?;
    
    // Print success message with repo details
    CommandHelpers::print_success(&format!("Repository created successfully:"));
    println!("\nName: {}", repo.name);
    println!("URL:  {}", repo.url);
    
    if let Some(desc) = repo.description {
        println!("Description: {}", desc);
    }
    
    println!("Visibility: {}", repo.visibility);
    
    // Ask if the user wants to clone the repository
    if CommandHelpers::prompt_confirmation("Would you like to clone this repository now?") {
        let dir = CommandHelpers::prompt_input("Clone directory (leave empty for default)", Some(""))?;
        
        let target_dir = if dir.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(dir))
        };
        
        RepositoryHelpers::clone_github_repo(&repo.name, target_dir.as_deref())?;
        CommandHelpers::print_success("Repository cloned successfully");
    }
    
    Ok(())
}

/// Clone a GitHub repository
async fn clone_repository(repo: &str, dir: Option<&std::path::Path>) -> Result<()> {
    CommandHelpers::print_info(&format!("Cloning repository '{}'...", repo));
    
    // Clone the repository
    RepositoryHelpers::clone_github_repo(repo, dir)?;
    
    // Print success message
    CommandHelpers::print_success(&format!("Repository '{}' cloned successfully", repo));
    
    let target_dir = if let Some(d) = dir {
        d.to_string_lossy()
    } else {
        // Extract repo name from URL or owner/repo format
        let repo_name = if repo.contains('/') {
            repo.split('/').last().unwrap_or(repo)
        } else {
            repo
        };
        repo_name.into()
    };
    
    println!("\nCloned to: {}", target_dir);
    
    Ok(())
}

/// View GitHub repository details
async fn view_repository(repo: &str) -> Result<()> {
    CommandHelpers::print_info(&format!("Fetching details for repository '{}'...", repo));
    
    // Fetch repository details
    let details = RepositoryHelpers::view_github_repo(repo)?;
    
    // Print repository details in a formatted way
    println!("\n{}", "Repository Details:".bold());
    println!("Name:        {}", details.name);
    
    if let Some(desc) = details.description {
        println!("Description: {}", desc);
    }
    
    println!("URL:         {}", details.url);
    println!("Visibility:  {}", details.visibility);
    println!("Owner:       {}", details.owner.login);
    println!("Stars:       {}", details.stargazer_count);
    println!("Forks:       {}", details.fork_count);
    
    if let Some(branch) = details.default_branch_ref {
        println!("Default Branch: {}", branch.name);
    }
    
    println!("Created:     {}", details.created_at);
    println!("Updated:     {}", details.updated_at);
    
    if let Some(languages) = details.languages {
        println!("\n{}", "Languages:".bold());
        for lang in languages {
            println!("  • {}", lang.name);
        }
    }
    
    if details.is_archived {
        println!("\n{}", "This repository is archived".yellow());
    }
    
    if details.is_fork {
        println!("\n{}", "This repository is a fork".cyan());
    }
    
    CommandHelpers::print_success("Repository details displayed");
    
    Ok(())
}