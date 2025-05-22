# Command Creation Guide

This guide explains how to create new commands for CI.

## Using Instant Command Creation

The fastest way to create a new command is to use the instant command creation pattern:

```bash
ci "CI:[command-name]" 
# or
ci "CI:[command-name] [description]"
```

If you don't provide a description, you'll be prompted for one. The system will automatically categorize the command and create all necessary files.

### Examples

```bash
# Create a new 'analyze' command with description
ci "CI:analyze Analyze project structure and dependencies"

# Create a 'sync' command (will prompt for description)
ci "CI:sync"
```

## How Instant Command Creation Works

When you use the instant command pattern:

1. CI extracts the command name from the format
2. It prompts for a description if one wasn't provided
3. It automatically categorizes the command based on keywords in the name and description
4. It generates the command implementation in the appropriate category file
5. It updates the main.rs file to add the command to the Commands enum and match statement
6. It creates documentation for the command

## Manual Command Creation

If you prefer to create commands manually, follow these steps:

1. Determine which category module your command belongs in:
   - `src/commands/intelligence.rs` for Intelligence & Discovery commands
   - `src/commands/source_control.rs` for Source Control commands
   - `src/commands/lifecycle.rs` for Project Lifecycle commands
   - `src/commands/system.rs` for System Management commands

2. Add your function to the appropriate module file using this template:

```rust
/// Description of what the command does
pub async fn command_name(config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Command description", 
        "🧠", // Category emoji
        "Intelligence & Discovery", // Category name
        "blue" // Category color
    );
    
    // Command implementation
    
    CommandHelpers::print_success("Command completed successfully");
    
    Ok(())
}
```

3. Update the Commands enum in main.rs:

```rust
enum Commands {
    // ...existing commands...
    
    /// Description of what the command does
    #[command(about = format!("{}", "Description of what the command does".blue()))]
    CommandName,
}
```

4. Add the command to the match statement in main.rs:

```rust
match cli.command.unwrap() {
    // ...existing matches...
    
    Commands::CommandName => {
        commands::intelligence::command_name(&config).await
    },
}
```

5. Create documentation in the docs/commands directory

## Standard Colors and Emojis

CI uses consistent colors and emojis for each command category:

- Intelligence & Discovery: 🧠 Blue
- Source Control: 📊 Green
- Project Lifecycle: 🚀 Yellow
- System Management: ⚙️ Cyan

Use these colors and emojis in your command headers for consistency.

## Using Helper Functions

CI provides several helper modules to make command implementation easier:

- `CommandHelpers`: UI formatting, progress indicators
- `RepositoryHelpers`: Git operations, status management
- `ConfigHelpers`: Configuration management, API key handling
- `ProjectHelpers`: Project information, statistics
- `PathHelpers`: Path resolution, directory operations

Use these helpers rather than implementing functionality directly to ensure consistent behavior across commands.

## Testing Your Command

After creating a new command, build CI and test your command:

```bash
cargo build
cargo install --path .
ci your-command
```

Make sure your command follows the CI style guide and provides appropriate feedback to the user.