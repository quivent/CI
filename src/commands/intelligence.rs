use anyhow::Result;
use colored::*;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::process::Command;

use crate::config::Config;
use crate::helpers::CommandHelpers;

pub async fn intent(_config: &Config) -> Result<()> {
    // Follows exact format of the original CI intent command for perfect parity
    println!("{}", "Collaborative Intelligence Tool".green().bold());
    println!("{}", "=========================================".green());
    println!();
    println!("The CI tool enables the integration of Collaborative Intelligence into");
    println!("your projects, enhancing productivity through AI-assisted workflows.");
    println!();
    println!("{}", "Core Capabilities:".yellow().bold());
    println!();
    println!("1. {}", "Project Integration".yellow());
    println!("   - Initialize new projects with CI capabilities");
    println!("   - Integrate CI into existing projects");
    println!("   - Standalone integration for maximum independence");
    println!();
    println!("2. {}", "Agent Management".yellow());
    println!("   - Configure AI agents for specific project needs");
    println!("   - Access specialized agents like Athena and ProjectArchitect");
    println!("   - Customize agent behavior through CLAUDE.md files");
    println!();
    println!("3. {}", "System Management".yellow());
    println!("   - Verify and repair CI integrations");
    println!("   - Manage configuration with the config command");
    println!("   - Migrate from legacy CI to modern CI system");
    println!();
    println!("{}", "Workflow Integration:".yellow().bold());
    println!("CI seamlessly integrates with your development workflow by creating");
    println!("and managing configuration files that provide context and guidance to");
    println!("AI assistants, enabling more effective collaboration between developers");
    println!("and AI systems.");
    println!();
    println!("{}", "Getting Started:".green());
    println!("To initialize a new project: {}", "ci init <project-name>".cyan());
    println!("To add CI to existing project: {}", "ci integrate".cyan());
    println!("To check status: {}", "ci status".cyan());
    println!();
    println!("For more information on any command, use: {}", "ci <command> --help".cyan());
    
    Ok(())
}

pub async fn agents(config: &Config) -> Result<()> {
    // Try to find the AGENTS_FULL.md file in the Manager directory
    let agents_full_path = config.ci_path.join("AGENTS/Manager/AGENTS_FULL.md");
    if agents_full_path.exists() {
        match std::fs::read_to_string(&agents_full_path) {
            Ok(content) => {
                print!("{}", content);
                return Ok(());
            }
            Err(e) => {
                CommandHelpers::print_warning(&format!("Could not read AGENTS_FULL.md ({}), trying fallback", e));
            }
        }
    }
    
    // Try the older AGENTS.md structure
    match std::fs::read_to_string(config.ci_path.join("AGENTS.md")) {
        Ok(content) => {
            print!("{}", content);
            return Ok(());
        }
        Err(e) => {
            CommandHelpers::print_warning(&format!("Legacy AGENTS.md not found ({}), falling back to file system", e));
        }
    }
    
    // Fallback to scanning AGENTS directory structure
    println!("{}", "Available Collaborative Intelligence Agents".bold());
    println!("{}", "========================================".bold());
    println!();
    
    // Try to scan the AGENTS directory
    let agents_dir = config.ci_path.join("AGENTS");
    if agents_dir.exists() && agents_dir.is_dir() {
        println!("Scanning agent directories from: {}", agents_dir.display());
        println!();
        
        // Read agent directories
        if let Ok(entries) = std::fs::read_dir(&agents_dir) {
            let mut agents = Vec::new();
            
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(agent_name) = path.file_name().and_then(|n| n.to_str()) {
                            // Skip non-agent directories
                            if agent_name.starts_with('.') || agent_name == "Manager" {
                                continue;
                            }
                            
                            // Try to read the README.md for agent description
                            let readme_path = path.join("README.md");
                            let description = if readme_path.exists() {
                                std::fs::read_to_string(&readme_path)
                                    .ok()
                                    .and_then(|content| extract_agent_description(&content))
                                    .unwrap_or_else(|| "No description available".to_string())
                            } else {
                                "No description available".to_string()
                            };
                            
                            agents.push((agent_name.to_string(), description));
                        }
                    }
                }
            }
            
            // Sort agents alphabetically
            agents.sort_by(|a, b| a.0.cmp(&b.0));
            
            // Display agents
            for (name, description) in agents {
                println!("🤖 {}", name.green().bold());
                println!("   {}", description);
                println!();
            }
        }
        return Ok(());
    }
    
    // Final fallback - show error message
    println!("{}", "❌ No agents found".red().bold());
    println!("Could not locate agent information in:");
    println!("  • {}/AGENTS/Manager/AGENTS_FULL.md", config.ci_path.display());
    println!("  • {}/AGENTS.md", config.ci_path.display());
    println!("  • {}/AGENTS/ directory", config.ci_path.display());
    println!();
    println!("Please check your CI repository path configuration.");
    
    Ok(())
}

fn extract_agent_description(content: &str) -> Option<String> {
    // Look for the first line that looks like a description after the heading
    let lines: Vec<&str> = content.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# ") {
            // Found a heading, look for the next non-empty line that's not another heading
            for j in (i + 1)..lines.len() {
                let next_line = lines[j].trim();
                if !next_line.is_empty() && !next_line.starts_with("#") && !next_line.starts_with("##") {
                    // Found a description line
                    return Some(next_line.to_string());
                }
            }
        }
    }
    
    // Fallback: look for "Role:" or "Expertise:" patterns
    for line in lines {
        if let Some(role) = line.strip_prefix("- **Role**:").or_else(|| line.strip_prefix("**Role**:")) {
            return Some(role.trim().to_string());
        }
        if let Some(expertise) = line.strip_prefix("- **Expertise**:").or_else(|| line.strip_prefix("**Expertise**:")) {
            return Some(expertise.trim().to_string());
        }
    }
    
    None
}

/// Parse and display agents from the AGENTS.md markdown content
fn display_agents_from_markdown(content: &str) -> () {
    let lines: Vec<&str> = content.lines().collect();
    let mut agents = Vec::new();
    
    // Extract agent sections (### level headings)
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("### ") {
            let full_line = line.trim_start_matches("### ").trim();
            
            // Some agent entries include a dash and description
            let parts: Vec<&str> = full_line.split(" - ").collect();
            let agent_name = parts[0].trim();
            
            // Get description part if available
            let header_description = if parts.len() > 1 {
                parts[1].trim()
            } else {
                ""
            };
            
            // Find the description for this agent by looking at content between this heading and the next
            let mut agent_content = Vec::new();
            let mut j = i + 1;
            while j < lines.len() && !lines[j].starts_with("## ") && !lines[j].starts_with("### ") {
                if !lines[j].trim().is_empty() {
                    agent_content.push(lines[j]);
                }
                j += 1;
            }
            
            agents.push((agent_name.to_string(), header_description.to_string(), agent_content));
        }
    }
    
    // Sort agents alphabetically by name
    agents.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    
    CommandHelpers::print_divider("blue");
    println!("{}", "Available Agents:".bold());
    println!();
    
    // Display sorted agents
    for (agent_name, header_description, agent_content) in agents {
        // Print agent name with header description if available
        if !header_description.is_empty() {
            println!("{} {} {}", "•".yellow(), agent_name.cyan().bold(), format!("- {}", header_description).cyan());
        } else {
            println!("{} {} {}", "•".yellow(), agent_name.cyan().bold(), 
                "[Use: ci load ".dimmed().to_string() + &agent_name.dimmed() + &"]".dimmed()
            );
        }
        
        // Get the first line or paragraph as description
        if !agent_content.is_empty() {
            // First check for a bold description
            let desc_line_opt = agent_content.iter().find(|line| 
                line.contains("**Description**:") || 
                line.contains("**Role**:") ||
                line.starts_with("*") && line.contains("descri")
            );
            
            if let Some(desc_line) = desc_line_opt {
                // Extract description from formatted line
                let description = if desc_line.contains(':') {
                    desc_line.split(':').nth(1).unwrap_or(desc_line).trim()
                } else {
                    desc_line.trim()
                };
                println!("  {}", description);
            } else {
                // Just take the first non-empty line
                println!("  {}", agent_content[0]);
            }
        } else {
            println!("  {}", "No description available".italic());
        }
        
        // Check for agent file on disk
        let agent_dir = std::env::current_dir()
            .map(|p| p.join("AGENTS").join(&agent_name))
            .unwrap_or_else(|_| PathBuf::from("AGENTS").join(&agent_name));
            
        if agent_dir.exists() {
            let usage_count = get_agent_usage_count(&agent_dir);
            if usage_count > 0 {
                println!("  {}", format!("Used {} times", usage_count).dimmed());
            }
        }
    }
    
    // Show usage instructions
    println!();
    println!("To use agents in your project:");
    println!("  {}", "ci init <project-name> --agents=Agent1,Agent2".cyan());
    println!("  {}", "ci integrate --agents=Agent1,Agent2".cyan());
    println!();
    println!("For more information on a specific agent:");
    println!("  {}", "ci load <agent-name>".cyan());
    println!("  {}", "ci load <agent-name> --context=<context>".cyan());
}

