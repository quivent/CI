use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};
use std::process::{Command as ProcessCommand, Output};

/// Test helper to create a temporary test environment
pub struct TestEnv {
    /// The temporary directory
    pub temp_dir: TempDir,
    /// Original working directory
    original_dir: PathBuf,
    /// Original CI_REPO_PATH environment variable value (if any)
    original_cir_repo_path: Option<String>,
}

impl TestEnv {
    /// Create a new test environment with a temporary directory
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let original_dir = env::current_dir().expect("Failed to get current directory");
        let original_cir_repo_path = env::var("CI_REPO_PATH").ok();

        let env = Self {
            temp_dir,
            original_dir,
            original_cir_repo_path,
        };

        // Set working directory to the temp dir
        env::set_current_dir(env.temp_dir.path()).expect("Failed to set current directory");

        env
    }

    /// Get path within the temporary directory
    pub fn path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.temp_dir.path().join(path)
    }

    /// Create a file with the given content in the test environment
    pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> PathBuf {
        let file_path = self.path(path);
        
        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(content.as_bytes()).expect("Failed to write to file");
        
        file_path
    }

    /// Create a directory in the test environment
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let dir_path = self.path(path);
        fs::create_dir_all(&dir_path).expect("Failed to create directory");
        dir_path
    }

    /// Set up a mock CI repository structure for testing
    pub fn setup_mock_cir_repo(&self) -> PathBuf {
        // Create CI repo directory
        let cir_repo_dir = self.create_dir("cir_repo");
        
        // Create a basic CLAUDE.md file
        self.create_file(
            "cir_repo/CLAUDE.md",
            r#"# CI - Collaborative Intelligence in Rust

CI is a modern CLI tool for working with the Collaborative Intelligence system.

## Project Behavior
1. Begin implementation of tasks immediately
2. Prioritize completing tasks in order
3. Make intelligent decisions without excessive consultation
4. Automatically test code changes when possible
5. Proactively request necessary information if required
"#,
        );
        
        // Create AGENTS directory with some test agents
        let agents_dir = self.create_dir("cir_repo/AGENTS");
        
        // Create a couple of agent directories
        self.create_dir("cir_repo/AGENTS/Athena");
        self.create_dir("cir_repo/AGENTS/ProjectArchitect");
        
        // Create some agent memory files
        self.create_file(
            "cir_repo/AGENTS/Athena/Athena_memory.md",
            "# Agent Memory: Athena\n\nKnowledge management and research specialist."
        );
        
        self.create_file(
            "cir_repo/AGENTS/ProjectArchitect/ProjectArchitect_memory.md",
            "# Agent Memory: ProjectArchitect\n\nSystem design and architecture planning."
        );
        
        // Create templates directory for command templates
        let templates_dir = self.create_dir("cir_repo/templates/basic");
        
        // Create a sample template file
        self.create_file(
            "cir_repo/templates/basic/example.txt",
            "This is a sample template file."
        );
        
        // Set CI_REPO_PATH environment variable
        env::set_var("CI_REPO_PATH", cir_repo_dir.to_string_lossy().to_string());
        
        cir_repo_dir
    }

    /// Create a mock git repository for testing
    pub fn setup_git_repo(&self) -> PathBuf {
        let repo_dir = self.create_dir("git_repo");
        
        // Set working directory to repo dir temporarily
        let current_dir = env::current_dir().expect("Failed to get current directory");
        env::set_current_dir(&repo_dir).expect("Failed to set current directory");
        
        // Initialize git repo
        ProcessCommand::new("git")
            .args(["init"])
            .output()
            .expect("Failed to initialize git repository");
            
        // Create a basic README.md
        let readme_path = repo_dir.join("README.md");
        let mut file = File::create(readme_path).expect("Failed to create README.md");
        file.write_all(b"# Test Repository\n\nThis is a test repository for CI testing.\n")
            .expect("Failed to write to README.md");
            
        // Create a basic .gitignore
        let gitignore_path = repo_dir.join(".gitignore");
        let mut file = File::create(gitignore_path).expect("Failed to create .gitignore");
        file.write_all(b".DS_Store\nnode_modules/\n")
            .expect("Failed to write to .gitignore");
            
        // Set git config for tests
        ProcessCommand::new("git")
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("Failed to set git user.name");
            
        ProcessCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("Failed to set git user.email");
            
        // Stage and commit the files
        ProcessCommand::new("git")
            .args(["add", "."])
            .output()
            .expect("Failed to stage files");
            
        ProcessCommand::new("git")
            .args(["commit", "-m", "Initial commit"])
            .output()
            .expect("Failed to create initial commit");
            
        // Restore working directory
        env::set_current_dir(current_dir).expect("Failed to restore current directory");
        
        repo_dir
    }
    
    /// Create a CI-integrated test repository
    pub fn setup_cir_integrated_repo(&self) -> PathBuf {
        let repo_dir = self.setup_git_repo();
        
        // Create CI integration files
        self.create_file(
            repo_dir.join("CLAUDE.md"),
            r#"# Project: TestProject

## Configuration
Created: 2023-01-01 12:00:00
Integration: embedded integration

## Active Agents
- Athena
- ProjectArchitect

## Project Settings
- Use helpers for common operations
- Maintain consistent code style
- Prioritize documentation and comments
- Add tests for new functionality
"#
        );
        
        repo_dir
    }
    
    /// Create a test CI binary (shell script)
    pub fn create_test_cir_binary(&self) -> PathBuf {
        let bin_dir = self.create_dir("bin");
        let bin_path = bin_dir.join("ci");
        
        self.create_file(
            &bin_path,
            r#"#!/bin/sh
echo "CI Test Binary"
echo "Args: $@"
"#
        );
        
        // Make it executable
        ProcessCommand::new("chmod")
            .args(["+x", bin_path.to_str().unwrap()])
            .output()
            .expect("Failed to make test binary executable");
            
        // Add to PATH
        let path = env::var("PATH").unwrap_or_default();
        env::set_var("PATH", format!("{}:{}", bin_dir.to_string_lossy(), path));
        
        bin_path
    }

    /// Run a git command in the test environment
    pub fn run_git(&self, args: &[&str], dir: &Path) -> Output {
        ProcessCommand::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("Failed to run git command")
    }

    /// Setup advanced mock project with commit history
    pub fn setup_advanced_git_repo(&self) -> PathBuf {
        let repo_dir = self.setup_git_repo();
        
        // Create additional test files with content
        self.create_file(
            repo_dir.join("src/main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}
"#
        );
        
        self.create_file(
            repo_dir.join("src/lib.rs"),
            r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#
        );
        
        // Stage and commit these files
        self.run_git(&["add", "."], &repo_dir);
        self.run_git(&["commit", "-m", "Add Rust source files"], &repo_dir);
        
        // Add a test config file
        self.create_file(
            repo_dir.join("config.toml"),
            r#"[general]
name = "Test Project"
version = "0.1.0"

[settings]
debug = true
log_level = "info"
"#
        );
        
        // Stage and commit this file too
        self.run_git(&["add", "."], &repo_dir);
        self.run_git(&["commit", "-m", "Add configuration file"], &repo_dir);
        
        // Create a branch for testing
        self.run_git(&["checkout", "-b", "test-branch"], &repo_dir);
        
        // Make changes on this branch
        self.create_file(
            repo_dir.join("README.md"),
            r#"# Test Repository

This is a test repository for CI testing.

## Features
- Testing git operations
- Verifying CI functionality
- Simulating real-world scenarios
"#
        );
        
        // Stage and commit on branch
        self.run_git(&["add", "."], &repo_dir);
        self.run_git(&["commit", "-m", "Update README with features"], &repo_dir);
        
        // Switch back to main branch
        self.run_git(&["checkout", "main"], &repo_dir);
        
        repo_dir
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        // Restore working directory
        let _ = env::set_current_dir(&self.original_dir);
        
        // Restore CI_REPO_PATH environment variable
        match &self.original_cir_repo_path {
            Some(path) => env::set_var("CI_REPO_PATH", path),
            None => env::remove_var("CI_REPO_PATH"),
        }
    }
}

