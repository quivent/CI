# Legacy Command Support

This guide provides information on using and managing legacy commands for compatibility with the original CI tool.

## Overview

CI provides backward compatibility with the original CI tool through legacy command support. This allows scripts and workflows designed for the original CI tool to work seamlessly with CI.

## Legacy Command Mapping

The following legacy commands are supported and mapped to their CI equivalents:

| Legacy Command | CI Equivalent |
|----------------|---------------|
| `status` | `ci status` |
| `init` | `ci init` |
| `integrate` | `ci integrate` |
| `fix` | `ci fix` |
| `verify` | `ci verify` |
| `agents` | `ci agents` |
| `load` | `ci load` |
| `commit` | `ci commit` |
| `push` | `ci deploy` |
| `evolve` | `ci evolve` |
| `key` | `ci key` |
| `local` | `ci local` |
| `build` | `ci build` |
| `install` | `ci install` |
| `stage-commit` | `ci commit` |
| `stage-commit-push` | `ci deploy` |
| `update-gitignore` | `ci ignore` |

## Setting Up Legacy Command Support

You can set up legacy command support in two ways:

### 1. Using the Legacy Command

```bash
# List available legacy commands
ci legacy --list

# Create symlinks for all legacy commands
ci legacy --create

# Remove legacy command symlinks
ci legacy --remove
```

### 2. Manual Setup

You can also create symlinks manually:

```bash
# Create a symlink for a specific legacy command
ln -s /path/to/ci /usr/local/bin/commit
```

## Usage

Once set up, legacy commands can be used directly:

```bash
# Using legacy command
commit -m "My commit message"

# Equivalent CI command
ci commit -m "My commit message"
```

## Common Issues

- **Path Issues**: Make sure the directory containing the legacy command symlinks is in your PATH.
- **Permission Denied**: You may need to use `sudo` when creating symlinks in system directories.
- **Command Not Found**: Verify that the symlinks were created correctly and point to the CI executable.

## Further Reading

For more information on CI installation and system management, see the main documentation index.