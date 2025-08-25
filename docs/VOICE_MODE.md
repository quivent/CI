# CI Voice Mode and Auto-Accept Configuration

## Overview

The CI tool now supports Voice Mode and configurable auto-accept behavior for launching agents with Claude Code, allowing for streamlined workflows with automatic tool use acceptance.

## Voice Mode

Voice Mode is a new way to launch agents that automatically enables auto-accept for all Claude Code prompts, allowing for uninterrupted execution of tool commands.

### Usage

```bash
# Launch an agent in voice mode (auto-accept enabled)
ci agent voice AGENT_NAME

# Example
ci agent voice CLIA
ci agent voice Athena
```

### Features

- **Automatic Launch**: No confirmation prompts - immediately launches Claude Code
- **Auto-Accept Enabled**: All tool use prompts are automatically accepted
- **Visual Indicators**: Clear indication that voice mode is active
- **Agent Memory Loading**: Full agent memory and capabilities are loaded

## Load Command with Free Mode

The `load` command now supports a `-free` flag for auto-accept mode:

```bash
# Regular load (prompts for confirmation)
ci agent load AGENT_NAME

# Free mode (auto-accepts all prompts)
ci agent load AGENT_NAME --free
# or
ci agent load AGENT_NAME -f
```

## Configuration-Based Auto-Accept

You can configure auto-accept behavior through a `.ci-config.json` file:

### Configuration File Location

The CI tool looks for `.ci-config.json` in the following order:
1. Current directory
2. Parent directories (up to root)
3. Home directory

### Configuration Structure

```json
{
  "project_name": "MyProject",
  "ci_version": "0.1.0",
  "created_at": "2025-08-16T00:00:00Z",
  "updated_at": "2025-08-16T00:00:00Z",
  "active_agents": ["Athena", "ProjectArchitect"],
  "fast_activation": true,
  "auto_accept": {
    "agent_load": false,        // Auto-accept for all agent load commands
    "agent_activate": false,     // Auto-accept for all agent activate commands
    "agents": ["CLIA", "Athena"], // Specific agents to auto-accept
    "global": false             // Global auto-accept (use with caution!)
  },
  "metadata": {}
}
```

### Auto-Accept Settings

The `auto_accept` configuration supports:

- **`agent_load`**: When `true`, all `ci agent load` commands will use auto-accept
- **`agent_activate`**: When `true`, all `ci agent activate` commands will use auto-accept
- **`agents`**: List of specific agent names that should always use auto-accept
- **`global`**: When `true`, ALL agents use auto-accept (override all other settings)

### Priority Order

1. **Command-line flags** (highest priority)
   - `--free` flag on load command
   - `ci agent voice` command
2. **Configuration file settings**
   - Global override (`auto_accept.global`)
   - Specific agent list (`auto_accept.agents`)
   - Command-specific settings (`auto_accept.agent_load`, etc.)
3. **Default behavior** (lowest priority)
   - Normal prompting mode

## Examples

### Example 1: Voice Mode for Quick Development

```bash
# Launch CLIA agent in voice mode for immediate development
ci agent voice CLIA
```

### Example 2: Configuration for Trusted Agents

Create `.ci-config.json`:
```json
{
  "auto_accept": {
    "agents": ["CLIA", "Athena", "ProjectArchitect"]
  }
}
```

Now these agents will always launch with auto-accept:
```bash
ci agent load CLIA  # Auto-accept enabled automatically
```

### Example 3: Project-Wide Auto-Accept

For development environments where you want all agents to auto-accept:
```json
{
  "auto_accept": {
    "agent_load": true
  }
}
```

## Security Considerations

⚠️ **WARNING**: Auto-accept mode allows Claude Code to execute commands without confirmation. Use with caution:

- **Voice Mode**: Best for interactive development sessions where you're actively monitoring
- **Configuration**: Best for trusted agents in secure development environments
- **Global Auto-Accept**: Use only in isolated development environments

## Command Reference

| Command | Description | Auto-Accept |
|---------|-------------|-------------|
| `ci agent voice AGENT` | Launch agent in voice mode | Always enabled |
| `ci agent load AGENT --free` | Load agent with free mode | Enabled via flag |
| `ci agent load AGENT` | Load agent (check config) | Depends on config |
| `ci agent activate AGENT` | Activate agent | Depends on config |

## Troubleshooting

### Claude CLI Not Found

If you see "Claude CLI not found", install it with:
```bash
npm install -g @anthropic-ai/claude-cli
```

### Configuration Not Loading

Check configuration file location:
```bash
# The tool looks for .ci-config.json in:
ls ./.ci-config.json
ls ~/.ci-config.json
```

### Auto-Accept Not Working

1. Check configuration syntax is valid JSON
2. Verify agent name matches exactly (case-sensitive)
3. Use `--free` flag to override configuration

## Best Practices

1. **Use Voice Mode** for interactive development sessions
2. **Configure specific agents** rather than global auto-accept
3. **Keep configurations** in version control (except sensitive settings)
4. **Review configurations** regularly to ensure they match your security requirements
5. **Use project-specific** configurations in project directories