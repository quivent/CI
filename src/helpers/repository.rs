//! Git repository and source control helpers for CI
//!
//! This module provides helper functions for working with git repositories,
//! including status checks, commits, and repository management.

use colored::*;
use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result, anyhow};
use crate::helpers::command::CommandHelpers;

/// Helper functions for repository operations
pub struct RepositoryHelpers;

impl RepositoryHelpers {
    /// Check if a path is inside a git repository
    pub fn is_inside_git_repo(path: &Path) -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(path)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Get the root directory of the git repository
    pub fn get_git_root(path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to run git command")?;
            
        if !output.status.success() {
            return Err(anyhow!("Not in a git repository"));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    /// Get the current branch name
    pub fn get_current_branch(path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to run git branch command")?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to get current branch"));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    /// Get repository status information
    pub fn get_repository_status(path: &Path) -> Result<RepositoryStatus> {
        let mut status = RepositoryStatus::default();
        
        // Check if it's a git repository
        if !Self::is_inside_git_repo(path) {
            status.is_git_repo = false;
            return Ok(status);
        }
        
        status.is_git_repo = true;
        
        // Get current branch
        status.current_branch = Self::get_current_branch(path).ok();
        
        // Check for uncommitted changes
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to run git status")?;
            
        status.has_uncommitted_changes = !output.stdout.is_empty();
        
        // Count commits
        let output = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to count commits")?;
            
        if output.status.success() {
            status.commit_count = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
        }
        
        // Check for remotes
        let output = Command::new("git")
            .args(["remote", "-v"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get remotes")?;
            
        status.has_remote = !output.stdout.is_empty();
        
        // Get remote URL if exists
        if status.has_remote {
            let output = Command::new("git")
                .args(["remote", "get-url", "origin"])
                .current_dir(path)
                .output();
                
            if let Ok(output) = output {
                if output.status.success() {
                    status.remote_url = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
                }
            }
        }
        
        // Check if branch is ahead/behind remote
        if let Some(branch) = &status.current_branch {
            let output = Command::new("git")
                .args(["rev-list", "--left-right", "--count", &format!("origin/{}...{}", branch, branch)])
                .current_dir(path)
                .output();
                
            if let Ok(output) = output {
                if output.status.success() {
                    let counts = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if let Some((behind, ahead)) = counts.split_once('\t') {
                        status.commits_behind = behind.parse().unwrap_or(0);
                        status.commits_ahead = ahead.parse().unwrap_or(0);
                    }
                }
            }
        }
        
        Ok(status)
    }
    
    /// Display repository status in a formatted way
    pub fn display_status(status: &RepositoryStatus) {
        if !status.is_git_repo {
            println!("{}", "Not a git repository".yellow());
            return;
        }
        
        println!("Repository Status:");
        println!("  Branch: {}", status.current_branch.as_ref()
            .map(|b| b.green().to_string())
            .unwrap_or_else(|| "unknown".yellow().to_string()));
            
        println!("  Commits: {}", status.commit_count.to_string().cyan());
        
        if status.has_uncommitted_changes {
            println!("  Changes: {}", "uncommitted changes".yellow());
        } else {
            println!("  Changes: {}", "clean".green());
        }
        
        if status.has_remote {
            println!("  Remote: {}", status.remote_url.as_ref()
                .unwrap_or(&"configured".to_string()).cyan());
            
            // Show ahead/behind status if available
            if status.commits_ahead > 0 || status.commits_behind > 0 {
                let mut status_parts = Vec::new();
                if status.commits_ahead > 0 {
                    status_parts.push(format!("{} commits ahead", status.commits_ahead));
                }
                if status.commits_behind > 0 {
                    status_parts.push(format!("{} commits behind", status.commits_behind));
                }
                println!("  Remote Status: {}", status_parts.join(", ").yellow());
            }
        } else {
            println!("  Remote: {}", "none".yellow());
        }
    }
    
    /// Initialize a git repository
    pub fn init_git_repository(path: &Path) -> Result<()> {
        let output = Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()
            .with_context(|| "Failed to run git init")?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Git init failed: {}", stderr));
        }
        
        Ok(())
    }
    
    /// Create a default .gitignore if it doesn't exist
    pub fn create_default_gitignore(path: &Path) -> Result<()> {
        let gitignore_path = path.join(".gitignore");
        
        if !gitignore_path.exists() {
            let default_content = r#"# CI-specific ignores
.ci/
CLAUDE.local.md
.env

# Build artifacts
target/
*.log

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db
"#;
            std::fs::write(&gitignore_path, default_content)
                .with_context(|| "Failed to write .gitignore file")?;
        }
        
        Ok(())
    }
    
    /// Add specific patterns to .gitignore if not present
    pub fn update_gitignore(path: &Path, patterns: &[&str]) -> Result<bool> {
        let gitignore_path = path.join(".gitignore");
        
        let content = if gitignore_path.exists() {
            std::fs::read_to_string(&gitignore_path)
                .with_context(|| "Failed to read .gitignore file")?
        } else {
            String::new()
        };
        
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut added = false;
        
        for pattern in patterns {
            if !lines.iter().any(|line| line.trim() == *pattern) {
                lines.push(pattern.to_string());
                added = true;
            }
        }
        
        if added {
            lines.push(String::new()); // Add trailing newline
            let new_content = lines.join("\n");
            std::fs::write(&gitignore_path, new_content)
                .with_context(|| "Failed to update .gitignore file")?;
        }
        
        Ok(added)
    }
    
    /// Get the current commit hash
    pub fn get_current_commit(path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get current commit hash")?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to get current commit hash"));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    /// Get the latest commit message
    pub fn get_latest_commit_message(path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["log", "-1", "--pretty=%B"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get latest commit message")?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to get latest commit message"));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    /// Stage files matching a pattern
    pub fn stage_files(path: &Path, pattern: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["add", pattern])
            .current_dir(path)
            .output()
            .with_context(|| format!("Failed to stage files matching pattern: {}", pattern))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to stage files: {}", stderr));
        }
        