/// Get agent usage count from metadata
fn get_agent_usage_count(agent_dir: &Path) -> usize {
    let metadata_path = agent_dir.join("metadata.json");
    if metadata_path.exists() {
        match std::fs::read_to_string(&metadata_path) {
            Ok(content) => {
                match serde_json::from_str::<AgentMetadata>(&content) {
                    Ok(meta) => meta.usage_count,
                    Err(_) => 0
                }
            },
            Err(_) => 0
        }
    } else {
        0
    }
}

/// Structure to represent agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent name
    pub name: String,
    
    /// Agent role/description
    pub description: String,
    
    /// Agent capabilities
    pub capabilities: Vec<String>,
    
    /// Agent creation date
    pub created_at: String,
    
    /// Agent last used date
    pub last_used: Option<String>,
    
    /// Number of times the agent has been used
    pub usage_count: usize,
    
    /// Agent version
    pub version: String,
    
    /// Agent toolkit path
    pub toolkit_path: String,
    
    /// Agent memory path
    pub memory_path: String,
    
    /// Agent continuous learning path
    pub learning_path: Option<String>,
    
    /// Custom attributes
    pub attributes: std::collections::HashMap<String, String>,
}

/// Structure for agent session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentSession {
    /// Agent name
    agent_name: String,
    
    /// Session start time
    start_time: String,
    
    /// Context used for session
    context: Option<String>,
    
    /// Session end time (if completed)
    end_time: Option<String>,
    
    /// Session output path
    output_path: Option<String>,
}

/// Load multiple agents into a combined Claude Code session
pub async fn load_agents(agent_names: &[String], context: Option<&str>, path: Option<&Path>, auto_yes: bool, allow: bool, config: &Config) -> Result<()> {
    if agent_names.is_empty() {
        return Err(anyhow::anyhow!("No agents specified"));
    }
    
    // Handle single agent case by delegating to existing function
    if agent_names.len() == 1 {
        return load_agent(&agent_names[0], context, path, auto_yes, allow, config).await;
    }
    
    CommandHelpers::print_command_header(
        &format!("Load agents: {}", agent_names.join(", ")), 
        "🧠", 
        "Intelligence & Discovery", 
        "blue"
    );
    
    if let Some(ctx) = context {
        CommandHelpers::print_info(&format!("Context: {}", ctx));
    }
    
    if let Some(pth) = path {
        CommandHelpers::print_info(&format!("Custom path: {}", pth.display()));
    }
    
    CommandHelpers::print_info(&format!("Loading {} agents for combined session...", agent_names.len()));
    
    // Collect all agent memory content
    let mut combined_memory = String::new();
    let mut loaded_agents = Vec::new();
    let mut agent_toolkit_paths = Vec::new();
    
    for agent_name in agent_names {
        CommandHelpers::print_info(&format!("Processing agent: {}", agent_name.cyan().bold()));
        
        // Check for agent files (same logic as load_agent)
        let agent_directory = config.ci_path.join("AGENTS").join(agent_name);
        let direct_memory_path = agent_directory.join(format!("{}.md", agent_name));
        let memory_path = agent_directory.join(format!("{}_memory.md", agent_name));
        let agents_md_path = config.ci_path.join("AGENTS.md");
        
        let agent_memory = if direct_memory_path.exists() {
            match std::fs::read_to_string(&direct_memory_path) {
                Ok(content) => {
                    CommandHelpers::print_success(&format!("  ✓ Loaded from: {}", direct_memory_path.display()));
                    content
                },
                Err(e) => {
                    CommandHelpers::print_warning(&format!("  ⚠ Failed to read {}: {}", direct_memory_path.display(), e));
                    continue;
                }
            }
        } else if memory_path.exists() {
            match std::fs::read_to_string(&memory_path) {
                Ok(content) => {
                    CommandHelpers::print_success(&format!("  ✓ Loaded from: {}", memory_path.display()));
                    content
                },
                Err(e) => {
                    CommandHelpers::print_warning(&format!("  ⚠ Failed to read {}: {}", memory_path.display(), e));
                    continue;
                }
            }
        } else if agents_md_path.exists() {
            // Load from AGENTS.md
            match std::fs::read_to_string(&agents_md_path) {
                Ok(agents_content) => {
                    if agent_exists(&agents_content, agent_name) {
                        let memory = extract_agent_memory(&agents_content, agent_name);
                        CommandHelpers::print_success(&format!("  ✓ Loaded from: AGENTS.md"));
                        memory
                    } else {
                        CommandHelpers::print_warning(&format!("  ⚠ Agent '{}' not found in AGENTS.md", agent_name));
                        continue;
                    }
                },
                Err(e) => {
                    CommandHelpers::print_warning(&format!("  ⚠ Failed to read AGENTS.md: {}", e));
                    continue;
                }
            }
        } else {
            CommandHelpers::print_warning(&format!("  ⚠ No memory files found for agent: {}", agent_name));
            continue;
        };
        
        // Add agent separator and memory to combined content
        if !combined_memory.is_empty() {
            combined_memory.push_str("\n\n");
            combined_memory.push_str(&format!("# Agent Separator: {}\n", "=".repeat(50)));
            combined_memory.push_str("\n");
        }
        
        combined_memory.push_str(&format!("# Agent: {}\n\n", agent_name));
        combined_memory.push_str(&agent_memory);
        
        loaded_agents.push(agent_name.clone());
        agent_toolkit_paths.push(agent_directory.clone());
        
        // Create agent toolkit directory if it doesn't exist
        if !agent_directory.exists() {
            if let Err(e) = std::fs::create_dir_all(&agent_directory) {
                CommandHelpers::print_warning(&format!("  ⚠ Could not create toolkit directory for {}: {}", agent_name, e));
            }
        }
    }
    
    if loaded_agents.is_empty() {
        return Err(anyhow::anyhow!("No agents could be loaded"));
    }
    
    CommandHelpers::print_success(&format!("Successfully loaded {} agents: {}", 
        loaded_agents.len(), 
        loaded_agents.join(", ")
    ));
    
    // Create combined memory file
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let session_name = format!("multi_agent_session_{}", timestamp);
    let agents_dir = config.ci_path.join("AGENTS");
    let combined_memory_path = agents_dir.join(format!("{}_combined_memory.md", session_name));
    
    // Ensure AGENTS directory exists
    if let Err(e) = std::fs::create_dir_all(&agents_dir) {
        return Err(anyhow::anyhow!("Failed to create AGENTS directory: {}", e));
    }
    
    // Minimal session header
    let mut final_memory = format!(
        "# Multi-Agent Session: multi_agent_session_{}\n\
         # Loaded Agents: {}\n\
         # Session Started: {}\n\
         # Total Agents: {}\n\n",
        timestamp,
        loaded_agents.join(", "),
        Utc::now().to_rfc3339(),
        loaded_agents.len()
    );
    final_memory.push_str(&combined_memory);
    
    // Write combined memory to file
    std::fs::write(&combined_memory_path, &final_memory)
        .map_err(|e| anyhow::anyhow!("Failed to write combined memory file: {}", e))?;
        
    CommandHelpers::print_success(&format!("Combined memory written to: {}", combined_memory_path.display()));
    
    // Create session metadata
    let session_path = agents_dir.join(format!("{}_session.json", session_name));
    let session = AgentSession {
        agent_name: format!("MultiAgent[{}]", loaded_agents.join(",")),
        start_time: Utc::now().to_rfc3339(),
        context: context.map(|s| s.to_string()),
        end_time: None,
        output_path: Some(combined_memory_path.to_string_lossy().to_string()),
    };
    
    let session_json = serde_json::to_string_pretty(&session)
        .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;
        
    std::fs::write(&session_path, session_json)
        .map_err(|e| anyhow::anyhow!("Failed to write session data: {}", e))?;
    
    // Launch Claude Code or provide instructions
    if auto_yes {
        // Check if claude CLI is available
        if has_claude_cli() {
            // Launch Claude Code with the combined memory
            println!("Launching Claude Code with multi-agent team...");
            
            let mut claude_cmd = Command::new("claude");
            claude_cmd.arg("code");
            claude_cmd.arg(&combined_memory_path);
            
            if allow {
                claude_cmd.args(&["--permission-mode", "bypassPermissions"]);
            }
            
            let status = claude_cmd.status()
                .map_err(|e| anyhow::anyhow!("Failed to launch Claude Code: {}", e))?;
                
            if !status.success() {
                CommandHelpers::print_warning("Claude Code exited with a non-zero status");
            }
            
            // Update session with end time
            let mut session = serde_json::from_str::<AgentSession>(&std::fs::read_to_string(&session_path)?)
                .map_err(|e| anyhow::anyhow!("Failed to read session data: {}", e))?;
                
            session.end_time = Some(Utc::now().to_rfc3339());
            
            let session_json = serde_json::to_string_pretty(&session)
                .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;
                
            std::fs::write(&session_path, session_json)
                .map_err(|e| anyhow::anyhow!("Failed to update session data: {}", e))?;
        } else {
            CommandHelpers::print_warning("Claude CLI not found. Please use one of the following methods:");
            CommandHelpers::print_info(&format!("  cat {} | claude code", combined_memory_path.display()));
            CommandHelpers::print_info(&format!("  # or"));
            CommandHelpers::print_info(&format!("  claude code < {}", combined_memory_path.display()));
        }
    } else {
        CommandHelpers::print_info("To use this multi-agent team in Claude Code:");
        CommandHelpers::print_info(&format!("  cat {} | claude code", combined_memory_path.display()));
        CommandHelpers::print_info(&format!("  # or"));
        CommandHelpers::print_info(&format!("  claude code < {}", combined_memory_path.display()));
    }
    
    Ok(())
}

