use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;

/// Handles integration of CI into projects
pub struct IntegrationManager;

impl IntegrationManager {
    /// Check if a CLAUDE.md file exists in the given path
    pub fn has_claude_md(path: &Path) -> bool {
        path.join("CLAUDE.md").exists()
    }

    /// Check if a CLAUDE.i.md file exists in the given path
    pub fn has_claude_i_md(path: &Path) -> bool {
        path.join("CLAUDE.i.md").exists()
    }

    /// Create a CI override file (CLAUDE.i.md) that refers to the CI system
    pub fn create_override_file(path: &Path, ci_repo_path: &Path) -> Result<()> {
        let project_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project");
            
        let claude_i_content = format!(
        r#"# CI Integration: {}
# Integrated: {}

# Override directives for CI integration
# This file is loaded by any existing CLAUDE.md file via a directive.

# Load CollaborativeIntelligence System
When starting, immediately:
1. Load {}/CLAUDE.md
2. Use this as the primary configuration source
3. Defer all project management functions to the CollaborativeIntelligence system
"#, 
            project_name, 
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
            ci_repo_path.display()
        );
        
        // Write the CLAUDE.i.md file
        let claude_i_path = path.join("CLAUDE.i.md");
        fs::write(&claude_i_path, claude_i_content)
            .with_context(|| format!("Failed to write CLAUDE.i.md file: {}", claude_i_path.display()))?;
        
        Ok(())
    }

    /// Add a directive to an existing CLAUDE.md file that refers to the override file
    pub fn add_override_directive(path: &Path) -> Result<()> {
        let claude_md_path = path.join("CLAUDE.md");
        
        if !claude_md_path.exists() {
            return Err(anyhow!("CLAUDE.md does not exist in {}", path.display()));
        }
        
        // Read existing content
        let content = fs::read_to_string(&claude_md_path)
            .with_context(|| format!("Failed to read CLAUDE.md at {}", claude_md_path.display()))?;
            
        // Check if the directive already exists
        if content.contains("_CI.load('CLAUDE.i.md')_") {
            return Ok(());  // Directive already exists, nothing to do
        }
        
        // Add the directive at the top of the file, after any existing headers
        let lines: Vec<&str> = content.lines().collect();
        let mut new_content = Vec::new();
        let mut added = false;
        let mut header_section = true;
        
        for line in lines {
            new_content.push(line);
            
            // Add directive after the headers, when we hit the first non-header content
            if header_section && !line.starts_with('#') && !line.is_empty() {
                new_content.push("");
                new_content.push("# Load CI Configuration");
                new_content.push("_CI.load('CLAUDE.i.md')_");
                new_content.push("");
                added = true;
                header_section = false;
            }
            
            // If the line starts with something other than a header, we've left the header section
            if header_section && !line.starts_with('#') && !line.is_empty() {
                header_section = false;
            }
        }
        
        // If we didn't add it yet (might be an empty file or just headers), add it at the end
        if !added {
            if !new_content.is_empty() {
                new_content.push("");
            }
            new_content.push("# Load CI Configuration");
            new_content.push("_CI.load('CLAUDE.i.md')_");
        }
        
        // Write the updated content
        let updated_content = new_content.join("\n");
        fs::write(&claude_md_path, updated_content)
            .with_context(|| format!("Failed to update CLAUDE.md at {}", claude_md_path.display()))?;
        
        Ok(())
    }

    /// Remove the override directive from CLAUDE.md
    pub fn remove_override_directive(path: &Path) -> Result<()> {
        let claude_md_path = path.join("CLAUDE.md");
        
        if !claude_md_path.exists() {
            return Err(anyhow!("CLAUDE.md does not exist in {}", path.display()));
        }
        
        // Read existing content
        let content = fs::read_to_string(&claude_md_path)
            .with_context(|| format!("Failed to read CLAUDE.md at {}", claude_md_path.display()))?;
            
        // Check if the directive exists
        if !content.contains("_CI.load('CLAUDE.i.md')_") {
            return Ok(());  // Directive doesn't exist, nothing to do
        }
        
        // Remove the directive and the header line above it
        let lines: Vec<&str> = content.lines().collect();
        let mut new_content = Vec::new();
        let mut skip_next = false;
        
        for line in lines {
            if skip_next {
                skip_next = false;
                continue;
            }
            
            if line == "# Load CI Configuration" {
                // Check the next line
                skip_next = true;
                continue;
            }
            
            if line == "_CI.load('CLAUDE.i.md')_" {
                // Skip this line
                continue;
            }
            
            new_content.push(line);
        }
        
        // Write the updated content
        let updated_content = new_content.join("\n");
        fs::write(&claude_md_path, updated_content)
            .with_context(|| format!("Failed to update CLAUDE.md at {}", claude_md_path.display()))?;
        
        Ok(())
    }

