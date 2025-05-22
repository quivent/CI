use anyhow::{Result, Context, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use chrono;
use colored::Colorize;
use std::os::unix::fs::PermissionsExt;
use serde::{Serialize, Deserialize};

use crate::config::Config;
use crate::helpers::CommandHelpers;

/// Repository integration type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationType {
    /// Standalone integration (fully independent)
    Standalone,
    /// Override integration (preserves existing CLAUDE.md)
    Override,
}

impl IntegrationType {
    /// Convert from string representation
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "override" => Ok(IntegrationType::Override),
            _ => Ok(IntegrationType::Standalone) // Default to standalone for any other value
        }
    }
    
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            IntegrationType::Standalone => "standalone",
            IntegrationType::Override => "override",
        }
    }
    
    /// Get a human-readable description of the integration type
    pub fn description(&self) -> &'static str {
        match self {
            IntegrationType::Standalone => "Fully independent CI integration",
            IntegrationType::Override => "Preserves existing CLAUDE.md with override directives",
        }
    }
}

pub async fn init(
    project_name: &str,
    agents: &str,
    _integration: &str, // Kept for backward compatibility but ignored
    no_fast: bool,
    _ci_path_override: Option<&Path>, // Kept for backward compatibility but ignored
    _config: &Config, // Not needed anymore for standalone mode
) -> Result<()> {
    println!("{}", "Initializing new Collaborative Intelligence project".yellow().bold());
    println!("{}", "=".repeat("Initializing new Collaborative Intelligence project".len()).yellow());
    println!();
    
    // Validate project name
    if project_name.is_empty() {
        println!("{} {}", "✗".red().bold(), "Project name cannot be empty".red());
        return Err(anyhow!("Project name cannot be empty"));
    }
    
    // Check if project directory already exists
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_dir = current_dir.join(project_name);
    
    if project_dir.exists() {
        println!("{} {}", "✗".red().bold(), format!("Directory '{}' already exists", project_name).red());
        println!("Use '{}' to add CI capabilities to an existing project", "ci integrate".cyan());
        return Err(anyhow!("Project directory already exists"));
    }
    
    // Always use standalone integration
    let _integration_type = IntegrationType::Standalone;
    
    // Create project directory
    println!("{} Creating project directory: {}", "→".cyan(), project_dir.display());
    fs::create_dir_all(&project_dir)
        .with_context(|| format!("Failed to create project directory: {}", project_name))?;
    
    // Initialize git repository
    println!("{} Initializing git repository", "→".cyan());
    let git_result = Command::new("git")
        .args(&["init"])
        .current_dir(&project_dir)
        .output();
        
    match git_result {
        Ok(_) => println!("{} {}", "✓".green().bold(), "Git repository initialized".green()),
        Err(e) => println!("{} {}", "!".yellow().bold(), format!("Failed to initialize git repository: {}", e).yellow()),
    }
    
    // Create .gitignore
    println!("{} Creating .gitignore", "→".cyan());
    let gitignore_content = r#".collaborative-intelligence.json
.ci-config.json
.env
.ci/
CLAUDE.local.md
"#;
    
    match fs::write(project_dir.join(".gitignore"), gitignore_content) {
        Ok(_) => println!("{} {}", "✓".green().bold(), "Created .gitignore".green()),
        Err(e) => println!("{} {}", "!".yellow().bold(), format!("Failed to create .gitignore: {}", e).yellow()),
    }
    
    // Create standard directory structure
    println!("{} Creating standard project structure", "→".cyan());
    let _ = fs::create_dir_all(project_dir.join("src"));
    let _ = fs::create_dir_all(project_dir.join("docs"));
    let _ = fs::create_dir_all(project_dir.join("tests"));
    println!("{} {}", "✓".green().bold(), "Created project directories".green());
    
    // Set up standalone integration
    println!("{} Creating CLAUDE.md configuration", "→".cyan());
    if let Err(e) = setup_standalone_integration(&project_dir, agents, !no_fast) {
        println!("{} {}", "✗".red().bold(), format!("Failed to create CLAUDE.md: {}", e).red());
        return Err(e);
    }
    println!("{} {}", "✓".green().bold(), "Created CLAUDE.md".green());
    
    // Create initial README.md
    println!("{} Creating README.md", "→".cyan());
    let readme_content = format!(r#"# {}

A project configured with Collaborative Intelligence in Rust.

## Getting Started

This project is integrated with the CI tool for AI-powered development assistance.

### Available Commands

- `ci verify` - Verify the CI integration is working correctly
- `ci agents` - List available AI agents
- `ci load <agent>` - Load a specific agent for assistance
- `ci status` - Check project status
- `ci help` - Show all available commands

## Project Structure

```
{}
```

## Development

This project uses Collaborative Intelligence for enhanced development workflow.
See CLAUDE.md for AI assistant configuration.

## License

[Add your license information here]
"#, project_name, project_name);

    match fs::write(project_dir.join("README.md"), readme_content) {
        Ok(_) => println!("{} {}", "✓".green().bold(), "Created README.md".green()),
        Err(e) => println!("{} {}", "!".yellow().bold(), format!("Failed to create README.md: {}", e).yellow()),
    }
    
    // Display completion message
    println!();
    println!("{} {}", "✓".green().bold(), format!("Successfully initialized '{}' with Collaborative Intelligence!", project_name.bold()).green());
    println!();
    println!("Next steps:");
    println!("  1. cd {}", project_name);
    println!("  2. ci verify     (verify the integration)");
    println!("  3. ci agents     (list available agents)");
    println!("  4. ci load <agent>  (start working with an agent)");
    
    Ok(())
}

/// Set up embedded integration (CI files directly in project)
#[allow(dead_code)]
fn setup_embedded_integration(
    project_path: &Path,
    ci_path: &Path,
    agents: &str,
    fast_activation: bool
) -> Result<()> {
    // Create CLAUDE.md with CI integration
    let claude_md_content = format!(
    r#"# Project: {}
# Created: {}
# Integration: Embedded

# Load CollaborativeIntelligence System
When starting, immediately:
1. Load {}/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system
"#, 
        project_path.file_name().unwrap_or_default().to_string_lossy(), 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
        ci_path.display()
    );
    
    let claude_md_path = project_path.join("CLAUDE.md");
    fs::write(&claude_md_path, claude_md_content)
        .with_context(|| format!("Failed to write CLAUDE.md file: {}", claude_md_path.display()))?;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create CI configuration
    let config_content = format!(
    r#"{{
  "integration_type": "embedded",
  "repository_path": "{}",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "home_repository_path": "{}",
  "allowed_operations": ["memory_read", "agent_consult", "project_development"]
}}"#, 
        ci_path.display(),
        agents_json,
        fast_activation,
        ci_path.display()
    );
    
    let config_path = project_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
    
    CommandHelpers::print_success("Embedded integration setup completed");
    Ok(())
}

