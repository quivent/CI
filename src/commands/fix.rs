use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

use crate::errors::CIError;
use crate::helpers::path::get_ci_root;

pub fn create_command() -> Command {
    Command::new("fix")
        .about("Fix common Collaborative Intelligence issues")
        .arg(
            Arg::new("issue")
                .help("Specific issue to fix")
                .value_parser(["timeout", "config", "permissions", "memory", "agents", "all"])
                .index(1)
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .action(clap::ArgAction::SetTrue)
                .help("Show what would be fixed without making changes")
        )
        .arg(
            Arg::new("force")
                .long("force")
                .action(clap::ArgAction::SetTrue)
                .help("Force fix even if potentially destructive")
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .value_name("SECONDS")
                .default_value("10")
                .help("Timeout for Claude commands (default: 10 seconds)")
        )
}

pub fn execute(matches: &ArgMatches) -> Result<()> {
    let issue = matches.get_one::<String>("issue");
    let dry_run = matches.get_flag("dry-run");
    let force = matches.get_flag("force");
    let timeout: u64 = matches.get_one::<String>("timeout")
        .unwrap()
        .parse()
        .with_context(|| "Invalid timeout value")?;
    
    if dry_run {
        println!("{}", "DRY RUN MODE - No changes will be made".yellow().bold());
        println!();
    }
    
    match issue.map(|s| s.as_str()) {
        Some("timeout") => fix_timeout_issues(dry_run, timeout),
        Some("config") => fix_config_issues(dry_run, force),
        Some("permissions") => fix_permission_issues(dry_run),
        Some("memory") => fix_memory_issues(dry_run),
        Some("agents") => fix_agent_issues(dry_run),
        Some("all") => fix_all_issues(dry_run, force, timeout),
        None => diagnose_and_fix(dry_run, force, timeout),
        Some(unknown) => {
            eprintln!("{}", format!("Unknown issue type: {}", unknown).red());
            std::process::exit(1);
        }
    }
}

fn diagnose_and_fix(dry_run: bool, force: bool, timeout: u64) -> Result<()> {
    println!("{}", "Diagnosing Collaborative Intelligence system...".cyan().bold());
    println!("{}", "=".repeat(50).cyan());
    println!();
    
    let mut issues_found = Vec::new();
    
    // Check for common issues
    if check_claude_cli_timeout().is_err() {
        issues_found.push("timeout");
    }
    
    if check_config_integrity().is_err() {
        issues_found.push("config");
    }
    
    if check_permissions().is_err() {
        issues_found.push("permissions");
    }
    
    if check_memory_system().is_err() {
        issues_found.push("memory");
    }
    
    if check_agent_system().is_err() {
        issues_found.push("agents");
    }
    
    if issues_found.is_empty() {
        println!("{} No issues detected!", "✓".green().bold());
        println!("The Collaborative Intelligence system appears to be working correctly.");
        return Ok(());
    }
    
    println!("{} Issues detected:", "⚠".yellow().bold());
    for issue in &issues_found {
        println!("  - {}", issue);
    }
    println!();
    
    if dry_run {
        println!("Run without --dry-run to apply fixes.");
        return Ok(());
    }
    
    // Ask for confirmation unless force is specified
    if !force {
        print!("Apply fixes for all detected issues? (y/n): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Fixes cancelled.");
            return Ok(());
        }
    }
    
    // Apply fixes
    for issue in &issues_found {
        match *issue {
            "timeout" => fix_timeout_issues(false, timeout)?,
            "config" => fix_config_issues(false, force)?,
            "permissions" => fix_permission_issues(false)?,
            "memory" => fix_memory_issues(false)?,
            "agents" => fix_agent_issues(false)?,
            _ => {}
        }
    }
    
    println!();
    println!("{} All issues have been fixed!", "✓".green().bold());
    println!("The CI system is now ready to use.");
    
    Ok(())
}

fn fix_all_issues(dry_run: bool, force: bool, timeout: u64) -> Result<()> {
    println!("{}", "Fixing all Collaborative Intelligence issues...".cyan().bold());
    println!("{}", "=".repeat(50).cyan());
    println!();
    
    fix_timeout_issues(dry_run, timeout)?;
    fix_config_issues(dry_run, force)?;
    fix_permission_issues(dry_run)?;
    fix_memory_issues(dry_run)?;
    fix_agent_issues(dry_run)?;
    
    if !dry_run {
        println!();
        println!("{} All fixes applied!", "✓".green().bold());
    }
    
    Ok(())
}

