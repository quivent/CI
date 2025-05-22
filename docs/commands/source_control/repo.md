# 📊 REPO

Manage GitHub repositories using gh CLI

*Category: Source Control*

## Description

The `repo` command provides a streamlined interface for managing GitHub repositories using the GitHub CLI (gh). It supports repository listing, creation, cloning, and viewing detailed repository information.

## Usage

```bash
ci repo [SUBCOMMAND] [OPTIONS]
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `list` | List all repositories you have access to |
| `create <name>` | Create a new GitHub repository |
| `clone <repo>` | Clone a GitHub repository |
| `view <repo>` | View detailed information about a repository |

## Options

### create options

| Option | Description |
|--------|-------------|
| `--description <description>` | Repository description |
| `--private` | Make the repository private |

### clone options

| Option | Description |
|--------|-------------|
| `--dir <directory>` | Directory to clone into (defaults to repo name) |

## Examples

```bash
# List all repositories
ci repo list

# Create a new repository
ci repo create my-repo --description "A new project" --private

# Clone a repository
ci repo clone username/repo-name

# Clone a repository to a specific directory
ci repo clone username/repo-name --dir /custom/path

# View repository details
ci repo view username/repo-name
```

## Notes

- Requires GitHub CLI (gh) to be installed and authenticated
- Repository creation will automatically initialize with a README.md
- When cloning, you can use either a full URL or shorthand owner/repo format

## Related Commands

- [status](status.md) - Display detailed status of the git repository
- [commit](commit.md) - Commit changes with detailed analysis
- [deploy](deploy.md) - Deploy changes to remote repository