/// Set up symlink integration (CI files symlinked from central repo)
#[allow(dead_code)]
fn setup_symlink_integration(
    project_path: &Path,
    ci_path: &Path,
    agents: &str,
    fast_activation: bool
) -> Result<()> {
    // Create symlink to CLAUDE.md in CI repository
    let claude_md_source = ci_path.join("CLAUDE.md");
    if !claude_md_source.exists() {
        return Err(anyhow!("CI repository CLAUDE.md not found at: {}", claude_md_source.display()));
    }
    
    let claude_md_path = project_path.join("CLAUDE.md");
    
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&claude_md_source, &claude_md_path)
            .with_context(|| format!("Failed to create symlink from {} to {}", 
                claude_md_source.display(), claude_md_path.display()))?;
    }
    
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(&claude_md_source, &claude_md_path)
            .with_context(|| format!("Failed to create symlink from {} to {}", 
                claude_md_source.display(), claude_md_path.display()))?;
    }
    
    // Create a .ci directory for local context
    let cir_dir = project_path.join(".ci");
    fs::create_dir_all(&cir_dir)?;
    
    // Create a local CI_PROJECT.md file with project-specific info
    let project_md_content = format!(
    r#"# Project: {}
# Created: {}
# Integration: Symlink

This project is integrated with the Collaborative Intelligence system using symlinks.
The CLAUDE.md file is a symbolic link to the main CI repository.

## Project-Specific Configuration
- Agents: {}
- Fast activation: {}
"#, 
        project_path.file_name().unwrap_or_default().to_string_lossy(), 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        agents,
        fast_activation
    );
    
    fs::write(cir_dir.join("CI_PROJECT.md"), project_md_content)
        .with_context(|| "Failed to write CI_PROJECT.md file")?;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create CI configuration
    let config_content = format!(
    r#"{{
  "integration_type": "symlink",
  "repository_path": "{}",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "home_repository_path": "{}",
  "allowed_operations": ["memory_read", "agent_consult", "project_development"],
  "symlink_source": "{}"
}}"#, 
        ci_path.display(),
        agents_json,
        fast_activation,
        ci_path.display(),
        claude_md_source.display()
    );
    
    let config_path = project_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
    
    CommandHelpers::print_success("Symlink integration setup completed");
    Ok(())
}

/// Set up sibling integration (CI files in sibling directory)
#[allow(dead_code)]
fn setup_sibling_integration(
    project_path: &Path,
    ci_path: &Path,
    agents: &str,
    fast_activation: bool
) -> Result<()> {
    // Create a .ci directory that will hold a copy of essential CI files
    let cir_dir = project_path.join(".ci");
    fs::create_dir_all(&cir_dir)?;
    
    // Copy CLAUDE.md from CI repo to local .ci directory
    let claude_md_source = ci_path.join("CLAUDE.md");
    if !claude_md_source.exists() {
        return Err(anyhow!("CI repository CLAUDE.md not found at: {}", claude_md_source.display()));
    }
    
    let claude_md_content = fs::read_to_string(&claude_md_source)
        .with_context(|| format!("Failed to read CLAUDE.md from CI repository: {}", claude_md_source.display()))?;
    
    fs::write(cir_dir.join("CLAUDE.md"), &claude_md_content)
        .with_context(|| "Failed to write CLAUDE.md to .ci directory")?;
    
    // Create a main CLAUDE.md in the project root that references the .ci version
    let project_claude_md = format!(
    r#"# Project: {}
# Created: {}
# Integration: Sibling

# Load CollaborativeIntelligence System
When starting, immediately:
1. Load .ci/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system

# Project-Specific Configuration
- Agents: {}
- Fast activation: {}
"#, 
        project_path.file_name().unwrap_or_default().to_string_lossy(), 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        agents,
        fast_activation
    );
    
    fs::write(project_path.join("CLAUDE.md"), project_claude_md)
        .with_context(|| "Failed to write project CLAUDE.md file")?;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create CI configuration
    let config_content = format!(
    r#"{{
  "integration_type": "sibling",
  "repository_path": "{}",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "home_repository_path": "{}",
  "allowed_operations": ["memory_read", "agent_consult", "project_development"],
  "sibling_directory": ".ci"
}}"#, 
        ci_path.display(),
        agents_json,
        fast_activation,
        ci_path.display()
    );
    
    let config_path = project_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
    
    // Create a local version file to track updates
    let version_content = format!(
    r#"{{
  "ci_version": "1.0.0",
  "last_sync": "{}",
  "source_path": "{}"
}}"#,
        chrono::Local::now().to_rfc3339(),
        ci_path.display()
    );
    
    fs::write(cir_dir.join("version.json"), version_content)
        .with_context(|| "Failed to write version file")?;
    
    CommandHelpers::print_success("Sibling integration setup completed");
    Ok(())
}

