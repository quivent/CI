# CI Helper Functions Documentation

This guide documents the helper functions created to streamline common operations in the CI command-line tool.

## Overview

The helper functions are organized into three main modules:

1. **CommandHelpers** - General command execution and UI helpers
2. **RepositoryHelpers** - Git repository operations
3. **ConfigHelpers** - Configuration file management

## CommandHelpers

Located in `src/helpers/mod.rs`, this module provides general utility functions for command execution and UI formatting.

### Core Functions

#### UI Formatting

```rust
// Print a standard command header with category and emoji
CommandHelpers::print_command_header(
    title: &str,
    emoji: &str,
    category: &str,
    color: &str
)

// Print various status messages
CommandHelpers::print_success(message: &str)
CommandHelpers::print_error(message: &str)
CommandHelpers::print_warning(message: &str)
CommandHelpers::print_step(message: &str)
CommandHelpers::print_info(message: &str)
CommandHelpers::print_status(message: &str)

// Print divider lines
CommandHelpers::print_divider(color: &str)
```

#### Path Operations

```rust
// Resolve project path with error handling
CommandHelpers::resolve_project_path(path: &Option<String>) -> Result<PathBuf>

// Get CI repository path
CommandHelpers::get_ci_repository_path(ci_path: &Option<String>) -> Result<PathBuf>

// Check if a directory is a git repository
CommandHelpers::is_git_repository(path: &Path) -> bool
```

#### File Operations

```rust
// Create file with content, ensuring parent directory exists
CommandHelpers::create_file_with_content(path: &Path, content: &str) -> Result<()>

// Read file content with error handling
CommandHelpers::read_file_content(path: &Path) -> Result<String>

// Update markdown section
CommandHelpers::update_markdown_section(
    path: &Path,
    section_name: &str,
    content: &str
) -> Result<()>
```

#### Command Execution

```rust
// Run command and capture output
CommandHelpers::run_command_with_output(
    command: &str,
    args: &[&str],
    dir: Option<&Path>
) -> Result<(bool, String, String)>

// Check if a command exists in PATH
CommandHelpers::command_exists(command: &str) -> bool

// Execute function with progress indicator
CommandHelpers::with_progress<F, R>(message: &str, f: F) -> Result<R>
```

#### Other Utilities

```rust
// Get current timestamp
CommandHelpers::get_timestamp() -> String

// Prompt user for confirmation
CommandHelpers::prompt_confirmation(message: &str) -> bool

// Check if running in verbose mode
CommandHelpers::is_verbose() -> bool

// Check CI integration
CommandHelpers::check_ci_integration(path: &Path) -> Result<bool>

// Format file list for display
CommandHelpers::format_file_list(files: &[String]) -> String
```

## RepositoryHelpers

Located in `src/helpers/repository.rs`, this module provides Git repository operations.

### Core Functions

```rust
// Initialize a git repository
RepositoryHelpers::init_git_repository(path: &Path) -> Result<()>

// Create a default .gitignore
RepositoryHelpers::create_default_gitignore(path: &Path) -> Result<()>

// Check if path is inside a git repository
RepositoryHelpers::is_inside_git_repo(path: &Path) -> bool

// Get git repository root
RepositoryHelpers::get_git_root(path: &Path) -> Result<String>

// Get current branch name
RepositoryHelpers::get_current_branch(path: &Path) -> Result<String>

// Get repository status
RepositoryHelpers::get_repository_status(path: &Path) -> Result<RepositoryStatus>

// Display repository status
RepositoryHelpers::display_status(status: &RepositoryStatus)

// Update .gitignore with patterns
RepositoryHelpers::update_gitignore(path: &Path, patterns: &[&str]) -> Result<bool>
```

### RepositoryStatus Structure

```rust
pub struct RepositoryStatus {
    pub is_git_repo: bool,
    pub current_branch: Option<String>,
    pub has_uncommitted_changes: bool,
    pub commit_count: usize,
    pub has_remote: bool,
    pub remote_url: Option<String>,
}
```