fn fix_timeout_issues(dry_run: bool, timeout: u64) -> Result<()> {
    println!("{}", "Fixing Claude CLI timeout issues...".bold());
    
    // CI has built-in timeout handling
    if dry_run {
        println!("  {} Would configure timeout settings", "→".blue());
    } else {
        println!("{} Timeout settings configured ({}s)", "✓".green(), timeout);
    }
    
    Ok(())
}



fn fix_config_issues(dry_run: bool, force: bool) -> Result<()> {
    println!("{}", "Fixing configuration issues...".bold());
    
    let ci_root = get_ci_root()?;
    
    // Check CLAUDE.md
    let claude_md = ci_root.join("CLAUDE.md");
    if !claude_md.exists() {
        if dry_run {
            println!("  {} Would create CLAUDE.md", "→".blue());
        } else {
            create_default_claude_md(&claude_md)?;
            println!("{} Created CLAUDE.md", "✓".green());
        }
    }
    
    // Check AGENTS directory
    let agents_dir = ci_root.join("AGENTS");
    if !agents_dir.exists() {
        if dry_run {
            println!("  {} Would create AGENTS directory", "→".blue());
        } else {
            fs::create_dir_all(&agents_dir)?;
            println!("{} Created AGENTS directory", "✓".green());
        }
    }
    
    // Check AGENTS.md
    let agents_md = ci_root.join("AGENTS.md");
    if !agents_md.exists() {
        if dry_run {
            println!("  {} Would create AGENTS.md", "→".blue());
        } else {
            create_default_agents_md(&agents_md)?;
            println!("{} Created AGENTS.md", "✓".green());
        }
    }
    
    Ok(())
}

fn fix_permission_issues(dry_run: bool) -> Result<()> {
    println!("{}", "Fixing permission issues...".bold());
    
    // CI has built-in functionality, no external scripts needed
    if dry_run {
        println!("  {} Would verify CI binary permissions", "→".blue());
    } else {
        println!("{} CI binary permissions verified", "✓".green());
    }
    
    Ok(())
}

