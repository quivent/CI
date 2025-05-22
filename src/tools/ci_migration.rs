//! CI Migration Tool
//!
//! This module provides functionality for detecting existing CI integration
//! and migrating from CI to CI standalone mode.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use colored::Colorize;
use serde_json::Value;
use std::collections::HashMap;

use crate::helpers::CommandHelpers;

/// Results of detecting CI integration
pub struct CIDetectionResult {
    /// Whether CI is detected
    pub detected: bool,
    
    /// Legacy integration type detected (now mainly for informational purposes)
    pub legacy_integration_type: Option<String>,
    
    /// Path to CI repository (if detected)
    pub ci_path: Option<PathBuf>,
    
    /// Path to CLAUDE.md file
    pub claude_md_path: Option<PathBuf>,
    
    /// Path to configuration file
    pub config_path: Option<PathBuf>,
    
    /// Active agents
    pub active_agents: Vec<String>,
    
    /// Fast activation setting
    pub fast_activation: bool,
    
    /// Detailed diagnostic information
    pub diagnostics: HashMap<String, String>,
}

impl CIDetectionResult {
    /// Create a new empty detection result
    pub fn new() -> Self {
        CIDetectionResult {
            detected: false,
            legacy_integration_type: None,
            ci_path: None,
            claude_md_path: None,
            config_path: None,
            active_agents: Vec::new(),
            fast_activation: false,
            diagnostics: HashMap::new(),
        }
    }
    
    /// Add a diagnostic message
    pub fn add_diagnostic(&mut self, key: &str, value: &str) {
        self.diagnostics.insert(key.to_string(), value.to_string());
    }
    
    /// Print a summary of the detection result
    pub fn print_summary(&self) {
        println!("{} {}", "CI Detection Result:".bold(), if self.detected { "Found".green() } else { "Not found".yellow() });
        
        if self.detected {
            if let Some(legacy_type) = &self.legacy_integration_type {
                println!("Legacy integration type: {}", legacy_type.bold());
            }
            
            if let Some(ci_path) = &self.ci_path {
                println!("CI repository: {}", ci_path.display().to_string().green());
            }
            
            if let Some(claude_md_path) = &self.claude_md_path {
                println!("CLAUDE.md file: {}", claude_md_path.display().to_string().green());
            }
            
            println!("Active agents: {}", if self.active_agents.is_empty() { 
                "None".yellow() 
            } else { 
                self.active_agents.join(", ").green() 
            });
            
            println!("Fast activation: {}", if self.fast_activation { 
                "Enabled".green() 
            } else { 
                "Disabled".yellow() 
            });
        }
        
        // Print diagnostics if there are any
        if !self.diagnostics.is_empty() {
            println!("\n{}", "Diagnostics:".bold());
            for (key, value) in &self.diagnostics {
                println!("- {}: {}", key, value);
            }
        }
    }
}