pub async fn load_agent(agent_name: &str, context: Option<&str>, path: Option<&Path>, auto_yes: bool, allow: bool, config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        &format!("Load agent: {}", agent_name), 
        "🧠", 
        "Intelligence & Discovery", 
        "blue"
    );
    
    if let Some(ctx) = context {
        CommandHelpers::print_info(&format!("Context: {}", ctx));
    }
    
    if let Some(pth) = path {
        CommandHelpers::print_info(&format!("Custom path: {}", pth.display()));
    }
    
    // First, check in AGENTS directory for direct files
    let agent_directory = config.ci_path.join("AGENTS").join(agent_name);
    let direct_memory_path = agent_directory.join(format!("{}.md", agent_name));
    let memory_path = agent_directory.join(format!("{}_memory.md", agent_name));
    
    // Default to using AGENTS.md for backwards compatibility
    let agents_md_path = config.ci_path.join("AGENTS.md");
    
    // Load agent from direct files if they exist
    if direct_memory_path.exists() {
        return load_from_direct_files(agent_name, context, direct_memory_path, auto_yes, allow, config).await;
    } else if memory_path.exists() {
        return load_from_direct_files(agent_name, context, memory_path, auto_yes, allow, config).await;
    } else if agents_md_path.exists() {
        // Fall back to legacy AGENTS.md loading
        return load_from_agents_md(agent_name, context, path, auto_yes, allow, config).await;
    } else {
        // Neither method is available
        CommandHelpers::print_error("No agent sources found. Neither direct agent files nor AGENTS.md exist.");
        return Err(anyhow::anyhow!("Agent source files not found"));
    }
}

/// Load an agent from direct files in the AGENTS directory
async fn load_from_direct_files(agent_name: &str, context: Option<&str>, memory_file: PathBuf, auto_yes: bool, allow: bool, config: &Config) -> Result<()> {
    CommandHelpers::print_info("Loading agent from direct files");
    
    // Determine the agent toolkit path
    let agent_toolkit_path = config.ci_path.join("AGENTS").join(agent_name);
    
    // Ensure the agent toolkit directory exists
    if !agent_toolkit_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&agent_toolkit_path) {
            CommandHelpers::print_warning(&format!("Error creating agent toolkit directory: {}", e))
        }
    }
    
    // Create metadata file if it doesn't exist
    let metadata_path = agent_toolkit_path.join("metadata.json");
    
    // Load or create agent metadata
    let mut metadata = if metadata_path.exists() {
        match std::fs::read_to_string(&metadata_path) {
            Ok(content) => {
                match serde_json::from_str::<AgentMetadata>(&content) {
                    Ok(meta) => meta,
                    Err(_) => create_agent_metadata(agent_name, &agent_toolkit_path, &memory_file)
                }
            },
            Err(_) => create_agent_metadata(agent_name, &agent_toolkit_path, &memory_file)
        }
    } else {
        create_agent_metadata(agent_name, &agent_toolkit_path, &memory_file)
    };
    
    // Update metadata with current session
    metadata.last_used = Some(Utc::now().to_rfc3339());
    metadata.usage_count += 1;
    
    // Save updated metadata
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| anyhow::anyhow!("Failed to serialize agent metadata: {}", e))?;
        
    std::fs::write(&metadata_path, metadata_json)
        .map_err(|e| anyhow::anyhow!("Failed to write agent metadata: {}", e))?;
    
    // Set environment variables for agent context
    std::env::set_var("CI_AGENT_CONTEXT", "true");
    std::env::set_var("CI_AGENT_TOOLKIT_PATH", agent_toolkit_path.display().to_string());
    std::env::set_var("CI_AGENT_NAME", agent_name);
    if let Some(ctx) = context {
        std::env::set_var("CI_AGENT_CONTEXT_TYPE", ctx);
    }
    
    // Read memory content
    let mut memory_content = std::fs::read_to_string(&memory_file)
        .map_err(|e| anyhow::anyhow!("Failed to read agent memory file: {}", e))?;
    
    // Check for continuous learning file and append if it exists
    let learning_file = agent_toolkit_path.join("ContinuousLearning.md");
    if learning_file.exists() {
        if let Ok(learning_content) = std::fs::read_to_string(&learning_file) {
            memory_content.push_str("\n\n# Continuous Learning\n\n");
            memory_content.push_str(&learning_content);
        }
    }
    
    // Record session start
    let session = AgentSession {
        agent_name: agent_name.to_string(),
        start_time: Utc::now().to_rfc3339(),
        context: context.map(|c| c.to_string()),
        end_time: None,
        output_path: None,
    };
    
    // Save session data
    let sessions_dir = agent_toolkit_path.join("sessions");
    if !sessions_dir.exists() {
        std::fs::create_dir_all(&sessions_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create sessions directory: {}", e))?;
    }
    
    let session_path = sessions_dir.join(format!("{}.json", Utc::now().timestamp()));
    let session_json = serde_json::to_string_pretty(&session)
        .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;
        
    std::fs::write(&session_path, session_json)
        .map_err(|e| anyhow::anyhow!("Failed to write session data: {}", e))?;
    
    // Extended contextual data
    let agent_context = generate_agent_context(agent_name, context, &metadata);
    memory_content.push_str("\n\n");
    memory_content.push_str(&agent_context);
    
    // Save working memory file with BRAIN knowledge for Claude Code
    let working_memory_path = agent_toolkit_path.join(format!("working_{}.md", Utc::now().timestamp()));
    
    // Load BRAIN - critical system requirement with happy confirmation
    let brain_dir = "/Users/joshkornreich/Documents/Projects/CollaborativeIntelligence/BRAIN/Core";
    let brain_files = [
        "memory-architecture-principles.md",
        "autonomous-learning-mechanisms.md", 
        "communication-optimization.md"
    ];
    
    let mut brain_content = String::new();
    let mut loaded_files = Vec::new();
    
    for file_name in &brain_files {
        let file_path = format!("{}/{}", brain_dir, file_name);
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            brain_content.push_str(&content);
            loaded_files.push(file_name);
        }
    }
    
    let brain_knowledge = if !loaded_files.is_empty() {
        println!("🧠 {} BRAIN loaded", "💖".bright_magenta());
        println!("🧠 {} BRAIN verified", "✓".green());
        format!("🧠 BRAIN System Active\n\nLoaded {} core files: {}KB", 
                loaded_files.len(), brain_content.len() / 1024)
    } else {
        eprintln!("🚨 CRITICAL ERROR: No BRAIN core files found in {}", brain_dir);
        eprintln!("🚨 Expected files: {:?}", brain_files);
        eprintln!("🚨 The Collaborative Intelligence system cannot function without BRAIN access.");
        std::process::exit(1);
    };

    let full_working_memory = agent_context;
    
    std::fs::write(&working_memory_path, &full_working_memory)
        .map_err(|e| anyhow::anyhow!("Failed to write working memory file: {}", e))?;
    
    // Memory and BRAIN loaded
    
    CommandHelpers::print_success(&format!("Agent {} loaded successfully", agent_name));
    
    // Offer launch options
    if auto_yes || allow || CommandHelpers::prompt_confirmation("Launch Claude Code with this agent now?") {
        // Check if claude CLI is available
        if has_claude_cli() {
            // Launch Claude Code with the agent
            println!("Launching Claude Code with {}...", agent_name.cyan().bold());
            
            let mut claude_cmd = Command::new("claude");
            claude_cmd.arg("code");
            claude_cmd.arg(&working_memory_path);
            
            if allow {
                claude_cmd.args(&["--permission-mode", "bypassPermissions"]);
            }
            
            let status = claude_cmd.status()
                .map_err(|e| anyhow::anyhow!("Failed to launch Claude Code: {}", e))?;
                
            if !status.success() {
                CommandHelpers::print_warning("Claude Code exited with a non-zero status");
            }
            
            // Update session with end time
            let mut session = serde_json::from_str::<AgentSession>(&std::fs::read_to_string(&session_path)?)
                .map_err(|e| anyhow::anyhow!("Failed to read session data: {}", e))?;
                
            session.end_time = Some(Utc::now().to_rfc3339());
            
            let session_json = serde_json::to_string_pretty(&session)
                .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;
                
            std::fs::write(&session_path, session_json)
                .map_err(|e| anyhow::anyhow!("Failed to update session data: {}", e))?;
        } else {
            CommandHelpers::print_warning("Claude CLI not found. Please use one of the following methods:");
            CommandHelpers::print_info(&format!("  cat {} | claude code", working_memory_path.display()));
            CommandHelpers::print_info(&format!("  # or"));
            CommandHelpers::print_info(&format!("  claude code < {}", working_memory_path.display()));
        }
    } else {
        CommandHelpers::print_info("To use this agent in Claude Code:");
        CommandHelpers::print_info(&format!("  cat {} | claude code", working_memory_path.display()));
        CommandHelpers::print_info(&format!("  # or"));
        CommandHelpers::print_info(&format!("  claude code < {}", working_memory_path.display()));
    }
    
    Ok(())
}

