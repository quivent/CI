# Command Helpers

The `CommandHelpers` module provides functions for formatting command output and executing commands with consistent UI.

## UI Functions

### `print_command_header`

Prints a stylized header for a command with category and emoji.

```rust
CommandHelpers::print_command_header(
    "My Command Description", 
    "🧠", // Category emoji
    "Intelligence & Discovery", 
    "blue" // Category color
);
```

Output:
```
┌─────────────────────────────────────────────────────────────────┐
│  🧠  Intelligence & Discovery
└─────────────────────────────────────────────────────────────────┘

My Command Description
```

### `print_success`

Prints a success message with a green checkmark.

```rust
CommandHelpers::print_success("Command completed successfully");
```

Output:
```
✓ Command completed successfully
```

### `print_error`

Prints an error message with a red cross.

```rust
CommandHelpers::print_error("An error occurred");
```

Output:
```
✗ An error occurred
```

### `print_warning`

Prints a warning message with a yellow exclamation mark.

```rust
CommandHelpers::print_warning("This operation might take a while");
```

Output:
```
! This operation might take a while
```

### `print_info`

Prints an informational message with a blue info symbol.

```rust
CommandHelpers::print_info("Processing 5 files");
```

Output:
```
ℹ Processing 5 files
```

### `print_step`

Prints a step message with number and total.

```rust
CommandHelpers::print_step(2, 5, "Processing file");
```

Output:
```
[2/5] • Processing file
```

### `print_status`

Prints a status message with a bullet point.

```rust
CommandHelpers::print_status("File processed");
```

Output:
```
  • File processed
```

### `print_arrow_step`

Prints a step message with arrow.

```rust
CommandHelpers::print_arrow_step("Loading configurations");
```

Output:
```
→ Loading configurations
```

### `print_divider`

Prints a divider line with the specified color.

```rust
CommandHelpers::print_divider("blue");
```

Output:
```
══════════════════════════════════════════════════════════════════════════════
```

## Command Execution Functions

### `run_command_with_output`

Runs a command and captures stdout, stderr, and success status.

```rust
let (success, stdout, stderr) = CommandHelpers::run_command_with_output(
    "git", 
    &["status", "--short"], 
    Some(Path::new("/path/to/repo"))
)?;
```

### `run_command_with_progress`

Runs a command asynchronously with a progress indicator.

```rust
let output = CommandHelpers::run_command_with_progress(
    "git", 
    &["pull", "origin", "main"], 
    &Path::new("/path/to/repo"), 
    "Pulling latest changes"
).await?;
```

### `with_progress`

Executes a function with a spinner/progress indicator.

```rust
let result = CommandHelpers::with_progress("Loading data", || {
    // Code that returns a Result
    Ok(42)
})?;
```

## User Input Functions

### `prompt_confirmation`

Prompts the user for confirmation (y/n).

```rust
if CommandHelpers::prompt_confirmation("Do you want to continue?") {
    // User confirmed
} else {
    // User declined
}
```

### `prompt_input`

Prompts the user for input with an optional default value.

```rust
let name = CommandHelpers::prompt_input("Enter your name", Some("User"))?;
```

## Utility Functions

### `format_file_list`

Formats a list of files with bullet points for display.

```rust
let files = vec!["file1.rs".to_string(), "file2.rs".to_string()];
let formatted = CommandHelpers::format_file_list(&files);
// "  • file1.rs\n  • file2.rs"
```

### `display_enhanced_help`

Displays command help with enhanced formatting.

```rust
CommandHelpers::display_enhanced_help(
    "commit",
    "Create a commit with the specified message",
    "ci commit [--message <message>]",
    &[
        "ci commit --message \"Fix bug in parser\"",
        "ci commit"
    ]
);
```

### `get_timestamp`

Gets the current timestamp in human-readable format.

```rust
let timestamp = CommandHelpers::get_timestamp();
// "2023-01-01 12:34:56"
```

### Environment Checks

#### `is_verbose`

Checks if running in verbose mode (CI_VERBOSE=true).

```rust
if CommandHelpers::is_verbose() {
    // Print additional information
}
```

#### `is_debug`

Checks if running in debug mode (CI_DEBUG=true).

```rust
if CommandHelpers::is_debug() {
    // Print debug information
}
```