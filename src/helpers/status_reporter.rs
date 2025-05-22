//! Status reporting helper for CI
//!
//! This module provides enhanced status reporting for the Collaborative Intelligence
//! integration, including both local and remote status details.

use colored::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Local};
use crate::helpers::repository::RepositoryHelpers;

/// Status report structure holding detailed information about CI and project status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusReport {
    /// Time when the report was generated
    pub timestamp: DateTime<Utc>,
    
    /// Path to the project
    pub project_path: PathBuf,
    
    /// Repository status
    pub repo_status: RepoStatus,
    
    /// CI integration status
    pub ci_status: CIStatus,
    
    /// Agent status information
    pub agent_status: Vec<AgentStatus>,
    
    /// System resource usage
    pub system_resources: SystemResources,
}

/// Repository status information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoStatus {
    /// Is this a git repository
    pub is_git_repo: bool,
    
    /// Current branch
    pub branch: Option<String>,
    
    /// Does the repo have uncommitted changes
    pub has_uncommitted_changes: bool,
    
    /// Number of commits in repository
    pub commit_count: usize,
    
    /// Remote repository information
    pub remote: Option<RemoteInfo>,
    
    /// Last commit information
    pub last_commit: Option<CommitInfo>,
}

/// Remote repository information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteInfo {
    /// Remote URL
    pub url: String,
    
    /// Remote name (origin, upstream, etc.)
    pub name: String,
    
    /// Commits ahead of remote
    pub commits_ahead: usize,
    
    /// Commits behind remote
    pub commits_behind: usize,
}

/// Commit information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitInfo {
    /// Commit hash
    pub hash: String,
    
    /// Commit message
    pub message: String,
    
    /// Commit author
    pub author: String,
    
    /// Commit date
    pub date: String,
}

/// CI integration status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CIStatus {
    /// Is CI integrated with this project
    pub is_integrated: bool,
    
    /// Integration type (embedded, symlink, sibling)
    pub integration_type: Option<String>,
    
    /// Path to CLAUDE.md file
    pub claude_md_path: Option<PathBuf>,
    
    /// Path to CLAUDE.local.md file
    pub claude_local_md_path: Option<PathBuf>,
    
    /// Last modified time of CLAUDE.md
    pub claude_md_modified: Option<DateTime<Utc>>,
    
    /// CI version info
    pub ci_version: Option<String>,
    
    /// CI version info
    pub cir_version: String,
}

/// Agent status information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStatus {
    /// Agent name
    pub name: String,
    
    /// Agent description
    pub description: Option<String>,
    
    /// Path to agent memory file
    pub memory_path: Option<PathBuf>,
    
    /// Last modified time of agent memory
    pub last_modified: Option<DateTime<Utc>>,
    
    /// Size of agent memory (in bytes)
    pub memory_size: Option<u64>,
    
    /// Agent capability summary
    pub capabilities: Vec<String>,
}

/// System resource usage
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemResources {
    /// System memory usage percentage
    pub memory_usage: f32,
    
    /// CPU usage percentage
    pub cpu_usage: f32,
    
    /// Disk space usage
    pub disk_usage: f32,
    
    /// Total project size (in bytes)
    pub project_size: u64,
}

/// StatusReporter provides methods for collecting and reporting detailed status information
pub struct StatusReporter;

impl StatusReporter {
    /// Generate a comprehensive status report for a project
    pub fn generate_status_report(project_path: &Path) -> Result<StatusReport> {
        // Create a new status report
        let report = StatusReport {
            timestamp: Utc::now(),
            project_path: project_path.to_path_buf(),
            repo_status: Self::collect_repo_status(project_path)?,
            ci_status: Self::collect_ci_status(project_path)?,
            agent_status: Self::collect_agent_status(project_path)?,
            system_resources: Self::collect_system_resources(project_path)?,
        };
        
        Ok(report)
    }
    