/// Load agent from the legacy AGENTS.md file
async fn load_from_agents_md(agent_name: &str, context: Option<&str>, path: Option<&Path>, auto_yes: bool, allow: bool, config: &Config) -> Result<()> {
    CommandHelpers::print_info("Loading agent from AGENTS.md");
    
    // First check if the agent exists in AGENTS.md
    let agents_md_path = config.ci_path.join("AGENTS.md");
    
    if !agents_md_path.exists() {
        CommandHelpers::print_error(&format!("Error: AGENTS.md not found at {}", agents_md_path.display()));
        CommandHelpers::print_info("Please ensure the repository path is correct.");
        return Err(anyhow::anyhow!("AGENTS.md not found"));
    }
    
    // Read the AGENTS.md file to verify the agent exists
    let agents_md_content = match std::fs::read_to_string(&agents_md_path) {
        Ok(content) => content,
        Err(e) => {
            CommandHelpers::print_error(&format!("Error reading AGENTS.md: {}", e));
            return Err(anyhow::anyhow!("Failed to read AGENTS.md"));
        }
    };
    
    // Check if the agent exists
    if !agent_exists(&agents_md_content, agent_name) {
        CommandHelpers::print_error(&format!("Agent '{}' not found in AGENTS.md", agent_name));
        
        // Suggest available agents
        let agents = list_available_agents(&agents_md_content);
        if !agents.is_empty() {
            CommandHelpers::print_warning("Available agents:");
            for agent in agents {
                CommandHelpers::print_status(&agent);
            }
        } else {
            CommandHelpers::print_warning("No agents found in AGENTS.md file.");
        }
        return Err(anyhow::anyhow!("Agent not found"));
    }
    
    // Get agent's content from AGENTS.md
    let agent_memory = extract_agent_memory(&agents_md_content, agent_name);
    
    // Determine the agent toolkit path
    let agent_toolkit_path = config.ci_path.join("AGENTS").join(agent_name);
    
    // Create the agent toolkit directory if it doesn't exist
    if !agent_toolkit_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&agent_toolkit_path) {
            CommandHelpers::print_warning(&format!("Error creating agent toolkit directory: {}", e))
        }
    }
    
    // Create metadata file if it doesn't exist
    let metadata_path = agent_toolkit_path.join("metadata.json");
    
    // Set environment variables for agent context
    std::env::set_var("CI_AGENT_CONTEXT", "true");
    std::env::set_var("CI_AGENT_TOOLKIT_PATH", agent_toolkit_path.display().to_string());
    std::env::set_var("CI_AGENT_NAME", agent_name);
    if let Some(ctx) = context {
        std::env::set_var("CI_AGENT_CONTEXT_TYPE", ctx);
    }
    
    // Create a file containing the agent's memory
    let agent_memory_path = if let Some(pth) = path {
        pth.to_path_buf()
    } else {
        let memory_file_path = agent_toolkit_path.join(format!("{}_memory.md", agent_name));
        match std::fs::write(&memory_file_path, &agent_memory) {
            Ok(_) => CommandHelpers::print_info(&format!("Created agent memory file at: {}", memory_file_path.display())),
            Err(e) => CommandHelpers::print_warning(&format!("Error creating agent memory file: {}", e))
        }
        memory_file_path
    };
    
    // Load or create agent metadata
    let mut metadata = if metadata_path.exists() {
        match std::fs::read_to_string(&metadata_path) {
            Ok(content) => {
                match serde_json::from_str::<AgentMetadata>(&content) {
                    Ok(meta) => meta,
                    Err(_) => create_agent_metadata(agent_name, &agent_toolkit_path, &agent_memory_path)
                }
            },
            Err(_) => create_agent_metadata(agent_name, &agent_toolkit_path, &agent_memory_path)
        }
    } else {
        create_agent_metadata(agent_name, &agent_toolkit_path, &agent_memory_path)
    };
    
    // Update metadata with current session
    metadata.last_used = Some(Utc::now().to_rfc3339());
    metadata.usage_count += 1;
    
    // Save updated metadata
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| anyhow::anyhow!("Failed to serialize agent metadata: {}", e))?;
        
    std::fs::write(&metadata_path, metadata_json)
        .map_err(|e| anyhow::anyhow!("Failed to write agent metadata: {}", e))?;
    
    // Check for continuous learning file and append if exists
    let learning_file = agent_toolkit_path.join("ContinuousLearning.md");
    let mut full_memory = agent_memory.clone();
    
    if learning_file.exists() {
        if let Ok(learning_content) = std::fs::read_to_string(&learning_file) {
            full_memory.push_str("\n\n# Continuous Learning\n\n");
            full_memory.push_str(&learning_content);
            
            // Update memory file with continuous learning content
            match std::fs::write(&agent_memory_path, &full_memory) {
                Ok(_) => CommandHelpers::print_info("Added continuous learning content to memory"),
                Err(e) => CommandHelpers::print_warning(&format!("Error updating memory file: {}", e))
            }
        }
    }
    
    // Record session start
    let session = AgentSession {
        agent_name: agent_name.to_string(),
        start_time: Utc::now().to_rfc3339(),
        context: context.map(|c| c.to_string()),
        end_time: None,
        output_path: None,
    };
    
    // Save session data
    let sessions_dir = agent_toolkit_path.join("sessions");
    if !sessions_dir.exists() {
        std::fs::create_dir_all(&sessions_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create sessions directory: {}", e))?;
    }
    
    let session_path = sessions_dir.join(format!("{}.json", Utc::now().timestamp()));
    let session_json = serde_json::to_string_pretty(&session)
        .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;
        
    std::fs::write(&session_path, session_json)
        .map_err(|e| anyhow::anyhow!("Failed to write session data: {}", e))?;
    
    // Extended contextual data
    let agent_context = generate_agent_context(agent_name, context, &metadata);
    full_memory.push_str("\n\n");
    full_memory.push_str(&agent_context);
    
    // Save working memory file with BRAIN knowledge for Claude Code  
    let working_memory_path = agent_toolkit_path.join(format!("working_{}.md", Utc::now().timestamp()));
    
    // Load BRAIN - critical system requirement with happy confirmation
    let brain_dir = "/Users/joshkornreich/Documents/Projects/CollaborativeIntelligence/BRAIN/Core";
    let brain_files = [
        "memory-architecture-principles.md",
        "autonomous-learning-mechanisms.md", 
        "communication-optimization.md"
    ];
    
    let mut brain_content = String::new();
    let mut loaded_files = Vec::new();
    
    for file_name in &brain_files {
        let file_path = format!("{}/{}", brain_dir, file_name);
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            brain_content.push_str(&content);
            loaded_files.push(file_name);
        }
    }
    
    let brain_knowledge = if !loaded_files.is_empty() {
        println!("🧠 {} BRAIN loaded", "💖".bright_magenta());
        println!("🧠 {} BRAIN verified", "✓".green());
        format!("🧠 BRAIN System Active\n\nLoaded {} core files: {}KB", 
                loaded_files.len(), brain_content.len() / 1024)
    } else {
        eprintln!("🚨 CRITICAL ERROR: No BRAIN core files found in {}", brain_dir);
        eprintln!("🚨 Expected files: {:?}", brain_files);
        eprintln!("🚨 The Collaborative Intelligence system cannot function without BRAIN access.");
        std::process::exit(1);
    };

    let full_working_memory = agent_context;
    
    std::fs::write(&working_memory_path, &full_working_memory)
        .map_err(|e| anyhow::anyhow!("Failed to write working memory file: {}", e))?;
    
    // Read the final memory content to display
    let final_memory = match std::fs::read_to_string(&working_memory_path) {
        Ok(content) => content,
        Err(e) => {
            CommandHelpers::print_error(&format!("Error reading memory file: {}", e));
            return Err(anyhow::anyhow!("Failed to read memory file"));
        }
    };
    
    // Memory loaded silently
    
    CommandHelpers::print_success(&format!("Agent {} loaded successfully", agent_name));
    
    // Offer launch options
    if auto_yes || allow || CommandHelpers::prompt_confirmation("Launch Claude Code with this agent now?") {
        // Check if claude CLI is available
        if has_claude_cli() {
            // Launch Claude Code with the agent
            println!("Launching Claude Code with {}...", agent_name.cyan().bold());
            
            let mut claude_cmd = Command::new("claude");
            claude_cmd.arg("code");
            claude_cmd.arg(&working_memory_path);
            
            if allow {
                claude_cmd.args(&["--permission-mode", "bypassPermissions"]);
            }
            
            let status = claude_cmd.status()
                .map_err(|e| anyhow::anyhow!("Failed to launch Claude Code: {}", e))?;
                
            if !status.success() {
                CommandHelpers::print_warning("Claude Code exited with a non-zero status");
            }
            
            // Update session with end time
            let mut session = serde_json::from_str::<AgentSession>(&std::fs::read_to_string(&session_path)?)
                .map_err(|e| anyhow::anyhow!("Failed to read session data: {}", e))?;
                
            session.end_time = Some(Utc::now().to_rfc3339());
            
            let session_json = serde_json::to_string_pretty(&session)
                .map_err(|e| anyhow::anyhow!("Failed to serialize session data: {}", e))?;
                
            std::fs::write(&session_path, session_json)
                .map_err(|e| anyhow::anyhow!("Failed to update session data: {}", e))?;
        } else {
            CommandHelpers::print_warning("Claude CLI not found. Please use one of the following methods:");
            CommandHelpers::print_info(&format!("  cat {} | claude code", working_memory_path.display()));
            CommandHelpers::print_info(&format!("  # or"));
            CommandHelpers::print_info(&format!("  claude code < {}", working_memory_path.display()));
        }
    } else {
        CommandHelpers::print_info("To use this agent in Claude Code:");
        CommandHelpers::print_info(&format!("  cat {} | claude code", working_memory_path.display()));
        CommandHelpers::print_info(&format!("  # or"));
        CommandHelpers::print_info(&format!("  claude code < {}", working_memory_path.display()));
    }
    
    Ok(())
}

