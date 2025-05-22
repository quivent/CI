# CI Documentation Index

## Overview

CI (Collaborative Intelligence) is a modern command-line interface for the Collaborative Intelligence system. It provides a comprehensive set of commands for project management, source control, and system operations.

This documentation index provides links to all available documentation for the CI tool.

## Command Reference

### 🧠 Intelligence & Discovery Commands

- [Intent](commands/intelligence/intent.md) - Display the intent and purpose of the CI tool
- [Agents](commands/intelligence/agents.md) - List all available Collaborative Intelligence agents
- [Load](commands/intelligence/load.md) - Start a Claude Code session with a specified agent loaded
- [Projects](commands/intelligence/projects.md) - List projects integrated with Collaborative Intelligence

### 📊 Source Control Commands

- [Status](commands/source_control/status.md) - Display detailed status of the git repository and working tree
- [Repo](commands/source_control/repo.md) - Manage GitHub repositories using gh CLI
- [Clean](commands/source_control/clean.md) - Clean build artifacts from the project
- [Ignore](commands/source_control/ignore.md) - Update .gitignore with appropriate patterns for CI
- [Stage](commands/source_control/stage.md) - Run ignore and then stage all untracked and unstaged files
- [Remotes](commands/source_control/remotes.md) - Configure git remotes for personal and organizational repositories
- [Commit](commands/source_control/commit.md) - Run ignore, stage files, analyze changes, and commit with a detailed message
- [Deploy](commands/source_control/deploy.md) - Run ignore, stage, commit, and push in one operation

### 🚀 Project Lifecycle Commands

- [Init](commands/lifecycle/init.md) - Initialize a project with Collaborative Intelligence
- [Integrate](commands/lifecycle/integrate.md) - Integrate CI into an existing project
- [Fix](commands/lifecycle/fix.md) - Repair CI integration issues
- [Verify](commands/lifecycle/verify.md) - Verify CI integration is working properly
- [Local](commands/lifecycle/local.md) - Create or update CLAUDE.local.md file for local CI configuration

### ⚙️ System Management Commands

- [Evolve](commands/system/evolve.md) - Evolve the CI tool with Claude Code assistance
- [Key](commands/system/key.md) - Manage API keys for external services
- [Build](commands/system/build.md) - Build and install the CI binary
- [Install](commands/system/install.md) - Install CI tool to system path
- [Link](commands/system/link.md) - Create symlinks to the CI binary in system paths
- [Unlink](commands/system/unlink.md) - Remove symlinks to the CI binary from system paths
- [Legacy](commands/system/legacy.md) - Create legacy command symlinks for CI compatibility
- [Docs](commands/system/docs.md) - Generate comprehensive documentation
- [Version](commands/system/version.md) - Print version information

## Helper Function Reference

- [Command Helpers](helpers/command.md) - Helper functions for command operations
- [Repository Helpers](helpers/repository.md) - Helper functions for repository operations
- [API Key Management](helpers/api_keys.md) - Helper functions for API key management
- [System Helpers](helpers/system.md) - Helper functions for system operations
- [Path Helpers](helpers/path.md) - Helper functions for path operations
- [Project Helpers](helpers/project.md) - Helper functions for project operations
- [Commit Analyzer](helpers/commit_analyzer.md) - Helper functions for analyzing commits

## Guides & Tutorials

- [Creating New Commands](guides/command_creation.md) - Guide for creating new commands
- [API Key Management](guides/api_key_management.md) - Guide for managing API keys
- [Repository Integration](guides/repository_integration.md) - Guide for integrating repositories
- [Legacy Command Support](guides/legacy_commands.md) - Guide for using legacy commands

## Development & Testing

- [Testing Framework](testing_framework.md) - Overview of the testing framework
- [Helper Testing](helpers/testing.md) - Guide for testing helper functions
- [Command Testing](guides/command_testing.md) - Guide for testing commands

## Other Documentation

- [README](../README.md) - Project overview and getting started
- [Style Guide](../STYLE_GUIDE.md) - Code style conventions
- [Contributing](../CONTRIBUTING.md) - Guidelines for contributors