## ConfigHelpers

Located in `src/helpers/config.rs`, this module manages CI configuration files.

### Core Functions

```rust
// Create or update .env file
ConfigHelpers::create_or_update_env_file(
    project_path: &Path,
    ci_repo_path: &Path,
    explicit_path: &Option<String>
) -> Result<()>

// Create CLAUDE.md configuration
ConfigHelpers::create_claude_config(
    project_path: &Path,
    project_name: &str,
    integration_type: &str,
    agents: &[String]
) -> Result<()>

// Create CLAUDE.local.md file
ConfigHelpers::create_claude_local_config(
    project_path: &Path,
    ci_repo_path: &Path
) -> Result<()>

// Check configuration version
ConfigHelpers::check_config_version(path: &Path) -> Result<ConfigStatus>
```

### ConfigStatus Structure

```rust
pub struct ConfigStatus {
    pub needs_update: bool,
    pub missing_sections: Vec<&'static str>,
}
```

## Usage Example

Here's how to use these helpers in a command implementation:

```rust
use crate::helpers::{CommandHelpers, ConfigHelpers, RepositoryHelpers};

pub fn my_command(project_path: &Option<String>) {
    // Print command header
    CommandHelpers::print_command_header(
        "My Command",
        "🚀",
        "Example Category",
        "green"
    );
    
    // Resolve project path
    let project_dir = match CommandHelpers::resolve_project_path(project_path) {
        Ok(path) => path,
        Err(e) => {
            CommandHelpers::print_error(&format!("Invalid path: {}", e));
            return;
        }
    };
    
    // Check if it's a git repository
    if !CommandHelpers::is_git_repository(&project_dir) {
        CommandHelpers::print_warning("Not a git repository");
        
        // Initialize git if user confirms
        if CommandHelpers::prompt_confirmation("Initialize git repository?") {
            if let Err(e) = RepositoryHelpers::init_git_repository(&project_dir) {
                CommandHelpers::print_error(&format!("Failed to init git: {}", e));
                return;
            }
            CommandHelpers::print_success("Git repository initialized");
        }
    }
    
    // Create configuration with progress indicator
    let result = CommandHelpers::with_progress("Creating configuration", || {
        ConfigHelpers::create_claude_config(
            &project_dir,
            "MyProject",
            "embedded",
            &vec!["fast".to_string()]
        )
    });
    
    match result {
        Ok(_) => CommandHelpers::print_success("Configuration created"),
        Err(e) => CommandHelpers::print_error(&format!("Failed: {}", e)),
    }
}
```

## Benefits

1. **Consistency**: All commands use the same formatting and error handling
2. **Code Reuse**: Common operations are centralized
3. **Better UX**: Consistent visual feedback across all commands
4. **Error Handling**: Standardized error messages and recovery
5. **Maintainability**: Changes to common functionality only need to be made once

## Migration Guide

To update existing commands to use the helper functions:

1. Import the helpers:
   ```rust
   use crate::helpers::{CommandHelpers, ConfigHelpers, RepositoryHelpers};
   ```

2. Replace direct `println!` calls with appropriate helper functions:
   ```rust
   // Before
   println!("{}", "Success!".green());
   
   // After
   CommandHelpers::print_success("Success!");
   ```

3. Use path resolution helpers:
   ```rust
   // Before
   let path = match path {
       Some(p) => PathBuf::from(p),
       None => env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
   };
   
   // After
   let path = CommandHelpers::resolve_project_path(path)?;
   ```

4. Use file operation helpers:
   ```rust
   // Before
   fs::write(&file_path, content)?;
   
   // After
   CommandHelpers::create_file_with_content(&file_path, content)?;
   ```

5. Use repository helpers for git operations:
   ```rust
   // Before
   Command::new("git").arg("init").current_dir(path).output()?;
   
   // After
   RepositoryHelpers::init_git_repository(path)?;
   ```

These helpers significantly reduce code duplication and make the CI tool more maintainable and consistent.