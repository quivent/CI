use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::env;
use std::collections::HashMap;

use anyhow::{Result, Context, anyhow};
use serde::{Serialize, Deserialize};
use toml;
use colored::*;
use dirs;

/// Struct to manage API keys for services
pub struct ApiKeyManager;

/// Structure for storing API keys
#[derive(Debug, Serialize, Deserialize, Default)]
struct KeyStore {
    /// Service-specific API keys (service -> key type -> value)
    #[serde(default)]
    services: HashMap<String, HashMap<String, String>>,
    
    /// Environment-specific API keys (environment -> service -> key type -> value)
    #[serde(default)]
    environments: HashMap<String, HashMap<String, HashMap<String, String>>>,
    
    /// Metadata about the keys
    #[serde(default)]
    metadata: KeyMetadata,
}

/// Metadata about the key store
#[derive(Debug, Serialize, Deserialize, Default)]
struct KeyMetadata {
    /// When the store was last updated
    #[serde(default)]
    #[serde(with = "chrono_serde_option")]
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Description of the key store
    #[serde(default)]
    description: Option<String>,
}

// Helper module for serializing/deserializing chrono::DateTime<Utc>
mod chrono_serde_option {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        date: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => serializer.serialize_str(&date.to_rfc3339()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                let dt = DateTime::parse_from_rfc3339(&s)
                    .map_err(serde::de::Error::custom)?;
                Ok(Some(Utc.from_utc_datetime(&dt.naive_utc())))
            }
            None => Ok(None),
        }
    }
}

impl ApiKeyManager {
    /// Get the path to the key store file
    fn get_key_store_path() -> Result<PathBuf> {
        // Check for path override from environment variable
        if let Ok(path) = env::var("CI_KEYS_PATH") {
            return Ok(PathBuf::from(path));
        }
        
        // Use user-specific configuration directory
        if let Some(user_config_dir) = dirs::config_dir() {
            let keys_dir = user_config_dir.join("ci");
            
            // Create directory if it doesn't exist
            if !keys_dir.exists() {
                fs::create_dir_all(&keys_dir)
                    .with_context(|| format!("Failed to create key store directory: {}", keys_dir.display()))?;
                
                // Set secure permissions on Unix-like systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = fs::metadata(&keys_dir)?;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o700); // User read/write/execute only
                    fs::set_permissions(&keys_dir, perms)?;
                }
            }
            
            return Ok(keys_dir.join("keys.toml"));
        }
        