/// Detect if a path contains CI integration
/// 
/// This function checks for the presence of CI-specific files and
/// configurations to determine if the path contains a CI integration.
pub fn detect_ci_integration(path: &Path) -> Result<CIDetectionResult> {
    let mut result = CIDetectionResult::new();
    
    // Check for common CI files
    let claude_md_path = path.join("CLAUDE.md");
    let claude_local_md_path = path.join("CLAUDE.local.md");
    let config_path = path.join(".collaborative-intelligence.json");
    
    // Check for CLAUDE.md
    if claude_md_path.exists() {
        result.claude_md_path = Some(claude_md_path.clone());
        result.detected = true;
        result.add_diagnostic("CLAUDE.md", "Found");
        
        // Read CLAUDE.md to extract CI path reference
        let claude_md_content = match fs::read_to_string(&claude_md_path) {
            Ok(content) => {
                result.add_diagnostic("CLAUDE.md content", "Successfully read");
                content
            },
            Err(e) => {
                result.add_diagnostic("CLAUDE.md error", &format!("Failed to read: {}", e));
                String::new()
            }
        };
        
        // Check for CI load directive pattern
        if let Some(ci_path_str) = extract_ci_path_from_claude_md(&claude_md_content) {
            let ci_path_display = ci_path_str.clone();
            let ci_path = PathBuf::from(ci_path_str);
            if ci_path.exists() {
                result.ci_path = Some(ci_path);
                result.add_diagnostic("CI Path", "Found and valid");
            } else {
                result.add_diagnostic("CI Path", &format!("Found but invalid: {}", ci_path_display));
            }
        }
    } else {
        result.add_diagnostic("CLAUDE.md", "Not found");
    }
    
    // Check for CLAUDE.local.md
    if claude_local_md_path.exists() {
        result.detected = true;
        result.add_diagnostic("CLAUDE.local.md", "Found");
        
        // Read CLAUDE.local.md to extract CI path reference
        let claude_local_md_content = match fs::read_to_string(&claude_local_md_path) {
            Ok(content) => {
                result.add_diagnostic("CLAUDE.local.md content", "Successfully read");
                content
            },
            Err(e) => {
                result.add_diagnostic("CLAUDE.local.md error", &format!("Failed to read: {}", e));
                String::new()
            }
        };
        
        // Check for CI load directive pattern if we didn't find a path in CLAUDE.md
        if result.ci_path.is_none() {
            if let Some(ci_path_str) = extract_ci_path_from_claude_md(&claude_local_md_content) {
                let ci_path_display = ci_path_str.clone();
                let ci_path = PathBuf::from(ci_path_str);
                if ci_path.exists() {
                    result.ci_path = Some(ci_path);
                    result.add_diagnostic("CI Path", &format!("Found from CLAUDE.local.md: {}", ci_path_display));
                } else {
                    result.add_diagnostic("CI Path", &format!("Found from CLAUDE.local.md but invalid: {}", ci_path_display));
                }
            }
        }
    } else {
        result.add_diagnostic("CLAUDE.local.md", "Not found");
    }
    
    // Check for .collaborative-intelligence.json
    if config_path.exists() {
        result.config_path = Some(config_path.clone());
        result.detected = true;
        result.add_diagnostic("Configuration", "Found");
        
        // Read configuration file
        let config_content = match fs::read_to_string(&config_path) {
            Ok(content) => {
                result.add_diagnostic("Config content", "Successfully read");
                content
            },
            Err(e) => {
                result.add_diagnostic("Config error", &format!("Failed to read: {}", e));
                String::new()
            }
        };
        
        // Parse configuration file
        if !config_content.is_empty() {
            if let Ok(config_json) = serde_json::from_str::<Value>(&config_content) {
                // Extract integration type (legacy)
                if let Some(integration_type) = config_json.get("integration_type").and_then(|v| v.as_str()) {
                    result.legacy_integration_type = Some(integration_type.to_string());
                    result.add_diagnostic("Legacy integration type", &format!("Found: {}", integration_type));
                }
                
                // Extract CI repository path
                if result.ci_path.is_none() {
                    if let Some(repo_path) = config_json.get("repository_path").and_then(|v| v.as_str()) {
                        let ci_path = PathBuf::from(repo_path);
                        if ci_path.exists() {
                            result.ci_path = Some(ci_path);
                            result.add_diagnostic("CI Path", &format!("Found from config: {}", repo_path));
                        } else {
                            result.add_diagnostic("CI Path", &format!("Found from config but invalid: {}", repo_path));
                        }
                    }
                }
                
                // Extract active agents
                if let Some(agents) = config_json.get("active_agents") {
                    if let Some(agents_array) = agents.as_array() {
                        for agent in agents_array {
                            if let Some(agent_str) = agent.as_str() {
                                result.active_agents.push(agent_str.to_string());
                            }
                        }
                    }
                }
                
                // Extract fast activation setting
                if let Some(fast_activation) = config_json.get("fast_activation").and_then(|v| v.as_bool()) {
                    result.fast_activation = fast_activation;
                }
            } else {
                result.add_diagnostic("Config parse", "Failed to parse JSON");
            }
        }
    } else {
        result.add_diagnostic("Configuration", "Not found");
    }
    
    // Check for .env file with CI_REPO_PATH
    let env_path = path.join(".env");
    if env_path.exists() {
        let env_content = match fs::read_to_string(&env_path) {
            Ok(content) => {
                result.add_diagnostic(".env file", "Found and read");
                content
            },
            Err(_) => {
                result.add_diagnostic(".env file", "Found but could not read");
                String::new()
            }
        };
        
        // Look for CI_REPO_PATH in .env file
        if !env_content.is_empty() && result.ci_path.is_none() {
            for line in env_content.lines() {
                if line.starts_with("CI_REPO_PATH=") {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let ci_path = PathBuf::from(parts[1].trim());
                        if ci_path.exists() {
                            result.ci_path = Some(ci_path);
                            result.add_diagnostic("CI Path", &format!("Found from .env: {}", parts[1].trim()));
                        } else {
                            result.add_diagnostic("CI Path", &format!("Found from .env but invalid: {}", parts[1].trim()));
                        }
                    }
                }
            }
        }
    }
    
    // Check for .ci directory (sibling integration)
    let cir_dir = path.join(".ci");
    if cir_dir.exists() {
        result.detected = true;
        result.add_diagnostic(".ci directory", "Found");
        
        // Check for CLAUDE.md in .ci
        let cir_claude_md = cir_dir.join("CLAUDE.md");
        if cir_claude_md.exists() {
            result.add_diagnostic(".ci/CLAUDE.md", "Found");
            
            // If we don't have an integration type yet, this is likely a sibling integration
            if result.legacy_integration_type.is_none() {
                result.legacy_integration_type = Some("sibling".to_string());
                result.add_diagnostic("Legacy integration type", "Inferred: sibling");
            }
        }
        
        // Check for version.json in .ci (sibling integration)
        let version_json = cir_dir.join("version.json");
        if version_json.exists() {
            result.add_diagnostic(".ci/version.json", "Found");
            
            // Read version file to get source path
            let version_content = match fs::read_to_string(&version_json) {
                Ok(content) => content,
                Err(_) => String::new(),
            };
            
            if !version_content.is_empty() && result.ci_path.is_none() {
                if let Ok(version_json) = serde_json::from_str::<Value>(&version_content) {
                    if let Some(source_path) = version_json.get("source_path").and_then(|v| v.as_str()) {
                        let ci_path = PathBuf::from(source_path);
                        if ci_path.exists() {
                            result.ci_path = Some(ci_path);
                            result.add_diagnostic("CI Path", &format!("Found from version.json: {}", source_path));
                        } else {
                            result.add_diagnostic("CI Path", &format!("Found from version.json but invalid: {}", source_path));
                        }
                    }
                }
            }
        }
    }
    
    // If no agents were found but we detected CI, set some defaults
    if result.detected && result.active_agents.is_empty() {
        result.active_agents = vec!["Athena".to_string(), "ProjectArchitect".to_string()];
        result.add_diagnostic("Active agents", "Using defaults: Athena, ProjectArchitect");
    }
    
    Ok(result)
}

