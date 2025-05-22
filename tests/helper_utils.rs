use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use anyhow::{Result, anyhow};

/// Utility struct for working with commands in tests
pub struct CommandUtils;

impl CommandUtils {
    /// Check if a path is a git repository
    pub fn is_git_repository(path: &Path) -> bool {
        let git_dir = path.join(".git");
        git_dir.exists() && git_dir.is_dir()
    }
    
    /// Create a file with content
    pub fn create_file_with_content(path: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Read file content as string
    pub fn read_file_content(path: &Path) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }
    
    /// Execute a command with progress indication in tests
    pub fn with_progress<F, T>(message: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        println!("- {} ...", message);
        let result = f()?;
        println!("✓ {} ... done", message);
        Ok(result)
    }
    
    /// Check if a command exists in the system PATH
    pub fn command_exists(command: &str) -> bool {
        let output = if cfg!(target_os = "windows") {
            Command::new("where")
                .arg(command)
                .output()
        } else {
            Command::new("which")
                .arg(command)
                .output()
        };

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    /// Run an external process with optional custom environment
    pub fn run_process(
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
        env_vars: Option<&[(String, String)]>,
    ) -> Result<Output> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        
        // Set working directory if provided
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        
        // Set environment variables if provided
        if let Some(vars) = env_vars {
            for (key, value) in vars {
                cmd.env(key, value);
            }
        }
        
        let output = cmd.output()?;
        Ok(output)
    }
}

/// Utility struct for working with repositories in tests
pub struct RepositoryUtils;

impl RepositoryUtils {
    /// Create a default .gitignore file for CI projects
    pub fn create_default_gitignore(path: &Path) -> Result<PathBuf> {
        let gitignore_path = path.join(".gitignore");
        let content = r#".DS_Store
Thumbs.db
.env
.env.local
.env.development.local
.env.test.local
.env.production.local
node_modules/
dist/
target/
CLAUDE.local.md
.ci/
.vscode/
.idea/
*.swp
*.swo
"#;
        fs::write(&gitignore_path, content)?;
        Ok(gitignore_path)
    }
    
    /// Create a basic CLAUDE.md file for testing
    pub fn create_claude_md(
        path: &Path,
        project_name: &str,
        integration_type: &str,
        agents: &[String],
    ) -> Result<PathBuf> {
        let claude_md_path = path.join("CLAUDE.md");
        
        let agents_section = agents.iter()
            .map(|agent| format!("- {}", agent))
            .collect::<Vec<_>>()
            .join("\n");
        
        let content = format!(
            r#"# Project: {}

## Configuration
Created: {}
Integration: {} integration

## Active Agents
{}

## Project Settings
- Use helpers for common operations
- Maintain consistent code style
- Prioritize documentation and comments
- Add tests for new functionality
"#,
            project_name,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            integration_type,
            agents_section
        );
        
        fs::write(&claude_md_path, content)?;
        Ok(claude_md_path)
    }
    
    /// Get the nearest git repository from a path (traverses upward)
    pub fn find_git_repository(path: &Path) -> Option<PathBuf> {
        let mut current = path.to_path_buf();
        
        loop {
            if CommandUtils::is_git_repository(&current) {
                return Some(current);
            }
            
            if !current.pop() {
                return None;
            }
        }
    }
    
    /// Get the current git branch name for a repository
    pub fn get_current_branch(repo_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(repo_path)
            .output()?;
            
        if output.status.success() {
            let branch = String::from_utf8(output.stdout)?;
            Ok(branch.trim().to_string())
        } else {
            Err(anyhow!("Failed to get current branch name"))
        }
    }
}

/// Utility struct for working with CI configuration in tests
pub struct ConfigUtils;

impl ConfigUtils {
    /// Get the CI repository path
    pub fn get_cir_repo_path() -> Result<PathBuf> {
        if let Ok(path) = env::var("CI_REPO_PATH") {
            return Ok(PathBuf::from(path));
        }
        
        // Try to find in default locations
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let default_path = home_dir.join(".ci");
        
        if default_path.exists() && default_path.is_dir() {
            return Ok(default_path);
        }
        
        Err(anyhow!("CI repository path not found"))
    }
    
    /// Check if a directory is a valid CI project
    pub fn is_cir_project(path: &Path) -> bool {
        let claude_md_path = path.join("CLAUDE.md");
        
        if !claude_md_path.exists() || !claude_md_path.is_file() {
            return false;
        }
        
        let content = fs::read_to_string(claude_md_path).unwrap_or_default();
        content.contains("Project:") && content.contains("Integration:")
    }
    
    /// Extract project name from CLAUDE.md
    pub fn extract_project_name(path: &Path) -> Result<String> {
        let claude_md_path = path.join("CLAUDE.md");
        
        if !claude_md_path.exists() || !claude_md_path.is_file() {
            return Err(anyhow!("CLAUDE.md does not exist"));
        }
        
        let content = fs::read_to_string(claude_md_path)?;
        
        // Try to extract project name from "# Project: NAME" line
        for line in content.lines() {
            if line.starts_with("# Project:") {
                let project_name = line.trim_start_matches("# Project:").trim().to_string();
                return Ok(project_name);
            }
        }
        
        Err(anyhow!("Could not extract project name from CLAUDE.md"))
    }
}

/// Utility struct for testing agent-specific functionality
pub struct AgentUtils;

impl AgentUtils {
    /// Get a list of available agents from a CI repository
    pub fn get_available_agents(cir_repo_path: &Path) -> Result<Vec<String>> {
        let agents_dir = cir_repo_path.join("AGENTS");
        
        if !agents_dir.exists() || !agents_dir.is_dir() {
            return Err(anyhow!("AGENTS directory not found in CI repository"));
        }
        
        let mut agents = Vec::new();
        
        for entry in fs::read_dir(agents_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        agents.push(name_str.to_string());
                    }
                }
            }
        }
        
        Ok(agents)
    }
    
    /// Check if an agent exists in the CI repository
    pub fn agent_exists(cir_repo_path: &Path, agent_name: &str) -> bool {
        let agent_dir = cir_repo_path.join("AGENTS").join(agent_name);
        agent_dir.exists() && agent_dir.is_dir()
    }
}