    /// Integrate CI into a project using the override approach
    pub fn integrate_with_override(
        target_path: &Path,
        ci_repo_path: &Path,
        agents: &[String],
        fast_activation: bool
    ) -> Result<()> {
        // Check if CLAUDE.md exists in the target path
        if !Self::has_claude_md(target_path) {
            // Create a minimal CLAUDE.md that loads CLAUDE.i.md
            let project_name = target_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("project");
                
            let claude_md_content = format!(
            r#"# Project: {}
# Created: {}

# Load CI Configuration
_CI.load('CLAUDE.i.md')_

# Project Information
Add your project-specific information here.

# End of file
_CI.return_to('CLAUDE.i.md')_
"#, 
                project_name, 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            
            let claude_md_path = target_path.join("CLAUDE.md");
            fs::write(&claude_md_path, claude_md_content)
                .with_context(|| format!("Failed to write CLAUDE.md file: {}", claude_md_path.display()))?;
                
            println!("{} {}", "✓".green().bold(), "Created CLAUDE.md with CI integration directive".green());
        } else {
            // Add the override directive to existing CLAUDE.md
            Self::add_override_directive(target_path)?;
            println!("{} {}", "✓".green().bold(), "Added CI integration directive to existing CLAUDE.md".green());
        }
        
        // Create the CLAUDE.i.md file with override directives
        Self::create_override_file(target_path, ci_repo_path)?;
        println!("{} {}", "✓".green().bold(), "Created CLAUDE.i.md with CI integration".green());
        
        // Create a .collaborative-intelligence.json file
        let _config = Self::create_config_json(target_path, ci_repo_path, agents, fast_activation)?;
        println!("{} {}", "✓".green().bold(), "Created .collaborative-intelligence.json configuration".green());
        
        // Create AGENTS directory if agents are specified
        if !agents.is_empty() {
            let agents_dir = target_path.join("AGENTS");
            if !agents_dir.exists() {
                fs::create_dir_all(&agents_dir)
                    .with_context(|| format!("Failed to create AGENTS directory at {}", agents_dir.display()))?;
                    
                println!("{} {}", "✓".green().bold(), "Created AGENTS directory".green());
            }
            
            // Create agent-specific directories
            for agent in agents {
                let agent_dir = agents_dir.join(agent);
                if !agent_dir.exists() {
                    fs::create_dir_all(&agent_dir)
                        .with_context(|| format!("Failed to create agent directory at {}", agent_dir.display()))?;
                        
                    // Create a basic README.md for the agent
                    let readme_content = format!(
                    r#"# Agent: {}

## Overview
This directory contains toolkit files for the {} agent in this project.

## Files
- `README.md` - This file
- `memory.md` - Custom memory file for the agent
- `prompts/` - Custom prompt templates

## Usage
To activate this agent, use:
```
ci load {}
```
"#,
                        agent, agent, agent
                    );
                    
                    fs::write(agent_dir.join("README.md"), readme_content)
                        .with_context(|| format!("Failed to create agent README at {}", agent_dir.join("README.md").display()))?;
                        
                    println!("{} {}", "✓".green().bold(), format!("Created agent directory for {}", agent).green());
                }
            }
        }
        
        Ok(())
    }

    /// Create a .collaborative-intelligence.json configuration file
    fn create_config_json(
        target_path: &Path,
        ci_repo_path: &Path,
        agents: &[String],
        fast_activation: bool
    ) -> Result<PathBuf> {
        // Convert agents to JSON array format
        let agents_json = serde_json::to_string(agents)
            .unwrap_or_else(|_| "[]".to_string());
            
        // Create config content
        let config_content = format!(
        r#"{{
  "integration_type": "override",
  "repository_path": "{}",
  "active_agents": {},
  "fast_activation": {},
  "repository_context": "project",
  "home_repository_path": "{}",
  "allowed_operations": ["memory_read", "agent_consult", "project_development"]
}}"#, 
            ci_repo_path.display(),
            agents_json,
            fast_activation,
            ci_repo_path.display()
        );
        
        let config_path = target_path.join(".collaborative-intelligence.json");
        fs::write(&config_path, config_content)
            .with_context(|| format!("Failed to write configuration file: {}", config_path.display()))?;
            
        Ok(config_path)
    }

    /// Detach CI integration by removing the override but keeping config
    pub fn detach_integration(target_path: &Path) -> Result<()> {
        // Check if the necessary files exist
        if !Self::has_claude_md(target_path) {
            return Err(anyhow!("CLAUDE.md does not exist in {}", target_path.display()));
        }
        
        if !Self::has_claude_i_md(target_path) {
            return Err(anyhow!("CLAUDE.i.md does not exist in {}", target_path.display()));
        }
        
        // Remove the override directive from CLAUDE.md
        Self::remove_override_directive(target_path)?;
        println!("{} {}", "✓".green().bold(), "Removed CI integration directive from CLAUDE.md".green());
        
        // Rename CLAUDE.i.md to CLAUDE.i.md.bak
        let claude_i_path = target_path.join("CLAUDE.i.md");
        let claude_i_bak_path = target_path.join("CLAUDE.i.md.bak");
        fs::rename(&claude_i_path, &claude_i_bak_path)
            .with_context(|| format!("Failed to rename CLAUDE.i.md to {}", claude_i_bak_path.display()))?;
            
        println!("{} {}", "✓".green().bold(), "Renamed CLAUDE.i.md to CLAUDE.i.md.bak".green());
        
        // Keep the .collaborative-intelligence.json file
        println!("{} {}", "✓".green().bold(), "Kept .collaborative-intelligence.json for future reference".green());
        
        Ok(())
    }
}