pub async fn integrate(
    path: &Path,
    agents: &str,
    integration: &str,
    no_fast: bool,
    _ci_path_override: Option<&Path>, // Kept for backward compatibility but ignored
    config: &Config,
) -> Result<()> {
    // Get absolute path to target directory
    let target_path = PathBuf::from(path);
    if !target_path.exists() {
        return Err(anyhow!("Error: Target directory '{}' does not exist", path.display()));
    }
    
    // Get project name from directory
    let project_name = target_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    
    CommandHelpers::print_command_header(
        &format!("Integrating CI into project: {}", project_name), 
        "🚀", 
        "Project Lifecycle", 
        "yellow"
    );
    
    CommandHelpers::print_info(&format!("Target directory: {}", path.display()));
    
    // Determine integration type
    let integration_type = match integration.to_lowercase().as_str() {
        "override" => {
            CommandHelpers::print_info("Using override integration - preserves existing CLAUDE.md");
            IntegrationType::Override
        },
        _ => {
            // Default to standalone for any other value
            CommandHelpers::print_info("Using standalone integration");
            IntegrationType::Standalone
        }
    };
    
    CommandHelpers::print_info(&format!("Integration: {} ({})", integration_type.as_str(), integration_type.description()));
    CommandHelpers::print_info(&format!("Agents: {}", agents));
    CommandHelpers::print_info(&format!("Fast activation: {}", !no_fast));
    
    // Set up integration based on type
    match integration_type {
        IntegrationType::Override => {
            use crate::helpers::integration_manager::IntegrationManager;
            
            // Create array of agents
            let agent_list = agents.split(',')
                .map(|a| a.trim().to_string())
                .filter(|a| !a.is_empty())
                .collect::<Vec<String>>();
                
            // Integrate with override
            IntegrationManager::integrate_with_override(&target_path, &config.ci_path, &agent_list, !no_fast)?
        },
        _ => {
            // Use standalone integration
            integrate_standalone(&target_path, agents, !no_fast)?
        }
    };
    
    // Add appropriate entries to gitignore
    let gitignore_path = target_path.join(".gitignore");
    let mut gitignore_entries = vec![".collaborative-intelligence.json", ".ci-config.json", ".env", ".ci/"];
    
    // Add CLAUDE.i.md to gitignore if using override integration
    if integration_type == IntegrationType::Override {
        gitignore_entries.push("CLAUDE.i.md.bak");
    }
    
    if gitignore_path.exists() {
        let mut gitignore_content = fs::read_to_string(&gitignore_path)?;
        let mut updated = false;
        
        for entry in gitignore_entries {
            if !gitignore_content.contains(entry) {
                if !gitignore_content.ends_with('\n') {
                    gitignore_content.push('\n');
                }
                gitignore_content.push_str(&format!("{}\n", entry));
                updated = true;
            }
        }
        
        if updated {
            fs::write(&gitignore_path, gitignore_content)?;
            CommandHelpers::print_info("Updated .gitignore with CI entries");
        }
    } else {
        let content = gitignore_entries.join("\n") + "\n";
        fs::write(&gitignore_path, content)?;
        CommandHelpers::print_info("Created .gitignore with CI entries");
    }
    
    CommandHelpers::print_success(&format!("CI integrated into project: {}", project_name.bold()));
    println!("");
    println!("To get started:");
    println!("  cd {}", if path.display().to_string() == "." { "this directory".to_string() } else { path.display().to_string() });
    println!("  claude");
    println!("");
    println!("Then try: '{}' or '{}'", "Athena".blue().bold(), "Recommend an agent".blue().bold());
    
    Ok(())
}

/// Integrate using embedded method (CI files directly in project)
#[allow(dead_code)]
fn integrate_embedded(
    target_path: &Path,
    ci_path: &Path,
    agents: &str,
    fast_activation: bool
) -> Result<()> {
    // Create CLAUDE.local.md with CI integration
    let project_name = target_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
        
    let claude_md_content = format!(
    r#"# Project: {}
# Integrated: {}
# Integration: Embedded

# Load CollaborativeIntelligence System
When starting, immediately:
1. Load {}/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
        ci_path.display()
    );
    
    let claude_md_path = target_path.join("CLAUDE.local.md");
    fs::write(&claude_md_path, claude_md_content)
        .with_context(|| format!("Failed to write CLAUDE.local.md file: {}", claude_md_path.display()))?;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create CI configuration
    let config_content = format!(
    r#"{{
  "integration_type": "embedded",
  "repository_path": "{}",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "home_repository_path": "{}",
  "allowed_operations": ["memory_read", "agent_consult", "project_development"]
}}"#, 
        ci_path.display(),
        agents_json,
        fast_activation,
        ci_path.display()
    );
    
    let config_path = target_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
    
    CommandHelpers::print_success("Embedded integration completed");
    Ok(())
}

