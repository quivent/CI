use crate::test_helpers::TestEnv;
use crate::helper_utils::{CommandUtils, RepositoryUtils, ConfigUtils, AgentUtils};
use std::path::Path;
use std::fs;
use anyhow::Result;

#[test]
fn test_command_utils_git_repository() {
    let test_env = TestEnv::new();
    let path = test_env.temp_dir.path();
    
    // Should not be a git repository initially
    assert!(!CommandUtils::is_git_repository(path));
    
    // Initialize git repository
    std::process::Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()
        .unwrap();
    
    // Now it should be a git repository
    assert!(CommandUtils::is_git_repository(path));
}

#[test]
fn test_command_utils_file_operations() -> Result<()> {
    let test_env = TestEnv::new();
    let file_path = test_env.path("test.txt");
    
    // Write file
    let content = "Hello, world!";
    CommandUtils::create_file_with_content(&file_path, content)?;
    
    // Read file
    let read_content = CommandUtils::read_file_content(&file_path)?;
    assert_eq!(read_content, content);
    
    Ok(())
}

#[test]
fn test_repository_utils_gitignore() -> Result<()> {
    let test_env = TestEnv::new();
    let path = test_env.temp_dir.path();
    
    // Create default gitignore
    RepositoryUtils::create_default_gitignore(path)?;
    
    let gitignore_path = path.join(".gitignore");
    assert!(gitignore_path.exists());
    
    let content = std::fs::read_to_string(&gitignore_path)?;
    assert!(content.contains(".ci/"));
    assert!(content.contains("CLAUDE.local.md"));
    assert!(content.contains(".env"));
    
    Ok(())
}

#[test]
fn test_config_utils_claude_config() -> Result<()> {
    let test_env = TestEnv::new();
    let path = test_env.temp_dir.path();
    
    // Create CLAUDE.md
    let agents = vec!["Athena".to_string(), "ProjectArchitect".to_string()];
    let claude_path = RepositoryUtils::create_claude_md(
        path,
        "TestProject",
        "embedded",
        &agents
    )?;
    
    assert!(claude_path.exists());
    
    let content = std::fs::read_to_string(&claude_path)?;
    assert!(content.contains("# Project: TestProject"));
    assert!(content.contains("embedded integration"));
    assert!(content.contains("- Athena"));
    assert!(content.contains("- ProjectArchitect"));
    
    // Test project name extraction
    let project_name = ConfigUtils::extract_project_name(path)?;
    assert_eq!(project_name, "TestProject");
    
    // Test is_cir_project
    assert!(ConfigUtils::is_cir_project(path));
    
    Ok(())
}

#[test]
fn test_command_exists() {
    // Git should exist on most systems
    assert!(CommandUtils::command_exists("git"));
    
    // Non-existent command
    assert!(!CommandUtils::command_exists("this_command_definitely_does_not_exist"));
}

#[test]
fn test_find_git_repository() {
    let test_env = TestEnv::new();
    let repo_dir = test_env.setup_git_repo();
    let nested_dir = test_env.create_dir(repo_dir.join("nested/deep/directory"));
    
    // Should find the repository from the nested directory
    let found_repo = RepositoryUtils::find_git_repository(&nested_dir).unwrap();
    assert_eq!(found_repo, repo_dir);
    
    // Non-git directory should return None
    let non_git_dir = test_env.create_dir("not_a_git_repo");
    assert!(RepositoryUtils::find_git_repository(&non_git_dir).is_none());
}

#[test]
fn test_get_current_branch() -> Result<()> {
    let test_env = TestEnv::new();
    let repo_dir = test_env.setup_advanced_git_repo();
    
    // Should be on main branch
    let branch = RepositoryUtils::get_current_branch(&repo_dir)?;
    assert_eq!(branch, "main");
    
    // Switch to test branch and verify
    test_env.run_git(&["checkout", "test-branch"], &repo_dir);
    let branch = RepositoryUtils::get_current_branch(&repo_dir)?;
    assert_eq!(branch, "test-branch");
    
    Ok(())
}

#[test]
fn test_agent_utils() -> Result<()> {
    let test_env = TestEnv::new();
    let cir_repo = test_env.setup_mock_cir_repo();
    
    // Test get_available_agents
    let agents = AgentUtils::get_available_agents(&cir_repo)?;
    assert!(agents.contains(&"Athena".to_string()));
    assert!(agents.contains(&"ProjectArchitect".to_string()));
    assert_eq!(agents.len(), 2);
    
    // Test agent_exists
    assert!(AgentUtils::agent_exists(&cir_repo, "Athena"));
    assert!(AgentUtils::agent_exists(&cir_repo, "ProjectArchitect"));
    assert!(!AgentUtils::agent_exists(&cir_repo, "NonExistentAgent"));
    
    Ok(())
}

#[test]
fn test_with_progress() -> Result<()> {
    // Test with successful operation
    let result = CommandUtils::with_progress("Testing progress", || {
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(100));
        Ok(42)
    })?;
    
    assert_eq!(result, 42);
    
    // Test with failing operation
    let result = CommandUtils::with_progress("Testing progress with error", || {
        // Simulate failed operation
        Err(anyhow::anyhow!("Test error"))
    });
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Test error");
    
    Ok(())
}

#[test]
fn test_run_process() -> Result<()> {
    let test_env = TestEnv::new();
    let path = test_env.temp_dir.path();
    
    // Create a test file
    let test_file = path.join("test_file.txt");
    fs::write(&test_file, "Hello, world!")?;
    
    // Run "cat" on the file
    let output = CommandUtils::run_process(
        "cat",
        &[test_file.to_str().unwrap()],
        Some(path),
        None
    )?;
    
    assert!(output.status.success());
    let output_str = String::from_utf8(output.stdout)?;
    assert_eq!(output_str, "Hello, world!");
    
    // Test with environment variables
    let env_vars = vec![
        ("TEST_VAR".to_string(), "test_value".to_string())
    ];
    
    let output = CommandUtils::run_process(
        "sh",
        &["-c", "echo $TEST_VAR"],
        Some(path),
        Some(&env_vars)
    )?;
    
    assert!(output.status.success());
    let output_str = String::from_utf8(output.stdout)?;
    assert_eq!(output_str.trim(), "test_value");
    
    Ok(())
}

#[test]
fn test_advanced_git_repo() -> Result<()> {
    let test_env = TestEnv::new();
    let repo_dir = test_env.setup_advanced_git_repo();
    
    // Verify basic structure
    assert!(repo_dir.join("src/main.rs").exists());
    assert!(repo_dir.join("src/lib.rs").exists());
    assert!(repo_dir.join("config.toml").exists());
    
    // Verify git log
    let output = test_env.run_git(&["log", "--oneline"], &repo_dir);
    let log = String::from_utf8(output.stdout)?;
    
    assert!(log.contains("Add configuration file"));
    assert!(log.contains("Add Rust source files"));
    assert!(log.contains("Initial commit"));
    
    // Verify branches
    let output = test_env.run_git(&["branch"], &repo_dir);
    let branches = String::from_utf8(output.stdout)?;
    
    assert!(branches.contains("main"));
    assert!(branches.contains("test-branch"));
    
    Ok(())
}