        Ok(())
    }
    
    /// Check if there are unstaged changes
    pub fn has_unstaged_changes(path: &Path) -> bool {
        let output = Command::new("git")
            .args(["diff", "--quiet"])
            .current_dir(path)
            .output();
            
        match output {
            Ok(output) => !output.status.success(),
            Err(_) => false
        }
    }
    
    /// Shows diff statistics
    pub fn show_diff_statistics(path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["diff", "--stat", "--staged"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get diff statistics")?;
            
        let stat_output = String::from_utf8_lossy(&output.stdout);
        if !stat_output.trim().is_empty() {
            CommandHelpers::print_info("Staged changes:");
            println!("{}", stat_output);
        } else {
            CommandHelpers::print_info("No staged changes");
        }
        
        // Also show unstaged changes
        let output = Command::new("git")
            .args(["diff", "--stat"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get unstaged diff statistics")?;
            
        let stat_output = String::from_utf8_lossy(&output.stdout);
        if !stat_output.trim().is_empty() {
            CommandHelpers::print_info("Unstaged changes:");
            println!("{}", stat_output);
        }
        
        // Show untracked files
        let output = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get untracked files")?;
            
        let untracked = String::from_utf8_lossy(&output.stdout);
        if !untracked.trim().is_empty() {
            let files: Vec<_> = untracked.lines().collect();
            CommandHelpers::print_info(&format!("Untracked files: {}", files.len()));
            for file in files.iter().take(10) {
                println!("  • {}", file);
            }
            if files.len() > 10 {
                println!("  • ... and {} more", files.len() - 10);
            }
        }
        
        Ok(())
    }
    
    /// Shows recent commits
    pub fn show_recent_commits(path: &Path, count: usize) -> Result<()> {
        let output = Command::new("git")
            .args(["log", "--oneline", "--graph", "--decorate", &format!("--max-count={}", count)])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get recent commits")?;
            
        let commits = String::from_utf8_lossy(&output.stdout);
        if !commits.trim().is_empty() {
            CommandHelpers::print_info("Recent commits:");
            println!("{}", commits);
        } else {
            CommandHelpers::print_info("No commits found");
        }
        
        Ok(())
    }
    
    /// Generates a commit message based on changes
    pub async fn generate_commit_message(path: &Path) -> Result<(String, String)> {
        // First get the diff
        let output = Command::new("git")
            .args(["diff", "--staged", "--stat"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get staged diff")?;
            
        let diff_stat = String::from_utf8_lossy(&output.stdout).to_string();
        if diff_stat.trim().is_empty() {
            return Err(anyhow!("No staged changes to analyze"));
        }
        
        // Get actual diff content
        let output = Command::new("git")
            .args(["diff", "--staged"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get staged diff content")?;
            
        let diff_content = String::from_utf8_lossy(&output.stdout).to_string();
        
        // Parse file changes
        let files_changed = diff_stat.lines()
            .filter(|line| line.contains("|") && !line.contains("files changed"))
            .map(|line| line.split('|').next().unwrap_or("").trim())
            .collect::<Vec<_>>();

        // For normal non-AI implementation, we'd analyze more carefully, but this is a mock implementation 
        // A real implementation would look at actual changes and patterns
        let files_desc = if files_changed.len() == 1 {
            format!("Update {}", files_changed[0])
        } else {
            let categories = Self::categorize_files(&files_changed);
            let primary_category = categories.first().map(|c| c.as_str()).unwrap_or("code");
            
            if diff_content.contains("new file") {
                format!("Add {} {}", files_changed.len(), primary_category)
            } else if diff_content.contains("deleted file") {
                format!("Remove {} {}", files_changed.len(), primary_category)
            } else {
                format!("Update {} {}", files_changed.len(), primary_category)
            }
        };
        
        // Build commit message
        let message = if diff_content.len() < 500 {
            // Simple change
            files_desc
        } else {
            // More complex change, try to be more descriptive
            let action = if diff_content.contains("+++ b/") && diff_content.contains("--- a/") {
                "Update"
            } else if diff_content.contains("new file") {
                "Add"
            } else {
                "Change"
            };
            
            format!("{} implementation for {}", action, files_changed.join(", "))
        };
        
        // Build detailed description
        let details = format!("Files changed:\n{}\n\nSummary:\n- Modified {} files\n- {} additions, {} deletions", 
            files_changed.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n"),
            files_changed.len(),
            diff_content.lines().filter(|l| l.starts_with("+")).count(),
            diff_content.lines().filter(|l| l.starts_with("-")).count()
        );
        
        Ok((message, details))
    }
    
    /// Categorize files by type
    fn categorize_files(files: &[&str]) -> Vec<String> {
        let mut categories = std::collections::HashMap::new();
        
        for file in files {
            let category = if file.ends_with(".rs") {
                "rust files"
            } else if file.ends_with(".js") || file.ends_with(".ts") {
                "javascript files"
            } else if file.ends_with(".css") || file.ends_with(".scss") {
                "style files"
            } else if file.ends_with(".html") {
                "html files"
            } else if file.ends_with(".md") {
                "documentation"
            } else if file.ends_with(".json") || file.ends_with(".toml") || file.ends_with(".yaml") {
                "configuration files"
            } else if file.ends_with(".sh") || file.ends_with(".bash") {
                "scripts"
            } else {
                "files"
            };
            
            *categories.entry(category).or_insert(0) += 1;
        }
        
        // Sort by count
        let mut category_counts: Vec<_> = categories.into_iter().collect();
        category_counts.sort_by(|a, b| b.1.cmp(&a.1));
        
        category_counts.into_iter().map(|(cat, _)| cat.to_string()).collect()
    }
    
    /// Get list of staged files
    pub fn get_staged_files(path: &Path) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["diff", "--name-only", "--cached"])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to get staged files")?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to get staged files: {}", stderr));
        }
        
        let files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect();
            
        Ok(files)
    }
    
    /// Create a commit with the given message
    pub fn create_commit(path: &Path, message: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(path)
            .output()
            .with_context(|| "Failed to create commit")?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create commit: {}", stderr));
        }
        
        Ok(())
    }
    
    /// Push to remote
    pub fn push_to_remote(path: &Path, remote: &str, branch: &str, force: bool) -> Result<()> {
        let mut args = vec!["push", remote, branch];
        if force {
            args.push("--force");
        }
        
        let output = Command::new("git")
            .args(&args)
            .current_dir(path)
            .output()
            .with_context(|| format!("Failed to push to remote {}/{}", remote, branch))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to push to remote: {}", stderr));
        }
        
        Ok(())
    }
    
    /// Check if gh cli is installed
    pub fn check_gh_installed() -> bool {
        Command::new("gh")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// List GitHub repositories using gh cli
    pub fn list_github_repos() -> Result<Vec<GitHubRepo>> {
        // Ensure gh is installed
        if !Self::check_gh_installed() {
            return Err(anyhow!("GitHub CLI not installed. Please install it first: https://cli.github.com/"));
        }
        
        // Get repositories in JSON format
        let output = Command::new("gh")
            .args(["repo", "list", "--json", "name,description,url,visibility,isArchived,isFork"])
            .output()
            .with_context(|| "Failed to list GitHub repositories")?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to list GitHub repositories: {}", stderr));
        }
        
        // Parse the JSON response
        let response = String::from_utf8_lossy(&output.stdout);
        let repos: Vec<GitHubRepo> = serde_json::from_str(&response)
            .with_context(|| "Failed to parse GitHub repository data")?;
            
        Ok(repos)
    }
    
    /// Create a new GitHub repository
    pub fn create_github_repo(name: &str, description: Option<&str>, private: bool) -> Result<GitHubRepo> {
        // Ensure gh is installed
        if !Self::check_gh_installed() {
            return Err(anyhow!("GitHub CLI not installed. Please install it first: https://cli.github.com/"));
        }
        
        // Build command args
        let mut args = vec!["repo", "create", name];
        
        if let Some(desc) = description {
            args.push("--description");
            args.push(desc);
        }
        
        if private {
            args.push("--private");
        } else {
            args.push("--public");
        }
        
        // Add JSON output format
        args.push("--json");
        args.push("name,description,url,visibility");
        
        // Execute command
        let output = Command::new("gh")
            .args(&args)
            .output()
            .with_context(|| format!("Failed to create GitHub repository: {}", name))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create GitHub repository: {}", stderr));
        }
        
        // Parse the JSON response
        let response = String::from_utf8_lossy(&output.stdout);
        let repo: GitHubRepo = serde_json::from_str(&response)
            .with_context(|| "Failed to parse GitHub repository data")?;
            
        Ok(repo)
    }
    
    /// Clone a GitHub repository
    pub fn clone_github_repo(repo: &str, dir: Option<&Path>) -> Result<()> {
        // Build command args
        let mut args = vec!["repo", "clone", repo];
        
        if let Some(target_dir) = dir {
            args.push(target_dir.to_str().unwrap_or("."));
        }
        
        // Execute command
        let output = Command::new("gh")
            .args(&args)
            .output()
            .with_context(|| format!("Failed to clone GitHub repository: {}", repo))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to clone GitHub repository: {}", stderr));
        }
        
        Ok(())
    }
    
    /// View GitHub repository details
    pub fn view_github_repo(repo: &str) -> Result<GitHubRepoDetails> {
        // Ensure gh is installed
        if !Self::check_gh_installed() {
            return Err(anyhow!("GitHub CLI not installed. Please install it first: https://cli.github.com/"));
        }
        
        // Get repository details in JSON format
        let output = Command::new("gh")
            .args(["repo", "view", repo, "--json", "name,description,url,visibility,stargazerCount,forkCount,defaultBranchRef,isArchived,isFork,owner,createdAt,updatedAt,languages"])
            .output()
            .with_context(|| format!("Failed to view GitHub repository: {}", repo))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to view GitHub repository: {}", stderr));
        }
        
        // Parse the JSON response
        let response = String::from_utf8_lossy(&output.stdout);
        let repo_details: GitHubRepoDetails = serde_json::from_str(&response)
            .with_context(|| "Failed to parse GitHub repository details")?;
            
        Ok(repo_details)
    }
    
    /// Configure repository with standard remotes
    pub fn configure_remotes(path: &Path, personal: &str, organization: Option<&str>) -> Result<()> {
        // Set the personal remote as 'origin'
        let output = Command::new("git")
            .args(["remote", "add", "origin", personal])
            .current_dir(path)
            .output()
            .with_context(|| format!("Failed to add personal remote: {}", personal))?;
            
        // If command fails because remote already exists, try setting the URL instead
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already exists") {
                let output = Command::new("git")
                    .args(["remote", "set-url", "origin", personal])
                    .current_dir(path)
                    .output()
                    .with_context(|| format!("Failed to update personal remote: {}", personal))?;
                    
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Failed to update personal remote: {}", stderr));
                }
            } else {
                return Err(anyhow!("Failed to add personal remote: {}", stderr));
            }
        }
        
        // If organization remote is provided, set it as 'upstream'
        if let Some(org_remote) = organization {
            let output = Command::new("git")
                .args(["remote", "add", "upstream", org_remote])
                .current_dir(path)
                .output()
                .with_context(|| format!("Failed to add organization remote: {}", org_remote))?;
                
            // If command fails because remote already exists, try setting the URL instead
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("already exists") {
                    let output = Command::new("git")
                        .args(["remote", "set-url", "upstream", org_remote])
                        .current_dir(path)
                        .output()
                        .with_context(|| format!("Failed to update organization remote: {}", org_remote))?;
                        
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(anyhow!("Failed to update organization remote: {}", stderr));
                    }
                } else {
                    return Err(anyhow!("Failed to add organization remote: {}", stderr));
                }
            }
        }
        
        Ok(())
    }
    
    /// Create a pull request
    pub fn create_pull_request(path: &Path, title: &str, body: &str, base_branch: Option<&str>) -> Result<String> {
        // Ensure gh is installed
        if !Self::check_gh_installed() {
            return Err(anyhow!("GitHub CLI not installed. Please install it first: https://cli.github.com/"));
        }
        
        // Create a temporary file to store the PR body
        let mut body_file = tempfile::NamedTempFile::new()
            .with_context(|| "Failed to create temporary file for PR body")?;
            
        std::io::Write::write_all(&mut body_file, body.as_bytes())
            .with_context(|| "Failed to write PR body to temporary file")?;
        
        // Build command args
        let mut args = vec!["pr", "create", "--title", title, "--body-file", body_file.path().to_str().unwrap()];
        
        if let Some(base) = base_branch {
            args.push("--base");
            args.push(base);
        }
        
        // Add web flag to open PR in browser
        args.push("--web");
        
        // Execute command
        let output = Command::new("gh")
            .args(&args)
            .current_dir(path)
            .output()
            .with_context(|| "Failed to create pull request")?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create pull request: {}", stderr));
        }
        
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(url)
    }
}