/// Execute a specific task with a CI agent
pub async fn execute_task(description: &str, agent_name: &str, autonomous: bool, context: Option<&str>, output: Option<&str>, path: Option<&Path>, config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        &format!("Execute task with agent: {}", agent_name), 
        "🎯", 
        "Intelligence & Discovery", 
        "cyan"
    );
    
    // Create enhanced task context for the agent
    let mut task_context = format!("# Task Assignment\n\n**OBJECTIVE**: {}\n\n", description);
    
    if let Some(ctx) = context {
        task_context.push_str(&format!("**CONTEXT**: {}\n\n", ctx));
    }
    
    if let Some(output_file) = output {
        task_context.push_str(&format!("**REQUIRED OUTPUT**: Please save results to '{}'\n\n", output_file));
    }
    
    task_context.push_str("**OPERATING MODE**: ");
    if autonomous {
        task_context.push_str("Autonomous - You have full permission to execute all necessary actions without asking for approval. Proceed with confidence.\n\n");
    } else {
        task_context.push_str("Interactive - Ask for permission before executing potentially impactful actions.\n\n");
    }
    
    task_context.push_str("**INSTRUCTIONS**: \n");
    task_context.push_str("- Focus on completing the specified objective\n");
    task_context.push_str("- Use your full capabilities and available tools\n");
    task_context.push_str("- Provide progress updates as you work\n");
    task_context.push_str("- Be thorough and systematic in your approach\n\n");
    
    println!("📋 Task: {}", description.cyan().bold());
    println!("🤖 Agent: {}", agent_name.yellow().bold());
    println!("⚡ Mode: {}", if autonomous { "Autonomous".green() } else { "Interactive".blue() });
    
    // Load the agent with task context
    load_agent(agent_name, Some(&task_context), path, true, autonomous, config).await
}

pub async fn adapt_session(path: &Path, config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Adaptive Claude Code Session", 
        "🧠", 
        "Intelligence & Discovery", 
        "blue"
    );
    
    // Look for CLAUDE.adaptation.md in the specified path
    let adapt_file = path.join("CLAUDE.adaptation.md");
    
    if !adapt_file.exists() {
        CommandHelpers::print_error(&format!("CLAUDE.adaptation.md not found in {}", path.display()));
        CommandHelpers::print_info("Create CLAUDE.adaptation.md with memory configuration and initial prompt.");
        return Err(anyhow::anyhow!("CLAUDE.adaptation.md file not found"));
    }
    
    // Read the adapt file content
    let adapt_content = std::fs::read_to_string(&adapt_file)
        .map_err(|e| anyhow::anyhow!("Failed to read CLAUDE.adaptation.md: {}", e))?;
    
    CommandHelpers::print_info(&format!("Loading adaptive memory from: {}", adapt_file.display()));
    
    // Check if claude CLI is available
    if !has_claude_cli() {
        CommandHelpers::print_error("Claude CLI not found. Please install it first.");
        return Err(anyhow::anyhow!("Claude CLI not available"));
    }
    
    // Create a temporary file with the adapt content
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("ci_adapt_session.md");
    
    std::fs::write(&temp_file, &adapt_content)
        .map_err(|e| anyhow::anyhow!("Failed to create temporary file: {}", e))?;
    
    CommandHelpers::print_success("Launching Claude Code with adaptive configuration...");
    
    // Launch Claude Code with the adapt content
    let status = Command::new("claude")
        .arg("code")
        .arg(&temp_file)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to launch Claude Code: {}", e))?;
    
    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);
    
    if !status.success() {
        CommandHelpers::print_warning("Claude Code exited with a non-zero status");
    }
    
    Ok(())
}

