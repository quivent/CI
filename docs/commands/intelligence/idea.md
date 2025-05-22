# 💡 IDEA

Manage ideas, concepts, and inspirations

*Category: Intelligence & Discovery*

## Description

The `idea` command provides a comprehensive system for managing creative ideas, concepts, and inspirations. It allows users to capture, organize, categorize, and track ideas throughout their lifecycle, ensuring valuable insights are preserved and can be revisited later.

## Usage

```bash
ci idea <SUBCOMMAND> [OPTIONS]
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `list` | List all ideas with optional filtering |
| `add` | Add a new idea |
| `view` | View details of a specific idea |
| `update` | Update an existing idea |
| `delete` | Delete an idea |
| `categories` | List all categories used in ideas |
| `tags` | List all tags used in ideas |

## Options

### Global Options

| Option | Description |
|--------|-------------|
| `--filter <filter>` | Filter ideas by text in title, description, or tags |
| `--category <category>` | Filter or set idea category |
| `--status <status>` | Filter by status or set idea status |

### Add/Update Options

| Option | Description |
|--------|-------------|
| `--title <title>` | Set idea title |
| `--description <description>` | Set idea description |
| `--tags <tags>` | Comma-separated list of tags |
| `--priority <priority>` | Set idea priority (low, medium, high, critical) |

### View/Update/Delete Options

| Option | Description |
|--------|-------------|
| `--id <id>` | Idea ID to view, update, or delete |

## Examples

```bash
# List all ideas
ci idea list

# Filter ideas by category
ci idea list --category "Project Ideas"

# Filter ideas by status
ci idea list --status "exploring"

# Text search across ideas
ci idea list --filter "authentication"

# Add a new idea
ci idea add --title "Implement OAuth" --description "Add OAuth support for GitHub integration" --category "Features" --tags "auth,security,github"

# View an idea
ci idea view --id abc123

# Update an idea
ci idea update --id abc123 --status "in_development" --priority "high"

# Delete an idea
ci idea delete --id abc123

# List categories
ci idea categories

# List tags
ci idea tags
```

## Statuses

Ideas can have the following statuses:

- `new` - Fresh idea, not yet evaluated
- `exploring` - Idea is being investigated
- `development` - Idea is in active development
- `implemented` - Idea has been implemented
- `onhold` - Development temporarily paused
- `archived` - Idea stored for future reference
- `rejected` - Idea was evaluated and rejected

## Storage

Ideas are stored in JSON format in:

- Project-specific ideas: `.ci/ideas.json` in the project directory
- Global ideas: In the user's data directory under `ci/ideas.json`

## Notes

- Each idea is assigned a unique ID for referencing
- Ideas can be linked to related ideas to build concept networks
- Idea metadata includes creation and update timestamps

## Related Commands

- [intent](intent.md) - Display the intent and purpose of the CI tool
- [agents](agents.md) - List all available Collaborative Intelligence agents