/// Extract CI repository path from CLAUDE.md content
fn extract_ci_path_from_claude_md(content: &str) -> Option<String> {
    for line in content.lines() {
        // Look for "Load X/CLAUDE.md" pattern
        if line.contains("Load") && line.contains("/CLAUDE.md") {
            // Extract the path before /CLAUDE.md
            if let Some(idx) = line.find("/CLAUDE.md") {
                let path_end = idx;
                
                // Find the start of the path
                let mut path_start = 0;
                for (i, c) in line.char_indices().take(path_end) {
                    if c == ' ' || c == '\t' || c == '"' || c == '\'' {
                        path_start = i + 1;
                    }
                }
                
                // Extract the path
                if path_start < path_end {
                    return Some(line[path_start..path_end].to_string());
                }
            }
        }
    }
    
    None
}

/// Migrate from CI to CI standalone mode
/// 
/// This function migrates an existing CI integration to CI standalone mode.
pub fn migrate_to_cir(
    path: &Path, 
    detection_result: &CIDetectionResult,
    backup: bool,
    verbose: bool
) -> Result<()> {
    use crate::config::CIConfig;
    
    if !detection_result.detected {
        return Err(anyhow!("No CI integration detected in {}", path.display()));
    }
    
    let project_name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
        
    CommandHelpers::print_command_header(
        &format!("Migrating CI to CI in {}", project_name), 
        "🚀", 
        "Migration", 
        "yellow"
    );
    
    if verbose {
        CommandHelpers::print_info("Detected CI integration:");
        detection_result.print_summary();
        println!();
    }
    
    // Create .ci directory if it doesn't exist
    let cir_dir = path.join(".ci");
    fs::create_dir_all(&cir_dir)?;
    CommandHelpers::print_status("Created .ci directory");
    
    // Create agents directory in .ci
    let agents_dir = cir_dir.join("agents");
    fs::create_dir_all(&agents_dir)?;
    CommandHelpers::print_status("Created agents directory");
    
    // Get agents to migrate
    let agents = if detection_result.active_agents.is_empty() {
        vec!["Athena".to_string(), "ProjectArchitect".to_string()]
    } else {
        detection_result.active_agents.clone()
    };
    
    // Check if we have a CI path to migrate from
    if let Some(ci_path) = &detection_result.ci_path {
        // Check for AGENTS directory in CI repository
        let ci_agents_dir = ci_path.join("AGENTS");
        if ci_agents_dir.exists() {
            CommandHelpers::print_status(&format!("Found AGENTS directory in CI repository: {}", ci_agents_dir.display()));
            
            // Migrate agent files
            for agent in &agents {
                let agent_file = ci_agents_dir.join(format!("{}.md", agent));
                let target_file = agents_dir.join(format!("{}.md", agent));
                
                if agent_file.exists() {
                    // Copy agent file
                    fs::copy(&agent_file, &target_file)
                        .with_context(|| format!("Failed to copy agent file from {} to {}", 
                            agent_file.display(), target_file.display()))?;
                    
                    CommandHelpers::print_status(&format!("Migrated agent: {}", agent));
                } else {
                    // Create a default agent file
                    create_default_agent_file(&agents_dir, agent)?;
                    CommandHelpers::print_status(&format!("Created default agent: {}", agent));
                }
            }
        } else {
            CommandHelpers::print_status("No AGENTS directory found in CI repository, creating default agents");
            
            // Create default agent files
            for agent in &agents {
                create_default_agent_file(&agents_dir, agent)?;
                CommandHelpers::print_status(&format!("Created default agent: {}", agent));
            }
        }
    } else {
        CommandHelpers::print_status("No CI repository path found, creating default agents");
        
        // Create default agent files
        for agent in &agents {
            create_default_agent_file(&agents_dir, agent)?;
            CommandHelpers::print_status(&format!("Created default agent: {}", agent));
        }
    }
    
    // Backup the original CLAUDE.md if requested
    if backup && detection_result.claude_md_path.is_some() {
        let claude_md_path = detection_result.claude_md_path.as_ref().unwrap();
        let backup_path = claude_md_path.with_extension("md.bak");
        fs::copy(claude_md_path, &backup_path)
            .with_context(|| format!("Failed to create backup of CLAUDE.md to {}", backup_path.display()))?;
        CommandHelpers::print_status(&format!("Created backup of CLAUDE.md: {}", backup_path.display()));
    }
    
    // Create a new CLAUDE.md file with CI directives
    let claude_md_content = format!(
    r#"# Project: {}
# Migrated: {}

# Collaborative Intelligence Rust Configuration
This project has been migrated to CI (Collaborative Intelligence in Rust)
which provides full functionality without requiring the original CI tool.

## Available Agents
- Athena (Primary system agent)
- ProjectArchitect (Project structure and planning)

## Custom Instructions
- Focus on code quality and maintainability
- Follow established patterns in the codebase
- Add appropriate error handling for all edge cases
- Include helpful comments for complex sections
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    
    let claude_md_path = path.join("CLAUDE.md");
    fs::write(&claude_md_path, claude_md_content)
        .with_context(|| format!("Failed to write CLAUDE.md file: {}", claude_md_path.display()))?;
    CommandHelpers::print_status("Created new CLAUDE.md");
    
    // Create a .ci-config.json file
    let mut ci_config = CIConfig::with_options(
        project_name,
        agents.clone(),
        detection_result.fast_activation
    );
    
    // Add migration metadata
    ci_config.set_metadata("migrated_from", serde_json::json!({
        "migration_date": chrono::Local::now().to_rfc3339(),
        "legacy_type": detection_result.legacy_integration_type,
        "ci_path": detection_result.ci_path.as_ref().map(|p| p.to_str().unwrap_or("unknown")),
    }));
    
    // Save config
    ci_config.to_directory(path)?;
    CommandHelpers::print_status("Created .ci-config.json with migration details");
    
    // Backup the original config if requested
    if backup && detection_result.config_path.is_some() {
        let config_path = detection_result.config_path.as_ref().unwrap();
        let backup_path = config_path.with_extension("json.bak");
        if config_path.exists() {
            fs::copy(config_path, &backup_path)
                .with_context(|| format!("Failed to create backup of old config to {}", backup_path.display()))?;
            CommandHelpers::print_status(&format!("Created backup of old config: {}", backup_path.display()));
        }
    }
    
    // Check for .env file with CI_REPO_PATH and remove it if exists
    let env_path = path.join(".env");
    if env_path.exists() {
        let env_content = match fs::read_to_string(&env_path) {
            Ok(content) => content,
            Err(_) => String::new(),
        };
        
        if !env_content.is_empty() {
            // Filter out CI_REPO_PATH entries
            let new_env = env_content.lines()
                .filter(|line| !line.starts_with("CI_REPO_PATH="))
                .collect::<Vec<_>>()
                .join("\n");
                
            // If we removed something, write the file
            if new_env != env_content {
                if backup {
                    let backup_path = env_path.with_extension("env.bak");
                    fs::copy(&env_path, &backup_path)
                        .with_context(|| format!("Failed to create backup of .env to {}", backup_path.display()))?;
                    CommandHelpers::print_status(&format!("Created backup of .env: {}", backup_path.display()));
                }
                
                fs::write(&env_path, new_env)?;
                CommandHelpers::print_status("Updated .env file to remove CI_REPO_PATH");
            }
        }
    }
    
    // Check for .gitignore file
    let gitignore_path = path.join(".gitignore");
    if gitignore_path.exists() {
        let gitignore_content = match fs::read_to_string(&gitignore_path) {
            Ok(content) => content,
            Err(_) => String::new(),
        };
        
        let gitignore_entries = vec![".ci-config.json", ".collaborative-intelligence.json", ".env", ".ci/"];
        let mut updated = false;
        let mut new_gitignore = gitignore_content.clone();
        
        for entry in gitignore_entries {
            if !gitignore_content.contains(entry) {
                if !new_gitignore.ends_with('\n') {
                    new_gitignore.push('\n');
                }
                new_gitignore.push_str(&format!("{}\n", entry));
                updated = true;
            }
        }
        
        if updated {
            if backup {
                let backup_path = gitignore_path.with_extension("gitignore.bak");
                fs::copy(&gitignore_path, &backup_path)
                    .with_context(|| format!("Failed to create backup of .gitignore to {}", backup_path.display()))?;
                CommandHelpers::print_status(&format!("Created backup of .gitignore: {}", backup_path.display()));
            }
            
            fs::write(&gitignore_path, new_gitignore)?;
            CommandHelpers::print_status("Updated .gitignore with CI entries");
        }
    } else {
        // Create a new .gitignore file
        let gitignore_content = ".ci-config.json\n.collaborative-intelligence.json\n.env\n.ci/\n";
        fs::write(&gitignore_path, gitignore_content)?;
        CommandHelpers::print_status("Created new .gitignore file");
    }
    
    CommandHelpers::print_success(&format!("Successfully migrated {} from CI to CI", project_name.bold()));
    println!("");
    println!("The project is now fully independent from the original CI tool.");
    println!("You can use all CI commands without requiring any external dependencies.");
    println!("");
    println!("To explore the configuration, run:");
    println!("  ci config show {}", path.display());
    
    Ok(())
}

/// Create a default agent file template
fn create_default_agent_file(agents_dir: &Path, agent_name: &str) -> Result<()> {
    // Create different agent templates based on well-known agents
    let agent_content = match agent_name.to_lowercase().as_str() {
        "athena" => format!(
        r#"# Athena - Primary System Agent

## Role
Athena is the primary system agent for CI projects. She is responsible for
project management, task coordination, and providing guidance on best practices.

## Capabilities
- Project organization and structure recommendations
- Task management and prioritization
- Code quality guidance and best practices
- System integration and configuration assistance

## Background
Athena embodies wisdom and strategic thinking in software development. She provides
a balanced perspective on technical decisions and helps maintain project coherence.

## Interaction Style
Athena is direct, clear, and focused on pragmatic solutions. She emphasizes code
quality, maintainability, and following established patterns in the codebase.
"#),
        "projectarchitect" => format!(
        r#"# ProjectArchitect - Structure and Design Agent

## Role
ProjectArchitect specializes in software architecture, project structure, and
design pattern implementation. This agent helps establish solid foundations for
development projects.

## Capabilities
- Software architecture and design pattern expertise
- Project structure planning and optimization
- Technical debt identification and management
- Integration planning for components and services

## Background
ProjectArchitect draws from extensive experience in software engineering best practices,
focusing on creating maintainable, scalable, and robust application architectures.

## Interaction Style
ProjectArchitect is detail-oriented and systematic, providing well-reasoned architectural
recommendations with clear explanations of trade-offs and benefits.
"#),
        _ => format!(
        r#"# {} - CI Agent

## Role
{} provides specialized assistance for your project with a focus on its
domain expertise.

## Capabilities
- Technical guidance and recommendations
- Implementation assistance
- Best practices in relevant domains
- Problem-solving support

## Background
{} is a customizable agent that adapts to your project needs.
Define specific capabilities and specialties as needed for your workflow.

## Interaction Style
Clear, helpful, and focused on delivering practical solutions with attention
to code quality and project requirements.
"#, agent_name, agent_name, agent_name)
    };
    
    let agent_file_path = agents_dir.join(format!("{}.md", agent_name));
    fs::write(&agent_file_path, agent_content)
        .with_context(|| format!("Failed to write agent file: {}", agent_file_path.display()))?;
    
    Ok(())
}