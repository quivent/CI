use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};
use std::fs;

use crate::config::Config;
use crate::helpers::CommandHelpers;
use crate::helpers::path::PathHelpers;
use crate::helpers::repository::RepositoryHelpers;
use crate::helpers::project::ProjectHelpers;

/// Integrate an agent into a project
pub async fn agent_integrate(
    agent_name: &str,
    path: &Option<String>,
    force: bool,
    __config: &Config
) -> Result<()> {
    println!("{}", "Integrate Collaborative Intelligence Agent".blue().bold());
    println!("{}", "=======================================".blue());
    println!();

    println!("Agent name: {}", agent_name.cyan().bold());
    
    // Resolve project path
    let project_dir = match PathHelpers::resolve_project_path(path) {
        Ok(path) => path,
        Err(e) => {
            println!("{} {}", "✗".red().bold(), format!("Invalid path: {}", e).red());
            return Err(e);
        }
    };
    
    // Get CI repository path
    let ci_repo_path = match PathHelpers::get_ci_repository_path(&None) {
        Ok(path) => path,
        Err(e) => {
            println!("{} {}", "✗".red().bold(), format!("Failed to locate CI repository: {}", e).red());
            return Err(e);
        }
    };
    
    // Check if project is CI-integrated
    if !ProjectHelpers::is_ci_project(&project_dir) {
        println!("{} {}", "✗".red().bold(), "Not a CI project. Run 'ci integrate' first.".red());
        return Err(anyhow::anyhow!("Not a CI project"));
    }
    
    // Check if agent exists
    let agent_path = ci_repo_path.join("AGENTS").join(agent_name);
    if !agent_path.exists() {
        // Check for agent in AGENTS.md
        let agents_md_path = ci_repo_path.join("AGENTS.md");
        if agents_md_path.exists() {
            let content = fs::read_to_string(&agents_md_path)
                .with_context(|| format!("Failed to read AGENTS.md at {}", agents_md_path.display()))?;
                
            // Quick check for the agent name in AGENTS.md
            if !content.contains(&format!("### {}", agent_name)) && 
               !content.contains(&format!("## {}", agent_name)) {
                println!("{} {}", "✗".red().bold(), format!("Agent '{}' not found", agent_name).red());
                return Err(anyhow::anyhow!("Agent not found"));
            }
        } else {
            println!("{} {}", "✗".red().bold(), format!("Agent '{}' not found", agent_name).red());
            return Err(anyhow::anyhow!("Agent not found"));
        }
    }
    
    // Get project info
    let mut project_info = ProjectHelpers::get_project_info(&project_dir)?;
    
    // Check if agent is already integrated
    if project_info.agents.contains(&agent_name.to_string()) && !force {
        println!("{} {}", "⚠".yellow().bold(), format!("Agent '{}' is already integrated. Use --force to update.", agent_name).yellow());
        return Ok(());
    }
    
    // Add agent to project
    if !project_info.agents.contains(&agent_name.to_string()) {
        project_info.agents.push(agent_name.to_string());
    }
    
    // Update CLAUDE.md
    print!("Updating project configuration... ");
    let result = (|| -> Result<()> {
        let claude_md_path = project_dir.join("CLAUDE.md");
        let claude_content = fs::read_to_string(&claude_md_path)
            .with_context(|| format!("Failed to read CLAUDE.md at {}", claude_md_path.display()))?;
            
        // Update the Active Agents section
        let agents_list = project_info.agents
            .iter()
            .map(|a| format!("- {}", a))
            .collect::<Vec<_>>()
            .join("\n");
            
        let section_header = "## Active Agents";
        let updated_content = if claude_content.contains(section_header) {
            // Replace existing section
            let lines: Vec<&str> = claude_content.lines().collect();
            let mut new_lines = Vec::new();
            let mut in_section = false;
            let mut _section_replaced = false;
            
            for line in lines {
                if line == section_header {
                    in_section = true;
                    new_lines.push(line);
                    new_lines.push("");
                    new_lines.push(&agents_list);
                    new_lines.push("");
                    _section_replaced = true;
                } else if in_section && line.starts_with("##") {
                    in_section = false;
                    new_lines.push(line);
                } else if !in_section {
                    new_lines.push(line);
                }
            }
            
            new_lines.join("\n")
        } else {
            // Append new section
            format!("{}\n\n{}\n{}\n", claude_content.trim_end(), section_header, agents_list)
        };
        
        fs::write(&claude_md_path, updated_content)
            .with_context(|| format!("Failed to write CLAUDE.md at {}", claude_md_path.display()))?;
        
        Ok(())
    })();
    
    match result {
        Ok(_) => println!("{}", "✓".green().bold()),
        Err(e) => {
            println!("{}", "✗".red().bold());
            return Err(e);
        }
    }
    
    // Create agent toolkit directory
    let project_agent_dir = project_dir.join("AGENTS").join(agent_name);
    if !project_agent_dir.exists() {
        print!("Creating agent toolkit directory for {}... ", agent_name);
        let result = fs::create_dir_all(&project_agent_dir)
            .with_context(|| format!("Failed to create agent toolkit directory at {}", project_agent_dir.display()));
            
        match result {
            Ok(_) => println!("{}", "✓".green().bold()),
            Err(e) => {
                println!("{}", "✗".red().bold());
                return Err(e);
            }
        }
    }
    
    // Create agent README
    let agent_readme_path = project_agent_dir.join("README.md");
    if !agent_readme_path.exists() {
        print!("Creating agent README... ");
        let result = (|| -> Result<()> {
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
                agent_name, agent_name, agent_name
            );
            
            fs::write(&agent_readme_path, readme_content)
                .with_context(|| format!("Failed to write agent README at {}", agent_readme_path.display()))?;
            
            Ok(())
        })();
        
        match result {
            Ok(_) => println!("{}", "✓".green().bold()),
            Err(e) => {
                println!("{}", "✗".red().bold());
                return Err(e);
            }
        }
    }
    
    // Copy agent memory if available
    let source_memory_path = agent_path.join(format!("{}_memory.md", agent_name));
    if source_memory_path.exists() {
        let target_memory_path = project_agent_dir.join("memory.md");
        if !target_memory_path.exists() || force {
            print!("Copying agent memory... ");
            let result = fs::copy(&source_memory_path, &target_memory_path)
                .with_context(|| format!("Failed to copy agent memory from {} to {}", 
                    source_memory_path.display(), target_memory_path.display()));
                    
            match result {
                Ok(_) => println!("{}", "✓".green().bold()),
                Err(e) => {
                    println!("{}", "✗".red().bold());
                    return Err(e);
                }
            }
        }
    }
    
    // Create prompts directory
    let prompts_dir = project_agent_dir.join("prompts");
    if !prompts_dir.exists() {
        print!("Creating prompts directory... ");
        let result = fs::create_dir_all(&prompts_dir)
            .with_context(|| format!("Failed to create prompts directory at {}", prompts_dir.display()));
            
        match result {
            Ok(_) => println!("{}", "✓".green().bold()),
            Err(e) => {
                println!("{}", "✗".red().bold());
                return Err(e);
            }
        }
    }
    
    // Create example prompt
    let example_prompt_path = prompts_dir.join("example.md");
    if !example_prompt_path.exists() {
        print!("Creating example prompt... ");
        let result = (|| -> Result<()> {
            let prompt_content = format!(
                r#"# Example Prompt for {}

## Context
This is an example prompt template for the {} agent.

## Instructions
1. Analyze the provided information
2. Apply domain-specific expertise
3. Generate a detailed response

## Input Format
```
[Input text here]
```

## Expected Output
```
[Output format here]
```
"#,
                agent_name, agent_name
            );
            
            fs::write(&example_prompt_path, prompt_content)
                .with_context(|| format!("Failed to write example prompt at {}", example_prompt_path.display()))?;
            
            Ok(())
        })();
        
        match result {
            Ok(_) => println!("{}", "✓".green().bold()),
            Err(e) => {
                println!("{}", "✗".red().bold());
                return Err(e);
            }
        }
    }
    
    // Check if we're in a git repo and offer to stage changes
    if RepositoryHelpers::is_inside_git_repo(&project_dir) {
        println!("{}", "Changes detected. Would you like to stage them?".cyan());
        if CommandHelpers::prompt_confirmation("Stage changes?") {
            print!("Staging changes... ");
            let result = (|| -> Result<()> {
                // Stage relevant files
                RepositoryHelpers::stage_files(&project_dir, "CLAUDE.md")?;
                RepositoryHelpers::stage_files(&project_dir, "AGENTS/")?;
                Ok(())
            })();
            
            match result {
                Ok(_) => {
                    println!("{}", "✓".green().bold());
                    println!("{} {}", "✓".green().bold(), "Changes staged".green());
                    println!("{}", "You can now commit these changes with:".cyan());
                    println!("  • ci commit -m \"Integrate agent: <message>\"");
                },
                Err(e) => {
                    println!("{}", "✗".red().bold());
                    return Err(e);
                }
            }
        }
    }
    
    println!("{} {}", "✓".green().bold(), format!("Agent '{}' integrated successfully", agent_name).green());
    println!("{}", "To use this agent:".cyan());
    println!("  • ci load {}", agent_name);
    
    Ok(())
}