        // Fallback to current directory if we can't find a user config dir
        Ok(PathBuf::from(".ci").join("keys.toml"))
    }
    
    /// Get the project-specific key store path
    fn get_project_key_store_path() -> Option<PathBuf> {
        // Try to find .ci directory in current or parent directories
        let mut current_dir = env::current_dir().ok()?;
        
        loop {
            let key_path = current_dir.join(".ci").join("keys.toml");
            
            if key_path.exists() {
                return Some(key_path);
            }
            
            // Check parent directory
            if !current_dir.pop() {
                break;
            }
        }
        
        None
    }
    
    /// Load the key store
    fn load_key_store() -> Result<KeyStore> {
        let key_path = Self::get_key_store_path()?;
        
        if !key_path.exists() {
            return Ok(KeyStore::default());
        }
        
        // Read and parse the file
        let mut file = File::open(&key_path)
            .with_context(|| format!("Failed to open key store: {}", key_path.display()))?;
            
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read key store: {}", key_path.display()))?;
            
        // Parse TOML
        let store: KeyStore = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse key store: {}", key_path.display()))?;
            
        Ok(store)
    }
    
    /// Save the key store
    fn save_key_store(store: &KeyStore) -> Result<()> {
        let key_path = Self::get_key_store_path()?;
        
        // Create parent directory if needed
        if let Some(parent) = key_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create key store directory: {}", parent.display()))?;
                
                // Set secure permissions on Unix-like systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = fs::metadata(parent)?;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o700); // User read/write/execute only
                    fs::set_permissions(parent, perms)?;
                }
            }
        }
        
        // Convert to TOML and write
        let toml_string = toml::to_string_pretty(store)
            .with_context(|| "Failed to serialize key store to TOML")?;
            
        let mut file = File::create(&key_path)
            .with_context(|| format!("Failed to create key store file: {}", key_path.display()))?;
            
        // Set secure permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = file.metadata()?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o600); // User read/write only
            file.set_permissions(perms)?;
        }
        
        file.write_all(toml_string.as_bytes())
            .with_context(|| format!("Failed to write key store: {}", key_path.display()))?;
            
        Ok(())
    }
    
    /// Check if an API key exists
    pub fn has_key(service: &str, key_name: &str) -> bool {
        if let Ok(store) = Self::load_key_store() {
            if let Some(service_keys) = store.services.get(service) {
                return service_keys.contains_key(key_name);
            }
        }
        
        // Check environment variables
        let env_var_name = format!("{}_{}",
            service.to_uppercase(),
            key_name.to_uppercase()
        );
        
        env::var(&env_var_name).is_ok()
    }
    
    /// Get an API key
    pub fn get_key(service: &str, key_name: &str) -> Result<String> {
        // Check environment variables first (highest precedence)
        let env_var_name = format!("{}_{}",
            service.to_uppercase(),
            key_name.to_uppercase()
        );
        
        if let Ok(key) = env::var(&env_var_name) {
            return Ok(key);
        }
        
        // Check user config
        let store = Self::load_key_store()?;
        
        if let Some(service_keys) = store.services.get(service) {
            if let Some(key) = service_keys.get(key_name) {
                return Ok(key.clone());
            }
        }
        
        // Check project-specific config
        if let Some(project_key_path) = Self::get_project_key_store_path() {
            if let Ok(contents) = fs::read_to_string(&project_key_path) {
                if let Ok(project_store) = toml::from_str::<KeyStore>(&contents) {
                    if let Some(service_keys) = project_store.services.get(service) {
                        if let Some(key) = service_keys.get(key_name) {
                            return Ok(key.clone());
                        }
                    }
                }
            }
        }
        
        Err(anyhow!("API key not found: {}.{}", service, key_name))
    }
    
    /// Get an API key for a specific environment
    pub fn get_key_for_environment(service: &str, key_name: &str, environment: &str) -> Result<String> {
        // Check environment variables first (highest precedence)
        let env_var_name = format!("{}_{}_{}",
            environment.to_uppercase(),
            service.to_uppercase(),
            key_name.to_uppercase()
        );
        
        if let Ok(key) = env::var(&env_var_name) {
            return Ok(key);
        }
        
        // Check user config
        let store = Self::load_key_store()?;
        
        if let Some(environments) = store.environments.get(environment) {
            if let Some(service_keys) = environments.get(service) {
                if let Some(key) = service_keys.get(key_name) {
                    return Ok(key.clone());
                }
            }
        }
        
        // Fall back to the default key if no environment-specific key exists
        Self::get_key(service, key_name)
    }
    
    /// Set an API key
    pub fn set_key(service: &str, key_name: &str, key_value: &str) -> Result<()> {
        // Load existing store
        let mut store = Self::load_key_store()?;
        
        // Update metadata
        store.metadata.last_updated = Some(chrono::Utc::now());
        
        // Add or update the key
        store.services
            .entry(service.to_string())
            .or_insert_with(HashMap::new)
            .insert(key_name.to_string(), key_value.to_string());
            
        // Save the updated store
        Self::save_key_store(&store)
    }
    
    /// Set an API key for a specific environment
    pub fn set_key_for_environment(
        service: &str,
        key_name: &str,
        key_value: &str,
        environment: &str
    ) -> Result<()> {
        // Load existing store
        let mut store = Self::load_key_store()?;
        
        // Update metadata
        store.metadata.last_updated = Some(chrono::Utc::now());
        
        // Add or update the key
        store.environments
            .entry(environment.to_string())
            .or_insert_with(HashMap::new)
            .entry(service.to_string())
            .or_insert_with(HashMap::new)
            .insert(key_name.to_string(), key_value.to_string());
            
        // Save the updated store
        Self::save_key_store(&store)
    }
    
    /// Set an API key in the project-specific store
    pub fn set_project_key(
        service: &str,
        key_name: &str,
        key_value: &str,
        project_dir: &Path
    ) -> Result<()> {
        let cir_dir = project_dir.join(".ci");
        
        // Create .ci directory if it doesn't exist
        if !cir_dir.exists() {
            fs::create_dir_all(&cir_dir)
                .with_context(|| format!("Failed to create .ci directory: {}", cir_dir.display()))?;
        }
        
        let key_path = cir_dir.join("keys.toml");
        
        // Load existing store or create new
        let mut store = if key_path.exists() {
            let contents = fs::read_to_string(&key_path)
                .with_context(|| format!("Failed to read project key store: {}", key_path.display()))?;
                
            toml::from_str(&contents)
                .with_context(|| format!("Failed to parse project key store: {}", key_path.display()))?
        } else {
            KeyStore::default()
        };
        
        // Update metadata
        store.metadata.last_updated = Some(chrono::Utc::now());
        
        // Add or update the key
        store.services
            .entry(service.to_string())
            .or_insert_with(HashMap::new)
            .insert(key_name.to_string(), key_value.to_string());
            
        // Convert to TOML and write
        let toml_string = toml::to_string_pretty(&store)
            .with_context(|| "Failed to serialize project key store to TOML")?;
            
        fs::write(&key_path, toml_string)
            .with_context(|| format!("Failed to write project key store: {}", key_path.display()))?;
        
        // Set secure permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&key_path)?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o600); // User read/write only
            fs::set_permissions(&key_path, perms)?;
        }
        
        Ok(())
    }
    
    /// Remove an API key
    pub fn remove_key(service: &str, key_name: &str) -> Result<bool> {
        // Load existing store
        let mut store = Self::load_key_store()?;
        
        // Try to remove the key
        let mut removed = false;
        
        if let Some(service_keys) = store.services.get_mut(service) {
            removed = service_keys.remove(key_name).is_some();
            
            // Remove service entry if empty
            if service_keys.is_empty() {
                store.services.remove(service);
            }
        }
        
        // Update metadata if something was removed
        if removed {
            store.metadata.last_updated = Some(chrono::Utc::now());
            
            // Save the updated store
            Self::save_key_store(&store)?;
        }
        
        Ok(removed)
    }
    
    /// Remove an API key for a specific environment
    pub fn remove_key_for_environment(
        service: &str,
        key_name: &str,
        environment: &str
    ) -> Result<bool> {
        // Load existing store
        let mut store = Self::load_key_store()?;
        
        // Try to remove the key
        let mut removed = false;
        
        if let Some(environments) = store.environments.get_mut(environment) {
            if let Some(service_keys) = environments.get_mut(service) {
                removed = service_keys.remove(key_name).is_some();
                
                // Remove service entry if empty
                if service_keys.is_empty() {
                    environments.remove(service);
                }
                
                // Remove environment entry if empty
                if environments.is_empty() {
                    store.environments.remove(environment);
                }
            }
        }
        
        // Update metadata if something was removed
        if removed {
            store.metadata.last_updated = Some(chrono::Utc::now());
            
            // Save the updated store
            Self::save_key_store(&store)?;
        }
        
        Ok(removed)
    }
    
    /// List all available keys (masked)
    pub fn list_keys() -> Result<HashMap<String, Vec<String>>> {
        let store = Self::load_key_store()?;
        let mut result = HashMap::new();
        
        // Process service keys
        for (service, keys) in &store.services {
            let key_names: Vec<String> = keys.keys().cloned().collect();
            if !key_names.is_empty() {
                result.insert(service.clone(), key_names);
            }
        }
        
        // Process environment keys
        for (env_name, services) in &store.environments {
            for (service, keys) in services {
                let key_names: Vec<String> = keys.keys()
                    .map(|key| format!("{}:{}", env_name, key))
                    .collect();
                
                if !key_names.is_empty() {
                    result
                        .entry(service.clone())
                        .or_insert_with(Vec::new)
                        .extend(key_names);
                }
            }
        }
        
        Ok(result)
    }
    
    /// Get a masked version of an API key (for display)
    pub fn mask_key(key: &str) -> String {
        if key.len() <= 8 {
            return "****".to_string();
        }
        
        let visible_prefix = &key[0..4];
        let visible_suffix = &key[key.len() - 4..];
        
        format!("{}****{}", visible_prefix, visible_suffix)
    }
    
    /// Reset the key store (for testing)
    #[cfg(test)]
    pub fn reset_key_store() -> Result<()> {
        let key_path = Self::get_key_store_path()?;
        
        if key_path.exists() {
            fs::remove_file(&key_path)
                .with_context(|| format!("Failed to remove key store: {}", key_path.display()))?;
        }
        
        Ok(())
    }
}

