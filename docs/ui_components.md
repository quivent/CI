# UI Components for CI CLI

This document describes the user interface components available in the CI CLI tool. These components provide consistent formatting and visual elements for a better user experience.

## Basic Output Functions

### Command Headers

```rust
CommandHelpers::print_command_header(title: &str, emoji: &str, category: &str, color: &str)
```

Prints a styled header for commands with a category and emoji indicator.

**Example:**
```rust
CommandHelpers::print_command_header(
    "Initialize a new project",
    "🚀",
    "Project Lifecycle",
    "yellow"
);
```

**Output:**
```
┌─────────────────────────────────────────────────────────────────┐
│  🚀  Project Lifecycle
└─────────────────────────────────────────────────────────────────┘

Initialize a new project
```

### Status Messages

For reporting operation outcomes:

```rust
// Success messages (green with checkmark)
CommandHelpers::print_success("Configuration file created successfully");

// Error messages (red with cross)
CommandHelpers::print_error("Failed to read configuration file");

// Warning messages (yellow with exclamation mark)
CommandHelpers::print_warning("Configuration contains deprecated settings");

// Info messages (blue with info symbol)
CommandHelpers::print_info("Processing configuration file");
```

### Dividers

To visually separate sections:

```rust
// Print a styled divider line
CommandHelpers::print_divider("blue");
```

### Sections

For breaking content into logical parts:

```rust
// Prints a section header with underline
CommandHelpers::print_section("System Configuration");
```

**Output:**
```
System Configuration
-------------------
```

## Advanced UI Components

### Progress Indicators

For long-running operations:

```rust
// Using the Spinner directly
let mut spinner = Spinner::new("Loading configuration");
spinner.start();
// ... perform work ...
spinner.stop(SpinnerResult::Success);

// Using the helper function
let result = with_spinner("Verifying project structure", || {
    // ... perform work ...
    Ok(())
});
```

### Step Tracking

For multi-step processes:

```rust
// Print a step with number and total
CommandHelpers::print_step(1, 3, "Creating project directory");
// ... code for this step ...

CommandHelpers::print_step(2, 3, "Initializing git repository");
// ... code for this step ...

CommandHelpers::print_step(3, 3, "Configuring CI integration");
// ... code for this step ...
```

### Boxed Messages

For highlighting important information:

```rust
// Print text in a box
CommandHelpers::print_box("Project ready for development!", "green");
```

**Output:**
```
┌──────────────────────────────────┐
│  Project ready for development!  │
└──────────────────────────────────┘
```

### List Items with Status

For checklist-style output:

```rust
// Print list items with different statuses
CommandHelpers::print_list_item("Verify configuration", Some("success"));
CommandHelpers::print_list_item("Check dependencies", Some("failure"));
CommandHelpers::print_list_item("Initialize database", Some("warning"));
CommandHelpers::print_list_item("Optional: Configure logging", Some("skipped"));
CommandHelpers::print_list_item("Additional tasks", None);
```

**Output:**
```
• Verify configuration [OK]
• Check dependencies [FAILED]
• Initialize database [WARNING]
• Optional: Configure logging [SKIPPED]
• Additional tasks
```

## User Interaction

For collecting user input:

```rust
// Prompt for confirmation
if CommandHelpers::prompt_confirmation("Do you want to continue?") {
    // User selected yes
} else {
    // User selected no
}

// Prompt for input with optional default
let name = CommandHelpers::prompt_input("Enter project name", Some("my-project"))?;
```

## Formatting Helpers

For consistent formatting of common elements:

```rust
// Format a list of files
let files = vec!["file1.rs".to_string(), "file2.rs".to_string()];
let formatted = CommandHelpers::format_file_list(&files);
println!("Files to process:\n{}", formatted);
```

## Best Practices

1. **Use Consistent Colors**:
   - `blue`: For general information, status, and details
   - `green`: For success messages and positive outcomes
   - `yellow`: For warnings and caution messages
   - `red`: For errors and critical failures
   - `cyan`: For commands, steps, and operations

2. **Use Standard Icons**:
   - ✓ (checkmark): For success
   - ✗ (cross): For failure
   - ! (exclamation): For warnings
   - ℹ (info): For information
   - • (bullet): For list items
   - → (arrow): For steps or actions

3. **Be Consistent with Styling**:
   - Use command headers for main commands
   - Use sections for logical grouping
   - Use steps for ordered, multi-part operations
   - Use dividers to separate major content areas