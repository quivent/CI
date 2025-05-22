# CI Testing Framework

This documentation describes the comprehensive testing framework for the CI project. The framework is designed to provide robust testing capabilities for all aspects of the CI CLI tool.

## Overview

The CI testing framework consists of several key components:

1. **TestEnv** - A test environment manager that provides isolated environments for tests
2. **Helper Utilities** - A collection of utility functions to simplify common testing tasks
3. **Test Suites** - Organized test modules for different aspects of CI functionality
4. **Integration Tests** - End-to-end tests for CI commands and workflows

## TestEnv

The `TestEnv` struct (defined in `tests/test_helpers.rs`) provides a temporary testing environment for isolated test execution. It handles:

- Creating temporary directories for test artifacts
- Managing environment variables and working directories
- Setting up mock repositories and CI structures
- Cleaning up resources after tests complete

### Key Features

```rust
// Create a new test environment
let test_env = TestEnv::new();

// Get a path within the temporary directory
let file_path = test_env.path("test_file.txt");

// Create a file with content
let created_path = test_env.create_file("test_file.txt", "Content");

// Create a directory
let dir_path = test_env.create_dir("test_dir");

// Set up a mock CI repository
let cir_repo = test_env.setup_mock_cir_repo();

// Set up a git repository
let git_repo = test_env.setup_git_repo();

// Set up an integrated CI repository
let integrated_repo = test_env.setup_cir_integrated_repo();

// Set up an advanced git repository with branches
let advanced_repo = test_env.setup_advanced_git_repo();
```

## Helper Utilities

The helper utilities (defined in `tests/helper_utils.rs`) provide specialized functionality for different testing scenarios:

### CommandUtils

Utilities for working with command execution and file operations:

```rust
// Check if a path is a git repository
let is_git_repo = CommandUtils::is_git_repository(path);

// Create a file with content
CommandUtils::create_file_with_content(path, content)?;

// Read file content
let content = CommandUtils::read_file_content(path)?;

// Execute a function with progress indication
let result = CommandUtils::with_progress("Test operation", || {
    // Operation to perform
    Ok(42)
})?;

// Check if a command exists in the system
let has_git = CommandUtils::command_exists("git");

// Run a process with custom environment
let output = CommandUtils::run_process(
    "command",
    &["arg1", "arg2"],
    Some(working_dir),
    Some(&env_vars)
)?;
```

### RepositoryUtils

Utilities for working with Git repositories and CI project files:

```rust
// Create a default .gitignore file
let gitignore = RepositoryUtils::create_default_gitignore(path)?;

// Create a CLAUDE.md file
let claude_md = RepositoryUtils::create_claude_md(
    path,
    "ProjectName",
    "embedded",
    &["Agent1".to_string(), "Agent2".to_string()]
)?;

// Find the nearest git repository
let repo_path = RepositoryUtils::find_git_repository(path);

// Get the current git branch
let branch = RepositoryUtils::get_current_branch(repo_path)?;
```

### ConfigUtils

Utilities for working with CI configuration:

```rust
// Get the CI repository path
let cir_repo = ConfigUtils::get_cir_repo_path()?;

// Check if a directory is a CI project
let is_cir_project = ConfigUtils::is_cir_project(path);

// Extract project name from CLAUDE.md
let project_name = ConfigUtils::extract_project_name(path)?;
```

### AgentUtils

Utilities for working with CI agents:

```rust
// Get available agents from a CI repository
let agents = AgentUtils::get_available_agents(cir_repo_path)?;

// Check if an agent exists
let exists = AgentUtils::agent_exists(cir_repo_path, "AgentName");
```

## Test Suites

The CI test framework includes several test suites:

### Command Tests (`command_tests.rs`)

Tests for CI commands like `init`, `status`, `verify`, etc.

### Helper Tests (`helper_tests.rs`)

Tests for CI's internal helper functions.

### Integration Tests (`integration_tests.rs`)

Tests for end-to-end workflows combining multiple CI commands.

### Advanced Tests (`advanced_tests.rs`)

Tests for the advanced helper utilities provided by the testing framework.

## Running Tests

To run the CI test suite:

```sh
cargo test
```

To run a specific test:

```sh
cargo test test_name
```

To run a specific test suite:

```sh
cargo test --test test_file_name
```

## Best Practices

When writing tests for CI:

1. **Use TestEnv** - Always use the `TestEnv` for isolated testing
2. **Clean Up** - Ensure tests clean up after themselves (TestEnv does this automatically)
3. **Descriptive Names** - Use clear, descriptive test names
4. **One Assertion** - Focus each test on one specific behavior
5. **Test Edge Cases** - Consider failures, not just happy paths
6. **Use Helper Utilities** - Leverage the helper utilities for common operations
7. **Document Tests** - Add comments explaining the test's purpose

## Example Test

Here's an example of a well-structured test:

```rust
#[test]
fn test_project_initialization() -> Result<()> {
    // Setup test environment
    let test_env = TestEnv::new();
    let cir_repo = test_env.setup_mock_cir_repo();
    
    // Run the command being tested
    let output = run_cir(&["init", "test-project", "--agents", "Athena"]);
    assert!(output.status.success());
    
    // Verify the expected results
    let project_dir = test_env.path("test-project");
    assert!(project_dir.exists());
    
    // Check specific file content
    let claude_md = project_dir.join("CLAUDE.md");
    assert!(claude_md.exists());
    
    let content = fs::read_to_string(claude_md)?;
    assert!(content.contains("# Project: test-project"));
    assert!(content.contains("- Athena"));
    
    Ok(())
}
```

## Extending the Framework

To extend the testing framework:

1. Add new helper methods to `TestEnv` for common setup operations
2. Add new utility functions to the appropriate utility class
3. Create new test files for specific areas of functionality
4. Update this documentation to reflect new capabilities