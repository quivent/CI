//! Tests for CI helper functions

use anyhow::Result;
use std::path::Path;
use std::fs;
use tempfile::TempDir;

// Import helpers
use CI::helpers::command::CommandHelpers;
use CI::helpers::repository::RepositoryHelpers;
use CI::helpers::config::ConfigHelpers;
use CI::helpers::project::{ProjectHelpers, ProjectType};
use CI::helpers::path::PathHelpers;

// Test command helpers
#[test]
fn test_format_file_list() {
    let files = vec!["file1.rs".to_string(), "file2.rs".to_string()];
    let formatted = CommandHelpers::format_file_list(&files);
    assert_eq!(formatted, "  • file1.rs\n  • file2.rs");
}

// Test path helpers
#[test]
fn test_ensure_directory_exists() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_dir = temp_dir.path().join("test_dir");
    
    // Directory should not exist initially
    assert!(!test_dir.exists());
    
    // Create the directory
    PathHelpers::ensure_directory_exists(&test_dir)?;
    
    // Directory should now exist
    assert!(test_dir.exists());
    assert!(test_dir.is_dir());
    
    // Calling again should not error
    PathHelpers::ensure_directory_exists(&test_dir)?;
    
    Ok(())
}

#[test]
fn test_create_file_with_content() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test_file.txt");
    let content = "Test content";
    
    // File should not exist initially
    assert!(!test_file.exists());
    
    // Create the file with content
    PathHelpers::create_file_with_content(&test_file, content)?;
    
    // File should now exist
    assert!(test_file.exists());
    assert!(test_file.is_file());
    
    // Content should match
    let read_content = fs::read_to_string(&test_file)?;
    assert_eq!(read_content, content);
    
    Ok(())
}

// Test config helpers
#[test]
fn test_update_markdown_section() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("CLAUDE.md");
    
    // Create file with initial content
    let initial_content = "# Test File\n\n## Section 1\nContent 1\n\n## Section 2\nContent 2\n";
    fs::write(&test_file, initial_content)?;
    
    // Update existing section
    ConfigHelpers::update_markdown_section(&test_file, "Section 1", "New content for section 1")?;
    
    // Check file content
    let updated_content = fs::read_to_string(&test_file)?;
    assert!(updated_content.contains("## Section 1"));
    assert!(updated_content.contains("New content for section 1"));
    assert!(updated_content.contains("## Section 2"));
    assert!(updated_content.contains("Content 2"));
    
    // Add new section
    ConfigHelpers::update_markdown_section(&test_file, "Section 3", "Content for section 3")?;
    
    // Check file content
    let updated_content = fs::read_to_string(&test_file)?;
    assert!(updated_content.contains("## Section 3"));
    assert!(updated_content.contains("Content for section 3"));
    assert!(updated_content.contains("## Section 1"));
    assert!(updated_content.contains("New content for section 1"));
    
    Ok(())
}

// Test project helpers setup function for test environment
fn setup_test_project() -> Result<(TempDir, ProjectType)> {
    let temp_dir = tempfile::tempdir()?;
    
    // Create basic rust project
    fs::write(temp_dir.path().join("Cargo.toml"), 
        r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#)?;
    
    fs::create_dir_all(temp_dir.path().join("src"))?;
    fs::write(temp_dir.path().join("src/main.rs"), 
        r#"fn main() {
    println!("Hello, world!");
}
"#)?;
    
    Ok((temp_dir, ProjectType::Rust))
}

#[test]
fn test_detect_project_type() -> Result<()> {
    let (temp_dir, expected_type) = setup_test_project()?;
    
    // Test detection
    let detected = ProjectHelpers::detect_project_type(temp_dir.path())?;
    assert_eq!(detected, expected_type);
    
    Ok(())
}

#[test]
fn test_create_claude_config() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let project_name = "Test Project";
    let integration_type = "embedded";
    let agents = vec!["Athena".to_string(), "ProjectArchitect".to_string()];
    
    ConfigHelpers::create_claude_config(temp_dir.path(), project_name, integration_type, &agents)?;
    
    let claude_path = temp_dir.path().join("CLAUDE.md");
    assert!(claude_path.exists());
    
    let content = fs::read_to_string(claude_path)?;
    assert!(content.contains("# Project: Test Project"));
    assert!(content.contains("Integration: embedded integration"));
    assert!(content.contains("- Athena"));
    assert!(content.contains("- ProjectArchitect"));
    
    Ok(())
}