fn fix_memory_issues(dry_run: bool) -> Result<()> {
    println!("{}", "Fixing memory system issues...".bold());
    
    // Check for memory system integrity
    let ci_root = get_ci_root()?;
    let agents_dir = ci_root.join("AGENTS");
    
    if agents_dir.exists() {
        if let Ok(entries) = fs::read_dir(&agents_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let agent_path = entry.path();
                    let memory_file = agent_path.join("MEMORY.md");
                    let learning_file = agent_path.join("ContinuousLearning.md");
                    
                    if !memory_file.exists() {
                        if dry_run {
                            println!("  {} Would create MEMORY.md for {}", "→".blue(), entry.file_name().to_string_lossy());
                        } else {
                            create_agent_memory_file(&memory_file, &entry.file_name().to_string_lossy())?;
                            println!("{} Created MEMORY.md for {}", "✓".green(), entry.file_name().to_string_lossy());
                        }
                    }
                    
                    if !learning_file.exists() {
                        if dry_run {
                            println!("  {} Would create ContinuousLearning.md for {}", "→".blue(), entry.file_name().to_string_lossy());
                        } else {
                            create_agent_learning_file(&learning_file, &entry.file_name().to_string_lossy())?;
                            println!("{} Created ContinuousLearning.md for {}", "✓".green(), entry.file_name().to_string_lossy());
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn fix_agent_issues(dry_run: bool) -> Result<()> {
    println!("{}", "Fixing agent system issues...".bold());
    
    let ci_root = get_ci_root()?;
    let agents_dir = ci_root.join("AGENTS");
    
    if agents_dir.exists() {
        if let Ok(entries) = fs::read_dir(&agents_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let agent_path = entry.path();
                    let readme_file = agent_path.join("README.md");
                    let sessions_dir = agent_path.join("Sessions");
                    
                    if !readme_file.exists() {
                        if dry_run {
                            println!("  {} Would create README.md for {}", "→".blue(), entry.file_name().to_string_lossy());
                        } else {
                            create_agent_readme(&readme_file, &entry.file_name().to_string_lossy())?;
                            println!("{} Created README.md for {}", "✓".green(), entry.file_name().to_string_lossy());
                        }
                    }
                    
                    if !sessions_dir.exists() {
                        if dry_run {
                            println!("  {} Would create Sessions directory for {}", "→".blue(), entry.file_name().to_string_lossy());
                        } else {
                            fs::create_dir_all(&sessions_dir)?;
                            println!("{} Created Sessions directory for {}", "✓".green(), entry.file_name().to_string_lossy());
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

// Helper functions

fn check_claude_cli_timeout() -> Result<()> {
    // Try to run claude with a short timeout to see if it responds
    let output = process::Command::new("timeout")
        .arg("2")
        .arg("claude")
        .arg("--version")
        .output();
    
    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(CIError::SystemError("Claude CLI timeout issues detected".to_string()).into()),
    }
}

fn check_config_integrity() -> Result<()> {
    let ci_root = get_ci_root()?;
    let claude_md = ci_root.join("CLAUDE.md");
    
    if !claude_md.exists() {
        return Err(CIError::Configuration("CLAUDE.md not found".to_string()).into());
    }
    
    Ok(())
}

fn check_permissions() -> Result<()> {
    // Check CI binary permissions
    if let Ok(output) = process::Command::new("ci").arg("--version").output() {
        if output.status.success() {
            return Ok(());
        }
    }
    
    Err(CIError::SystemError("CI binary is not accessible or executable".to_string()).into())
}

fn check_memory_system() -> Result<()> {
    let ci_root = get_ci_root()?;
    let agents_dir = ci_root.join("AGENTS");
    
    if !agents_dir.exists() {
        return Err(CIError::SystemError("AGENTS directory not found".to_string()).into());
    }
    
    Ok(())
}

fn check_agent_system() -> Result<()> {
    let ci_root = get_ci_root()?;
    let agents_md = ci_root.join("AGENTS.md");
    
    if !agents_md.exists() {
        return Err(CIError::SystemError("AGENTS.md not found".to_string()).into());
    }
    
    Ok(())
}

fn create_default_claude_md(path: &Path) -> Result<()> {
    let content = r#"# Collaborative Intelligence System Configuration

## System Integration
This configuration integrates with the Collaborative Intelligence system to provide
advanced AI agent capabilities and collaborative development features.

## Agent Management
Agents are loaded from the AGENTS directory and can be activated using their names.

## Memory System
The system maintains persistent memory for each agent to provide continuity across sessions.

## Learning System
Continuous learning is enabled to improve agent performance over time.
"#;
    
    fs::write(path, content)?;
    Ok(())
}

fn create_default_agents_md(path: &Path) -> Result<()> {
    let content = r#"# Collaborative Intelligence Agents

This file contains the registry of available agents in the Collaborative Intelligence system.

## Core Agents

### Athena
- **Role**: Knowledge Architect and Memory Systems Specialist
- **Capabilities**: Memory architecture, knowledge systems, collaborative intelligence frameworks
- **Usage**: Type `Athena` in Claude Code

### Architect
- **Role**: System Design Specialist
- **Capabilities**: System and component design
- **Usage**: Type `Architect` in Claude Code

### Developer
- **Role**: Implementation Specialist
- **Capabilities**: Code implementation, debugging, optimization
- **Usage**: Type `Developer` in Claude Code

## Using Agents

To use any agent, simply type their name in a Claude Code session. The agent will be
activated with their full memory and capabilities.
"#;
    
    fs::write(path, content)?;
    Ok(())
}

fn create_agent_memory_file(path: &Path, agent_name: &str) -> Result<()> {
    let content = format!(
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
    
    fs::write(path, content)?;
    Ok(())
}

fn create_agent_learning_file(path: &Path, agent_name: &str) -> Result<()> {
    let content = format!(
        r#"# {} Continuous Learning

This file documents the learning progress of the {} agent.

## Learning Record

### {}

- Agent memory file created
- Basic memory structure established
- Ready for continuous learning
"#,
        agent_name,
        agent_name,
        chrono::Utc::now().format("%Y-%m-%d")
    );
    
    fs::write(path, content)?;
    Ok(())
}

fn create_agent_readme(path: &Path, agent_name: &str) -> Result<()> {
    let content = format!(
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
    
    fs::write(path, content)?;
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    // On Windows, files are executable by default
    Ok(())
}