# CI Helper Functions

This directory contains documentation for the helper functions provided by CI. These helper functions are designed to make common tasks easier and more consistent across the codebase.

## Helper Modules

CI provides several helper modules that encapsulate related functionality:

- [Command Helpers](command.md) - Functions for command output formatting and execution
- [Repository Helpers](repository.md) - Git and source control operations
- [Config Helpers](config.md) - Configuration management
- [Project Helpers](project.md) - Project information and management
- [Path Helpers](path.md) - File system path operations
- [System Helpers](system.md) - System operations and information

## Using Helpers in Commands

Helper functions are designed to be used in command implementations to ensure consistent behavior and reduce code duplication. Here's an example of how to use the helpers in a command:

```rust
pub async fn my_command(path: &Path, config: &Config) -> Result<()> {
    // Print command header with category styling
    CommandHelpers::print_command_header(
        "My Command Description", 
        "🧠", // Category emoji
        "Intelligence & Discovery", 
        "blue" // Category color
    );
    
    // Check if path is in a git repository
    if !RepositoryHelpers::is_inside_git_repo(path) {
        CommandHelpers::print_error("Not in a git repository");
        return Err(anyhow!("Not in a git repository"));
    }
    
    // Get repository status
    let status = RepositoryHelpers::get_repository_status(path)?;
    
    // Display repository status
    RepositoryHelpers::display_status(&status);
    
    // Create a directory if it doesn't exist
    let output_dir = path.join("output");
    PathHelpers::ensure_directory_exists(&output_dir)?;
    
    // Create a file with content
    PathHelpers::create_file_with_content(
        &output_dir.join("output.txt"),
        "Hello, world!"
    )?;
    
    // Print success message
    CommandHelpers::print_success("Command completed successfully");
    
    Ok(())
}
```

## UI Consistency

The Command Helpers module provides functions for ensuring consistent UI output across commands:

- `print_command_header` - Prints a stylized header for a command
- `print_success` - Prints a success message with a checkmark
- `print_error` - Prints an error message with a cross
- `print_warning` - Prints a warning message with a warning symbol
- `print_info` - Prints an informational message
- `print_step` - Prints a step in a multi-step process
- `print_divider` - Prints a divider line to separate sections

## Error Handling

All helper functions use the `anyhow` crate for error handling, which allows for contextual error messages. When using helper functions, you can use the `?` operator to propagate errors:

```rust
// Example of error handling with helpers
let project_info = ProjectHelpers::get_project_info(path)
    .with_context(|| format!("Failed to get project info for {}", path.display()))?;
```

## Best Practices

1. **Use helpers whenever possible** - They ensure consistent behavior and reduce code duplication
2. **Add appropriate context to errors** - Use `with_context` to provide useful error messages
3. **Follow UI conventions** - Use the command helpers to maintain consistent UI
4. **Add new helper functions when needed** - If you find yourself duplicating code across commands, consider adding a new helper function