/// API Key commands for CLI
pub struct ApiKeyCommands;

impl ApiKeyCommands {
    /// Set an API key from CLI
    pub fn set_key_cli(
        service: &str,
        key_name: &str,
        key_value: &str,
        environment: Option<&str>,
        project: bool
    ) -> Result<()> {
        if let Some(env_name) = environment {
            ApiKeyManager::set_key_for_environment(service, key_name, key_value, env_name)?;
            println!("{} API key {}.{} for environment {} set successfully", 
                "✓".green(), 
                service.bold(), 
                key_name.bold(),
                env_name.bold()
            );
        } else if project {
            let current_dir = env::current_dir()
                .with_context(|| "Failed to get current directory")?;
                
            ApiKeyManager::set_project_key(service, key_name, key_value, &current_dir)?;
            println!("{} Project-specific API key {}.{} set successfully", 
                "✓".green(), 
                service.bold(), 
                key_name.bold()
            );
        } else {
            ApiKeyManager::set_key(service, key_name, key_value)?;
            println!("{} API key {}.{} set successfully", 
                "✓".green(), 
                service.bold(), 
                key_name.bold()
            );
        }
        
        Ok(())
    }
    
    /// Remove an API key from CLI
    pub fn remove_key_cli(
        service: &str,
        key_name: &str,
        environment: Option<&str>
    ) -> Result<()> {
        let removed = if let Some(env_name) = environment {
            ApiKeyManager::remove_key_for_environment(service, key_name, env_name)?
        } else {
            ApiKeyManager::remove_key(service, key_name)?
        };
        
        if removed {
            if let Some(env_name) = environment {
                println!("{} API key {}.{} for environment {} removed successfully", 
                    "✓".green(), 
                    service.bold(), 
                    key_name.bold(),
                    env_name.bold()
                );
            } else {
                println!("{} API key {}.{} removed successfully", 
                    "✓".green(), 
                    service.bold(), 
                    key_name.bold()
                );
            }
        } else {
            if let Some(env_name) = environment {
                println!("{} API key {}.{} for environment {} not found", 
                    "!".yellow(), 
                    service.bold(), 
                    key_name.bold(),
                    env_name.bold()
                );
            } else {
                println!("{} API key {}.{} not found", 
                    "!".yellow(), 
                    service.bold(), 
                    key_name.bold()
                );
            }
        }
        
        Ok(())
    }
    