    /// Collect repository status information
    fn collect_repo_status(project_path: &Path) -> Result<RepoStatus> {
        let mut repo_status = RepoStatus {
            is_git_repo: false,
            branch: None,
            has_uncommitted_changes: false,
            commit_count: 0,
            remote: None,
            last_commit: None,
        };
        
        // Check if it's a git repository
        if !RepositoryHelpers::is_inside_git_repo(project_path) {
            return Ok(repo_status);
        }
        
        repo_status.is_git_repo = true;
        
        // Get current branch
        repo_status.branch = RepositoryHelpers::get_current_branch(project_path).ok();
        
        // Check for uncommitted changes
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(project_path)
            .output()
            .with_context(|| "Failed to run git status")?;
            
        repo_status.has_uncommitted_changes = !output.stdout.is_empty();
        
        // Count commits
        let output = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .current_dir(project_path)
            .output()
            .with_context(|| "Failed to count commits")?;
            
        if output.status.success() {
            repo_status.commit_count = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
        }
        
        // Check for remotes
        let output = Command::new("git")
            .args(["remote", "-v"])
            .current_dir(project_path)
            .output()
            .with_context(|| "Failed to get remotes")?;
            
        let has_remote = !output.stdout.is_empty();
        
        // If remote exists, gather details
        if has_remote {
            let remote_name = "origin";  // Default to origin
            
            // Get remote URL
            let output = Command::new("git")
                .args(["remote", "get-url", remote_name])
                .current_dir(project_path)
                .output();
                
            if let Ok(output) = output {
                if output.status.success() {
                    let remote_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    
                    // Check if branch is ahead/behind remote
                    let mut ahead = 0;
                    let mut behind = 0;
                    
                    if let Some(branch) = &repo_status.branch {
                        let ahead_behindoutput = Command::new("git")
                            .args(["rev-list", "--left-right", "--count", &format!("origin/{}...{}", branch, branch)])
                            .current_dir(project_path)
                            .output();
                            
                        if let Ok(output) = ahead_behindoutput {
                            if output.status.success() {
                                let counts = String::from_utf8_lossy(&output.stdout).trim().to_string();
                                if let Some((behind_str, ahead_str)) = counts.split_once('\t') {
                                    behind = behind_str.parse().unwrap_or(0);
                                    ahead = ahead_str.parse().unwrap_or(0);
                                }
                            }
                        }
                    }
                    
                    repo_status.remote = Some(RemoteInfo {
                        url: remote_url,
                        name: remote_name.to_string(),
                        commits_ahead: ahead,
                        commits_behind: behind,
                    });
                }
            }
        }
        
        // Get last commit information
        if repo_status.commit_count > 0 {
            // Get commit hash
            let output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(project_path)
                .output()
                .with_context(|| "Failed to get commit hash")?;
                
            if output.status.success() {
                let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                
                // Get commit details
                let output = Command::new("git")
                    .args(["log", "-1", "--pretty=%B%n%an%n%ad"])
                    .current_dir(project_path)
                    .output()
                    .with_context(|| "Failed to get commit details")?;
                    
                if output.status.success() {
                    let details = String::from_utf8_lossy(&output.stdout);
                    let mut lines = details.lines();
                    
                    let message = lines.next().unwrap_or("").to_string();
                    let author = lines.next().unwrap_or("").to_string();
                    let date = lines.next().unwrap_or("").to_string();
                    
                    repo_status.last_commit = Some(CommitInfo {
                        hash,
                        message,
                        author,
                        date,
                    });
                }
            }
        }
        
        Ok(repo_status)
    }
    