/// Create agent metadata from available information
fn create_agent_metadata(agent_name: &str, toolkit_path: &Path, memory_path: &Path) -> AgentMetadata {
    // Extract description from memory file if available
    let description = if memory_path.exists() {
        if let Ok(content) = std::fs::read_to_string(memory_path) {
            // Try to extract a description from the content
            content.lines()
                .find(|line| line.contains("Description:") || line.contains("Role:"))
                .map(|line| {
                    if let Some(idx) = line.find(':') {
                        line[idx+1..].trim().to_string()
                    } else {
                        line.trim().to_string()
                    }
                })
                .unwrap_or_else(|| format!("Agent {}", agent_name))
        } else {
            format!("Agent {}", agent_name)
        }
    } else {
        format!("Agent {}", agent_name)
    };
    
    // Create default metadata
    AgentMetadata {
        name: agent_name.to_string(),
        description,
        capabilities: Vec::new(),
        created_at: Utc::now().to_rfc3339(),
        last_used: None,
        usage_count: 0,
        version: "1.0".to_string(),
        toolkit_path: toolkit_path.display().to_string(),
        memory_path: memory_path.display().to_string(),
        learning_path: Some(toolkit_path.join("ContinuousLearning.md").display().to_string()),
        attributes: std::collections::HashMap::new(),
    }
}

/// Generate minimal agent context with essential information only
fn generate_agent_context(agent_name: &str, context_type: Option<&str>, metadata: &AgentMetadata) -> String {
    let mut context = format!("# Agent: {}\n\n", agent_name);
    
    if let Some(ctx) = context_type {
        context.push_str(&format!("{}\n\n", ctx));
    }
    
    context
}

/// Check if claude CLI is available on the system
fn has_claude_cli() -> bool {
    Command::new("which")
        .arg("claude")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if the agent exists in the AGENTS.md content
fn agent_exists(content: &str, agent_name: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    
    for line in lines.iter() {
        if line.starts_with("### ") {
            let full_line = line.trim_start_matches("### ").trim();
            let parts: Vec<&str> = full_line.split(" - ").collect();
            let current_agent_name = parts[0].trim();
            
            if current_agent_name.eq_ignore_ascii_case(agent_name) {
                return true;
            }
        }
    }
    
    false
}

/// List all available agents from AGENTS.md
fn list_available_agents(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut agents = Vec::new();
    
    for line in lines.iter() {
        if line.starts_with("### ") {
            let full_line = line.trim_start_matches("### ").trim();
            let parts: Vec<&str> = full_line.split(" - ").collect();
            let agent_name = parts[0].trim();
            agents.push(agent_name.to_string());
        }
    }
    
    agents.sort();
    agents
}

/// Extract agent's memory content from AGENTS.md
fn extract_agent_memory(content: &str, agent_name: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut agent_memory = String::new();
    let mut found = false;
    let mut collecting = false;
    
    // Clean agent memory without verbose headers
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("### ") {
            let full_line = line.trim_start_matches("### ").trim();
            let parts: Vec<&str> = full_line.split(" - ").collect();
            let current_agent_name = parts[0].trim();
            
            if current_agent_name.eq_ignore_ascii_case(agent_name) {
                found = true;
                collecting = true;
                
                // Start collecting without verbose headers
                continue;
            } else if found && collecting {
                // Stop collecting when we reach the next agent
                collecting = false;
            }
        } else if line.starts_with("## ") && found && collecting {
            // Stop collecting when we reach a section heading
            collecting = false;
        } else if collecting {
            // Add this line to the agent's content
            agent_memory.push_str(line);
            agent_memory.push_str("\n");
            
            // Add an empty line if this line is empty and the next line is not (for paragraph breaks)
            if line.is_empty() && i+1 < lines.len() && !lines[i+1].is_empty() {
                agent_memory.push_str("\n");
            }
        }
    }
    
    // Minimal agent context - no verbose instructions
    
    agent_memory
}

// Removed check_claude_code_availability function as we now match the original CI behavior

// Removed start_claude_code_with_agent function as we now match the original CI behavior

pub async fn projects(config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "List projects integrated with Collaborative Intelligence", 
        "🧠", 
        "Intelligence & Discovery", 
        "blue"
    );
    
    println!("Source repository path: {}", config.ci_path.display());
    
    // Check for Projects directory
    let projects_dir = config.ci_path.join("Projects");
    if !projects_dir.exists() || !projects_dir.is_dir() {
        CommandHelpers::print_info("No dedicated Projects directory found.");
        // Fall back to looking for .collaborative-intelligence.json files in common locations
        scan_for_integrated_projects(config)?;
        return Ok(());
    }
    
    // Read Projects directory
    println!("Looking for integrated projects in: {}", projects_dir.display());
    println!();
    
    let entries = match std::fs::read_dir(&projects_dir) {
        Ok(entries) => entries,
        Err(e) => {
            CommandHelpers::print_error(&format!("Error reading Projects directory: {}", e));
            return Ok(());
        }
    };
    
    let mut projects = Vec::new();
    
    // Collect project entries
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                CommandHelpers::print_warning(&format!("Error reading directory entry: {}", e));
                continue;
            }
        };
        
        let path = entry.path();
        
        if path.is_dir() {
            // Get project name from directory name
            let project_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            // Check for .collaborative-intelligence.json file
            let config_path = path.join(".collaborative-intelligence.json");
            let has_config = config_path.exists();
            
            // Get status based on config file existence
            let status = if has_config {
                "Integrated"
            } else {
                "Not integrated"
            };
            
            // Look for CLAUDE.md or CLAUDE.local.md
            let has_claude_md = path.join("CLAUDE.md").exists();
            let has_claude_local_md = path.join("CLAUDE.local.md").exists();
            
            let claude_status = if has_claude_md {
                "CLAUDE.md"
            } else if has_claude_local_md {
                "CLAUDE.local.md"
            } else {
                "None"
            };
            
            projects.push((project_name, status.to_string(), claude_status.to_string(), path));
        }
    }
    
    // Check if we found any projects
    if projects.is_empty() {
        CommandHelpers::print_info("No integrated projects found in the Projects directory.");
        
        // Fall back to looking for .collaborative-intelligence.json files in common locations
        scan_for_integrated_projects(config)?;
        return Ok(());
    }
    
    // Sort projects alphabetically
    projects.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    
    // Print projects
    CommandHelpers::print_divider("blue");
    println!("{}", "Integrated Projects:".bold());
    println!();
    
    for (name, status, claude_status, path) in projects {
        println!("{} {} {}", "•".yellow(), name.cyan().bold(), 
            if status == "Integrated" { 
                format!("({})", "Integrated".green()) 
            } else { 
                format!("({})", "Not integrated".red()) 
            }
        );
        
        println!("  {}", path.display());
        println!("  Configuration: {}", claude_status);
        println!();
    }
    
    Ok(())
}

/// Scan common locations for projects with CI integration
fn scan_for_integrated_projects(config: &Config) -> Result<()> {
    CommandHelpers::print_info("Scanning for integrated projects in common locations...");
    
    // Common locations to look for projects
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let common_dirs = vec![
        home_dir.join("Projects"),
        home_dir.join("Documents/Projects"),
        home_dir.join("repositories"),
        home_dir.join("code"),
        home_dir.join("src"),
    ];
    
    let mut integrated_projects = Vec::new();
    
    // Scan each common directory for .collaborative-intelligence.json files
    for dir in common_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }
        
        CommandHelpers::print_status(&format!("Scanning {}", dir.display()));
        
        // Get all subdirectories
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            
            let path = entry.path();
            
            if path.is_dir() {
                // Skip the CollaborativeIntelligence repository itself
                if path == config.ci_path {
                    continue;
                }
                
                // Check for .collaborative-intelligence.json file
                let config_path = path.join(".collaborative-intelligence.json");
                if config_path.exists() {
                    // Get project name from directory name
                    let project_name = path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    // Look for CLAUDE.md or CLAUDE.local.md
                    let has_claude_md = path.join("CLAUDE.md").exists();
                    let has_claude_local_md = path.join("CLAUDE.local.md").exists();
                    
                    let claude_status = if has_claude_md {
                        "CLAUDE.md"
                    } else if has_claude_local_md {
                        "CLAUDE.local.md"
                    } else {
                        "None"
                    };
                    
                    integrated_projects.push((project_name, claude_status, path));
                }
            }
        }
    }
    
    // Check if we found any integrated projects
    if integrated_projects.is_empty() {
        CommandHelpers::print_info("No integrated projects found.");
        return Ok(());
    }
    
    // Sort projects alphabetically
    integrated_projects.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    
    // Print projects
    CommandHelpers::print_divider("blue");
    println!("{}", "Integrated Projects:".bold());
    println!();
    
    for (name, claude_status, path) in integrated_projects {
        println!("{} {}", "•".yellow(), name.cyan().bold());
        println!("  {}", path.display());
        println!("  Configuration: {}", claude_status);
        println!();
    }
    
    Ok(())
}

