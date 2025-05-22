# CI - Style Guide

This document outlines the coding style and conventions used in the CI project.

## Code Organization

### Module Structure

The codebase is organized into the following modules:

- `main.rs` - Entry point and command definitions
- `commands/` - Command implementation modules
  - `intelligence.rs` - Intelligence & Discovery commands
  - `lifecycle.rs` - Project Lifecycle commands
  - `source_control.rs` - Source Control commands
  - `system.rs` - System Management commands
- `helpers/` - Helper functions and utilities
- `config.rs` - Configuration loading and management
- `error.rs` - Error type definitions

### Command Implementation

Each command should follow this general pattern:

```rust
pub async fn command_name(arg1: Type1, arg2: Type2, config: &Config) -> Result<()> {
    // Print command header with category
    CommandHelpers::print_command_header(
        "Command description", 
        "🧠", // Category emoji
        "Category Name", 
        "blue" // Category color
    );
    
    // Command implementation
    
    // Print success message
    CommandHelpers::print_success("Operation completed successfully");
    
    Ok(())
}
```

## Coding Style

### Formatting

- Use 4 spaces for indentation
- Maximum line length should be 100 characters
- Use blank lines to group related code
- Follow Rust's naming conventions:
  - `snake_case` for variables, functions, and modules
  - `CamelCase` for types and enum variants
  - `SCREAMING_SNAKE_CASE` for constants and static variables

### Error Handling

- Use `anyhow::Result` for public functions
- Use `anyhow::Context` to add context to errors
- For specific error types, define them in the `error.rs` module
- Provide helpful error messages that suggest possible solutions

Example:
```rust
let content = std::fs::read_to_string(&file_path)
    .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
```

### Path Handling

- Always use absolute paths when interacting with the file system
- Use `PathBuf` for paths that are modified
- Use `&Path` for paths that are only read
- Check that paths exist before accessing them

### Output Formatting

- Use the `CommandHelpers` module for consistent output formatting
- Use colored output consistently:
  - Blue for Intelligence & Discovery commands
  - Green for Source Control commands
  - Yellow for Project Lifecycle commands
  - Cyan for System Management commands
- Use emoji icons for visual categorization
- Format important information in bold
- Use dividers for separating sections of output

## Testing

### Unit Tests

- Place unit tests in the same file as the code they test
- Use the `#[cfg(test)]` module attribute
- Name test functions with a `test_` prefix
- Test both success and error cases

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name_success() {
        // Test successful execution
    }
    
    #[test]
    fn test_function_name_error() {
        // Test error handling
    }
}
```

### Integration Tests

- Place integration tests in the `tests/` directory
- Test complete command workflows
- Mock external dependencies when necessary
- Use a test environment with temporary directories

## Documentation

### Function Documentation

- Document all public functions with doc comments
- Include:
  - A brief description of what the function does
  - Parameter descriptions
  - Return value description
  - Example usage if appropriate
  - Any errors that might be returned

Example:
```rust
/// Extract agent's memory content from AGENTS.md
///
/// # Arguments
///
/// * `content` - The content of the AGENTS.md file
/// * `agent_name` - The name of the agent to extract memory for
///
/// # Returns
///
/// The extracted agent memory as a formatted string
fn extract_agent_memory(content: &str, agent_name: &str) -> String {
    // ...
}
```

### File Headers

- Include a brief description of the file's purpose at the top of each file
- List the main functionality provided by the file

Example:
```rust
//! Intelligence & Discovery commands for the CI tool.
//!
//! This module contains commands for interacting with agents and projects,
//! including listing agents, loading agent memory, and managing projects.
```

## Commit Style

When committing changes, use the following format:

```
[Category] Brief description of the change

More detailed explanation of what the change does, why it was needed,
and any important implementation details.

[Category] can be one of:
- Feature: New functionality
- Fix: Bug fixes
- Refactor: Code restructuring without behavior change
- Docs: Documentation updates
- Style: Code style changes
- Test: Test additions or modifications
- Chore: Maintenance tasks
```

## Pull Requests

Pull requests should:

1. Reference an issue or enhancement item
2. Have a clear title describing the change
3. Include a detailed description of the changes
4. Be focused on a single logical change
5. Include tests for new functionality
6. Pass all CI checks
7. Be reviewed by at least one maintainer