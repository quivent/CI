use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Configuration for agent activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActivationConfig {
    pub primary_agent: String,
    pub secondary_agents: Vec<String>,
    pub auto_activate: bool,
    pub signature_protocol_required: bool,
    pub memory_architecture_enabled: bool,
}

impl Default for AgentActivationConfig {
    fn default() -> Self {
        Self {
            primary_agent: "Athena".to_string(),
            secondary_agents: vec!["ProjectArchitect".to_string()],
            auto_activate: true,
            signature_protocol_required: true,
            memory_architecture_enabled: true,
        }
    }
}

/// Agent autoload system for ensuring persistent agent activation
pub struct AgentAutoload;

impl AgentAutoload {
    /// Check if the current project has CI integration with agent requirements
    pub fn is_agent_required(project_path: &Path) -> Result<bool> {
        let claude_md = project_path.join("CLAUDE.md");
        
        if !claude_md.exists() {
            return Ok(false);
        }
        
        let content = fs::read_to_string(&claude_md)
            .with_context(|| format!("Failed to read CLAUDE.md at {}", claude_md.display()))?;
        
        // Check for agent activation protocol markers
        Ok(content.contains("Agent Activation Protocol") ||
           content.contains("_CI.load_agents") ||
           content.contains("Available Agents"))
    }
    
    /// Parse agent configuration from CLAUDE.md
    pub fn parse_agent_config(project_path: &Path) -> Result<Option<AgentActivationConfig>> {
        let claude_md = project_path.join("CLAUDE.md");
        
        if !claude_md.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&claude_md)
            .with_context(|| format!("Failed to read CLAUDE.md at {}", claude_md.display()))?;
        
        let mut config = AgentActivationConfig::default();
        
        // Parse _CI.load_agents directive
        if let Some(agents_line) = content.lines().find(|line| line.contains("_CI.load_agents")) {
            if let Some(start) = agents_line.find('(') {
                if let Some(end) = agents_line[start..].find(')') {
                    let end = start + end;
                    let agents_str = &agents_line[start+1..end];
                    let agents_str = agents_str.trim_matches('\'').trim_matches('"');
                    let agents: Vec<String> = agents_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    
                    if !agents.is_empty() {
                        config.primary_agent = agents[0].clone();
                        config.secondary_agents = agents[1..].to_vec();
                    }
                }
            }
        }
        
        // Check for signature protocol requirement
        config.signature_protocol_required = content.contains("signature protocol") || 
                                            content.contains("ALWAYS implement agent signature protocol");
        
        // Check for auto-activation requirement
        config.auto_activate = content.contains("IMMEDIATELY activate") ||
                              content.contains("Auto-Activation Instructions");
        
        // Check for memory architecture requirement
        config.memory_architecture_enabled = content.contains("memory architecture") ||
                                            content.contains("Memory Systems Specialist");
        