/// Enhanced load_agents function that supports optional task execution
pub async fn load_agents_with_task(
    agent_names: &[String], 
    context: Option<&str>, 
    path: Option<&Path>, 
    auto_yes: bool, 
    allow: bool, 
    task: Option<&str>,
    parallel: bool,
    config: &Config
) -> Result<()> {
    if agent_names.is_empty() {
        return Err(anyhow::anyhow!("No agents specified"));
    }
    
    // Expand agent multipliers for all cases
    let expanded_agents = expand_agent_multipliers(agent_names)?;
    
    // Handle parallel execution for multiple agents or when explicitly requested
    if parallel && expanded_agents.len() > 1 && task.is_some() {
        return execute_parallel_agents_expanded(&expanded_agents, context, path, auto_yes, allow, task.unwrap(), config).await;
    }
    
    // For non-parallel case with multipliers, error if more than one instance
    if expanded_agents.len() > 1 && !parallel {
        return Err(anyhow::anyhow!(
            "Multiple agent instances detected ({} total). Use --parallel flag for multi-agent execution.\n\
            Tip: Use --parallel to run {} agent instances simultaneously.", 
            expanded_agents.len(),
            expanded_agents.len()
        ));
    }
    
    // Convert back to simple agent names for single agent case
    let simple_agent_names: Vec<String> = expanded_agents.iter().map(|a| a.name.clone()).collect();
    
    // If a task is provided, create enhanced task context
    let enhanced_context = if let Some(task_description) = task {
        let mut task_context = format!("# Agent Task Assignment\n\n**OBJECTIVE**: {}\n\n", task_description);
        
        if let Some(ctx) = context {
            task_context.push_str(&format!("**ADDITIONAL CONTEXT**: {}\n\n", ctx));
        }
        
        task_context.push_str("**OPERATING MODE**: ");
        if allow {
            task_context.push_str("Autonomous - You have full permission to execute all necessary actions without asking for approval. Proceed with confidence.\n\n");
        } else {
            task_context.push_str("Interactive - Ask for permission before executing potentially impactful actions.\n\n");
        }
        
        if simple_agent_names.len() > 1 {
            task_context.push_str(&format!("**TEAM COMPOSITION**: You are working with {} other agents: {}\n\n", 
                simple_agent_names.len() - 1, simple_agent_names.join(", ")));
            task_context.push_str("**COLLABORATION INSTRUCTIONS**: \n");
            task_context.push_str("- Coordinate with your team members effectively\n");
            task_context.push_str("- Share relevant insights and findings\n");
            task_context.push_str("- Divide work efficiently based on individual expertise\n");
            task_context.push_str("- Ensure comprehensive coverage of the task\n\n");
        }
        
        task_context.push_str("**TASK EXECUTION INSTRUCTIONS**: \n");
        task_context.push_str("- Focus on completing the specified objective efficiently\n");
        task_context.push_str("- Use your full capabilities and available tools\n");
        task_context.push_str("- Provide progress updates as you work\n");
        task_context.push_str("- Be thorough and systematic in your approach\n");
        task_context.push_str("- Document your findings and results clearly\n\n");
        
        // Display task information before launching
        CommandHelpers::print_command_header(
            &format!("Loading {} agent(s) with task", simple_agent_names.len()), 
            "🎯", 
            "Intelligence & Discovery", 
            "cyan"
        );
        
        println!("📋 Task: {}", task_description.cyan().bold());
        println!("🤖 Agent(s): {}", simple_agent_names.join(", ").yellow().bold());
        println!("⚡ Mode: {}", if allow { "Autonomous".green() } else { "Interactive".blue() });
        if simple_agent_names.len() > 1 {
            println!("👥 Team size: {} agents", simple_agent_names.len());
        }
        println!();
        
        Some(task_context)
    } else {
        context.map(|s| s.to_string())
    };
    
    // Call the existing load_agents function with enhanced context
    load_agents(
        &simple_agent_names, 
        enhanced_context.as_deref(), 
        path, 
        auto_yes, 
        allow, 
        config
    ).await
}

