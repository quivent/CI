use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::errors::CIError;
use crate::helpers::path::get_ci_root;

#[derive(Debug, Serialize, Deserialize)]
struct SessionMetadata {
    session_name: String,
    agent_name: String,
    created_at: String,
    description: String,
    status: String,
    tags: Vec<String>,
}

pub fn create_command() -> Command {
    Command::new("session")
        .about("Manage Collaborative Intelligence sessions")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list")
                .about("List sessions")
                .arg(
                    Arg::new("agent")
                        .short('a')
                        .long("agent")
                        .value_name("AGENT_NAME")
                        .help("Filter sessions by agent")
                )
                .arg(
                    Arg::new("status")
                        .short('s')
                        .long("status")
                        .value_name("STATUS")
                        .help("Filter sessions by status (active, completed, archived)")
                )
                .arg(
                    Arg::new("recent")
                        .short('r')
                        .long("recent")
                        .value_name("COUNT")
                        .default_value("10")
                        .help("Show only recent sessions")
                )
        )
        .subcommand(
            Command::new("create")
                .about("Create a new session")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent for this session")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("session_name")
                        .help("Name of the session")
                        .required(true)
                        .index(2)
                )
                .arg(
                    Arg::new("description")
                        .short('d')
                        .long("description")
                        .value_name("DESCRIPTION")
                        .help("Session description")
                )
                .arg(
                    Arg::new("tags")
                        .short('t')
                        .long("tags")
                        .value_name("TAGS")
                        .help("Comma-separated tags")
                )
        )
        .subcommand(
            Command::new("info")
                .about("Show session information")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("session_name")
                        .help("Name of the session")
                        .required(true)
                        .index(2)
                )
        )
        .subcommand(
            Command::new("archive")
                .about("Archive a session")
                .arg(
                    Arg::new("agent_name")
                        .help("Name of the agent")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("session_name")
                        .help("Name of the session")
                        .required(true)
                        .index(2)
                )
        )
        .subcommand(
            Command::new("cleanup")
                .about("Clean up old sessions")
                .arg(
                    Arg::new("days")
                        .short('d')
                        .long("days")
                        .value_name("DAYS")
                        .default_value("30")
                        .help("Archive sessions older than N days")
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show what would be cleaned up without making changes")
                )
        )
}

pub fn execute(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => list_sessions(sub_matches),
        Some(("create", sub_matches)) => create_session(sub_matches),
        Some(("info", sub_matches)) => show_session_info(sub_matches),
        Some(("archive", sub_matches)) => archive_session(sub_matches),
        Some(("cleanup", sub_matches)) => cleanup_sessions(sub_matches),
        _ => {
            eprintln!("{}", "No valid subcommand provided".red());
            std::process::exit(1);
        }
    }
}

fn list_sessions(matches: &ArgMatches) -> Result<()> {
    let agent_filter = matches.get_one::<String>("agent");
    let status_filter = matches.get_one::<String>("status");
    let recent_count: usize = matches.get_one::<String>("recent")
        .unwrap()
        .parse()
        .with_context(|| "Invalid recent count")?;
    
    let ci_root = get_ci_root()?;
    let agents_dir = ci_root.join("AGENTS");
    
    if !agents_dir.exists() {
        return Err(CIError::NotFound("AGENTS directory not found".to_string()).into());
    }
    
    println!("{}", "Collaborative Intelligence Sessions".cyan().bold());
    println!("{}", "=".repeat(40).cyan());
    println!();
    
    let mut all_sessions = Vec::new();
    
    // Collect sessions from all agents
    if let Ok(entries) = fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let agent_name = entry.file_name().to_string_lossy().to_string();
                
                // Apply agent filter
                if let Some(filter) = agent_filter {
                    if !agent_name.contains(filter) {
                        continue;
                    }
                }
                
                let sessions_dir = entry.path().join("Sessions");
                if sessions_dir.exists() {
                    let agent_sessions = collect_agent_sessions(&agent_name, &sessions_dir)?;
                    all_sessions.extend(agent_sessions);
                }
            }
        }
    }
    
    // Apply status filter
    if let Some(status) = status_filter {
        all_sessions.retain(|session| session.status == *status);
    }
    
    // Sort by creation date (newest first)
    all_sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Limit to recent count
    if all_sessions.len() > recent_count {
        all_sessions.truncate(recent_count);
    }
    
    if all_sessions.is_empty() {
        println!("{}", "No sessions found matching the criteria.".yellow());
        println!("Create a new session with: ci session create <agent> <session_name>");
        return Ok(());
    }
    
    // Display sessions
    for session in &all_sessions {
        let status_color = match session.status.as_str() {
            "active" => session.status.green(),
            "completed" => session.status.blue(),
            "archived" => session.status.yellow(),
            _ => session.status.white(),
        };
        
        println!("{} {} - {}", 
            "●".white(),
            session.session_name.bold(),
            session.agent_name.dimmed()
        );
        
        println!("  Status: {} | Created: {}", 
            status_color,
            session.created_at.dimmed()
        );
        
        if !session.description.is_empty() {
            println!("  Description: {}", session.description);
        }
        
        if !session.tags.is_empty() {
            println!("  Tags: {}", session.tags.join(", ").dimmed());
        }
        
        println!();
    }
    
    println!("Total sessions: {}", all_sessions.len());
    
    Ok(())
}

