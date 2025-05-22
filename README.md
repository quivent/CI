# CI - Collaborative Intelligence CLI

A modern implementation of the Collaborative Intelligence CLI, providing a structured interface for agent management, project integration, source control operations, and system management tasks.

## Features

- ✅ Complete agent listing and loading functionality with markdown parsing
- ✅ Project initialization and integration with the Collaborative Intelligence system
- ✅ Project verification and repair functionality
- ✅ Local configuration management with CLAUDE.local.md files
- ✅ Integrated projects discovery and listing
- ✅ Source control management with git operations and enhanced commit analysis
- ✅ System management commands for installation and configuration
- ✅ Organized command structure with color-coded categories
- ✅ Enhanced error handling and user feedback
- ✅ Native Rust installation with automatic PATH management
- ✅ Instant command creation with `CI:command_name` pattern
- ✅ Automated code generation with templates
- ✅ Comprehensive testing framework with test environment management
- ✅ API key management system for secure handling of credentials

## Commands

CI is organized into four main categories:

### 🧠 Intelligence & Discovery

- `ci agents` - List all available Collaborative Intelligence agents
- `ci load <agent>` - Start a Claude Code session with a specified agent
- `ci projects` - List projects integrated with Collaborative Intelligence
- `ci intent` - Display the intent and purpose of the CI tool

### 📊 Source Control

- `ci status` - Display detailed status of the git repository
- `ci commit` - Stage files, analyze changes, and commit with a detailed message
- `ci deploy` - Stage, commit, and push in one operation
- `ci clean` - Clean build artifacts from the project
- `ci ignore` - Update .gitignore with appropriate patterns
- `ci stage` - Stage all untracked and unstaged files
- `ci repo` - Manage GitHub repositories using gh CLI
- `ci remotes` - Configure git remotes

### 🚀 Project Lifecycle

- `ci init <project-name>` - Initialize a project with CI
- `ci integrate` - Integrate CI into an existing project
- `ci fix` - Repair CI integration issues
- `ci verify` - Verify CI integration is working properly
- `ci local` - Create or update CLAUDE.local.md for local configuration

### ⚙️ System Management

- `ci build` - Build the CI binary with cargo
- `ci install` - Build and install CI tool to system path
- `ci link` - Create symlinks to the CI binary
- `ci unlink` - Remove symlinks to the CI binary
- `ci fix-warnings` - Automatically fix common compiler warnings
- `ci docs` - Generate comprehensive documentation
- `ci add-command` - Add a new command to CI
- `ci version` - Print version information
- `ci key` - Manage API keys for external services:
  - `ci key list` - List all stored API keys
  - `ci key add <service> <key_name> <key_value>` - Add a new API key
  - `ci key add <service> <key_name> <key_value> --env <env>` - Add environment-specific key
  - `ci key add <service> <key_name> <key_value> --project` - Add project-specific key
  - `ci key remove <service> <key_name>` - Remove an API key
  - `ci key export` - Export API keys for shell environment
- `ci evolve` - Evolve the CI tool with Claude Code assistance

## Installation

### Quick Start

```bash
# Build and install to ~/.local/bin
cargo build
cargo run -- install

# Create symlinks
cargo run -- link
```

### Alternative Installation Methods

```bash
# Build the binary directly
ci build

# Install to ~/.local/bin
ci install

# Create symlinks manually
ci link
```

### Manual Installation

```bash
# Build the release version
cargo build --release

# Copy the binary to a location in your PATH
cp target/release/CI ~/.local/bin/ci
chmod +x ~/.local/bin/ci
```

## Working with Agents

CI provides enhanced functionality for working with the Collaborative Intelligence agents:

### Listing Available Agents

```bash
ci agents
```

This command displays all available agents with descriptions from the AGENTS.md file, including:
- Agent name and role
- Brief description of capabilities
- Usage instructions

### Loading an Agent

```bash
ci load Athena
```

This command:
1. Verifies the agent exists in AGENTS.md
2. Extracts agent information and context
3. Sets up agent toolkit directory
4. Loads agent memory and continuous learning content
5. Outputs the agent memory to stdout for piping to Claude Code

Typical usage with Claude Code:
```bash
# Pipe the output to Claude Code
ci load Athena | claude code
```

### Advanced Agent Loading

```bash
# Load with specific context
ci load Fixer --context="Fix compilation errors" | claude code

# Load from a custom memory file
ci load Recommender --path=/path/to/custom/memory.md | claude code

# Save agent memory to a file for later use
ci load Fixer > fixer_memory.md
```

## Project Management

### Creating a New Project

```bash
ci init my-new-project
```

This creates a new directory with:
- Basic project structure
- CI configuration files
- Environment setup
- Git repository initialization

### Integrating with an Existing Project

```bash
cd existing-project
ci integrate
```

Adds CI capabilities to an existing project by:
- Creating configuration files
- Setting up environment variables
- Preparing for agent use

### Verifying Integration

```bash
ci verify
```

Tests that the CI integration is working correctly.

## Troubleshooting

### Command Not Found

If you encounter a "command not found" error, ensure that the installation directory is in your PATH.

### Missing Repository

If CI cannot find the Collaborative Intelligence repository, set the `CI_PATH` environment variable:

```bash
export CI_PATH=/path/to/CollaborativeIntelligence
```

### Agent Loading Issues

If agent loading fails, check that:
- The agent name is spelled correctly
- The CollaborativeIntelligence repository path is set correctly
- You have the proper permissions to access the agent files

## Design Philosophy

CI was built with a focus on:

1. **Pure Rust Implementation** - Fully implemented in Rust without bash scripts or external dependencies
2. **Maintainability** - Clean, organized code structure with clear separation of concerns
3. **User Experience** - Consistent command interface with helpful feedback and error messages
4. **Extensibility** - Easy to add new commands and features
5. **Performance** - Efficient implementation with minimal dependencies

## Testing Framework

CI includes a comprehensive testing framework to ensure reliability and maintainability:

### Test Environment

The `TestEnv` provides an isolated environment for tests:

```rust
let test_env = TestEnv::new();
let repo_dir = test_env.setup_git_repo();
```

### Utility Functions

Helper utilities simplify testing common operations:

```rust
// Repository utilities
RepositoryUtils::create_default_gitignore(path)?;
RepositoryUtils::get_current_branch(repo_path)?;

// Command utilities
CommandUtils::is_git_repository(path);
CommandUtils::run_process("git", &["status"], Some(path), None)?;

// Config utilities
ConfigUtils::is_ci_project(path);
ConfigUtils::extract_project_name(path)?;

// Agent utilities
AgentUtils::get_available_agents(cir_repo_path)?;
```

### Advanced Testing

The framework supports advanced testing scenarios:

```rust
// Create repository with commit history and branches
let repo = test_env.setup_advanced_git_repo();

// Test CI integration
let ci_repo = test_env.setup_ci_integrated_repo();
```

For more details, see [testing_framework.md](docs/testing_framework.md).

## API Key Management

CI includes a secure API key management system:

```rust
// Get an API key
let api_key = ApiKeyManager::get_key("service", "key_name")?;

// Set an API key
ApiKeyManager::set_key("service", "key_name", "key_value")?;

// CLI usage
ci config set-key openai api_key sk-xxxxxxxxxxxx
ci config list-keys
```

For more details, see [api_key_management.md](docs/api_key_management.md).

## Contributing

Contributions to CI are welcome! Please feel free to submit pull requests or open issues for bugs, feature requests, or documentation improvements.

## License

MIT