    /// Collect CI integration status information
    fn collect_ci_status(project_path: &Path) -> Result<CIStatus> {
        let mut ci_status = CIStatus {
            is_integrated: false,
            integration_type: None,
            claude_md_path: None,
            claude_local_md_path: None,
            claude_md_modified: None,
            ci_version: None,
            cir_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        // Check for CLAUDE.md
        let claude_md_path = project_path.join("CLAUDE.md");
        if claude_md_path.exists() {
            ci_status.is_integrated = true;
            ci_status.claude_md_path = Some(claude_md_path.clone());
            
            // Get last modified time
            if let Ok(metadata) = fs::metadata(&claude_md_path) {
                if let Ok(modified) = metadata.modified() {
                    let datetime: DateTime<Utc> = modified.into();
                    ci_status.claude_md_modified = Some(datetime);
                }
            }
        }
        
        // Check for CLAUDE.local.md
        let claude_local_md_path = project_path.join("CLAUDE.local.md");
        if claude_local_md_path.exists() {
            ci_status.claude_local_md_path = Some(claude_local_md_path);
        }
        
        // Determine integration type
        let cir_dir = project_path.join(".ci");
        let sibling_ci_path = project_path.parent().map(|p| p.join("CollaborativeIntelligence"));
        
        if cir_dir.exists() {
            // Check for integration type file
            let integration_type_path = cir_dir.join("integration_type");
            if integration_type_path.exists() {
                if let Ok(integration_type) = fs::read_to_string(integration_type_path) {
                    ci_status.integration_type = Some(integration_type.trim().to_string());
                }
            } else if let Some(sibling_path) = sibling_ci_path {
                if sibling_path.exists() {
                    ci_status.integration_type = Some("sibling".to_string());
                } else {
                    ci_status.integration_type = Some("embedded".to_string());
                }
            }
        }
        
        // Get CI version if available
        let ci_command = Command::new("CI")
            .arg("version")
            .output();
            
        if let Ok(output) = ci_command {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stdout);
                if let Some(version_line) = version_output.lines().next() {
                    if version_line.contains("version") {
                        let parts: Vec<&str> = version_line.split_whitespace().collect();
                        if parts.len() > 1 {
                            ci_status.ci_version = Some(parts[parts.len() - 1].to_string());
                        }
                    }
                }
            }
        }
        