fn create_session(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let session_name = matches.get_one::<String>("session_name").unwrap();
    let description = matches.get_one::<String>("description").unwrap_or(&String::new()).clone();
    let empty_tags = String::new();
    let tags_str = matches.get_one::<String>("tags").unwrap_or(&empty_tags);
    
    let tags: Vec<String> = if tags_str.is_empty() {
        Vec::new()
    } else {
        tags_str.split(',').map(|s| s.trim().to_string()).collect()
    };
    
    let ci_root = get_ci_root()?;
    let agent_dir = ci_root.join("AGENTS").join(agent_name);
    
    if !agent_dir.exists() {
        return Err(CIError::NotFound(format!(
            "Agent '{}' not found. Use 'ci agent list' to see available agents.",
            agent_name
        )).into());
    }
    
    let sessions_dir = agent_dir.join("Sessions");
    let session_dir = sessions_dir.join(session_name);
    
    if session_dir.exists() {
        return Err(CIError::AlreadyExists(format!(
            "Session '{}' already exists for agent '{}'",
            session_name, agent_name
        )).into());
    }
    
    println!("{}", format!("Creating session: {} for {}", session_name, agent_name).cyan().bold());
    
    // Create session directory
    fs::create_dir_all(&session_dir)?;
    
    // Create session metadata
    let metadata = SessionMetadata {
        session_name: session_name.clone(),
        agent_name: agent_name.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        description,
        status: "active".to_string(),
        tags,
    };
    
    let metadata_path = session_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, metadata_json)?;
    
    // Create README.md
    let readme_content = format!(
        r#"# {} Session: {}

**Agent**: {}
**Created**: {}
**Status**: Active

## Description

{}

## Session Notes

Add your session notes here as you work with the agent.

## Outcomes

Document the outcomes and learnings from this session.

## Files

- `metadata.json` - Session metadata
- `README.md` - This session documentation
- `notes.md` - Detailed session notes (create as needed)
- `outcomes.md` - Session outcomes and results (create as needed)
"#,
        agent_name,
        session_name,
        agent_name,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        if metadata.description.is_empty() { 
            "No description provided." 
        } else { 
            &metadata.description 
        }
    );
    
    fs::write(session_dir.join("README.md"), readme_content)?;
    
    println!("{} Session created successfully!", "✓".green());
    println!("Session directory: {}", session_dir.display());
    println!();
    println!("To work with this session:");
    println!("  cd {}", session_dir.display());
    println!("  ci agent activate {}", agent_name);
    
    Ok(())
}

fn show_session_info(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let session_name = matches.get_one::<String>("session_name").unwrap();
    
    let ci_root = get_ci_root()?;
    let session_dir = ci_root.join("AGENTS").join(agent_name).join("Sessions").join(session_name);
    
    if !session_dir.exists() {
        return Err(CIError::NotFound(format!(
            "Session '{}' not found for agent '{}'",
            session_name, agent_name
        )).into());
    }
    
    println!("{}", format!("Session Information: {}", session_name).cyan().bold());
    println!("{}", "=".repeat(40).cyan());
    println!();
    
    // Load metadata
    let metadata_path = session_dir.join("metadata.json");
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        if let Ok(metadata) = serde_json::from_str::<SessionMetadata>(&metadata_content) {
            println!("{}: {}", "Agent".bold(), metadata.agent_name);
            println!("{}: {}", "Created".bold(), metadata.created_at);
            
            let status_colored = match metadata.status.as_str() {
                "active" => metadata.status.green(),
                "completed" => metadata.status.blue(),
                "archived" => metadata.status.yellow(),
                _ => metadata.status.white(),
            };
            println!("{}: {}", "Status".bold(), status_colored);
            
            if !metadata.description.is_empty() {
                println!("{}: {}", "Description".bold(), metadata.description);
            }
            
            if !metadata.tags.is_empty() {
                println!("{}: {}", "Tags".bold(), metadata.tags.join(", "));
            }
        }
    }
    
    println!("{}: {}", "Path".bold(), session_dir.display());
    println!();
    
    // Show session files
    println!("{}", "Session Files:".bold());
    if let Ok(entries) = fs::read_dir(&session_dir) {
        let mut files: Vec<_> = entries.flatten().collect();
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        
        for entry in files {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_type = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                "dir".dimmed()
            } else {
                "file".dimmed()
            };
            
            let size = if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    format!(" ({})", format_file_size(metadata.len())).dimmed().to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            
            println!("  - {} {}{}", file_name, file_type, size);
        }
    }
    
    Ok(())
}