    /// List all API keys from CLI
    pub fn list_keys_cli() -> Result<()> {
        let keys = ApiKeyManager::list_keys()?;
        
        if keys.is_empty() {
            println!("{} No API keys configured", "!".yellow());
            println!("\nTo set a key, use: ci key set <service> <key_name> <key_value>");
            return Ok(());
        }
        
        println!("{} Configured API keys:", "ℹ".blue());
        
        for (service, key_names) in keys {
            println!("\n{}", service.to_uppercase().bold());
            
            for key_name in key_names {
                // Process key name by environment or standard
                if key_name.contains(':') {
                    // This is an environment-specific key
                    let parts: Vec<&str> = key_name.split(':').collect();
                    let env_name = parts[0];
                    let actual_key_name = parts[1];
                    
                    if let Ok(key) = ApiKeyManager::get_key_for_environment(&service, actual_key_name, env_name) {
                        println!("  {} [{} environment]: {}", 
                            actual_key_name.bold(),
                            env_name.cyan(),
                            ApiKeyManager::mask_key(&key).dimmed()
                        );
                    } else {
                        println!("  {} [{}]: {}", 
                            actual_key_name.bold(),
                            env_name.cyan(),
                            "(error)".red()
                        );
                    }
                } else {
                    // Standard key
                    if let Ok(key) = ApiKeyManager::get_key(&service, &key_name) {
                        println!("  {}: {}", 
                            key_name.bold(),
                            ApiKeyManager::mask_key(&key).dimmed()
                        );
                    } else {
                        println!("  {}: {}", 
                            key_name.bold(),
                            "(error)".red()
                        );
                    }
                }
            }
        }
        
        println!("\n{} Key values are masked for security", "ℹ".blue());
        
        Ok(())
    }
    
