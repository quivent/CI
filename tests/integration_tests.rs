use std::fs;
use assert_cmd::Command;
use predicates::prelude::*;
use crate::test_helpers::{TestEnv, run_cir};

// Test project initialization and verification
#[test]
fn test_project_lifecycle() {
    let test_env = TestEnv::new();
    let cir_repo = test_env.setup_mock_cir_repo();
    
    // Test init command
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["init", "test-project", "--agents", "Athena,ProjectArchitect"])
        .env("CI_REPO_PATH", cir_repo.to_string_lossy().to_string())
        .assert()
        .success();
    
    // Verify project was created correctly
    let project_dir = test_env.path("test-project");
    let claude_md_path = project_dir.join("CLAUDE.md");
    assert!(claude_md_path.exists());
    
    let claude_md_content = fs::read_to_string(&claude_md_path).unwrap();
    assert!(claude_md_content.contains("# Project: test-project"));
    assert!(claude_md_content.contains("- Athena"));
    assert!(claude_md_content.contains("- ProjectArchitect"));
    
    // Test verify command
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["verify"])
        .current_dir(&project_dir)
        .env("CI_REPO_PATH", cir_repo.to_string_lossy().to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("Verification successful"));
}

// Test git operations integration
#[test]
fn test_git_operations() {
    let test_env = TestEnv::new();
    let repo_dir = test_env.setup_git_repo();
    
    // Test status command
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["status"])
        .current_dir(&repo_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository Status"))
        .stdout(predicate::str::contains("Branch"));
    
    // Create a new file
    let test_file = repo_dir.join("test.txt");
    fs::write(&test_file, "Test content").unwrap();
    
    // Test stage command
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["stage"])
        .current_dir(&repo_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Staged files"));
    
    // Verify git status shows staged files
    let output = std::process::Command::new("git")
        .args(["status"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    
    let status_output = String::from_utf8_lossy(&output.stdout);
    assert!(status_output.contains("Changes to be committed") || 
            status_output.contains("new file:") || 
            status_output.contains("test.txt"));
}

// Test CI integration with existing project
#[test]
fn test_integrate_command() {
    let test_env = TestEnv::new();
    let repo_dir = test_env.setup_git_repo();
    let cir_repo = test_env.setup_mock_cir_repo();
    
    // Test integrate command
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["integrate", "--agents", "Athena"])
        .current_dir(&repo_dir)
        .env("CI_REPO_PATH", cir_repo.to_string_lossy().to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration successful"));
    
    // Verify CLAUDE.md was created
    let claude_md_path = repo_dir.join("CLAUDE.md");
    assert!(claude_md_path.exists());
    
    let claude_md_content = fs::read_to_string(&claude_md_path).unwrap();
    assert!(claude_md_content.contains("Integration:"));
    assert!(claude_md_content.contains("- Athena"));
}

// Test command sequence (workflow integration)
#[test]
fn test_command_sequence() {
    let test_env = TestEnv::new();
    let cir_repo = test_env.setup_mock_cir_repo();
    
    // Create a new project
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["init", "workflow-test", "--agents", "Athena"])
        .env("CI_REPO_PATH", cir_repo.to_string_lossy().to_string())
        .assert()
        .success();
    
    let project_dir = test_env.path("workflow-test");
    
    // Create a test file
    let test_file = project_dir.join("test.txt");
    fs::write(&test_file, "Test content").unwrap();
    
    // Stage, verify and status should work in sequence
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["stage"])
        .current_dir(&project_dir)
        .assert()
        .success();
    
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["verify"])
        .current_dir(&project_dir)
        .env("CI_REPO_PATH", cir_repo.to_string_lossy().to_string())
        .assert()
        .success();
    
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["status"])
        .current_dir(&project_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository Status"));
}