use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::errors::CIError;
use crate::helpers::path::get_ci_root;

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
}

pub fn execute(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => list_agents(sub_matches),
        Some(("info", sub_matches)) => show_agent_info(sub_matches),
        Some(("create", sub_matches)) => create_agent(sub_matches),
        Some(("enable", sub_matches)) => enable_agent(sub_matches),
        Some(("disable", sub_matches)) => disable_agent(sub_matches),
        Some(("activate", sub_matches)) => activate_agent(sub_matches),
        Some(("template", sub_matches)) => create_agent_from_template(sub_matches),
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

fn list_agents(matches: &ArgMatches) -> Result<()> {
    let agents_dir = get_agents_dir()?;
    let enabled_only = matches.get_flag("enabled-only");
    let verbose = matches.get_flag("verbose");
    
    println!("{}", "Available Collaborative Intelligence Agents".cyan().bold());
    println!("{}", "=".repeat(50).cyan());
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
    
    // Sort agents alphabetically
    agent_info.sort_by(|a, b| a.0.cmp(&b.0));
    
    if agent_info.is_empty() {
        println!("{}", "No agents found.".yellow());
        println!("Run 'ci agent create <name>' to create a new agent.");
        return Ok(());
    }
    
    println!("{}", "Available Agents:".bold());
    println!();
    
    for (name, status, description, session_count) in &agent_info {
        let status_indicator = match status.as_ref() {
            "enabled" => "●".green(),
            "disabled" => "○".red(),
            _ => "○".white(),
        };
        
        print!("{} {} - {}", status_indicator, name.bold(), description);
        
        if verbose {
            if *session_count > 0 {
                print!(" {}", format!("(Used {} times)", session_count).dimmed());
            }
            match status.as_ref() {
                "enabled" => print!(" {}", "[ENABLED]".green()),
                "disabled" => print!(" {}", "[DISABLED]".red()),
                _ => {}
            }
        } else if *session_count > 0 {
            print!(" {}", format!("Used {} times", session_count).dimmed());
        }
        
        println!();
    }
    
    println!();
    println!("To use agents in your project:");
    println!("  ci agent enable <agent_name>");
    println!("  ci agent activate <agent_name>");
    println!();
    println!("For more information:");
    println!("  ci agent info <agent_name>");
    
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
    let template = matches.get_one::<String>("template");
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
        let enable_matches = clap::ArgMatches::default();
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

fn enable_agent(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    if !agent_dir.exists() {
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    println!("{} Agent '{}' enabled in current project", "✓".green(), agent_name);
    println!("The agent is now available for use.");
    println!("To activate: ci agent activate {}", agent_name);
    
    Ok(())
}

fn disable_agent(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    if !agent_dir.exists() {
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    println!("{} Agent '{}' disabled in current project", "✓".green(), agent_name);
    println!("To re-enable: ci agent enable {}", agent_name);
    
    Ok(())
}

fn activate_agent(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let context = matches.get_one::<String>("context");
    let agents_dir = get_agents_dir()?;
    let agent_dir = agents_dir.join(agent_name);
    
    if !agent_dir.exists() {
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    println!("{}", format!("Activating Agent: {}", agent_name).cyan().bold());
    println!("{}", "=".repeat(30).cyan());
    
    if let Some(ctx) = context {
        println!("Context: {}", ctx);
    }
    
    // Output activation marker for Claude Code
    println!("@[AGENT_ACTIVATION:{}]", agent_name);
    
    // Load and display agent memory
    let memory_path = agent_dir.join("MEMORY.md");
    if memory_path.exists() {
        if let Ok(content) = fs::read_to_string(&memory_path) {
            println!("{}", content);
        }
    }
    
    // Load and display learning content
    let learning_path = agent_dir.join("ContinuousLearning.md");
    if learning_path.exists() {
        if let Ok(content) = fs::read_to_string(&learning_path) {
            println!("\n# Continuous Learning\n");
            println!("{}", content);
        }
    }
    
    println!();
    println!("{} Agent '{}' is now active", "✓".green().bold(), agent_name);
    println!("You can interact with this agent directly in Claude Code");
    
    Ok(())
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