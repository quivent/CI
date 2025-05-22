use anyhow::Result;
use colored::*;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::process::Command;

use crate::config::Config;
use crate::helpers::CommandHelpers;

pub async fn intent(__config: &Config) -> Result<()> {
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

pub async fn load_agent(agent_name: &str, context: Option<&str>, path: Option<&Path>, auto_yes: bool, config: &Config) -> Result<()> {
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
        return load_from_direct_files(agent_name, context, direct_memory_path, auto_yes, config).await;
    } else if memory_path.exists() {
        return load_from_direct_files(agent_name, context, memory_path, auto_yes, config).await;
    } else if agents_md_path.exists() {
        // Fall back to legacy AGENTS.md loading
        return load_from_agents_md(agent_name, context, path, auto_yes, config).await;
    } else {
        // Neither method is available
        CommandHelpers::print_error("No agent sources found. Neither direct agent files nor AGENTS.md exist.");
        return Err(anyhow::anyhow!("Agent source files not found"));
    }
}

/// Load an agent from direct files in the AGENTS directory
async fn load_from_direct_files(agent_name: &str, context: Option<&str>, memory_file: PathBuf, auto_yes: bool, config: &Config) -> Result<()> {
    CommandHelpers::print_info("Loading agent from direct files");
    
    // Determine the agent toolkit path
    let agent_toolkit_path = config.ci_path.join("AGENTS").join(agent_name);
    
    // Ensure the agent toolkit directory exists
    if !agent_toolkit_path.exists() {
        match std::fs::create_dir_all(&agent_toolkit_path) {
            Ok(_) => CommandHelpers::print_info(&format!("Created agent toolkit directory at: {}", agent_toolkit_path.display())),
            Err(e) => CommandHelpers::print_warning(&format!("Error creating agent toolkit directory: {}", e))
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
    
    // Save the enhanced memory to a working file
    let working_memory_path = agent_toolkit_path.join(format!("working_{}.md", Utc::now().timestamp()));
    std::fs::write(&working_memory_path, &memory_content)
        .map_err(|e| anyhow::anyhow!("Failed to write working memory file: {}", e))?;
    
    // Print information about the operation
    CommandHelpers::print_divider("blue");
    CommandHelpers::print_info(&format!("Agent Memory: {}", agent_name));
    CommandHelpers::print_divider("blue");
    
    // Print the memory content to stdout if not launching directly
    print!("{}", memory_content);
    
    CommandHelpers::print_success(&format!("Agent {} loaded successfully", agent_name));
    
    // Offer launch options
    if auto_yes || CommandHelpers::prompt_confirmation("Launch Claude Code with this agent now?") {
        // Check if claude CLI is available
        if has_claude_cli() {
            // Launch Claude Code with the agent
            println!("Launching Claude Code with {}...", agent_name.cyan().bold());
            
            let status = Command::new("cat")
                .arg(&working_memory_path)
                .stdout(std::process::Stdio::piped())
                .spawn()
                .and_then(|output| {
                    Command::new("claude")
                        .arg("code")
                        .stdin(output.stdout.unwrap())
                        .status()
                })
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
async fn load_from_agents_md(agent_name: &str, context: Option<&str>, path: Option<&Path>, auto_yes: bool, config: &Config) -> Result<()> {
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
        match std::fs::create_dir_all(&agent_toolkit_path) {
            Ok(_) => CommandHelpers::print_info(&format!("Created agent toolkit directory at: {}", agent_toolkit_path.display())),
            Err(e) => CommandHelpers::print_warning(&format!("Error creating agent toolkit directory: {}", e))
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
    
    // Save the enhanced memory to a working file
    let working_memory_path = agent_toolkit_path.join(format!("working_{}.md", Utc::now().timestamp()));
    std::fs::write(&working_memory_path, &full_memory)
        .map_err(|e| anyhow::anyhow!("Failed to write working memory file: {}", e))?;
    
    // Read the final memory content to display
    let final_memory = match std::fs::read_to_string(&working_memory_path) {
        Ok(content) => content,
        Err(e) => {
            CommandHelpers::print_error(&format!("Error reading memory file: {}", e));
            return Err(anyhow::anyhow!("Failed to read memory file"));
        }
    };
    
    // Print information about the operation
    CommandHelpers::print_divider("blue");
    CommandHelpers::print_info(&format!("Agent Memory: {}", agent_name));
    CommandHelpers::print_divider("blue");
    
    // Print the memory content to stdout
    print!("{}", final_memory);
    
    CommandHelpers::print_success(&format!("Agent {} loaded successfully", agent_name));
    
    // Offer launch options
    if auto_yes || CommandHelpers::prompt_confirmation("Launch Claude Code with this agent now?") {
        // Check if claude CLI is available
        if has_claude_cli() {
            // Launch Claude Code with the agent
            println!("Launching Claude Code with {}...", agent_name.cyan().bold());
            
            let status = Command::new("cat")
                .arg(&working_memory_path)
                .stdout(std::process::Stdio::piped())
                .spawn()
                .and_then(|output| {
                    Command::new("claude")
                        .arg("code")
                        .stdin(output.stdout.unwrap())
                        .status()
                })
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

/// Generate enhanced agent context with additional information
fn generate_agent_context(agent_name: &str, context_type: Option<&str>, metadata: &AgentMetadata) -> String {
    let mut context = String::new();
    
    context.push_str("# Agent Context Information\n\n");
    
    // Basic agent information
    context.push_str(&format!("## Agent: {}\n\n", agent_name));
    context.push_str(&format!("Role: {}\n\n", metadata.description));
    
    if !metadata.capabilities.is_empty() {
        context.push_str("### Capabilities\n\n");
        for capability in &metadata.capabilities {
            context.push_str(&format!("- {}\n", capability));
        }
        context.push_str("\n");
    }
    
    // Session information
    context.push_str("### Session Information\n\n");
    context.push_str(&format!("- Started: {}\n", Utc::now().to_rfc3339()));
    if let Some(ctx) = context_type {
        context.push_str(&format!("- Context: {}\n", ctx));
    }
    context.push_str(&format!("- Previous sessions: {}\n", metadata.usage_count));
    if let Some(last_used) = &metadata.last_used {
        context.push_str(&format!("- Last used: {}\n", last_used));
    }
    context.push_str("\n");
    
    // Environment information
    context.push_str("### Environment\n\n");
    context.push_str(&format!("- Toolkit path: {}\n", metadata.toolkit_path));
    let current_dir = std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "<unknown>".to_string());
    context.push_str(&format!("- Working directory: {}\n", current_dir));
    
    context.push_str("\n### Usage Instructions\n\n");
    context.push_str("This agent has its own toolkit directory and capabilities.\n");
    context.push_str("When working with this agent, refer to its specific role and capabilities.\n");
    context.push_str("The agent will prioritize its own resources before checking parent repositories.\n");
    
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
    
    // Add header for context
    agent_memory.push_str("# Agent Memory: ");
    agent_memory.push_str(agent_name);
    agent_memory.push_str("\n\n");
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("### ") {
            let full_line = line.trim_start_matches("### ").trim();
            let parts: Vec<&str> = full_line.split(" - ").collect();
            let current_agent_name = parts[0].trim();
            
            if current_agent_name.eq_ignore_ascii_case(agent_name) {
                found = true;
                collecting = true;
                
                // Add the agent header to the content
                agent_memory.push_str(&format!("## {}\n\n", full_line));
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
    
    // Add footer with usage instructions
    agent_memory.push_str("\n\n## Agent Usage Instructions\n\n");
    agent_memory.push_str("This agent has been loaded into the current Claude Code session.\n");
    agent_memory.push_str("You can interact with it as usual, and the agent will have access to its own memory and capabilities.\n\n");
    agent_memory.push_str("The agent has its own toolkit directory at:\n");
    
    // Add toolkit path info
    if let Ok(toolkit_path) = std::env::var("CI_AGENT_TOOLKIT_PATH") {
        agent_memory.push_str(&format!("```\n{}\n```\n\n", toolkit_path));
    } else {
        agent_memory.push_str("```\n[Will be set when the agent is loaded]\n```\n\n");
    }
    
    agent_memory.push_str("**IMPORTANT:** The agent will prioritize resources in its own toolkit before checking the parent repository.\n");
    agent_memory.push_str("This allows the agent to operate with its own specialized tools and knowledge.\n");
    
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