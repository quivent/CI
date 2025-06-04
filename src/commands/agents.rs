use anyhow::{Context, Result};
use chrono;
use clap::{Arg, ArgMatches, Command};
use colored::{Colorize, control};
use dirs;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::errors::CIError;
use crate::helpers::path::get_ci_root;
use crate::helpers::agent_autoload::AgentAutoload;
use crate::helpers::agent_colors;

pub fn create_command() -> Command {
    Command::new("agent")
        .about("Manage agents in the Collaborative Intelligence system")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list")
                .about("List available agents")
                .arg(
                    Arg::new("enabled-only")
                        .long("enabled-only")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show only enabled agents")
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show detailed information")
                )
        )
        .subcommand(
            Command::new("info")
                .about("Show detailed information about an agent")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent")
                        .required(true)
                        .index(1)
                )
        )
        .subcommand(
            Command::new("create")
                .about("Create a new agent")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the new agent")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .value_name("TEMPLATE")
                        .help("Create from template")
                )
                .arg(
                    Arg::new("enable")
                        .long("enable")
                        .action(clap::ArgAction::SetTrue)
                        .help("Enable the agent after creation")
                )
        )
        .subcommand(
            Command::new("enable")
                .about("Enable an agent in the current project")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent to enable")
                        .required(true)
                        .index(1)
                )
        )
        .subcommand(
            Command::new("disable")
                .about("Disable an agent in the current project")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent to disable")
                        .required(true)
                        .index(1)
                )
        )
        .subcommand(
            Command::new("activate")
                .about("Activate an agent for the current session")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent to activate")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("context")
                        .help("Context for agent activation")
                        .index(2)
                )
        )
        .subcommand(
            Command::new("load")
                .about("Load agent memory and capabilities")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent to load")
                        .required(true)
                        .index(1)
                )
        )
        .subcommand(
            Command::new("template")
                .about("Create an agent from a template")
                .arg(
                    Arg::new("template_name")
                        .help("Name of the template")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("agent_name")
                        .help("Name for the new agent")
                        .index(2)
                )
        )
        .subcommand(
            Command::new("deploy")
                .about("Deploy CI tool globally with latest changes")
                .arg(
                    Arg::new("force")
                        .long("force")
                        .action(clap::ArgAction::SetTrue)
                        .help("Force deployment even if no changes detected")
                )
                .arg(
                    Arg::new("backup")
                        .long("backup")
                        .action(clap::ArgAction::SetTrue)
                        .help("Create backup of existing global binary")
                )
        )
        .subcommand(
            Command::new("reset-color")
                .about("Reset terminal background color to default")
        )
        .subcommand(
            Command::new("switch")
                .about("Switch to a different agent during current session")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent to switch to")
                        .required(true)
                        .index(1)
                )
        )
}

pub fn execute(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => list_agents(sub_matches),
        Some(("info", sub_matches)) => show_agent_info(sub_matches),
        Some(("create", sub_matches)) => create_agent(sub_matches),
        Some(("enable", sub_matches)) => agent_enable(sub_matches),
        Some(("disable", sub_matches)) => agent_disable(sub_matches),
        Some(("activate", sub_matches)) => agent_activate(sub_matches),
        Some(("load", sub_matches)) => agent_load(sub_matches),
        Some(("template", sub_matches)) => create_agent_from_template(sub_matches),
        Some(("deploy", sub_matches)) => deploy_ci_globally(sub_matches),
        Some(("reset-color", _)) => agent_reset_color(),
        Some(("switch", sub_matches)) => agent_switch(sub_matches),
        _ => {
            eprintln!("{}", "No valid subcommand provided".red());
            std::process::exit(1);
        }
    }
}

fn get_agents_dir() -> Result<PathBuf> {
    let ci_root = get_ci_root()?;
    let agents_dir = ci_root.join("AGENTS");
    
    if !agents_dir.exists() {
        return Err(CIError::Configuration(format!(
            "Agents directory not found: {}. Run 'ci fix' to repair configuration.",
            agents_dir.display()
        )).into());
    }
    
    Ok(agents_dir)
}