/// Runs the CI binary with the given arguments and returns the output
pub fn run_cir(args: &[&str]) -> Output {
    // Try to find binary by cargo built binary
    let binary_path = std::env::current_exe()
        .ok()
        .and_then(|mut path| {
            path.pop(); // Remove the test executable name
            path.pop(); // Remove the "deps" directory
            // Try to find the CI binary
            let bin_path = path.join("ci");
            if bin_path.exists() {
                Some(bin_path)
            } else {
                // Fallback for compatibility
                let bin_path = path.join("CollaborativeIntelligenceRust");
                if bin_path.exists() {
                    Some(bin_path)
                } else {
                    None
                }
            }
        })
        .unwrap_or_else(|| {
            // Fallback to the command in PATH
            PathBuf::from("ci")
        });

    ProcessCommand::new(binary_path)
        .args(args)
        .output()
        .expect("Failed to execute CI binary")
}

/// Checks if a command exists in the system PATH
pub fn command_exists(command: &str) -> bool {
    let output = if cfg!(target_os = "windows") {
        ProcessCommand::new("where")
            .arg(command)
            .output()
    } else {
        ProcessCommand::new("which")
            .arg(command)
            .output()
    };

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Runs a shell command with the given arguments and returns the output
pub fn run_command(command: &str, args: &[&str]) -> Output {
    ProcessCommand::new(command)
        .args(args)
        .output()
        .expect(&format!("Failed to execute command: {}", command))
}

/// Creates a CLAUDE.md file with the given project name and agents
pub fn create_claude_md(path: &Path, project_name: &str, agents: &[&str]) -> std::io::Result<PathBuf> {
    let claude_md_path = path.join("CLAUDE.md");
    
    let agents_content = agents.iter()
        .map(|agent| format!("- {}", agent))
        .collect::<Vec<_>>()
        .join("\n");
    
    let content = format!(
        r#"# Project: {}

## Configuration
Created: {}
Integration: CI integration

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
        agents_content
    );
    
    fs::write(&claude_md_path, content)?;
    
    Ok(claude_md_path)
}