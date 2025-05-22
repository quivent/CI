#[cfg(test)]
mod integration_override_tests {
    use std::path::Path;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    
    // Mock helper for testing
    fn setup_test_project() -> Result<TempDir, Box<dyn std::error::Error>> {
        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        
        // Create a simple CLAUDE.md file
        let claude_md_path = temp_dir.path().join("CLAUDE.md");
        let claude_md_content = r#"# Project: Test Project
# Created: 2025-01-01

## Project Information
This is a test project for integration testing.

## Configuration
Some existing configuration that should be preserved.
"#;
        let mut file = fs::File::create(claude_md_path)?;
        file.write_all(claude_md_content.as_bytes())?;
        
        // Create a simple file structure
        fs::create_dir(temp_dir.path().join("src"))?;
        fs::create_dir(temp_dir.path().join("docs"))?;
        
        Ok(temp_dir)
    }
    
    #[test]
    fn test_integration_manager_detects_files() {
        use crate::helpers::integration_manager::IntegrationManager;
        
        let temp_dir = setup_test_project().unwrap();
        
        // Check detection of CLAUDE.md
        assert!(IntegrationManager::has_claude_md(temp_dir.path()));
        
        // CLAUDE.i.md should not exist yet
        assert!(!IntegrationManager::has_claude_i_md(temp_dir.path()));
    }
    
    #[test]
    fn test_add_override_directive() {
        use crate::helpers::integration_manager::IntegrationManager;
        
        let temp_dir = setup_test_project().unwrap();
        
        // Add override directive
        IntegrationManager::add_override_directive(temp_dir.path()).unwrap();
        
        // Read the updated file
        let claude_md_path = temp_dir.path().join("CLAUDE.md");
        let content = fs::read_to_string(claude_md_path).unwrap();
        
        // Check if the directive was added
        assert!(content.contains("_CI.load('CLAUDE.i.md')_"));
    }
    
    #[test]
    fn test_create_override_file() {
        use crate::helpers::integration_manager::IntegrationManager;
        
        let temp_dir = setup_test_project().unwrap();
        let ci_repo_path = Path::new("/tmp/ci_repo"); // Mock path
        
        // Create override file
        IntegrationManager::create_override_file(temp_dir.path(), ci_repo_path).unwrap();
        
        // Check if file was created
        let claude_i_md_path = temp_dir.path().join("CLAUDE.i.md");
        assert!(claude_i_md_path.exists());
        
        // Check content
        let content = fs::read_to_string(claude_i_md_path).unwrap();
        assert!(content.contains("# Override directives for CI integration"));
        assert!(content.contains(&format!("Load {}/CLAUDE.md", ci_repo_path.display())));
    }
    
    #[test]
    fn test_remove_override_directive() {
        use crate::helpers::integration_manager::IntegrationManager;
        
        let temp_dir = setup_test_project().unwrap();
        
        // First add the directive
        IntegrationManager::add_override_directive(temp_dir.path()).unwrap();
        
        // Then remove it
        IntegrationManager::remove_override_directive(temp_dir.path()).unwrap();
        
        // Read the updated file
        let claude_md_path = temp_dir.path().join("CLAUDE.md");
        let content = fs::read_to_string(claude_md_path).unwrap();
        
        // Check if the directive was removed
        assert!(!content.contains("_CI.load('CLAUDE.i.md')_"));
        assert!(!content.contains("# Load CI Configuration"));
    }
    
    #[test]
    fn test_full_integration_cycle() {
        use crate::helpers::integration_manager::IntegrationManager;
        
        let temp_dir = setup_test_project().unwrap();
        let ci_repo_path = Path::new("/tmp/ci_repo"); // Mock path
        
        // Check initial state
        assert!(IntegrationManager::has_claude_md(temp_dir.path()));
        assert!(!IntegrationManager::has_claude_i_md(temp_dir.path()));
        
        // Perform integration
        let agents = vec!["Athena".to_string(), "ProjectArchitect".to_string()];
        IntegrationManager::integrate_with_override(temp_dir.path(), ci_repo_path, &agents, true).unwrap();
        
        // Check post-integration state
        assert!(IntegrationManager::has_claude_md(temp_dir.path()));
        assert!(IntegrationManager::has_claude_i_md(temp_dir.path()));
        
        // Read the updated CLAUDE.md
        let claude_md_path = temp_dir.path().join("CLAUDE.md");
        let content = fs::read_to_string(claude_md_path).unwrap();
        
        // Should contain the directive
        assert!(content.contains("_CI.load('CLAUDE.i.md')_"));
        
        // Check .collaborative-intelligence.json existence
        let config_path = temp_dir.path().join(".collaborative-intelligence.json");
        assert!(config_path.exists());
        
        // Now detach integration
        IntegrationManager::detach_integration(temp_dir.path()).unwrap();
        
        // Check post-detach state
        assert!(IntegrationManager::has_claude_md(temp_dir.path()));
        assert!(!IntegrationManager::has_claude_i_md(temp_dir.path()));
        
        // Should have a backup of CLAUDE.i.md
        let claude_i_md_bak_path = temp_dir.path().join("CLAUDE.i.md.bak");
        assert!(claude_i_md_bak_path.exists());
        
        // CLAUDE.md should no longer contain the directive
        let content = fs::read_to_string(claude_md_path).unwrap();
        assert!(!content.contains("_CI.load('CLAUDE.i.md')_"));
        
        // Config file should still exist
        assert!(config_path.exists());
    }
}