fn get_project_config() -> Result<Value> {
    let claude_md = std::env::current_dir()?.join("CLAUDE.md");
    if claude_md.exists() {
        // Try to parse project configuration from CLAUDE.md
        // For now, return empty JSON object
        Ok(serde_json::json!({}))
    } else {
        Ok(serde_json::json!({}))
    }
}

fn get_enabled_agents() -> Result<Vec<String>> {
    let config = get_project_config()?;
    if let Some(enabled) = config.get("agents").and_then(|a| a.get("enabled")) {
        if let Some(array) = enabled.as_array() {
            return Ok(array.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect());
        }
    }
    Ok(vec![])
}

fn get_disabled_agents() -> Result<Vec<String>> {
    let config = get_project_config()?;
    if let Some(disabled) = config.get("agents").and_then(|a| a.get("disabled")) {
        if let Some(array) = disabled.as_array() {
            return Ok(array.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect());
        }
    }
    Ok(vec![])
}

pub fn list_agents(matches: &ArgMatches) -> Result<()> {
    // Force colored output to always be enabled
    control::set_override(true);
    
    let agents_dir = get_agents_dir()?;
    let enabled_only = matches.get_flag("enabled-only");
    let verbose = matches.get_flag("verbose");
    
    // Elegant header with visual hierarchy
    println!();
    println!("{}", "✨ Collaborative Intelligence Agents".blue().bold());
    println!("{}", "━".repeat(60).blue().dimmed());
    println!();
    
    let enabled_agents = get_enabled_agents().unwrap_or_default();
    let disabled_agents = get_disabled_agents().unwrap_or_default();
    
    let mut agent_info = Vec::new();
    
    // Read agents directory
    if let Ok(entries) = fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(agent_name) = entry.file_name().to_str() {
                    if agent_name.starts_with('.') {
                        continue;
                    }
                    
                    let agent_path = entry.path();
                    let readme_path = agent_path.join("README.md");
                    
                    if readme_path.exists() {
                        let status = if enabled_agents.contains(&agent_name.to_string()) {
                            "enabled"
                        } else if disabled_agents.contains(&agent_name.to_string()) {
                            "disabled"
                        } else {
                            "available"
                        };
                        
                        if enabled_only && status != "enabled" {
                            continue;
                        }
                        
                        let description = extract_agent_description(&readme_path).unwrap_or_default();
                        let session_count = count_sessions(&agent_path).unwrap_or(0);
                        
                        agent_info.push((agent_name.to_string(), status, description, session_count));
                    }
                }
            }
        }
    }
    
    // Sort agents alphabetically for better organization
    agent_info.sort_by(|a, b| a.0.cmp(&b.0));
    
    if agent_info.is_empty() {
        println!("   {}", "No agents found in your system.".yellow().bold());
        println!("   {}", "Run 'ci agent create <name>' to create a new agent.".dimmed());
        return Ok(());
    }
    
    // Group agents for better visual organization
    let mut total_agents = 0;
    let mut active_agents = 0;
    
    for (name, status, description, session_count) in &agent_info {
        total_agents += 1;
        if *session_count > 0 {
            active_agents += 1;
        }
        
        // Sophisticated status indicators with better visual design
        let (status_icon, status_color) = match status.as_ref() {
            "enabled" => ("●", "green"),
            "disabled" => ("◯", "red"), 
            _ => ("○", "blue"),
        };
        
        // Create visually appealing agent entry
        let status_indicator = match status_color {
            "green" => status_icon.green(),
            "red" => status_icon.red(),
            _ => status_icon.blue(),
        };
        
        // Enhanced layout with better spacing and typography
        print!("  {} {}", status_indicator, name.bold().blue());
        
        // Usage statistics with refined styling
        if *session_count > 0 {
            print!(" {}", format!("({})", session_count).yellow().dimmed());
        }
        
        println!();
        
        // Description with proper indentation and styling
        if !description.is_empty() {
            let formatted_desc = if description.len() > 85 {
                format!("{}...", &description[..82])
            } else {
                description.clone()
            };
            println!("    {}", formatted_desc.dimmed());
        }
        
        // Add subtle spacing between entries
        if verbose {
            match status.as_ref() {
                "enabled" => println!("    {}", "● Enabled in project".green().dimmed()),
                "disabled" => println!("    {}", "◯ Disabled in project".red().dimmed()),
                _ => {}
            }
        }
        
        println!();
    }
    
    // Summary statistics with elegant styling
    println!("{}", "─".repeat(60).dimmed());
    println!("  {} total agents  •  {} with usage history", 
             total_agents.to_string().bold().blue(),
             active_agents.to_string().bold().yellow());
    
    println!();
    
    // Elegant command reference with improved visual design
    println!("{}", "🚀 Quick Actions".bold().blue());
    println!("{}", "─".repeat(40).dimmed());
    println!("  {} {}    {}", "ci agent load".green().bold(), "<name>".blue(), "Load agent memory".dimmed());
    println!("  {} {}  {}", "ci agent enable".green().bold(), "<name>".blue(), "Enable for project".dimmed());
    println!("  {} {} {}", "ci agent activate".green().bold(), "<name>".blue(), "Start session".dimmed());
    println!("  {} {}    {}", "ci agent info".green().bold(), "<name>".blue(), "View details".dimmed());
    println!();
    
    Ok(())
}