    /// Export API keys for shell sourcing
    pub fn export_keys_cli() -> Result<()> {
        let keys = ApiKeyManager::list_keys()?;
        
        for (service, key_names) in keys {
            for key_name in key_names {
                if key_name.contains(':') {
                    // Skip environment-specific keys in export
                    continue;
                }
                
                if let Ok(key) = ApiKeyManager::get_key(&service, &key_name) {
                    if service.to_lowercase() == "anthropic" && key_name.to_lowercase() == "api_key" {
                        println!("export ANTHROPIC_API_KEY=\"{}\"", key);
                    } else {
                        println!("export {}_{}=\"{}\"", 
                            service.to_uppercase(),
                            key_name.to_uppercase(),
                            key
                        );
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_api_key_management() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path().join("test_keys.toml");
        
        // Set the environment variable to use our test path
        env::set_var("CI_KEYS_PATH", temp_path.to_string_lossy().to_string());
        
        // Set a test key
        ApiKeyManager::set_key("test_service", "api_key", "test_value")?;
        
        // Check that the key exists
        assert!(ApiKeyManager::has_key("test_service", "api_key"));
        
        // Get the key
        let key = ApiKeyManager::get_key("test_service", "api_key")?;
        assert_eq!(key, "test_value");
        
        // Set an environment-specific key
        ApiKeyManager::set_key_for_environment("test_service", "api_key", "env_test_value", "dev")?;
        
        // Get the environment-specific key
        let env_key = ApiKeyManager::get_key_for_environment("test_service", "api_key", "dev")?;
        assert_eq!(env_key, "env_test_value");
        
        // List keys
        let keys = ApiKeyManager::list_keys()?;
        assert!(keys.contains_key("test_service"));
        
        // Remove the key
        let removed = ApiKeyManager::remove_key("test_service", "api_key")?;
        assert!(removed);
        
        // Check that the key is gone
        assert!(!ApiKeyManager::has_key("test_service", "api_key"));
        
        // The environment-specific key should still exist
        assert!(ApiKeyManager::get_key_for_environment("test_service", "api_key", "dev").is_ok());
        
        // Remove the environment-specific key
        let removed = ApiKeyManager::remove_key_for_environment("test_service", "api_key", "dev")?;
        assert!(removed);
        
        // Check that the environment-specific key is gone
        assert!(ApiKeyManager::get_key_for_environment("test_service", "api_key", "dev").is_err());
        
        // Clean up
        env::remove_var("CI_KEYS_PATH");
        
        Ok(())
    }
    
    #[test]
    fn test_key_masking() {
        assert_eq!(ApiKeyManager::mask_key("short"), "****");
        assert_eq!(ApiKeyManager::mask_key("sk-abcdefghijklmnopq"), "sk-a****mnopq");
        assert_eq!(ApiKeyManager::mask_key("1234567890"), "1234****7890");
    }
}