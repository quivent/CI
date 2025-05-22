use crate::test_helpers::TestEnv;
use crate::helper_utils::{CommandUtils, RepositoryUtils, ConfigUtils};
use std::path::Path;
use std::fs;
use anyhow::Result;
use std::env;
use std::process::{Command, Output};
use tempfile::tempdir;

use CI::helpers::api_keys::{ApiKeyManager, ApiKeyCommands};

#[test]
fn test_api_key_manager() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path().join("test_keys.toml");
    
    // Set the environment variable to use our test path
    env::set_var("CI_KEYS_PATH", temp_path.to_string_lossy().to_string());
    
    // Test setting a key
    ApiKeyManager::set_key("test_service", "api_key", "test_value")?;
    
    // Verify the key exists and can be retrieved
    assert!(ApiKeyManager::has_key("test_service", "api_key"));
    let key = ApiKeyManager::get_key("test_service", "api_key")?;
    assert_eq!(key, "test_value");
    
    // Test setting an environment-specific key
    ApiKeyManager::set_key_for_environment("test_service", "api_key", "env_test_value", "development")?;
    
    // Verify the environment-specific key
    let env_key = ApiKeyManager::get_key_for_environment("test_service", "api_key", "development")?;
    assert_eq!(env_key, "env_test_value");
    
    // Test listing keys
    let keys = ApiKeyManager::list_keys()?;
    assert!(keys.contains_key("test_service"));
    
    // Test key masking
    let masked_key = ApiKeyManager::mask_key("sk-1234567890abcdef");
    assert_eq!(masked_key, "sk-1****cdef");
    
    // Verify regular key still has precedence for regular get_key
    let key = ApiKeyManager::get_key("test_service", "api_key")?;
    assert_eq!(key, "test_value");
    
    // Test removing a key
    let removed = ApiKeyManager::remove_key("test_service", "api_key")?;
    assert!(removed);
    
    // Verify the key is gone
    assert!(!ApiKeyManager::has_key("test_service", "api_key"));
    
    // The environment-specific key should still exist
    assert!(ApiKeyManager::get_key_for_environment("test_service", "api_key", "development").is_ok());
    
    // Test removing an environment-specific key
    let removed = ApiKeyManager::remove_key_for_environment("test_service", "api_key", "development")?;
    assert!(removed);
    
    // Verify the environment-specific key is gone
    assert!(ApiKeyManager::get_key_for_environment("test_service", "api_key", "development").is_err());
    
    // Clean up
    env::remove_var("CI_KEYS_PATH");
    
    Ok(())
}

#[test]
fn test_project_specific_keys() -> Result<()> {
    let test_env = TestEnv::new();
    let project_dir = test_env.create_dir("test_project");
    
    // Set up a temporary key store path
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path().join("test_keys.toml");
    env::set_var("CI_KEYS_PATH", temp_path.to_string_lossy().to_string());
    
    // Set a project-specific key
    ApiKeyManager::set_project_key("project_service", "project_key", "project_value", &project_dir)?;
    
    // Set a user-level key
    ApiKeyManager::set_key("user_service", "user_key", "user_value")?;
    
    // Set the current directory to the project dir to test project-specific key retrieval
    let original_dir = env::current_dir()?;
    env::set_current_dir(&project_dir)?;
    
    // Verify the .ci directory and keys.toml file were created
    let cir_dir = project_dir.join(".ci");
    let keys_file = cir_dir.join("keys.toml");
    assert!(cir_dir.exists());
    assert!(keys_file.exists());
    
    // Verify the content of the keys file
    let content = fs::read_to_string(&keys_file)?;
    assert!(content.contains("project_service"));
    assert!(content.contains("project_key"));
    
    // Clean up
    env::set_current_dir(original_dir)?;
    env::remove_var("CI_KEYS_PATH");
    
    Ok(())
}

#[test]
fn test_api_key_commands() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path().join("test_keys.toml");
    
    // Set the environment variable to use our test path
    env::set_var("CI_KEYS_PATH", temp_path.to_string_lossy().to_string());
    
    // Test set_key_cli
    ApiKeyCommands::set_key_cli("cli_service", "cli_key", "cli_value", None, false)?;
    
    // Verify the key exists
    assert!(ApiKeyManager::has_key("cli_service", "cli_key"));
    let key = ApiKeyManager::get_key("cli_service", "cli_key")?;
    assert_eq!(key, "cli_value");
    
    // Test list_keys_cli
    ApiKeyCommands::list_keys_cli()?;
    
    // Test remove_key_cli
    ApiKeyCommands::remove_key_cli("cli_service", "cli_key", None)?;
    
    // Verify the key is gone
    assert!(!ApiKeyManager::has_key("cli_service", "cli_key"));
    
    // Test export_keys_cli
    // Add a key for testing export
    ApiKeyManager::set_key("export_service", "export_key", "export_value")?;
    ApiKeyCommands::export_keys_cli()?;
    
    // Clean up
    env::remove_var("CI_KEYS_PATH");
    
    Ok(())
}

#[test]
fn test_environment_variables_precedence() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path().join("test_keys.toml");
    
    // Set the environment variable to use our test path
    env::set_var("CI_KEYS_PATH", temp_path.to_string_lossy().to_string());
    
    // Set a key in the store
    ApiKeyManager::set_key("env_test_service", "env_test_key", "store_value")?;
    
    // Set an environment variable with higher precedence
    env::set_var("ENV_TEST_SERVICE_ENV_TEST_KEY", "env_value");
    
    // Get the key - should prefer environment variable
    let key = ApiKeyManager::get_key("env_test_service", "env_test_key")?;
    assert_eq!(key, "env_value");
    
    // Clean up
    env::remove_var("CI_KEYS_PATH");
    env::remove_var("ENV_TEST_SERVICE_ENV_TEST_KEY");
    
    Ok(())
}

#[test]
fn test_api_key_manager_edge_cases() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path().join("test_keys.toml");
    
    // Set the environment variable to use our test path
    env::set_var("CI_KEYS_PATH", temp_path.to_string_lossy().to_string());
    
    // Test with empty key
    ApiKeyManager::set_key("empty_service", "empty_key", "")?;
    let key = ApiKeyManager::get_key("empty_service", "empty_key")?;
    assert_eq!(key, "");
    
    // Test with special characters
    let special_key = "!@#$%^&*()";
    ApiKeyManager::set_key("special_service", "special_key", special_key)?;
    let key = ApiKeyManager::get_key("special_service", "special_key")?;
    assert_eq!(key, special_key);
    
    // Test masking with short key
    let masked = ApiKeyManager::mask_key("short");
    assert_eq!(masked, "****");
    
    // Test masking with exact 8-char key
    let masked = ApiKeyManager::mask_key("12345678");
    assert_eq!(masked, "1234****5678");
    
    // Test non-existent key
    let result = ApiKeyManager::get_key("nonexistent", "key");
    assert!(result.is_err());
    
    // Test removing non-existent key
    let removed = ApiKeyManager::remove_key("nonexistent", "key")?;
    assert!(!removed);
    
    // Clean up
    env::remove_var("CI_KEYS_PATH");
    
    Ok(())
}