/// Integrate using symlink method (CI files symlinked from central repo)
#[allow(dead_code)]
fn integrate_symlink(
    target_path: &Path,
    ci_path: &Path,
    agents: &str,
    fast_activation: bool
) -> Result<()> {
    // Create CLAUDE.local.md with CI integration
    let project_name = target_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
        
    let claude_md_content = format!(
    r#"# Project: {}
# Integrated: {}
# Integration: Symlink

# THIS IS A FALLBACK FILE. The main configuration is symlinked as CLAUDE.md.
# If the symlink doesn't work, this file will be used instead.
# 
# Load CollaborativeIntelligence System
When starting, immediately:
1. Load {}/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
        ci_path.display()
    );
    
    let claude_local_md_path = target_path.join("CLAUDE.local.md");
    fs::write(&claude_local_md_path, claude_md_content)
        .with_context(|| format!("Failed to write CLAUDE.local.md file: {}", claude_local_md_path.display()))?;
    
    // Create symlink to CLAUDE.md in CI repository
    let claude_md_source = ci_path.join("CLAUDE.md");
    if !claude_md_source.exists() {
        return Err(anyhow!("CI repository CLAUDE.md not found at: {}", claude_md_source.display()));
    }
    
    let claude_md_target = target_path.join("CLAUDE.md");
    // Remove existing CLAUDE.md if it exists
    if claude_md_target.exists() {
        if fs::metadata(&claude_md_target)?.file_type().is_symlink() {
            // Remove existing symlink
            fs::remove_file(&claude_md_target)?;
            CommandHelpers::print_info("Removed existing CLAUDE.md symlink");
        } else {
            // Rename existing file to CLAUDE.md.bak
            let backup_path = target_path.join("CLAUDE.md.bak");
            fs::rename(&claude_md_target, &backup_path)?;
            CommandHelpers::print_info(&format!("Renamed existing CLAUDE.md to {}", backup_path.display()));
        }
    }
    
    // Create symlink
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&claude_md_source, &claude_md_target)
            .with_context(|| format!("Failed to create symlink from {} to {}", 
                claude_md_source.display(), claude_md_target.display()))?;
    }
    
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(&claude_md_source, &claude_md_target)
            .with_context(|| format!("Failed to create symlink from {} to {}", 
                claude_md_source.display(), claude_md_target.display()))?;
    }
    
    CommandHelpers::print_info(&format!("Created symlink to {} as CLAUDE.md", claude_md_source.display()));
    
    // Create a .ci directory for local context
    let cir_dir = target_path.join(".ci");
    fs::create_dir_all(&cir_dir)?;
    
    // Create a local CI_PROJECT.md file with project-specific info
    let project_md_content = format!(
    r#"# Project: {}
# Integrated: {}
# Integration: Symlink

This project is integrated with the Collaborative Intelligence system using symlinks.
The CLAUDE.md file is a symbolic link to the main CI repository.

## Project-Specific Configuration
- Agents: {}
- Fast activation: {}
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        agents,
        fast_activation
    );
    
    fs::write(cir_dir.join("CI_PROJECT.md"), project_md_content)
        .with_context(|| "Failed to write CI_PROJECT.md file")?;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create CI configuration
    let config_content = format!(
    r#"{{
  "integration_type": "symlink",
  "repository_path": "{}",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "home_repository_path": "{}",
  "allowed_operations": ["memory_read", "agent_consult", "project_development"],
  "symlink_source": "{}"
}}"#, 
        ci_path.display(),
        agents_json,
        fast_activation,
        ci_path.display(),
        claude_md_source.display()
    );
    
    let config_path = target_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
    
    CommandHelpers::print_success("Symlink integration completed");
    Ok(())
}

/// Integrate using sibling method (CI files in sibling directory)
#[allow(dead_code)]
fn integrate_sibling(
    target_path: &Path,
    ci_path: &Path,
    agents: &str,
    fast_activation: bool
) -> Result<()> {
    // Create a .ci directory that will hold a copy of essential CI files
    let project_name = target_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
        
    let cir_dir = target_path.join(".ci");
    fs::create_dir_all(&cir_dir)?;
    
    // Copy CLAUDE.md from CI repo to local .ci directory
    let claude_md_source = ci_path.join("CLAUDE.md");
    if !claude_md_source.exists() {
        return Err(anyhow!("CI repository CLAUDE.md not found at: {}", claude_md_source.display()));
    }
    
    let claude_md_content = fs::read_to_string(&claude_md_source)
        .with_context(|| format!("Failed to read CLAUDE.md from CI repository: {}", claude_md_source.display()))?;
    
    fs::write(cir_dir.join("CLAUDE.md"), &claude_md_content)
        .with_context(|| "Failed to write CLAUDE.md to .ci directory")?;
    
    CommandHelpers::print_info(&format!("Copied CI repository CLAUDE.md to {}", cir_dir.join("CLAUDE.md").display()));
    
    // Create CLAUDE.local.md with CI integration
    let claude_local_md_content = format!(
    r#"# Project: {}
# Integrated: {}
# Integration: Sibling

# THIS IS A FALLBACK FILE. The main configuration is in the .ci directory.
# 
# Load CollaborativeIntelligence System
When starting, immediately:
1. Load .ci/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    
    let claude_local_md_path = target_path.join("CLAUDE.local.md");
    fs::write(&claude_local_md_path, claude_local_md_content)
        .with_context(|| format!("Failed to write CLAUDE.local.md file: {}", claude_local_md_path.display()))?;
    
    // Create a main CLAUDE.md in the project root that references the .ci version
    // Check if there's an existing CLAUDE.md
    let claude_md_path = target_path.join("CLAUDE.md");
    if claude_md_path.exists() {
        // Create a backup
        let backup_path = target_path.join("CLAUDE.md.bak");
        fs::rename(&claude_md_path, &backup_path)?;
        CommandHelpers::print_info(&format!("Renamed existing CLAUDE.md to {}", backup_path.display()));
    }
    
    let project_claude_md = format!(
    r#"# Project: {}
# Integrated: {}
# Integration: Sibling

# Load CollaborativeIntelligence System
When starting, immediately:
1. Load .ci/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system

# Project-Specific Configuration
- Agents: {}
- Fast activation: {}
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        agents,
        fast_activation
    );
    
    fs::write(claude_md_path, project_claude_md)
        .with_context(|| "Failed to write project CLAUDE.md file")?;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create CI configuration
    let config_content = format!(
    r#"{{
  "integration_type": "sibling",
  "repository_path": "{}",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "home_repository_path": "{}",
  "allowed_operations": ["memory_read", "agent_consult", "project_development"],
  "sibling_directory": ".ci"
}}"#, 
        ci_path.display(),
        agents_json,
        fast_activation,
        ci_path.display()
    );
    
    let config_path = target_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
    
    // Create a local version file to track updates
    let version_content = format!(
    r#"{{
  "ci_version": "1.0.0",
  "last_sync": "{}",
  "source_path": "{}"
}}"#,
        chrono::Local::now().to_rfc3339(),
        ci_path.display()
    );
    
    fs::write(cir_dir.join("version.json"), version_content)
        .with_context(|| "Failed to write version file")?;
    
    CommandHelpers::print_success("Sibling integration completed");
    Ok(())
}

