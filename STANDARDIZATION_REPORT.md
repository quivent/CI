# CI Standardization Report

Generated: 2025-06-04 01:23:41 UTC
Protocol Version: 1.0.0

## Violations Found

### agent_function_naming

- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:183**: Function 'get_agents_dir' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:208**: Function 'get_enabled_agents' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:220**: Function 'get_disabled_agents' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:232**: Function 'list_agents' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:371**: Function 'extract_agent_description' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:408**: Function 'show_agent_info' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:804**: Function 'load_agent_memory_from_ci' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:896**: Function 'display_loaded_agent_memory' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:927**: Function 'launch_claude_code_with_agent' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agents.rs:1204**: Function 'get_current_agent' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/visualize/agents.rs:9**: Function 'show_agents' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/visualize/agents.rs:60**: Function 'show_agent_network' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/visualize/agents.rs:73**: Function 'show_agent_categories' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:36**: Function 'is_agent_required' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:53**: Function 'parse_agent_config' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:140**: Function 'is_agent_active' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:173**: Function 'load_agent_capabilities' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:340**: Function 'set_agent_session_window_title' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:371**: Function 'update_agent_session_title' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:376**: Function 'restore_agent_session_title' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:396**: Function 'validate_agent_protocols' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:417**: Function 'test_agent_config_parsing' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_autoload.rs:440**: Function 'test_agent_activation_detection' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_colors.rs:11**: Function 'get_agent_color' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_colors.rs:73**: Function 'apply_agent_color' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_colors.rs:90**: Function 'update_current_agent_state' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_colors.rs:98**: Function 'get_current_agent' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_colors.rs:154**: Function 'get_agent_color_with_config' violates naming standard
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_colors.rs:171**: Function 'test_agent_color_mapping' violates naming standard

### error_handling

- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agent_integrate.rs:62**: Use CIError instead of anyhow::anyhow! for agent errors
- **/Users/joshkornreich/Documents/Projects/CI/src/commands/agent_integrate.rs:66**: Use CIError instead of anyhow::anyhow! for agent errors
- **/Users/joshkornreich/Documents/Projects/CI/src/helpers/agent_colors.rs:93**: Use CIError instead of anyhow::anyhow! for agent errors

## Standardization Protocol

```json
{
  "version": "1.0.0",
  "enforcement_level": "Error",
  "standards": {
    "error_handling": {
      "category": "Error Management",
      "description": "Use CIError for all agent-related errors with context",
      "required_pattern": "CIError::[A-Z][a-zA-Z]*\\(.*\\)\\.into\\(\\)",
      "examples": [
        "CIError::AgentNotFound(name.clone()).into()",
        "CIError::ActivationFailed(msg).into()"
      ],
      "violations": [
        "anyhow::anyhow!()",
        "panic!()"
      ],
      "enforcement": "Warning"
    },
    "agent_activation": {
      "category": "Agent Management",
      "description": "Use signature protocol detection for agent activation",
      "required_pattern": "\\[AGENT_NAME\\].*--\\s*\\[AGENT_NAME\\]",
      "examples": [
        "[ATHENA]: content -- [ATHENA]"
      ],
      "violations": [
        "@[AGENT_ACTIVATION:{}]"
      ],
      "enforcement": "Error"
    },
    "agent_function_naming": {
      "category": "Naming Conventions",
      "description": "All agent-related functions must use agent_ prefix",
      "required_pattern": "^agent_[a-z_]+$",
      "examples": [
        "agent_activate()",
        "agent_load()",
        "agent_configure()"
      ],
      "violations": [
        "enable_agent()",
        "activate_agent()",
        "loadAgent()"
      ],
      "enforcement": "Error"
    },
    "claude_md_generation": {
      "category": "Configuration",
      "description": "Use unified CLAUDE.md template with agent activation protocol",
      "required_pattern": "agent_activation_protocol_template",
      "examples": [
        "StandardizationEngine::generate_claude_md()"
      ],
      "violations": [
        "Multiple different CLAUDE.md formats"
      ],
      "enforcement": "Blocking"
    }
  },
  "global_policies": [
    {
      "name": "Agent Loading Policy",
      "description": "All agents must follow standardization protocols",
      "applies_to": [
        "Athena",
        "ProjectArchitect",
        "Standardist"
      ],
      "requirements": [
        "Read standardization protocols on initialization",
        "Validate implementations against standards",
        "Report violations immediately"
      ]
    }
  ],
  "validation_rules": [
    {
      "name": "Function Name Validation",
      "pattern": "fn\\s+(\\w+)\\(",
      "scope": "FunctionNames",
      "action": "Report"
    },
    {
      "name": "Error Pattern Validation",
      "pattern": "(anyhow::anyhow!|CIError::|panic!)",
      "scope": "ErrorHandling",
      "action": "Fix"
    }
  ]
}
```