fn archive_session(matches: &ArgMatches) -> Result<()> {
    let agent_name = matches.get_one::<String>("agent_name").unwrap();
    let session_name = matches.get_one::<String>("session_name").unwrap();
    
    let ci_root = get_ci_root()?;
    let session_dir = ci_root.join("AGENTS").join(agent_name).join("Sessions").join(session_name);
    
    if !session_dir.exists() {
        return Err(CIError::NotFound(format!(
            "Session '{}' not found for agent '{}'",
            session_name, agent_name
        )).into());
    }
    
    // Update metadata status
    let metadata_path = session_dir.join("metadata.json");
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        if let Ok(mut metadata) = serde_json::from_str::<SessionMetadata>(&metadata_content) {
            metadata.status = "archived".to_string();
            let updated_json = serde_json::to_string_pretty(&metadata)?;
            fs::write(&metadata_path, updated_json)?;
        }
    }
    
    println!("{} Session '{}' archived for agent '{}'", 
        "✓".green(), session_name, agent_name);
    
    Ok(())
}

fn cleanup_sessions(matches: &ArgMatches) -> Result<()> {
    let days: i64 = matches.get_one::<String>("days")
        .unwrap()
        .parse()
        .with_context(|| "Invalid days value")?;
    let dry_run = matches.get_flag("dry-run");
    
    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days);
    
    println!("{}", format!("Cleaning up sessions older than {} days", days).cyan().bold());
    if dry_run {
        println!("{}", "DRY RUN - No changes will be made".yellow());
    }
    println!();
    
    let ci_root = get_ci_root()?;
    let agents_dir = ci_root.join("AGENTS");
    
    let mut archived_count = 0;
    
    if let Ok(entries) = fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let agent_name = entry.file_name().to_string_lossy().to_string();
                let sessions_dir = entry.path().join("Sessions");
                
                if sessions_dir.exists() {
                    archived_count += cleanup_agent_sessions(&agent_name, &sessions_dir, cutoff_date, dry_run)?;
                }
            }
        }
    }
    
    if dry_run {
        println!("Would archive {} sessions", archived_count);
    } else {
        println!("{} Archived {} sessions", "✓".green(), archived_count);
    }
    
    Ok(())
}

fn collect_agent_sessions(agent_name: &str, sessions_dir: &Path) -> Result<Vec<SessionMetadata>> {
    let mut sessions = Vec::new();
    
    if let Ok(entries) = fs::read_dir(sessions_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let session_name = entry.file_name().to_string_lossy().to_string();
                let metadata_path = entry.path().join("metadata.json");
                
                let metadata = if metadata_path.exists() {
                    let content = fs::read_to_string(&metadata_path)?;
                    serde_json::from_str::<SessionMetadata>(&content).unwrap_or_else(|_| {
                        // Fallback metadata if JSON is invalid
                        create_fallback_metadata(agent_name, &session_name, &entry.path())
                    })
                } else {
                    // Create metadata for sessions without metadata.json
                    create_fallback_metadata(agent_name, &session_name, &entry.path())
                };
                
                sessions.push(metadata);
            }
        }
    }
    
    Ok(sessions)
}

fn create_fallback_metadata(agent_name: &str, session_name: &str, session_path: &Path) -> SessionMetadata {
    let created_at = if let Ok(metadata) = fs::metadata(session_path) {
        if let Ok(created) = metadata.created() {
            chrono::DateTime::<chrono::Utc>::from(created).to_rfc3339()
        } else {
            chrono::Utc::now().to_rfc3339()
        }
    } else {
        chrono::Utc::now().to_rfc3339()
    };
    
    SessionMetadata {
        session_name: session_name.to_string(),
        agent_name: agent_name.to_string(),
        created_at,
        description: "Legacy session (no metadata)".to_string(),
        status: "completed".to_string(),
        tags: vec!["legacy".to_string()],
    }
}

fn cleanup_agent_sessions(
    agent_name: &str,
    sessions_dir: &Path,
    cutoff_date: chrono::DateTime<chrono::Utc>,
    dry_run: bool,
) -> Result<usize> {
    let mut archived_count = 0;
    
    if let Ok(entries) = fs::read_dir(sessions_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let session_name = entry.file_name().to_string_lossy().to_string();
                let metadata_path = entry.path().join("metadata.json");
                
                if let Ok(metadata) = fs::metadata(&entry.path()) {
                    if let Ok(created) = metadata.created() {
                        let created_time = chrono::DateTime::<chrono::Utc>::from(created);
                        
                        if created_time < cutoff_date {
                            if dry_run {
                                println!("  Would archive: {} / {}", agent_name, session_name);
                            } else {
                                // Update metadata to archived status
                                if metadata_path.exists() {
                                    let content = fs::read_to_string(&metadata_path)?;
                                    if let Ok(mut session_metadata) = serde_json::from_str::<SessionMetadata>(&content) {
                                        session_metadata.status = "archived".to_string();
                                        let updated_json = serde_json::to_string_pretty(&session_metadata)?;
                                        fs::write(&metadata_path, updated_json)?;
                                    }
                                }
                                println!("  Archived: {} / {}", agent_name, session_name);
                            }
                            archived_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    Ok(archived_count)
}

fn format_file_size(size: u64) -> String {
    if size < 1024 {
        format!("{}B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1}KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}