        Ok(ci_status)
    }
    
    /// Collect agent status information
    fn collect_agent_status(project_path: &Path) -> Result<Vec<AgentStatus>> {
        let mut agents = Vec::new();
        
        // Determine where to look for agents
        let agents_dir = project_path.join("AGENTS");
        if !agents_dir.exists() {
            return Ok(agents);
        }
        
        // List agent files
        if let Ok(entries) = fs::read_dir(agents_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Skip non-markdown files
                if !path.is_file() || path.extension().map_or(true, |ext| ext != "md") {
                    continue;
                }
                
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                let agent_name = file_name.trim_end_matches(".md").to_string();
                
                // Skip non-agent files
                if agent_name.starts_with("README") || agent_name.contains("NOTIFICATION") {
                    continue;
                }
                
                // Get file metadata
                let metadata = fs::metadata(&path).ok();
                let last_modified = metadata.as_ref().and_then(|m| {
                    m.modified().ok().map(|modified| {
                        let datetime: DateTime<Utc> = modified.into();
                        datetime
                    })
                });
                
                let memory_size = metadata.map(|m| m.len());
                
                // Get agent description
                let mut description = None;
                let mut capabilities = Vec::new();
                
                if let Ok(content) = fs::read_to_string(&path) {
                    // Try to extract description from first paragraph
                    for line in content.lines().take(10) {
                        if !line.is_empty() && !line.starts_with('#') {
                            description = Some(line.trim().to_string());
                            break;
                        }
                    }
                    
                    // Extract capabilities
                    for line in content.lines() {
                        if line.contains("capability") || line.contains("Capability") || 
                           line.contains("responsible for") || line.contains("expertise in") {
                            capabilities.push(line.trim().to_string());
                        }
                        
                        // Limit to 5 capabilities
                        if capabilities.len() >= 5 {
                            break;
                        }
                    }
                }
                
                agents.push(AgentStatus {
                    name: agent_name,
                    description,
                    memory_path: Some(path.clone()),
                    last_modified,
                    memory_size,
                    capabilities,
                });
            }
        }
        
        Ok(agents)
    }
    
    /// Collect system resource usage information
    fn collect_system_resources(project_path: &Path) -> Result<SystemResources> {
        // Default values
        let mut resources = SystemResources {
            memory_usage: 0.0,
            cpu_usage: 0.0,
            disk_usage: 0.0,
            project_size: 0,
        };
        
        // Get project size
        resources.project_size = Self::get_directory_size(project_path)?;
        
        // Get system memory usage
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("free").arg("-m").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(mem_line) = output_str.lines().nth(1) {
                    let parts: Vec<&str> = mem_line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let (Ok(total), Ok(used)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>()) {
                            resources.memory_usage = (used / total) * 100.0;
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("vm_stat").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut _page_size = 4096; // Default page size in bytes
                let mut free_pages = 0;
                let mut total_pages = 0;
                
                // Extract page size if available
                if let Some(line) = output_str.lines().find(|l| l.contains("page size of")) {
                    if let Some(size_str) = line.split("page size of").nth(1) {
                        if let Ok(size) = size_str.trim().parse::<usize>() {
                            _page_size = size;
                        }
                    }
                }
                
                // Extract memory statistics
                for line in output_str.lines() {
                    if line.contains("Pages free:") {
                        if let Some(num_str) = line.split(':').nth(1) {
                            if let Ok(num) = num_str.trim().replace('.', "").parse::<usize>() {
                                free_pages = num;
                            }
                        }
                    } else if line.contains("Pages active:") || line.contains("Pages inactive:") || 
                              line.contains("Pages wired down:") || line.contains("Pages purgeable:") {
                        if let Some(num_str) = line.split(':').nth(1) {
                            if let Ok(num) = num_str.trim().replace('.', "").parse::<usize>() {
                                total_pages += num;
                            }
                        }
                    }
                }
                
                // Calculate memory usage
                total_pages += free_pages;
                if total_pages > 0 {
                    let used_pages = total_pages - free_pages;
                    resources.memory_usage = (used_pages as f32 / total_pages as f32) * 100.0;
                }
            }
        }
        
        // Get CPU usage
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("top").args(["-bn1"]).output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(cpu_line) = output_str.lines().find(|l| l.contains("Cpu(s)")) {
                    if let Some(usage_str) = cpu_line.split(':').nth(1) {
                        if let Some(user_usage) = usage_str.split(',').next() {
                            if let Ok(usage) = user_usage.trim().split('%').next().unwrap_or("0").parse::<f32>() {
                                resources.cpu_usage = usage;
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("top").args(["-l", "1", "-n", "0"]).output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(cpu_line) = output_str.lines().find(|l| l.contains("CPU usage")) {
                    if let Some(usage_str) = cpu_line.split(':').nth(1) {
                        let parts: Vec<&str> = usage_str.split(',').collect();
                        if parts.len() >= 2 {
                            if let Some(user_str) = parts[0].split('%').next() {
                                if let Ok(user) = user_str.trim().parse::<f32>() {
                                    if let Some(sys_str) = parts[1].split('%').next() {
                                        if let Ok(sys) = sys_str.trim().parse::<f32>() {
                                            resources.cpu_usage = user + sys;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Get disk usage
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            if let Ok(output) = Command::new("df").args(["-h", "."]).output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(usage_line) = output_str.lines().nth(1) {
                    let parts: Vec<&str> = usage_line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        let usage_str = parts[4].trim_end_matches('%');
                        if let Ok(usage) = usage_str.parse::<f32>() {
                            resources.disk_usage = usage;
                        }
                    }
                }
            }
        }
        
        Ok(resources)
    }
    
    /// Calculate the size of a directory recursively
    fn get_directory_size(path: &Path) -> Result<u64> {
        let mut total_size = 0;
        
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                return Ok(metadata.len());
            }
            return Ok(0);
        }
        
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                
                // Skip git directory
                if entry_path.file_name().map_or(false, |name| name == ".git") {
                    continue;
                }
                
                // Skip node_modules
                if entry_path.file_name().map_or(false, |name| name == "node_modules") {
                    continue;
                }
                
                // Skip target directory
                if entry_path.file_name().map_or(false, |name| name == "target") {
                    continue;
                }
                
                if entry_path.is_dir() {
                    total_size += Self::get_directory_size(&entry_path)?;
                } else if let Ok(metadata) = fs::metadata(&entry_path) {
                    total_size += metadata.len();
                }
            }
        }
        
        Ok(total_size)
    }
    
    /// Display the full status report
    pub fn display_status_report(report: &StatusReport) {
        println!("\n{}", "═════════════════════════════════════════════════════".cyan());
        println!(" {} {} {}", "📊".cyan(), "CI Status Report".cyan().bold(), format!("[{}]", Local::now().format("%Y-%m-%d %H:%M:%S")).cyan());
        println!("{}", "═════════════════════════════════════════════════════".cyan());
        
        // Repository Status
        println!("\n{}", "Repository Status:".green().bold());
        if report.repo_status.is_git_repo {
            println!("  Branch: {}", report.repo_status.branch.as_ref()
                .map(|b| b.green().to_string())
                .unwrap_or_else(|| "unknown".yellow().to_string()));
            
            println!("  Commits: {}", report.repo_status.commit_count.to_string().cyan());
            
            if report.repo_status.has_uncommitted_changes {
                println!("  Changes: {}", "uncommitted changes".yellow());
            } else {
                println!("  Changes: {}", "clean".green());
            }
            
            // Display remote info
            if let Some(remote) = &report.repo_status.remote {
                println!("  Remote: {}", remote.url.cyan());
                
                // Show ahead/behind status
                let mut status_parts = Vec::new();
                if remote.commits_ahead > 0 {
                    status_parts.push(format!("{} commits ahead", remote.commits_ahead));
                }
                if remote.commits_behind > 0 {
                    status_parts.push(format!("{} commits behind", remote.commits_behind));
                }
                
                if !status_parts.is_empty() {
                    println!("  Remote Status: {}", status_parts.join(", ").yellow());
                } else {
                    println!("  Remote Status: {}", "up to date".green());
                }
            } else {
                println!("  Remote: {}", "none".yellow());
            }
            
            // Display last commit info
            if let Some(commit) = &report.repo_status.last_commit {
                println!("  Last Commit: {} ({})", commit.message.cyan(), 
                    commit.hash.chars().take(8).collect::<String>().yellow());
                println!("  Author: {} on {}", commit.author.cyan(), commit.date.yellow());
            }
        } else {
            println!("  {}", "Not a git repository".yellow());
        }
        
        // CI Integration Status
        println!("\n{}", "CI Integration Status:".blue().bold());
        if report.ci_status.is_integrated {
            println!("  Integration: {}", "Active".green());
            
            if let Some(integration_type) = &report.ci_status.integration_type {
                println!("  Type: {}", integration_type.cyan());
            }
            
            if let Some(claude_md_path) = &report.ci_status.claude_md_path {
                println!("  CLAUDE.md: {}", claude_md_path.display().to_string().cyan());
                
                if let Some(modified) = report.ci_status.claude_md_modified {
                    let local_time: DateTime<Local> = modified.into();
                    println!("  Last Modified: {}", local_time.format("%Y-%m-%d %H:%M:%S").to_string().yellow());
                }
            }
            
            if let Some(claude_local_md_path) = &report.ci_status.claude_local_md_path {
                println!("  CLAUDE.local.md: {}", claude_local_md_path.display().to_string().cyan());
            }
            
            println!("  CI Version: {}", report.ci_status.cir_version.cyan());
            
            if let Some(ci_version) = &report.ci_status.ci_version {
                println!("  CI Version: {}", ci_version.cyan());
            }
        } else {
            println!("  {}", "CI not integrated".yellow());
            println!("  Run 'ci integrate' to integrate CI with this project");
        }
        
        // Agent Status
        println!("\n{}", "Agent Status:".magenta().bold());
        if report.agent_status.is_empty() {
            println!("  {}", "No agents found".yellow());
            println!("  Use 'ci load <agent>' to load an agent");
        } else {
            println!("  Found {} agents:", report.agent_status.len());
            for agent in &report.agent_status {
                println!("  • {} - {}", agent.name.magenta(), 
                    agent.description.as_ref().unwrap_or(&"No description".to_string()).cyan());
                
                if let Some(last_modified) = agent.last_modified {
                    let local_time: DateTime<Local> = last_modified.into();
                    println!("    Last Modified: {}", local_time.format("%Y-%m-%d %H:%M:%S").to_string().yellow());
                }
                
                if !agent.capabilities.is_empty() {
                    let cap_summary = if agent.capabilities.len() > 2 {
                        format!("{} and {} more", 
                            agent.capabilities[0].chars().take(50).collect::<String>(),
                            agent.capabilities.len() - 1)
                    } else {
                        agent.capabilities[0].chars().take(60).collect::<String>()
                    };
                    println!("    Capabilities: {}", cap_summary.cyan());
                }
            }
        }
        
        // System Resources
        println!("\n{}", "System Resources:".yellow().bold());
        
        // Format project size for display
        let size_display = if report.system_resources.project_size > 1_000_000_000 {
            format!("{:.2} GB", report.system_resources.project_size as f64 / 1_000_000_000.0)
        } else if report.system_resources.project_size > 1_000_000 {
            format!("{:.2} MB", report.system_resources.project_size as f64 / 1_000_000.0)
        } else if report.system_resources.project_size > 1_000 {
            format!("{:.2} KB", report.system_resources.project_size as f64 / 1_000.0)
        } else {
            format!("{} bytes", report.system_resources.project_size)
        };
        
        println!("  Project Size: {}", size_display.cyan());
        println!("  Memory Usage: {:.1}%", report.system_resources.memory_usage);
        println!("  CPU Usage: {:.1}%", report.system_resources.cpu_usage);
        println!("  Disk Usage: {:.1}%", report.system_resources.disk_usage);
        
        println!("\n{}", "═════════════════════════════════════════════════════".cyan());
    }
    
    /// Display a compact status report
    pub fn display_compact_status(report: &StatusReport) {
        println!("\n{} {} {}", "📊".cyan(), "CI Status".cyan().bold(), 
            format!("[{}]", Local::now().format("%Y-%m-%d %H:%M:%S")).cyan());
            
        // Repository
        if report.repo_status.is_git_repo {
            let branch = report.repo_status.branch.as_ref().map_or("unknown", |b| b);
            let changes = if report.repo_status.has_uncommitted_changes {
                "uncommitted changes".yellow()
            } else {
                "clean".green()
            };
            
            println!("  Repo: {} ({}) - {}", branch.green(), report.repo_status.commit_count, changes);
            
            // Show remote status if available
            if let Some(remote) = &report.repo_status.remote {
                let status = if remote.commits_ahead > 0 || remote.commits_behind > 0 {
                    let mut parts = vec![];
                    if remote.commits_ahead > 0 {
                        parts.push(format!("{}↑", remote.commits_ahead));
                    }
                    if remote.commits_behind > 0 {
                        parts.push(format!("{}↓", remote.commits_behind));
                    }
                    parts.join(" ").yellow()
                } else {
                    "↔".green()
                };
                
                println!("  Remote: {} {}", remote.name.cyan(), status);
            }
        } else {
            println!("  Repo: {}", "not a git repository".yellow());
        }
        
        // CI Integration
        if report.ci_status.is_integrated {
            let integration_type = report.ci_status.integration_type.as_ref().map_or("embedded", |t| t);
            println!("  CI: {} ({})", "integrated".green(), integration_type);
            println!("  Version: {} CI, {}", 
                report.ci_status.cir_version.cyan(),
                report.ci_status.ci_version.as_ref().map_or("CI not found".yellow().to_string(), |v| v.cyan().to_string())
            );
        } else {
            println!("  CI: {}", "not integrated".yellow());
        }
        
        // Agents
        let agent_count = report.agent_status.len();
        if agent_count > 0 {
            let agent_names: Vec<String> = report.agent_status.iter()
                .take(3)
                .map(|a| a.name.clone())
                .collect();
                
            let display = if agent_count <= 3 {
                agent_names.join(", ")
            } else {
                format!("{} and {} more", agent_names.join(", "), agent_count - 3)
            };
            
            println!("  Agents: {}", display.magenta());
        } else {
            println!("  Agents: {}", "none".yellow());
        }
        
        // Size
        let size_display = if report.system_resources.project_size > 1_000_000_000 {
            format!("{:.2} GB", report.system_resources.project_size as f64 / 1_000_000_000.0)
        } else if report.system_resources.project_size > 1_000_000 {
            format!("{:.2} MB", report.system_resources.project_size as f64 / 1_000_000.0)
        } else if report.system_resources.project_size > 1_000 {
            format!("{:.2} KB", report.system_resources.project_size as f64 / 1_000.0)
        } else {
            format!("{} bytes", report.system_resources.project_size)
        };
        
        println!("  Size: {}", size_display.cyan());
    }
}