/// Structure to hold repository status information
#[derive(Default)]
pub struct RepositoryStatus {
    pub is_git_repo: bool,
    pub current_branch: Option<String>,
    pub has_uncommitted_changes: bool,
    pub commit_count: usize,
    pub has_remote: bool,
    pub remote_url: Option<String>,
    pub commits_ahead: usize,
    pub commits_behind: usize,
}

/// Structure to represent a GitHub repository
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GitHubRepo {
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub visibility: String,
    #[serde(rename = "isArchived")]
    pub is_archived: Option<bool>,
    #[serde(rename = "isFork")]
    pub is_fork: Option<bool>,
}

/// Structure to represent detailed GitHub repository information
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GitHubRepoDetails {
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub visibility: String,
    #[serde(rename = "stargazerCount")]
    pub stargazer_count: usize,
    #[serde(rename = "forkCount")]
    pub fork_count: usize,
    #[serde(rename = "defaultBranchRef")]
    pub default_branch_ref: Option<DefaultBranchRef>,
    #[serde(rename = "isArchived")]
    pub is_archived: bool,
    #[serde(rename = "isFork")]
    pub is_fork: bool,
    pub owner: Owner,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub languages: Option<Vec<Language>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct DefaultBranchRef {
    pub name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Owner {
    pub login: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Language {
    pub name: String,
    pub color: Option<String>,
}