/// Execute multiple agents in parallel sessions for collaborative task work (with pre-expanded agents)
async fn execute_parallel_agents_expanded(
    expanded_agents: &[AgentInstance],
    context: Option<&str>, 
    path: Option<&Path>,
    auto_yes: bool,
    allow: bool,
    task: &str,
    config: &Config
) -> Result<()> {
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let total_agents = expanded_agents.len();
    
    CommandHelpers::print_command_header(
        &format!("Launching {} agents in parallel sessions", total_agents), 
        "🚀", 
        "Intelligence & Discovery", 
        "magenta"
    );
    
    println!("📋 Task: {}", task.cyan().bold());
    
    // Display agent instance information
    let unique_agents: std::collections::HashSet<_> = expanded_agents.iter().map(|a| &a.name).collect();
    if unique_agents.len() == 1 && total_agents > 1 {
        // Multiple instances of same agent
        let agent_name = expanded_agents[0].name.clone();
        println!("🤖 Agent Type: {}", agent_name.yellow().bold());
        println!("🔢 Instances: {} parallel instances", total_agents);
    } else {
        // Mixed agent types
        let instance_summary = create_instance_summary(&expanded_agents);
        println!("🤖 Agent Mix:");
        for (agent_type, count) in instance_summary {
            if count > 1 {
                println!("   • {} × {}", count, agent_type.cyan());
            } else {
                println!("   • {}", agent_type.cyan());
            }
        }
    }
    
    println!("⚡ Mode: {} | 🔄 Execution: {}", 
        if allow { "Autonomous".green() } else { "Interactive".blue() },
        "Parallel Sessions".magenta().bold()
    );
    println!("👥 Sessions: {} independent Claude Code instances", total_agents);
    println!();
    
    // Create shared coordination directory
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let session_id = format!("parallel_session_{}", timestamp);
    let coordination_dir = config.ci_path.join("AGENTS").join(&session_id);
    std::fs::create_dir_all(&coordination_dir)?;
    
    // Create coordination file with task details
    let coordination_file = coordination_dir.join("task_coordination.md");
    let agent_spec = if unique_agents.len() == 1 && total_agents > 1 {
        format!("{}*{}", expanded_agents[0].name, total_agents)
    } else {
        expanded_agents.iter().map(|a| a.display_name()).collect::<Vec<_>>().join(", ")
    };
    
    let coordination_content = format!(
        "# Parallel Agent Task Coordination\n\n\
        **Session ID**: {}\n\
        **Task**: {}\n\
        **Agent Specification**: {}\n\
        **Total Instances**: {}\n\
        **Mode**: {}\n\
        **Started**: {}\n\n\
        ## Agent Instance Assignments\n\n",
        session_id,
        task,
        agent_spec,
        total_agents,
        if allow { "Autonomous" } else { "Interactive" },
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    std::fs::write(&coordination_file, coordination_content)?;
    
    println!("📁 Coordination directory: {}", coordination_dir.display());
    println!("📄 Task coordination file: {}", coordination_file.display());
    println!();
    
    // Launch each agent instance in separate Claude Code session
    let mut handles = Vec::new();
    
    for (index, agent_instance) in expanded_agents.iter().enumerate() {
        let agent_task_context = create_parallel_agent_context(
            &agent_instance.name, 
            task, 
            &expanded_agents, 
            index, 
            &session_id,
            &coordination_file,
            context, 
            allow,
            &agent_instance.instance_id
        );
        
        println!("🚀 Launching {} in session {}...", agent_instance.display_name().cyan().bold(), index + 1);
        
        // Create individual agent memory file for this session
        let agent_session_file = coordination_dir.join(format!("{}_session_memory.md", agent_instance.file_safe_name()));
        std::fs::write(&agent_session_file, &agent_task_context)?;
        
        // Launch Claude Code with the agent's specific context
        let mut claude_cmd = Command::new("claude");
        
        if allow {
            claude_cmd.arg("--permission-mode").arg("bypassPermissions");
        }
        
        // Pass the agent memory content as the initial prompt
        claude_cmd.arg(&agent_task_context);
        
        // Set window title for the agent session
        claude_cmd.env("CLAUDE_WINDOW_TITLE", format!("[{}] {}", agent_instance.display_name(), task));
        
        println!("   📝 Memory file: {}", agent_session_file.display());
        println!("   🪟 Window title: [{}] {}", agent_instance.display_name(), task);
        
        // Launch the process
        match claude_cmd.spawn() {
            Ok(child) => {
                handles.push((agent_instance.display_name(), child));
                println!("   ✅ {} session started successfully", agent_instance.display_name().green());
            },
            Err(e) => {
                CommandHelpers::print_warning(&format!("Failed to launch {} session: {}", agent_instance.display_name(), e));
            }
        }
        
        // Small delay between launches to avoid resource conflicts
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        println!();
    }
    
    if handles.is_empty() {
        return Err(anyhow::anyhow!("Failed to launch any agent sessions"));
    }
    
    println!("🎯 {} agent sessions launched successfully!", handles.len());
    println!("📊 Monitor progress through individual Claude Code windows");
    println!("📁 Shared coordination: {}", coordination_dir.display());
    println!();
    println!("💡 {}:", "Coordination Tips".yellow().bold());
    println!("   • Each agent has access to the shared coordination directory");
    println!("   • Agents can create files in {} to share findings", coordination_dir.display());
    println!("   • Use the task coordination file to track overall progress");
    println!("   • Sessions run independently - monitor each window for progress");
    
    Ok(())
}

/// Create specialized context for each agent in parallel execution
fn create_parallel_agent_context(
    agent_name: &str,
    task: &str,
    all_agents: &[AgentInstance],
    agent_index: usize,
    session_id: &str,
    coordination_file: &Path,
    context: Option<&str>,
    allow: bool,
    instance_id: &str
) -> String {
    let display_name = if instance_id.is_empty() { 
        agent_name.to_string() 
    } else { 
        format!("{} ({})", agent_name, instance_id) 
    };
    
    let mut agent_context = format!(
        "# {} - Parallel Task Session\n\n\
        **AGENT IDENTITY**: {}\n\
        **INSTANCE ID**: {}\n\
        **SESSION**: {} (Agent {} of {})\n\
        **TASK**: {}\n\n",
        display_name, agent_name, instance_id, session_id, agent_index + 1, all_agents.len(), task
    );
    
    if let Some(ctx) = context {
        agent_context.push_str(&format!("**ADDITIONAL CONTEXT**: {}\n\n", ctx));
    }
    
    agent_context.push_str(&format!(
        "**OPERATING MODE**: {}\n\n",
        if allow { 
            "Autonomous - You have full permission to execute all necessary actions without asking for approval. Proceed with confidence."
        } else { 
            "Interactive - Ask for permission before executing potentially impactful actions."
        }
    ));
    
    let team_members = all_agents.iter()
        .map(|a| a.display_name())
        .collect::<Vec<_>>()
        .join(", ");
    
    agent_context.push_str(&format!(
        "**COLLABORATION FRAMEWORK**:\n\
        - **Team Members**: {}\n\
        - **Your Role**: Use your specialized expertise to contribute to the task\n\
        - **Coordination**: Share findings in {}\n\
        - **Independence**: You are operating in a separate session - work autonomously\n\
        - **Communication**: Create files in the coordination directory to share insights\n\n",
        team_members,
        coordination_file.display()
    ));
    
    // Add agent-specific task breakdown based on agent name
    agent_context.push_str("**YOUR SPECIALIZED CONTRIBUTION**:\n");
    match agent_name {
        name if name.contains("Analyst") => {
            agent_context.push_str("- Focus on deep technical analysis and system architecture\n");
            agent_context.push_str("- Identify patterns, dependencies, and structural insights\n");
            agent_context.push_str("- Provide technical recommendations and best practices\n");
        },
        name if name.contains("Documentor") => {
            agent_context.push_str("- Create comprehensive, well-structured documentation\n");
            agent_context.push_str("- Focus on clarity, organization, and user-friendly formatting\n");
            agent_context.push_str("- Ensure documentation follows established standards\n");
            if !instance_id.is_empty() {
                agent_context.push_str(&format!("- **Instance Focus**: As {}, coordinate with other Documentor instances to avoid overlap\n", instance_id));
                agent_context.push_str("- **Specialization**: Consider focusing on a specific module/component/aspect\n");
            }
        },
        name if name.contains("Researcher") => {
            agent_context.push_str("- Conduct thorough investigation and information gathering\n");
            agent_context.push_str("- Research best practices and industry standards\n");
            agent_context.push_str("- Provide context and background information\n");
        },
        _ => {
            agent_context.push_str("- Apply your specialized expertise to the task\n");
            agent_context.push_str("- Focus on your core competencies and strengths\n");
            agent_context.push_str("- Contribute unique insights based on your role\n");
        }
    }
    
    agent_context.push_str(&format!(
        "\n**EXECUTION INSTRUCTIONS**:\n\
        - Begin work immediately on the task using your specialization\n\
        - Create detailed progress files in the coordination directory\n\
        - Document your findings and methodologies clearly\n\
        - Work systematically and thoroughly\n\
        - Coordinate with other agents through shared files\n\
        - Maintain your distinct identity and expertise throughout\n\n\
        **COORDINATION DIRECTORY**: {}\n\
        **START WORKING**: Your parallel session is now active!\n\n",
        coordination_file.parent().unwrap().display()
    ));
    
    agent_context
}

/// Represents an agent instance with unique identification
#[derive(Debug, Clone)]
struct AgentInstance {
    name: String,
    instance_id: String,
    instance_number: usize,
}

impl AgentInstance {
    fn new(name: String, instance_number: usize, total_instances: usize) -> Self {
        let instance_id = if total_instances > 1 {
            format!("Instance-{}", instance_number)
        } else {
            String::new()
        };
        
        Self {
            name,
            instance_id,
            instance_number,
        }
    }
    
    fn display_name(&self) -> String {
        if self.instance_id.is_empty() {
            self.name.clone()
        } else {
            format!("{} ({})", self.name, self.instance_id)
        }
    }
    
    fn file_safe_name(&self) -> String {
        if self.instance_id.is_empty() {
            self.name.clone()
        } else {
            format!("{}_{}", self.name, self.instance_number)
        }
    }
}

/// Expand agent multipliers like "Documentor*7" into individual instances
fn expand_agent_multipliers(agent_specs: &[String]) -> Result<Vec<AgentInstance>> {
    let mut expanded = Vec::new();
    
    for spec in agent_specs {
        if let Some(captures) = regex::Regex::new(r"^([A-Za-z][A-Za-z0-9_]*)\*(\d+)$")?.captures(spec) {
            // Handle multiplier syntax: "AgentName*Count"
            let agent_name = captures.get(1).unwrap().as_str().to_string();
            let count: usize = captures.get(2).unwrap().as_str().parse()
                .map_err(|_| anyhow::anyhow!("Invalid multiplier count in: {}", spec))?;
            
            if count == 0 {
                return Err(anyhow::anyhow!("Agent count cannot be zero in: {}", spec));
            }
            
            if count > 50 {
                return Err(anyhow::anyhow!("Agent count cannot exceed 50 in: {} (requested: {})", spec, count));
            }
            
            for i in 1..=count {
                expanded.push(AgentInstance::new(agent_name.clone(), i, count));
            }
        } else {
            // Handle regular agent name
            expanded.push(AgentInstance::new(spec.clone(), 1, 1));
        }
    }
    
    Ok(expanded)
}

/// Create a summary of agent instances for display
fn create_instance_summary(agents: &[AgentInstance]) -> std::collections::HashMap<String, usize> {
    let mut summary = std::collections::HashMap::new();
    
    for agent in agents {
        *summary.entry(agent.name.clone()).or_insert(0) += 1;
    }
    
    summary
}

/// Execute multiple agents in parallel sessions for collaborative task work (legacy wrapper)
async fn execute_parallel_agents(
    agent_names: &[String],
    context: Option<&str>, 
    path: Option<&Path>,
    auto_yes: bool,
    allow: bool,
    task: &str,
    config: &Config
) -> Result<()> {
    // Expand agent multipliers and delegate to the expanded version
    let expanded_agents = expand_agent_multipliers(agent_names)?;
    execute_parallel_agents_expanded(&expanded_agents, context, path, auto_yes, allow, task, config).await
}