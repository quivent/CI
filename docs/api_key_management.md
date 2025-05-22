# API Key Management in CI

This document outlines the API key management system for CI, providing secure storage and access to API keys used for various service integrations.

## Overview

CI integrates with various external services, each requiring API keys or tokens for authentication. The API key management system provides:

1. **Secure Storage** - Keys are stored securely on the user's system
2. **Easy Access** - A simple, consistent interface for accessing keys
3. **Persistent Configuration** - Keys are persisted between CI sessions
4. **Multiple Environments** - Support for different environments (development, production)

## Key Storage Location

API keys are stored in the following locations (in order of precedence):

1. Environment variables (highest precedence)
2. User-specific configuration directory:
   - **macOS**: `~/Library/Application Support/CI/keys.toml`
   - **Linux**: `~/.config/ci/keys.toml`
   - **Windows**: `%APPDATA%\CI\keys.toml`
3. Project-specific configuration (`.ci/keys.toml` in project root)

## Configuration Format

API keys are stored in TOML format for readability and structure:

```toml
# API Keys configuration for CI

[openai]
api_key = "sk-xxxxxxxxxxxxxxxxxxx"
organization_id = "org-xxxxxxxxxxxxxxx"

[anthropic]
api_key = "sk-ant-xxxxxxxxxxxxxxx"

[github]
access_token = "ghp_xxxxxxxxxxxxxxxxxxxx"

[environments.development]
openai.api_key = "sk-dev-xxxxxxxxxxxxxxx"
```

## Usage in Code

The API key management is accessed through the `ApiKeyManager` class:

```rust
use ci::helpers::api_keys::ApiKeyManager;

// Get an API key
let openai_key = ApiKeyManager::get_key("openai", "api_key")?;

// Get a key for a specific environment
let openai_dev_key = ApiKeyManager::get_key_for_environment("openai", "api_key", "development")?;

// Check if a key exists
if ApiKeyManager::has_key("github", "access_token") {
    // Use the key
}

// Set a key (will be stored in user-specific configuration)
ApiKeyManager::set_key("anthropic", "api_key", "sk-ant-xxxxxxxxxxxxxxx")?;

// Set a key for a specific environment
ApiKeyManager::set_key_for_environment("openai", "api_key", "sk-dev-xxxxxxxxxxxxxxx", "development")?;
```

## Command Line Interface

CI provides commands for managing API keys:

```sh
# Set an API key
ci config set-key openai api_key sk-xxxxxxxxxxxxxxxxxxx

# Set an API key for a specific environment
ci config set-key openai api_key sk-dev-xxxxxxxxxxxxxxx --env development

# List all configured keys (values are masked)
ci config list-keys

# Remove a key
ci config remove-key openai api_key
```

## Security Considerations

The API key manager implements several security practices:

1. **File Permissions** - Key files are created with restricted permissions (600)
2. **No Logging** - API keys are never logged or printed in full
3. **Memory Management** - Keys are handled as sensitive data in memory
4. **Masking** - When displaying keys in output, they are masked (e.g., `sk-***********`)

## Implementation Details

The API key management system consists of:

1. **ApiKeyManager** - Core functionality for managing keys
2. **KeyStorage** - Handles reading and writing key files
3. **Environment** - Manages environment-specific configurations
4. **Commands** - CLI commands for interacting with keys

## Testing

Tests for the API key management system use a temporary environment to avoid affecting real configurations:

```rust
#[test]
fn test_api_key_manager() {
    // Setup a test environment
    let test_env = TestEnv::new();
    let test_keys_path = test_env.path(".ci/keys.toml");
    
    // Override the default keys path for testing
    std::env::set_var("CI_KEYS_PATH", test_keys_path.to_str().unwrap());
    
    // Test setting and getting keys
    ApiKeyManager::set_key("test", "api_key", "test-value").unwrap();
    
    let key = ApiKeyManager::get_key("test", "api_key").unwrap();
    assert_eq!(key, "test-value");
    
    // Test environment-specific keys
    ApiKeyManager::set_key_for_environment("test", "api_key", "env-value", "testing").unwrap();
    
    let env_key = ApiKeyManager::get_key_for_environment("test", "api_key", "testing").unwrap();
    assert_eq!(env_key, "env-value");
}
```

## Future Enhancements

Planned enhancements for the API key management system:

1. **Encryption** - Optional encryption of stored keys
2. **Service Integration** - Pre-configured setups for popular services
3. **Validation** - Key format validation for common APIs
4. **Cloud Sync** - Optional secure syncing of keys between devices
5. **Rotation** - Key rotation management and scheduling