pub async fn fix(
    path: &Path,
    fix_type: Option<&str>,
    _config: &Config,
) -> Result<()> {
    // Handle specific fix types
    if let Some(fix_type) = fix_type {
        if fix_type == "verify" {
            println!("CI has built-in verification functionality.");
            println!("{} CI verification is handled internally", "✓".green());
            println!("Use 'ci verify .' to test your integration");
            return Ok(());
        }
    }
    
    // Get absolute path to target directory
    let target_path = PathBuf::from(path);
    if !target_path.exists() {
        return Err(anyhow!("Error: Target directory '{}' does not exist", path.display()));
    }
    
    // Get project name from directory
    let project_name = target_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    
    println!("Repairing CI integration for project: {}", project_name.bold());
    
    let ci_path = &_config.ci_path;
    println!("CI Repository: {}", ci_path.display());
    
    // Create the CLAUDE.local.md file with correct absolute paths
    let claude_md_content = format!(
    r#"# Project: {}
# Integrated: {}

# Load CollaborativeIntelligence System
When starting, immediately:
1. Load {}/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
        ci_path.display()
    );
    
    let claude_md_path = target_path.join("CLAUDE.local.md");
    fs::write(&claude_md_path, claude_md_content)?;
    println!("{} Created CLAUDE.local.md with correct absolute paths", "✓".green());
    
    // Default values
    let agents = "Athena,ProjectArchitect";
    let integration_type = "embedded";
    let fast_activation = true;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create correct .collaborative-intelligence.json configuration
    let config_content = format!(
    r#"{{
  "integration_type": "{}",
  "repository_path": "{}",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "home_repository_path": "{}",
  "allowed_operations": ["memory_read", "agent_consult", "project_development"]
}}"#, 
        integration_type, 
        ci_path.display(),
        agents_json,
        fast_activation,
        ci_path.display()
    );
    
    let config_path = target_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)?;
    println!("{} Created .collaborative-intelligence.json with correct configuration", "✓".green());
    
    // Add to gitignore if it exists
    let gitignore_path = target_path.join(".gitignore");
    if gitignore_path.exists() {
        let gitignore_content = fs::read_to_string(&gitignore_path)?;
        if !gitignore_content.contains(".collaborative-intelligence.json") {
            let new_content = format!("{}
.collaborative-intelligence.json", gitignore_content);
            fs::write(&gitignore_path, new_content)?;
            println!("{} Added configuration to .gitignore", "✓".green());
        }
    } else {
        fs::write(&gitignore_path, ".collaborative-intelligence.json
")?;
        println!("{} Created .gitignore file", "✓".green());
    }
    
    // Update or create .env file with CI_PATH
    let env_path = target_path.join(".env");
    let env_content = format!("CI_PATH={}", ci_path.display());
    
    if env_path.exists() {
        let existing_env = fs::read_to_string(&env_path)?;
        if existing_env.contains("CI_PATH=") || existing_env.contains("CI_REPO_PATH=") {
            // Replace existing CI_PATH or CI_REPO_PATH
            let new_env = existing_env.lines()
                .map(|line| {
                    if line.starts_with("CI_PATH=") || line.starts_with("CI_REPO_PATH=") {
                        &env_content
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(&env_path, new_env)?;
            println!("{} Updated CI_PATH in .env file", "✓".green());
        } else {
            // Add CI_PATH to existing .env file
            let new_env = format!("{}
{}", existing_env, env_content);
            fs::write(&env_path, new_env)?;
            println!("{} Added CI_PATH to .env file", "✓".green());
        }
    } else {
        // Create new .env file with CI_PATH
        fs::write(&env_path, env_content)?;
        println!("{} Created .env file with CI_PATH", "✓".green());
        
        // Add .env to gitignore if it's not already there
        if gitignore_path.exists() {
            let gitignore_content = fs::read_to_string(&gitignore_path)?;
            if !gitignore_content.contains(".env") {
                let new_content = format!("{}
.env", gitignore_content);
                fs::write(&gitignore_path, new_content)?;
                println!("{} Added .env to .gitignore", "✓".green());
            }
        }
    }
    
    println!("\n{} CI integration fixed for {}", "✓".green(), project_name.bold());
    println!("To verify integration, run:");
    println!("  ci verify {}", path.display());
    
    Ok(())
}

pub async fn verify(
    path: &Path,
    _config: &Config,
) -> Result<()> {
    println!("{}", "Verifying Collaborative Intelligence integration".green().bold());
    println!("{}", "==========================================".green());
    println!();
    
    // Determine project directory
    let project_dir = PathBuf::from(path);
    
    // Check if project directory exists
    if !project_dir.exists() || !project_dir.is_dir() {
        println!("{}", format!("Error: Directory '{}' does not exist", project_dir.display()).red());
        return Ok(());
    }
    
    // Run a series of verification checks
    let mut issues_found = false;
    
    // Check for CLAUDE.md
    issues_found |= !check_claude_md(&project_dir);
    
    // Check for CLAUDE.local.md
    issues_found |= !check_claude_local_md(&project_dir);
    
    // Check git repository
    issues_found |= !check_git_repository(&project_dir);
    
    // Check for proper gitignore entries
    issues_found |= !check_gitignore(&project_dir);
    
    println!();
    if issues_found {
        println!("{}", "Verification completed with issues".yellow().bold());
        println!("Use '{}'", "ci fix".cyan());
        println!("to attempt to resolve identified issues.");
    } else {
        println!("{}", "Verification successful!".green().bold());
        println!("Your Collaborative Intelligence integration appears to be working correctly.");
    }
    
    Ok(())
}

/// Check if CLAUDE.md exists and has required sections or references CLAUDE.CI.md
fn check_claude_md(project_dir: &Path) -> bool {
    let claude_md_path = project_dir.join("CLAUDE.md");
    let claude_ci_md_path = project_dir.join("CLAUDE.CI.md");
    
    println!("Checking for CLAUDE.md and CLAUDE.CI.md...");
    
    if !claude_md_path.exists() {
        CommandHelpers::print_status_error("CLAUDE.md not found");
        return false;
    }
    
    // Read and check content
    let content = match fs::read_to_string(&claude_md_path) {
        Ok(content) => content,
        Err(e) => {
            CommandHelpers::print_status_error(&format!("Failed to read CLAUDE.md: {}", e));
            return false;
        }
    };
    
    // Check if we're using the CLAUDE.CI.md approach
    let has_ci_load = content.contains("_CI.load('CLAUDE.CI.md')_");
    let has_ci_return = content.contains("_CI.return_to('CLAUDE.CI.md')_");
    
    if has_ci_load && has_ci_return {
        // Check for CLAUDE.CI.md file
        if !claude_ci_md_path.exists() {
            CommandHelpers::print_status_error("CLAUDE.md references CLAUDE.CI.md, but it doesn't exist");
            return false;
        }
        
        // Check CLAUDE.CI.md content
        let ci_content = match fs::read_to_string(&claude_ci_md_path) {
            Ok(content) => content,
            Err(e) => {
                CommandHelpers::print_status_error(&format!("Failed to read CLAUDE.CI.md: {}", e));
                return false;
            }
        };
        
        // Check for essential CI sections
        let has_ci_config = ci_content.contains("## CI Configuration") || 
                           ci_content.contains("### Integration Type");
        
        if !has_ci_config {
            CommandHelpers::print_status_warning("CLAUDE.CI.md is missing required CI configuration sections");
            return false;
        }
        
        CommandHelpers::print_status_check("CLAUDE.md correctly references CLAUDE.CI.md with CI configuration");
        return true;
    } else {
        // Using the traditional approach, check for essential sections
        let has_project_section = content.contains("# Project:") || content.contains("# Project ");
        let has_configuration = content.contains("## Configuration") || content.contains("Configuration");
        
        if !has_project_section || !has_configuration {
            CommandHelpers::print_status_warning("CLAUDE.md is missing required sections");
            return false;
        }
        
        CommandHelpers::print_status_check("CLAUDE.md exists and contains required sections");
        return true;
    }
}

/// Check if CLAUDE.local.md exists with proper CI references
fn check_claude_local_md(project_dir: &Path) -> bool {
    let claude_local_md_path = project_dir.join("CLAUDE.local.md");
    
    println!("Checking for CLAUDE.local.md...");
    
    if !claude_local_md_path.exists() {
        CommandHelpers::print_status_warning("CLAUDE.local.md not found");
        println!("    Use '{}' to create the local configuration file", "ci local".cyan());
        return false;
    }
    
    // Read and check content
    let content = match fs::read_to_string(&claude_local_md_path) {
        Ok(content) => content,
        Err(e) => {
            CommandHelpers::print_status_error(&format!("Failed to read CLAUDE.local.md: {}", e));
            return false;
        }
    };
    
    // Check for essential sections
    let has_project_section = content.contains("# Project:") || content.contains("# Project ");
    let has_integrated = content.contains("# Integrated:") || content.contains("# Updated:");
    let has_ci_references = content.contains("Load") && 
                          content.contains("CollaborativeIntelligence") && 
                          content.contains("system");
    
    if !has_project_section || !has_integrated || !has_ci_references {
        CommandHelpers::print_status_warning("CLAUDE.local.md is missing required CI references");
        println!("    Use '{}' to update the local configuration file", "ci local".cyan());
        return false;
    }
    
    CommandHelpers::print_status_check("CLAUDE.local.md exists with proper CI references");
    return true;
}

/// Check if git repository is initialized
fn check_git_repository(project_dir: &Path) -> bool {
    let git_dir = project_dir.join(".git");
    println!("Checking git repository...");
    
    if !git_dir.exists() || !git_dir.is_dir() {
        CommandHelpers::print_status_error("Not a git repository");
        return false;
    }
    
    // Check git status
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_dir)
        .output();
        
    match status {
        Ok(_) => {
            CommandHelpers::print_status_check("Git repository is properly initialized");
            true
        },
        Err(e) => {
            CommandHelpers::print_status_error(&format!("Git repository error: {}", e));
            false
        }
    }
}

/// Check for proper .gitignore entries
fn check_gitignore(project_dir: &Path) -> bool {
    let gitignore_path = project_dir.join(".gitignore");
    println!("Checking .gitignore configuration...");
    
    if !gitignore_path.exists() {
        CommandHelpers::print_status_warning(".gitignore not found");
        return false;
    }
    
    // Read and check content
    let content = match fs::read_to_string(&gitignore_path) {
        Ok(content) => content,
        Err(e) => {
            CommandHelpers::print_status_error(&format!("Failed to read .gitignore: {}", e));
            return false;
        }
    };
    
    // Check for CI-specific entries
    let has_ci_entries = content.contains("# Collaborative Intelligence") ||
                        content.contains("CLAUDE.local.md") || 
                        content.contains(".ci/") ||
                        content.contains(".ci-config.json");
    
    if !has_ci_entries {
        CommandHelpers::print_status_warning(".gitignore missing CI-specific entries");
        return false;
    }
    
    CommandHelpers::print_status_check(".gitignore contains required CI-specific entries");
    true
}

pub async fn local(
    path: &Path,
    _config: &Config,
) -> Result<()> {
    CommandHelpers::print_command_header(
        &format!("Create/update CLAUDE.local.md for {}", path.display()), 
        "🚀", 
        "Project Lifecycle", 
        "yellow"
    );
    
    // Get absolute path to target directory
    let target_path = PathBuf::from(path);
    if !target_path.exists() {
        return Err(anyhow!("Error: Target directory '{}' does not exist", path.display()));
    }
    
    // Get project name from directory
    let project_name = target_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    
    // Check for existing CLAUDE.local.md
    let claude_local_md_path = target_path.join("CLAUDE.local.md");
    let existing = claude_local_md_path.exists();
    
    if existing {
        CommandHelpers::print_info(&format!("Updating existing CLAUDE.local.md file for project: {}", project_name.bold()));
    } else {
        CommandHelpers::print_info(&format!("Creating new CLAUDE.local.md file for project: {}", project_name.bold()));
    }
    
    let ci_path = &_config.ci_path;
    CommandHelpers::print_status(&format!("CI Repository: {}", ci_path.display()));
    
    // Create CLAUDE.local.md with CI integration
    let claude_md_content = format!(
    r#"# Project: {}
# Updated: {}

# Load CollaborativeIntelligence System
When starting, immediately:
1. Load {}/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system

# Project-Specific Configuration
The following settings are specific to this project:

## Project Context
This project is integrated with the Collaborative Intelligence system using the CI CLI tool.
All Claude Code sessions will have access to the CI agents and capabilities.

## Active Agents
Default agents for this project:
- Athena (Primary system agent)
- ProjectArchitect (Project structure and planning)

## Custom Instructions
- Focus on code quality and maintainability
- Follow established patterns in the codebase
- Add appropriate error handling for all edge cases
- Include helpful comments for complex sections
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
        ci_path.display()
    );
    
    // Write the file
    fs::write(&claude_local_md_path, claude_md_content)
        .with_context(|| format!("Failed to write CLAUDE.local.md file: {}", claude_local_md_path.display()))?;
    
    CommandHelpers::print_success(&format!("CLAUDE.local.md file {} for project: {}", 
        if existing { "updated" } else { "created" }, 
        project_name.bold()
    ));
    
    // Check for .env file to ensure CI_PATH is set
    let env_path = target_path.join(".env");
    
    if !env_path.exists() {
        // Create new .env file with CI_PATH
        let env_content = format!("CI_PATH={}", ci_path.display());
        fs::write(&env_path, env_content)
            .with_context(|| format!("Failed to write .env file"))?;
        
        CommandHelpers::print_info("Created .env file with CI_PATH");
        
        // Add .env to gitignore if it exists
        let gitignore_path = target_path.join(".gitignore");
        if gitignore_path.exists() {
            let gitignore_content = fs::read_to_string(&gitignore_path)?;
            if !gitignore_content.contains(".env") {
                let new_content = format!("{}
.env", gitignore_content);
                fs::write(&gitignore_path, new_content)?;
                CommandHelpers::print_info("Added .env to .gitignore");
            }
        } else {
            fs::write(&gitignore_path, ".env
")?;
            CommandHelpers::print_info("Created .gitignore file with .env entry");
        }
    } else {
        // Update existing .env file if needed
        let existing_env = fs::read_to_string(&env_path)?;
        let env_var = format!("CI_PATH={}", ci_path.display());
        
        if existing_env.contains("CI_PATH=") || existing_env.contains("CI_REPO_PATH=") {
            // Replace existing CI_PATH or CI_REPO_PATH
            let new_env = existing_env.lines()
                .map(|line| {
                    if line.starts_with("CI_PATH=") || line.starts_with("CI_REPO_PATH=") {
                        &env_var
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            if new_env != existing_env {
                fs::write(&env_path, new_env)?;
                CommandHelpers::print_info("Updated CI_PATH in .env file");
            }
        } else {
            // Add CI_PATH to existing .env file
            let new_env = format!("{}
{}", existing_env, env_var);
            fs::write(&env_path, new_env)?;
            CommandHelpers::print_info("Added CI_PATH to .env file");
        }
    }
    
    println!();
    CommandHelpers::print_info("To use CLAUDE.local.md:");
    CommandHelpers::print_status("1. Restart any running Claude Code sessions");
    CommandHelpers::print_status("2. New sessions will automatically load this configuration");
    
    Ok(())
}

/// Migrate from CI to CI standalone mode
pub async fn migrate(
    path: &Path,
    backup: bool,
    detect_only: bool,
    verbose: bool,
    _config: &Config,
) -> Result<()> {
    // Get absolute path to target directory
    let target_path = PathBuf::from(path);
    if !target_path.exists() {
        return Err(anyhow!("Error: Target directory '{}' does not exist", path.display()));
    }
    
    // Get project name from directory
    let project_name = target_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
        
    CommandHelpers::print_command_header(
        &format!("CI to CI Migration for {}", project_name), 
        "🚀", 
        "Project Lifecycle", 
        "yellow"
    );
    
    // Detect existing CI integration
    CommandHelpers::print_status("Detecting CI integration...");
    let detection_result = crate::tools::ci_migration::detect_ci_integration(&target_path)?;
    
    if !detection_result.detected {
        println!();
        CommandHelpers::print_info(&format!("No CI integration detected in {}.", path.display().to_string().yellow()));
        println!("To integrate CI in standalone mode, run:");
        println!("  ci integrate {} --integration standalone", path.display());
        return Ok(());
    }
    
    // Print detection summary
    println!();
    CommandHelpers::print_success("CI integration detected!");
    detection_result.print_summary();
    
    // If detect_only flag is set, stop here
    if detect_only {
        println!();
        CommandHelpers::print_info("Detection completed. No migration performed (--detect-only flag was set).");
        println!("To perform the migration, run the command without the --detect-only flag:");
        println!("  ci migrate {}", path.display());
        return Ok(());
    }
    
    // Confirm migration
    println!();
    CommandHelpers::print_info("Ready to migrate from CI to CI standalone mode.");
    println!("This will make your project fully independent from the CI repository.");
    println!("Backups of existing files will {} created.", if backup { "be".green() } else { "NOT be".yellow() });
    
    // Ask for confirmation
    use std::io::{stdin, stdout, Write};
    print!("\nProceed with migration? [Y/n] ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let input = input.trim();
    
    if !input.is_empty() && input.to_lowercase() != "y" && input.to_lowercase() != "yes" {
        println!("Migration cancelled by user.");
        return Ok(());
    }
    
    // Perform migration
    println!();
    CommandHelpers::print_status("Performing migration...");
    crate::tools::ci_migration::migrate_to_cir(&target_path, &detection_result, backup, verbose)?;
    
    Ok(())
}

/// Set up standalone integration (fully independent from CI repository)
fn setup_standalone_integration(
    project_path: &Path,
    agents: &str,
    fast_activation: bool
) -> Result<()> {
    
    
    
    // Create a .ci directory for CI-specific files
    let cir_dir = project_path.join(".ci");
    fs::create_dir_all(&cir_dir)?;
    
    // Create a standalone CLAUDE.md file with CI directives
    let claude_md_content = format!(
    r#"# Project: {}
# Created: {}
# Integration: Standalone

# Collaborative Intelligence Rust Configuration
This project is configured to use the CI (Collaborative Intelligence in Rust) system
in standalone mode, which provides full functionality without requiring the original
CI repository.

_CI.config('project_name', '{}')_
_CI.config('created_at', '{}')_
_CI.config('integration_type', 'standalone')_

## Available Agents
_CI.load_agents('{}')_

## Project Context
This project uses the standalone CI integration, which processes directives
directly within the CI tool. No external CI repository is required.

## Custom Instructions
- Focus on code quality and maintainability
- Follow established patterns in the codebase
- Add appropriate error handling for all edge cases
- Include helpful comments for complex sections
"#, 
        project_path.file_name().unwrap_or_default().to_string_lossy(), 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        project_path.file_name().unwrap_or_default().to_string_lossy(),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        agents
    );
    
    let claude_md_path = project_path.join("CLAUDE.md");
    fs::write(&claude_md_path, claude_md_content)
        .with_context(|| format!("Failed to write CLAUDE.md file: {}", claude_md_path.display()))?;
    
    // Create a metadata.json file in the .ci directory
    let metadata = serde_json::json!({
        "project_name": project_path.file_name().unwrap_or_default().to_string_lossy(),
        "created_at": chrono::Local::now().to_rfc3339(),
        "integration_type": "standalone",
        "cir_version": env!("CARGO_PKG_VERSION"),
        "active_agents": agents.split(',').collect::<Vec<_>>(),
        "fast_activation": fast_activation
    });
    
    fs::write(cir_dir.join("metadata.json"), serde_json::to_string_pretty(&metadata)?)
        .with_context(|| "Failed to write metadata.json file")?;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create CI configuration
    let config_content = format!(
    r#"{{
  "integration_type": "standalone",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "cir_version": "{}",
  "standalone_processor": true,
  "allowed_operations": ["memory_read", "agent_consult", "project_development"]
}}"#, 
        agents_json,
        fast_activation,
        env!("CARGO_PKG_VERSION")
    );
    
    let config_path = project_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
    
    // Create default agent files for specified agents
    let agents_dir = cir_dir.join("agents");
    fs::create_dir_all(&agents_dir)?;
    
    for agent in agents.split(',') {
        let agent_name = agent.trim();
        if agent_name.is_empty() {
            continue;
        }
        
        // Create a basic agent template
        create_default_agent_file(&agents_dir, agent_name)?;
    }
    
    CommandHelpers::print_success("Standalone integration setup completed");
    Ok(())
}

/// Integrate using standalone method (fully independent of CI repository)
fn integrate_standalone(
    target_path: &Path,
    agents: &str,
    fast_activation: bool
) -> Result<()> {
    
    
    // Get project name from directory
    let project_name = target_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    
    // Create a .ci directory for CI-specific files
    let cir_dir = target_path.join(".ci");
    fs::create_dir_all(&cir_dir)?;
    
    // Create a standalone CLAUDE.md file with CI directives
    let claude_md_content = format!(
    r#"# Project: {}
# Created: {}
# Integration: Standalone

# Collaborative Intelligence Rust Configuration
This project is configured to use the CI (Collaborative Intelligence in Rust) system
in standalone mode, which provides full functionality without requiring the original
CI repository.

_CI.config('project_name', '{}')_
_CI.config('created_at', '{}')_
_CI.config('integration_type', 'standalone')_

## Available Agents
_CI.load_agents('{}')_

## Project Context
This project uses the standalone CI integration, which processes directives
directly within the CI tool. No external CI repository is required.

## Custom Instructions
- Focus on code quality and maintainability
- Follow established patterns in the codebase
- Add appropriate error handling for all edge cases
- Include helpful comments for complex sections
"#, 
        project_name, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        project_name,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        agents
    );
    
    // Check for existing CLAUDE.md
    let claude_md_path = target_path.join("CLAUDE.md");
    if claude_md_path.exists() {
        // Create a backup
        let backup_path = target_path.join("CLAUDE.md.bak");
        fs::rename(&claude_md_path, &backup_path)?;
        CommandHelpers::print_info(&format!("Renamed existing CLAUDE.md to {}", backup_path.display()));
    }
    
    fs::write(&claude_md_path, claude_md_content)
        .with_context(|| format!("Failed to write CLAUDE.md file: {}", claude_md_path.display()))?;
    
    // Create a metadata.json file in the .ci directory
    let metadata = serde_json::json!({
        "project_name": project_name,
        "created_at": chrono::Local::now().to_rfc3339(),
        "integration_type": "standalone",
        "cir_version": env!("CARGO_PKG_VERSION"),
        "active_agents": agents.split(',').collect::<Vec<_>>(),
        "fast_activation": fast_activation
    });
    
    fs::write(cir_dir.join("metadata.json"), serde_json::to_string_pretty(&metadata)?)
        .with_context(|| "Failed to write metadata.json file")?;
    
    // Convert comma-separated list to proper JSON array
    let agents_json = format!("[\"{}\"]", agents.replace(',', "\",\""));
    
    // Create CI configuration
    let config_content = format!(
    r#"{{
  "integration_type": "standalone",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "cir_version": "{}",
  "standalone_processor": true,
  "allowed_operations": ["memory_read", "agent_consult", "project_development"]
}}"#, 
        agents_json,
        fast_activation,
        env!("CARGO_PKG_VERSION")
    );
    
    let config_path = target_path.join(".collaborative-intelligence.json");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
    
    // Create default agent files for specified agents
    let agents_dir = cir_dir.join("agents");
    fs::create_dir_all(&agents_dir)?;
    
    for agent in agents.split(',') {
        let agent_name = agent.trim();
        if agent_name.is_empty() {
            continue;
        }
        
        // Create a basic agent template
        create_default_agent_file(&agents_dir, agent_name)?;
    }
    
    CommandHelpers::print_success("Standalone integration completed");
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