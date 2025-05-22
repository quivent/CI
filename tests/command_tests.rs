use std::fs;
use assert_cmd::Command;
use predicates::prelude::*;
use crate::test_helpers::TestEnv;

// Basic test to ensure the CI binary loads and runs
#[test]
fn test_cir_binary_runs() {
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        // Try alternative binary name
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("CI"));
}

// Test the intent command
#[test]
fn test_intent_command() {
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.arg("intent")
        .assert()
        .success()
        .stdout(predicate::str::contains("Collaborative Intelligence"))
        .stdout(predicate::str::contains("modern CLI tool"));
}

// Test agents command with mock repository
#[test]
fn test_agents_command() {
    let test_env = TestEnv::new();
    let cir_repo = test_env.setup_mock_cir_repo();
    
    // Test basic agents list
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.arg("agents")
        .env("CI_REPO_PATH", cir_repo.to_string_lossy().to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("Athena"))
        .stdout(predicate::str::contains("ProjectArchitect"));
}

// Test ignore command functionality
#[test]
fn test_ignore_command() {
    let test_env = TestEnv::new();
    let repo_dir = test_env.setup_git_repo();
    
    // Create a basic .gitignore file
    let gitignore_path = repo_dir.join(".gitignore");
    fs::write(&gitignore_path, "node_modules/\n").unwrap();
    
    // Run the ignore command
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["ignore"])
        .current_dir(&repo_dir)
        .assert()
        .success();
    
    // Verify .gitignore was updated
    let gitignore_content = fs::read_to_string(&gitignore_path).unwrap();
    assert!(gitignore_content.contains("node_modules/"));
    assert!(gitignore_content.contains("CLAUDE.local.md"));
}

// Test verify command with a properly set up repository
#[test]
fn test_verify_command() {
    let test_env = TestEnv::new();
    let repo_dir = test_env.setup_cir_integrated_repo();
    let cir_repo = test_env.setup_mock_cir_repo();
    
    // Run the verify command
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["verify"])
        .current_dir(&repo_dir)
        .env("CI_REPO_PATH", cir_repo.to_string_lossy().to_string())
        .assert()
        .success();
    
    // Test failing verification (missing CLAUDE.md)
    let bad_repo_dir = test_env.setup_git_repo();
    
    let mut cmd = Command::cargo_bin("ci").unwrap_or_else(|_| {
        Command::cargo_bin("CollaborativeIntelligenceRust")
            .expect("Failed to find CI binary")
    });
    
    cmd.args(["verify"])
        .current_dir(&bad_repo_dir)
        .env("CI_REPO_PATH", cir_repo.to_string_lossy().to_string())
        .assert()
        .success() // Command should still succeed, but...
        .stdout(predicate::str::contains("Not a CI project")); // ...should report issues
}