        Ok(Some(config))
    }
    
    /// Generate agent activation instructions for Claude Code
    pub fn generate_activation_instructions(config: &AgentActivationConfig) -> String {
        let mut instructions = String::new();
        
        instructions.push_str("# Agent Activation Required\n\n");
        instructions.push_str("**CRITICAL**: This session requires immediate agent activation.\n\n");
        
        instructions.push_str(&format!("## Primary Agent: {}\n", config.primary_agent));
        
        if config.signature_protocol_required {
            instructions.push_str("### Signature Protocol\n");
            instructions.push_str("- **REQUIRED**: All responses must use agent signature format\n");
            instructions.push_str(&format!("- **Format**: [{}]: <response content> -- [{}]\n", 
                                         config.primary_agent, config.primary_agent));
            instructions.push_str("- **Consistency**: Maintain agent identity throughout session\n\n");
        }
        
        if config.memory_architecture_enabled {
            instructions.push_str("### Memory Architecture\n");
            instructions.push_str("- Apply agent-specific memory and learning principles\n");
            instructions.push_str("- Follow collaborative intelligence operational guidelines\n");
            instructions.push_str("- Implement knowledge organization frameworks\n\n");
        }
        
        if !config.secondary_agents.is_empty() {
            instructions.push_str("### Available Secondary Agents\n");
            for agent in &config.secondary_agents {
                instructions.push_str(&format!("- {}\n", agent));
            }
            instructions.push_str("\n");
        }
        
        instructions.push_str("### Activation Confirmation\n");
        instructions.push_str("Please confirm agent activation by responding with proper signature protocol.\n");
        
        instructions
    }
    
    /// Check if an agent is currently active (by detecting signature protocol in recent context)
    pub fn is_agent_active(recent_context: &str) -> bool {
        // Look for agent signature patterns in recent context
        let signature_patterns = [
            "[ATHENA]:",
            "[PROJECTARCHITECT]:",
            "-- [ATHENA]",
            "-- [PROJECTARCHITECT]",
        ];
        
        signature_patterns.iter().any(|pattern| recent_context.contains(pattern))
    }
    
    /// Get the path to CI repository for agent data
    pub fn get_ci_repository_path() -> Result<PathBuf> {
        // Try multiple possible locations for CI repository
        let possible_paths = vec![
            PathBuf::from("/Users/joshkornreich/Documents/Projects/CollaborativeIntelligence"),
            dirs::home_dir().map(|p| p.join("Documents/Projects/CollaborativeIntelligence")).unwrap_or_else(|| PathBuf::from("/tmp")),
            std::env::current_dir().map(|p| p.join("../CollaborativeIntelligence")).unwrap_or_else(|_| PathBuf::from("/tmp")),
        ];
        
        for path in possible_paths.iter() {
            if path.exists() && path.join("AGENTS").exists() {
                return Ok(path.clone());
            }
        }
        
        Err(crate::errors::CIError::Configuration(
            "Could not locate CollaborativeIntelligence repository".to_string()
        ).into())
    }
    
    /// Load agent memory and capabilities from CI repository
    pub fn load_agent_capabilities(agent_name: &str) -> Result<String> {
        let ci_repo = Self::get_ci_repository_path()?;
        let agent_dir = ci_repo.join("AGENTS").join(agent_name);
        
        if !agent_dir.exists() {
            return Err(crate::errors::CIError::NotFound(
                format!("Agent '{}' not found in CI repository", agent_name)
            ).into());
        }
        
        let mut capabilities = String::new();
        
        // Load README.md for basic capabilities
        let readme_path = agent_dir.join("README.md");
        if readme_path.exists() {
            let readme_content = fs::read_to_string(&readme_path)
                .with_context(|| format!("Failed to read agent README at {}", readme_path.display()))?;
            capabilities.push_str("# Agent Capabilities\n\n");
            capabilities.push_str(&readme_content);
            capabilities.push_str("\n\n");
        }
        
        // Load MEMORY.md for core identity
        let memory_path = agent_dir.join("MEMORY.md");
        if memory_path.exists() {
            let memory_content = fs::read_to_string(&memory_path)
                .with_context(|| format!("Failed to read agent memory at {}", memory_path.display()))?;
            capabilities.push_str("# Agent Memory\n\n");
            capabilities.push_str(&memory_content);
            capabilities.push_str("\n\n");
        }
        
        // Load ContinuousLearning.md for operational guidelines
        let learning_path = agent_dir.join("ContinuousLearning.md");
        if learning_path.exists() {
            let learning_content = fs::read_to_string(&learning_path)
                .with_context(|| format!("Failed to read agent learning at {}", learning_path.display()))?;
            capabilities.push_str("# Continuous Learning\n\n");
            capabilities.push_str(&learning_content);
        }
        
        Ok(capabilities)
    }
    
    /// Display agent activation notification to user
    pub fn display_activation_notification(config: &AgentActivationConfig) {
        // Set window title for agent session
        Self::set_agent_session_window_title(&config.primary_agent, "Activating");
        
        println!("{}", "🤖 Agent Activation Required".cyan().bold());
        println!("{}", "=" .repeat(50).cyan());
        println!();
        
        println!("{} {}", "Primary Agent:".bold(), config.primary_agent.green());
        
        if config.signature_protocol_required {
            println!("{} {}", "Signature Protocol:".bold(), "REQUIRED".red().bold());
        }
        
        if config.memory_architecture_enabled {
            println!("{} {}", "Memory Architecture:".bold(), "ENABLED".green());
        }
        
        if !config.secondary_agents.is_empty() {
            println!("{} {}", "Secondary Agents:".bold(), config.secondary_agents.join(", ").dimmed());
        }
        
        println!();
        println!("{}", "Agent should be automatically loaded from CLAUDE.md configuration.".yellow());
        println!("{}", "If no agent is active, please activate manually.".yellow());
    }
    
    /// Install agent activation hooks for CI integration
    pub fn install_activation_hooks(project_path: &Path) -> Result<()> {
        let config_dir = project_path.join(".ci");
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create .ci directory at {}", config_dir.display()))?;
        
        let hook_file = config_dir.join("agent-activation.json");
        let config = AgentActivationConfig::default();
        
        let config_json = serde_json::to_string_pretty(&config)
            .context("Failed to serialize agent activation config")?;
        
        fs::write(&hook_file, config_json)
            .with_context(|| format!("Failed to write agent activation config to {}", hook_file.display()))?;
        
        println!("{} {}", "✓".green().bold(), "Agent activation hooks installed".green());
        
        Ok(())
    }
    
    /// Generate unified CLAUDE.md configuration with agent activation protocol
    pub fn generate_unified_claude_md(
        project_name: &str,
        integration_type: &str,
        agents: &[String],
    ) -> String {
        let default_agent = "Athena".to_string();
        let primary_agent = agents.first().unwrap_or(&default_agent);
        let secondary_agents = if agents.len() > 1 { &agents[1..] } else { &[] };
        
        format!(
            r#"# Project: {}
# Created: {}
# Integration: {}

# Collaborative Intelligence Configuration
This project is configured to use the CI (Collaborative Intelligence) system.

_CI.config('project_name', '{}')_
_CI.config('created_at', '{}')_
_CI.config('integration_type', '{}')_

## Agent Activation Protocol
**CRITICAL**: This project MUST always have an active agent with signature protocol enabled.

### Auto-Activation Instructions
When any Claude Code session starts in this project:
1. **IMMEDIATELY** activate {} agent with full protocol compliance
2. **ALWAYS** implement agent signature protocol ([AGENT_NAME]: ... -- [AGENT_NAME])
3. **MAINTAIN** persistent agent identity throughout session
4. **FOLLOW** Collaborative Intelligence operational guidelines

### Available Agents
_CI.load_agents('{}')_

**Primary Agent**: {} (Knowledge Architect and Memory Systems Specialist)
- Memory architecture and knowledge organization
- System design and collaborative intelligence frameworks
- Repository management and session organization
- Performance optimization and efficient data structures

{}

## Agent Persistence Requirements
- Agent activation MUST persist for entire session duration
- Signature protocol MUST be consistently applied to all responses
- Agent memory and learning principles MUST guide all operations
- Collaborative Intelligence protocols MUST be followed

## Custom Instructions
- Focus on code quality and maintainability
- Follow established patterns in the codebase
- Add appropriate error handling for all edge cases
- Include helpful comments for complex sections
- **ALWAYS operate as an active Collaborative Intelligence agent**
- **NEVER respond without proper agent identification and signature protocol**
"#,
            project_name,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            integration_type,
            project_name,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            integration_type,
            primary_agent,
            agents.join(","),
            primary_agent,
            if !secondary_agents.is_empty() {
                format!("\n**Secondary Agents**: {}\n- Additional specialized capabilities as needed\n", secondary_agents.join(", "))
            } else {
                String::new()
            }
        )
    }
    
    /// Set window title for agent session
    pub fn set_agent_session_window_title(agent_name: &str, status: &str) {
        // Check if we're in a terminal that supports window titles
        let is_terminal = atty::is(atty::Stream::Stdout);
        let has_term_env = env::var("TERM").is_ok();
        let in_known_terminal = env::var("TERM_PROGRAM").is_ok() || 
                               env::var("ITERM_SESSION_ID").is_ok() ||
                               env::var("TERMINAL_EMULATOR").is_ok();
        let force_title = env::var("CI_FORCE_WINDOW_TITLE").unwrap_or_default() == "true";
        
        // Debug output for window title debugging
        if env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
            eprintln!("DEBUG: Setting agent session window title: '{}' - {}", agent_name, status);
            eprintln!("DEBUG: Terminal detection - is_terminal: {}, has_term_env: {}, in_known_terminal: {}", 
                     is_terminal, has_term_env, in_known_terminal);
        }
        
        // Set title if we're in a terminal that supports it
        if is_terminal || has_term_env || in_known_terminal || force_title {
            // OSC sequence to set window title: \x1b]0;title\x07
            print!("\x1b]0;CI Agent: {} - {}\x07", agent_name, status);
            let _ = io::stdout().flush();
            
            if env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
                eprintln!("DEBUG: Window title set to 'CI Agent: {} - {}'", agent_name, status);
            }
        } else if env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
            eprintln!("DEBUG: Skipping window title (no terminal detection)");
        }
    }
    
    /// Update agent session window title with progress
    pub fn update_agent_session_title(agent_name: &str, task: &str, progress: &str) {
        Self::set_agent_session_window_title(agent_name, &format!("{}: {}", task, progress));
    }
    
    /// Restore window title when agent session ends
    pub fn restore_agent_session_title() {
        let is_terminal = atty::is(atty::Stream::Stdout);
        let has_term_env = env::var("TERM").is_ok();
        let in_known_terminal = env::var("TERM_PROGRAM").is_ok() || 
                               env::var("ITERM_SESSION_ID").is_ok() ||
                               env::var("TERMINAL_EMULATOR").is_ok();
        let force_title = env::var("CI_FORCE_WINDOW_TITLE").unwrap_or_default() == "true";
        
        if is_terminal || has_term_env || in_known_terminal || force_title {
            // Restore to generic CI title or terminal default
            print!("\x1b]0;CI - Collaborative Intelligence\x07");
            let _ = io::stdout().flush();
            
            if env::var("CI_DEBUG_WINDOW_TITLE").is_ok() {
                eprintln!("DEBUG: Window title restored to default");
            }
        }
    }
    
    /// Validate that agent protocols are working correctly
    pub fn validate_agent_protocols(project_path: &Path) -> Result<bool> {
        let config = Self::parse_agent_config(project_path)?;
        
        if let Some(config) = config {
            if config.auto_activate && config.signature_protocol_required {
                println!("{} Agent protocols configured correctly", "✓".green().bold());
                return Ok(true);
            }
        }
        
        println!("{} Agent protocols need configuration", "⚠".yellow().bold());
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_agent_config_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let claude_md = temp_dir.path().join("CLAUDE.md");
        
        let content = r#"
# Test Project

_CI.load_agents('Athena,ProjectArchitect')_

## Agent Activation Protocol
IMMEDIATELY activate agents with signature protocol
"#;
        
        fs::write(&claude_md, content).unwrap();
        
        let config = AgentAutoload::parse_agent_config(temp_dir.path()).unwrap().unwrap();
        assert_eq!(config.primary_agent, "Athena");
        assert_eq!(config.secondary_agents, vec!["ProjectArchitect"]);
        assert!(config.auto_activate);
        assert!(config.signature_protocol_required);
    }
    
    #[test]
    fn test_agent_activation_detection() {
        let context_with_agent = "[ATHENA]: Working on the project -- [ATHENA]";
        let context_without_agent = "Just a regular response without agent signature";
        
        assert!(AgentAutoload::is_agent_active(context_with_agent));
        assert!(!AgentAutoload::is_agent_active(context_without_agent));
    }
}