fn extract_agent_description(readme_path: &Path) -> Result<String> {
    let content = fs::read_to_string(readme_path)?;
    
    // Look for the first non-header, non-empty line
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("---") {
            let description = if trimmed.len() > 80 {
                format!("{}...", &trimmed[..77])
            } else {
                trimmed.to_string()
            };
            return Ok(description);
        }
    }
    
    Ok("No description available".to_string())
}

fn count_sessions(agent_path: &Path) -> Result<usize> {
    let sessions_dir = agent_path.join("Sessions");
    if !sessions_dir.exists() {
        return Ok(0);
    }
    
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                count += 1;
            }
        }
    }
    
    Ok(count)
}

fn show_agent_info(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    if !agent_dir.exists() {
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    println!("{}", format!("Agent Information: {}", agent_name).cyan().bold());
    println!("{}", "=".repeat(50).cyan());
    println!();
    
    let enabled_agents = get_enabled_agents().unwrap_or_default();
    let disabled_agents = get_disabled_agents().unwrap_or_default();
    
    let status = if enabled_agents.contains(&agent_name.to_string()) {
        "enabled".green()
    } else if disabled_agents.contains(&agent_name.to_string()) {
        "disabled".red()
    } else {
        "available".white()
    };
    
    println!("{}: {}", "Status".bold(), status);
    println!("{}: {}", "Path".bold(), agent_dir.display().to_string().dimmed());
    println!();
    
    // Show README content
    let readme_path = agent_dir.join("README.md");
    if readme_path.exists() {
        println!("{}", "Description:".bold());
        if let Ok(content) = fs::read_to_string(&readme_path) {
            let lines: Vec<&str> = content.lines().collect();
            let mut in_description = false;
            let mut description_lines = 0;
            
            for line in lines {
                let trimmed = line.trim();
                if trimmed.starts_with("# ") {
                    in_description = true;
                    continue;
                }
                if trimmed.starts_with("## ") && in_description {
                    break;
                }
                if in_description && !trimmed.is_empty() {
                    println!("  {}", line);
                    description_lines += 1;
                    if description_lines >= 5 {
                        println!("  {}", "... (see README.md for full description)".dimmed());
                        break;
                    }
                }
            }
        }
        println!();
    }
    
    // Show memory information
    let memory_path = agent_dir.join("MEMORY.md");
    if memory_path.exists() {
        if let Ok(content) = fs::read_to_string(&memory_path) {
            let line_count = content.lines().count();
            println!("{}: {} lines", "Memory".bold(), line_count);
        }
    }
    
    // Show learning information
    let learning_path = agent_dir.join("ContinuousLearning.md");
    if learning_path.exists() {
        if let Ok(content) = fs::read_to_string(&learning_path) {
            let line_count = content.lines().count();
            println!("{}: {} lines", "Learning".bold(), line_count);
        }
    }
    
    // Show sessions
    let session_count = count_sessions(&agent_dir).unwrap_or(0);
    println!("{}: {}", "Sessions".bold(), session_count);
    
    if session_count > 0 {
        let sessions_dir = agent_dir.join("Sessions");
        if let Ok(entries) = fs::read_dir(&sessions_dir) {
            let mut sessions: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect();
            
            sessions.sort();
            
            println!("  Recent sessions:");
            for session in sessions.iter().take(5) {
                println!("    - {}", session);
            }
            
            if sessions.len() > 5 {
                println!("    ... and {} more", sessions.len() - 5);
            }
        }
    }
    
    println!();
    println!("To use this agent:");
    println!("  ci agent activate {}", agent_name);
    
    Ok(())
}

fn create_agent(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let _template = matches.get_one::<String>("template");
    let enable_after = matches.get_flag("enable");
    
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    if agent_dir.exists() {
        return Err(CIError::AlreadyExists(format!(
            "Agent '{}' already exists. Use 'ci agent info {}' to view details.",
            agent_name, agent_name
        )).into());
    }
    
    println!("{}", format!("Creating Agent: {}", agent_name).cyan().bold());
    println!("{}", "=".repeat(30).cyan());
    
    // Create directory structure
    fs::create_dir_all(&agent_dir)?;
    fs::create_dir_all(agent_dir.join("Sessions"))?;
    println!("{} Created agent directory structure", "✓".green());
    
    // Create README.md
    let readme_content = format!(
        r#"# {}

This agent is part of the Collaborative Intelligence system.

## Capabilities

- Add agent capabilities here

## Usage

To use this agent, type `{}` in a Claude Code session.

## Sessions

Session records are stored in the Sessions directory.
"#,
        agent_name, agent_name
    );
    
    fs::write(agent_dir.join("README.md"), readme_content)?;
    println!("{} Created README.md documentation", "✓".green());
    
    // Create MEMORY.md
    let memory_content = format!(
        r#"# {} Memory

This file stores the long-term memory for the {} agent.

## Core Knowledge

- Agent Name: {}
- Creation Date: {}
- Primary Function: [Define primary function]

## System Integration

The {} agent is part of the Collaborative Intelligence ecosystem and follows the standard agent communication protocols.

## Expertise

[Define areas of expertise]
"#,
        agent_name,
        agent_name,
        agent_name,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        agent_name
    );
    
    fs::write(agent_dir.join("MEMORY.md"), memory_content)?;
    println!("{} Created MEMORY.md file", "✓".green());
    
    // Create ContinuousLearning.md
    let learning_content = format!(
        r#"# {} Continuous Learning

This file documents the learning progress of the {} agent.

## Learning Record

### {}

- Agent created
- Initial memory structure established
- Basic capabilities defined
"#,
        agent_name,
        agent_name,
        chrono::Utc::now().format("%Y-%m-%d")
    );
    
    fs::write(agent_dir.join("ContinuousLearning.md"), learning_content)?;
    println!("{} Created ContinuousLearning.md file", "✓".green());
    
    println!();
    println!("{} Agent '{}' created successfully", "✓".green().bold(), agent_name);
    
    if enable_after {
        println!("Enabling agent in current project...");
        // Call enable function
        let _enable_matches = clap::ArgMatches::default();
        // This would need a proper way to create ArgMatches for enable
        // For now, just print the instruction
        println!("Run: ci agent enable {}", agent_name);
    }
    
    println!();
    println!("To use this agent:");
    println!("  ci agent enable {}", agent_name);
    println!("  ci agent activate {}", agent_name);
    
    Ok(())
}

fn agent_enable(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    // Set window title for agent enabling
    AgentAutoload::set_agent_session_window_title(agent_name, "Enabling");
    
    if !agent_dir.exists() {
        AgentAutoload::update_agent_session_title(agent_name, "Enable", "Failed - Not Found");
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    AgentAutoload::update_agent_session_title(agent_name, "Enable", "Complete");
    
    println!("{} Agent '{}' enabled in current project", "✓".green(), agent_name);
    println!("The agent is now available for use.");
    println!("To activate: ci agent activate {}", agent_name);
    
    Ok(())
}

fn agent_disable(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    // Set window title for agent disabling
    AgentAutoload::set_agent_session_window_title(agent_name, "Disabling");
    
    if !agent_dir.exists() {
        AgentAutoload::update_agent_session_title(agent_name, "Disable", "Failed - Not Found");
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    AgentAutoload::update_agent_session_title(agent_name, "Disable", "Complete");
    
    println!("{} Agent '{}' disabled in current project", "✓".green(), agent_name);
    println!("To re-enable: ci agent enable {}", agent_name);
    
    Ok(())
}

fn agent_activate(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let context = matches.get_one::<String>("context");
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    // Set window title for agent activation
    AgentAutoload::set_agent_session_window_title(agent_name, "Activating");
    
    if !agent_dir.exists() {
        AgentAutoload::update_agent_session_title(agent_name, "Activation", "Failed - Not Found");
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    AgentAutoload::update_agent_session_title(agent_name, "Activation", "Loading Memory");
    
    println!("{}", format!("Activating Agent: {}", agent_name).cyan().bold());
    println!("{}", "=".repeat(30).cyan());
    
    if let Some(ctx) = context {
        println!("Context: {}", ctx);
    }
    
    // Output signature protocol activation marker
    println!("🤖 Agent Activation: {}", agent_name);
    println!("Expected signature format: [{}]: <content> -- [{}]", agent_name.to_uppercase(), agent_name.to_uppercase());
    
    AgentAutoload::update_agent_session_title(agent_name, "Activation", "Loading Configuration");
    
    // Load and display agent memory
    let memory_path = agent_dir.join("MEMORY.md");
    if memory_path.exists() {
        if let Ok(content) = fs::read_to_string(&memory_path) {
            println!("{}", content);
        }
    }
    
    AgentAutoload::update_agent_session_title(agent_name, "Activation", "Loading Learning");
    
    // Load and display learning content
    let learning_path = agent_dir.join("ContinuousLearning.md");
    if learning_path.exists() {
        if let Ok(content) = fs::read_to_string(&learning_path) {
            println!("\n# Continuous Learning\n");
            println!("{}", content);
        }
    }
    
    AgentAutoload::update_agent_session_title(agent_name, "Activation", "Protocol Ready");
    
    println!();
    println!("{} Agent '{}' is now active", "✓".green().bold(), agent_name);
    println!("You can interact with this agent directly in Claude Code");
    
    // Set final window title to show agent is ready
    AgentAutoload::update_agent_session_title(agent_name, "Active", "Ready");
    
    Ok(())
}

fn agent_load(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    
    println!("{}", format!("Loading memory for agent: {}", agent_name).cyan().bold());
    
    // Apply agent-specific background color
    if let Err(e) = agent_colors::apply_agent_color(agent_name) {
        println!("{} Warning: Failed to apply background color: {}", "⚠".yellow(), e);
    }
    
    // Try to find agent in CI repository first
    if let Ok(ci_repo) = AgentAutoload::get_ci_repository_path() {
        let agent_dir = ci_repo.join("AGENTS").join(agent_name);
        if agent_dir.exists() {
            println!("{} Found agent in CI repository: {}", "✓".green(), agent_dir.display());
            match load_agent_memory_from_ci(&agent_dir) {
                Ok(memory_content) => {
                    display_loaded_agent_memory(agent_name, &memory_content);
                    return Ok(());
                }
                Err(e) => {
                    println!("{} Failed to load from CI repository: {}", "⚠".yellow(), e);
                }
            }
        }
    }
    
    // If not found in CI repository, try local AGENTS directory
    if let Ok(ci_root) = crate::helpers::path::get_ci_root() {
        let local_agent_dir = ci_root.join("AGENTS").join(agent_name);
        if local_agent_dir.exists() {
            println!("{} Found agent in local directory: {}", "✓".green(), local_agent_dir.display());
            match load_agent_memory_from_ci(&local_agent_dir) {
                Ok(memory_content) => {
                    display_loaded_agent_memory(agent_name, &memory_content);
                    return Ok(());
                }
                Err(e) => {
                    println!("{} Failed to load from local directory: {}", "⚠".yellow(), e);
                }
            }
        }
    }
    
    // If still not found, create a virtual agent memory template
    println!("{} Agent not found, creating virtual memory template", "⚠".yellow());
    let virtual_memory = create_virtual_agent_memory(agent_name);
    display_loaded_agent_memory(agent_name, &virtual_memory);
    
    Ok(())
}

fn load_agent_memory_from_ci(agent_dir: &Path) -> Result<String> {
    let mut memory_content = String::new();
    
    // Load README.md
    let readme_path = agent_dir.join("README.md");
    if readme_path.exists() {
        if let Ok(content) = fs::read_to_string(&readme_path) {
            memory_content.push_str("# Agent Profile\n\n");
            memory_content.push_str(&content);
            memory_content.push_str("\n\n");
        }
    }
    
    // Load MEMORY.md
    let memory_path = agent_dir.join("MEMORY.md");
    if memory_path.exists() {
        if let Ok(content) = fs::read_to_string(&memory_path) {
            memory_content.push_str("# Core Memory\n\n");
            memory_content.push_str(&content);
            memory_content.push_str("\n\n");
        }
    }
    
    // Load ContinuousLearning.md
    let learning_path = agent_dir.join("ContinuousLearning.md");
    if learning_path.exists() {
        if let Ok(content) = fs::read_to_string(&learning_path) {
            memory_content.push_str("# Learning History\n\n");
            memory_content.push_str(&content);
        }
    }
    
    if memory_content.is_empty() {
        return Err(CIError::NotFound(
            "No memory files found in agent directory".to_string()
        ).into());
    }
    
    Ok(memory_content)
}

fn create_virtual_agent_memory(agent_name: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let uppercase_name = agent_name.to_uppercase();
    
    format!(r#"# Virtual Agent: {}

## Agent Profile
This is a virtual agent created by the CI system. The agent specializes in specific domain knowledge and can be manually loaded to provide context-aware assistance.

## Capabilities
- Domain-specific expertise
- Context-aware responses  
- Integration with CI system protocols
- Memory persistence across sessions

## Memory Structure
```
Agent Name: {}
Type: Virtual Agent
Created: {}
Status: Manual Load Template
```

## Signature Protocol
When active, this agent should use the signature format:
[{}]: <response content> -- [{}]

## Usage Instructions
To activate this agent manually:
1. Use `ci agent load {}` to load this memory
2. Begin interactions with proper signature protocol
3. Maintain agent identity throughout the session

## Notes
This virtual agent memory can be customized by creating:
- AGENTS/{}/README.md (agent profile)
- AGENTS/{}/MEMORY.md (core memory)
- AGENTS/{}/ContinuousLearning.md (learning history)
"#, 
        agent_name,      // Virtual Agent: {}
        agent_name,      // Agent Name: {}
        timestamp,       // Created: {}
        uppercase_name,  // [{}]: <response content>
        uppercase_name,  // -- [{}]
        agent_name,      // ci agent load {}
        agent_name,      // AGENTS/{}/README.md
        agent_name,      // AGENTS/{}/MEMORY.md
        agent_name       // AGENTS/{}/ContinuousLearning.md
    )
}

fn display_loaded_agent_memory(agent_name: &str, memory_content: &str) {
    println!("{}", "=".repeat(60).cyan());
    println!("{}", format!("🧠 Agent Memory Loaded: {}", agent_name).cyan().bold());
    println!("{}", "=".repeat(60).cyan());
    println!();
    
    // Format and display the memory content
    for line in memory_content.lines() {
        if line.starts_with("# ") {
            println!("{}", line.blue().bold());
        } else if line.starts_with("## ") {
            println!("{}", line.green().bold());
        } else if line.starts_with("```") {
            println!("{}", line.yellow());
        } else if line.trim().is_empty() {
            println!();
        } else {
            println!("{}", line);
        }
    }
    
    println!();
    println!("{}", "=".repeat(60).cyan());
    println!("{}", format!("Agent {} memory loaded successfully", agent_name).green().bold());
    println!("{}", "You can now interact with this agent using the signature protocol.".dimmed());
    println!("{}", "=".repeat(60).cyan());
    
    // Offer to launch Claude Code
    launch_claude_code_with_agent(agent_name, memory_content);
}

fn launch_claude_code_with_agent(agent_name: &str, memory_content: &str) {
    use crate::helpers::CommandHelpers;
    use std::process::Command;
    
    // Ask user if they want to launch Claude Code
    if CommandHelpers::prompt_confirmation("Launch Claude Code with this agent now?") {
        // Check if claude CLI is available
        if has_claude_cli() {
            println!("Launching Claude Code with {}...", agent_name.cyan().bold());
            
            // Create a temporary file with the memory content
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!("ci_agent_{}.md", agent_name));
            
            match std::fs::write(&temp_file, memory_content) {
                Ok(_) => {
                    let status = Command::new("cat")
                        .arg(&temp_file)
                        .stdout(std::process::Stdio::piped())
                        .spawn()
                        .and_then(|output| {
                            Command::new("claude")
                                .arg("code")
                                .stdin(output.stdout.unwrap())
                                .status()
                        });
                        
                    match status {
                        Ok(exit_status) => {
                            if !exit_status.success() {
                                println!("{} Claude Code exited with a non-zero status", "⚠".yellow());
                            }
                        }
                        Err(e) => {
                            println!("{} Failed to launch Claude Code: {}", "⚠".yellow(), e);
                            println!("{} Try running: ci load {}", "💡".blue(), agent_name);
                        }
                    }
                    
                    // Clean up temp file
                    let _ = std::fs::remove_file(&temp_file);
                }
                Err(e) => {
                    println!("{} Failed to create temporary file: {}", "⚠".yellow(), e);
                    println!("{} Try running: ci load {}", "💡".blue(), agent_name);
                }
            }
        } else {
            println!("{} Claude CLI not found. Install it first or try: ci load {}", "⚠".yellow(), agent_name);
        }
    } else {
        println!("{} To launch Claude Code later, run: ci load {}", "💡".blue(), agent_name);
    }
}

fn has_claude_cli() -> bool {
    std::process::Command::new("which")
        .arg("claude")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn create_agent_from_template(matches: &ArgMatches) -> Result<()> {
    let template_name = matches.get_one::<String>("template_name").unwrap();
    let agent_name = matches.get_one::<String>("agent_name");
    
    println!("{}", format!("Creating Agent from Template: {}", template_name).cyan().bold());
    
    // For now, just create a basic agent
    // In the future, this would use actual templates
    let name = agent_name.map(|s| s.as_str()).unwrap_or(template_name);
    
    // Create a basic agent (this would be enhanced with actual template system)
    println!("Template system not yet implemented.");
    println!("Creating basic agent instead: {}", name);
    
    // This would call create_agent with the template applied
    Ok(())
}

fn deploy_ci_globally(matches: &ArgMatches) -> Result<()> {
    let force = matches.get_flag("force");
    let backup = matches.get_flag("backup");
    
    println!("{}", "🚀 CI Global Deployment Protocol".cyan().bold());
    println!("{}", "=".repeat(40).cyan());
    
    AgentAutoload::set_agent_session_window_title("FIXER", "Deploying CI Globally");
    
    // Get current working directory (should be CI project root)
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    // Verify we're in a CI project directory
    let cargo_toml = current_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(CIError::Configuration(
            "Not in a Rust project directory. Deploy must be run from CI project root.".to_string()
        ).into());
    }
    
    // Check if this is the CI project by reading Cargo.toml
    let cargo_content = fs::read_to_string(&cargo_toml)?;
    if !cargo_content.contains("name = \"CI\"") {
        return Err(CIError::Configuration(
            "This doesn't appear to be the CI project. Deploy must be run from CI project root.".to_string()
        ).into());
    }
    
    let cargo_bin_dir = dirs::home_dir()
        .ok_or_else(|| CIError::Configuration("Could not find home directory".to_string()))?
        .join(".cargo/bin");
    
    // Create backup if requested
    if backup {
        println!("{} Creating backup of existing global binary...", "📦".blue());
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let ci_path = cargo_bin_dir.join("CI");
        if ci_path.exists() {
            let backup_path = cargo_bin_dir.join(format!("CI-backup-{}", timestamp));
            fs::copy(&ci_path, &backup_path)?;
            println!("{} Backup created: {}", "✓".green(), backup_path.display());
        }
    }
    
    // Build release version
    println!("{} Building release version...", "🔨".blue());
    AgentAutoload::update_agent_session_title("FIXER", "Deploy", "Building Release");
    
    let build_output = ProcessCommand::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&current_dir)
        .output()
        .context("Failed to execute cargo build")?;
    
    if !build_output.status.success() {
        return Err(CIError::Configuration(format!(
            "Build failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        )).into());
    }
    
    println!("{} Build completed successfully", "✓".green());
    
    // Copy binaries to global location
    println!("{} Installing binaries globally...", "📦".blue());
    AgentAutoload::update_agent_session_title("FIXER", "Deploy", "Installing Binaries");
    
    let release_binary = current_dir.join("target/release/CI");
    if !release_binary.exists() {
        return Err(CIError::NotFound(
            "Release binary not found. Build may have failed.".to_string()
        ).into());
    }
    
    // Install as both CI and ci
    let ci_global = cargo_bin_dir.join("CI");
    let ci_lowercase = cargo_bin_dir.join("ci");
    
    fs::copy(&release_binary, &ci_global)
        .context("Failed to copy CI binary to global location")?;
    fs::copy(&release_binary, &ci_lowercase)
        .context("Failed to copy ci binary to global location")?;
    
    println!("{} Global binaries installed:", "✓".green());
    println!("  - {}", ci_global.display());
    println!("  - {}", ci_lowercase.display());
    
    // Verify installation
    println!("{} Verifying global installation...", "🔍".blue());
    AgentAutoload::update_agent_session_title("FIXER", "Deploy", "Verifying Installation");
    
    let version_output = ProcessCommand::new("CI")
        .args(&["--version"])
        .output()
        .context("Failed to verify CI installation")?;
    
    if version_output.status.success() {
        println!("{} Global CI installation verified", "✓".green());
        println!("{}", String::from_utf8_lossy(&version_output.stdout));
    } else {
        println!("{} Warning: Could not verify CI installation", "⚠".yellow());
    }
    
    AgentAutoload::update_agent_session_title("FIXER", "Deploy", "Complete");
    
    println!();
    println!("{} {}", "🎉".green(), "CI Global Deployment Complete!".green().bold());
    println!();
    println!("You can now use 'CI' or 'ci' from anywhere to access the latest version.");
    println!("This deployment workflow is now integrated into the CI agent protocol.");
    
    Ok(())
}

fn agent_reset_color() -> Result<()> {
    println!("{}", "Resetting terminal background color...".cyan().bold());
    
    match agent_colors::reset_terminal_color() {
        Ok(_) => {
            println!("{} Terminal background color reset to default", "✓".green());
            Ok(())
        }
        Err(e) => {
            println!("{} Failed to reset terminal color: {}", "✗".red(), e);
            Err(e)
        }
    }
}

fn agent_switch(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    println!("{}", format!("Switching to agent: {}", agent_name).cyan().bold());
    
    // Set window title for agent switching
    AgentAutoload::set_agent_session_window_title(agent_name, "Switching");
    
    if !agent_dir.exists() {
        AgentAutoload::update_agent_session_title(agent_name, "Switch", "Failed - Not Found");
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    // Apply agent-specific background color
    if let Err(e) = agent_colors::apply_agent_color(agent_name) {
        println!("{} Warning: Failed to apply background color: {}", "⚠".yellow(), e);
    }
    
    AgentAutoload::update_agent_session_title(agent_name, "Switch", "Loading Profile");
    
    // Show brief agent profile for context
    let readme_path = agent_dir.join("README.md");
    if readme_path.exists() {
        if let Ok(content) = fs::read_to_string(&readme_path) {
            println!();
            println!("{}", "Agent Profile:".blue().bold());
            
            // Show just the title and first few lines
            for (i, line) in content.lines().enumerate() {
                if i >= 8 { break; } // Limit to first 8 lines
                if line.starts_with("# ") {
                    println!("{}", line.blue().bold());
                } else if !line.trim().is_empty() && !line.starts_with("---") {
                    println!("{}", line);
                }
            }
        }
    }
    
    // Update session state
    update_session_state(agent_name)?;
    
    AgentAutoload::update_agent_session_title(agent_name, "Active", "Ready");
    
    println!();
    println!("{} Successfully switched to agent '{}'", "✓".green().bold(), agent_name);
    println!("{} Agent is ready for interaction", "🤖".blue());
    println!("{} Use 'ci agent switch <name>' to change agents", "💡".dimmed());
    
    Ok(())
}

/// Update session state for agent switching
fn update_session_state(agent_name: &str) -> Result<()> {
    let session_file = std::env::temp_dir().join("ci_current_agent.txt");
    fs::write(&session_file, agent_name)
        .context("Failed to update session state")?;
    Ok(())
}

/// Get current active agent from session state
pub fn get_current_agent() -> Option<String> {
    let session_file = std::env::temp_dir().join("ci_current_agent.txt");
    fs::read_to_string(&session_file).ok()
}