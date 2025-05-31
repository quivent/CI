use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::errors::CIError;
use crate::helpers::agent_autoload::AgentAutoload;

pub fn create_command() -> Command {
    Command::new("init")
        .about("Initialize a new project with Collaborative Intelligence integration")
        .arg(
            Arg::new("project_name")
                .help("Name of the project to create")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("agents")
                .long("agents")
                .value_name("AGENT_LIST")
                .help("Comma-separated list of agents to enable (e.g., Athena,Optimizer)")
        )
        .arg(
            Arg::new("template")
                .short('t')
                .long("template")
                .value_name("TEMPLATE")
                .help("Project template to use")
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Directory to create the project in")
        )
        .arg(
            Arg::new("no-git")
                .long("no-git")
                .action(clap::ArgAction::SetTrue)
                .help("Don't initialize git repository")
        )
        .arg(
            Arg::new("standalone")
                .long("standalone")
                .action(clap::ArgAction::SetTrue)
                .help("Create standalone CI integration")
        )
}

pub fn execute(matches: &ArgMatches) -> Result<()> {
    let project_name = matches.get_one::<String>("project_name").unwrap();
    let agents = matches.get_one::<String>("agents");
    let template = matches.get_one::<String>("template");
    let custom_path = matches.get_one::<String>("path");
    let no_git = matches.get_flag("no-git");
    let standalone = matches.get_flag("standalone");
    
    initialize_project(project_name, agents, template, custom_path, no_git, standalone)
}

fn initialize_project(
    project_name: &str,
    agents: Option<&String>,
    template: Option<&String>,
    custom_path: Option<&String>,
    no_git: bool,
    standalone: bool,
) -> Result<()> {
    println!("{}", format!("Creating project: {}", project_name).cyan().bold());
    println!("{}", "=".repeat(40).cyan());
    
    // Determine project directory
    let project_dir = if let Some(path) = custom_path {
        PathBuf::from(path).join(project_name)
    } else {
        std::env::current_dir()?.join(project_name)
    };
    
    // Check if directory already exists
    if project_dir.exists() {
        return Err(CIError::AlreadyExists(format!(
            "Directory '{}' already exists",
            project_dir.display()
        )).into());
    }
    
    // Create project directory
    fs::create_dir_all(&project_dir)
        .with_context(|| format!("Failed to create project directory: {}", project_dir.display()))?;
    
    println!("{} Created project directory: {}", "✓".green(), project_dir.display());
    
    // Create basic directory structure
    create_project_structure(&project_dir)?;
    
    // Create CLAUDE.md configuration
    create_claude_config(&project_dir, project_name, standalone)?;
    
    // Initialize git repository
    if !no_git {
        initialize_git_repo(&project_dir)?;
    }
    
    // Apply template if specified
    if let Some(template_name) = template {
        apply_project_template(&project_dir, template_name)?;
    }
    
    // Enable specified agents
    if let Some(agent_list) = agents {
        enable_project_agents(&project_dir, agent_list)?;
    }
    
    // Create basic files
    create_basic_files(&project_dir, project_name)?;
    
    // Install agent activation hooks
    AgentAutoload::install_activation_hooks(&project_dir)?;
    
    println!();
    println!("{} Project '{}' created successfully!", "✓".green().bold(), project_name);
    println!();
    println!("To get started:");
    println!("  cd {}", project_name);
    println!("  claude");
    println!();
    
    if agents.is_some() {
        println!("Available agents:");
        if let Some(agent_list) = agents {
            for agent in agent_list.split(',') {
                println!("  - {}", agent.trim());
            }
        }
        println!();
    }
    
    println!("Try these commands:");
    println!("  'Athena' - Activate the knowledge architect");
    println!("  'ci agent list' - List all available agents");
    println!("  'Recommend an agent' - Get agent recommendations");
    
    Ok(())
}

fn create_project_structure(project_dir: &Path) -> Result<()> {
    let directories = ["src", "docs", "tests"];
    
    for dir in &directories {
        let dir_path = project_dir.join(dir);
        fs::create_dir_all(&dir_path)
            .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
    }
    
    println!("{} Created project structure (src, docs, tests)", "✓".green());
    Ok(())
}

fn create_claude_config(project_dir: &Path, project_name: &str, standalone: bool) -> Result<()> {
    let default_agents = vec!["Athena".to_string(), "ProjectArchitect".to_string()];
    
    let claude_content = if standalone {
        AgentAutoload::generate_unified_claude_md(
            project_name,
            "Standalone", 
            &default_agents
        )
    } else {
        AgentAutoload::generate_unified_claude_md(
            project_name,
            "External Repository",
            &default_agents
        )
    };
    
    let claude_path = project_dir.join("CLAUDE.md");
    fs::write(&claude_path, claude_content)
        .with_context(|| format!("Failed to create CLAUDE.md: {}", claude_path.display()))?;
    
    println!("{} Created CLAUDE.md configuration", "✓".green());
    Ok(())
}

fn initialize_git_repo(project_dir: &Path) -> Result<()> {
    let output = process::Command::new("git")
        .arg("init")
        .current_dir(project_dir)
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            println!("{} Initialized git repository", "✓".green());
            
            // Create .gitignore
            let gitignore_content = r#"# Dependencies
node_modules/
target/
*.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Environment
.env
.env.local

# CI Cache
.ci_cache/
"#;
            
            let gitignore_path = project_dir.join(".gitignore");
            fs::write(&gitignore_path, gitignore_content)
                .with_context(|| format!("Failed to create .gitignore: {}", gitignore_path.display()))?;
            
            println!("{} Created .gitignore", "✓".green());
        }
        Ok(_) => {
            println!("{} Git initialization failed (non-zero exit)", "⚠".yellow());
        }
        Err(_) => {
            println!("{} Git not available (repository not initialized)", "⚠".yellow());
        }
    }
    
    Ok(())
}

fn apply_project_template(project_dir: &Path, template_name: &str) -> Result<()> {
    // This would be expanded to handle actual templates
    println!("{} Applied template: {}", "✓".green(), template_name);
    
    match template_name {
        "rust" => create_rust_template(project_dir)?,
        "python" => create_python_template(project_dir)?,
        "web" => create_web_template(project_dir)?,
        "cli" => create_cli_template(project_dir)?,
        _ => {
            println!("{} Unknown template '{}', using default", "⚠".yellow(), template_name);
        }
    }
    
    Ok(())
}

fn create_rust_template(project_dir: &Path) -> Result<()> {
    // Create Cargo.toml
    let cargo_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
clap = {{ version = "4.0", features = ["derive"] }}
tokio = {{ version = "1.0", features = ["full"] }}
"#,
        project_dir.file_name().unwrap().to_str().unwrap()
    );
    
    fs::write(project_dir.join("Cargo.toml"), cargo_content)?;
    
    // Create main.rs
    let main_content = r#"use anyhow::Result;

fn main() -> Result<()> {
    println!("Hello from Collaborative Intelligence!");
    Ok(())
}
"#;
    
    fs::write(project_dir.join("src").join("main.rs"), main_content)?;
    
    println!("{} Created Rust project files", "✓".green());
    Ok(())
}

fn create_python_template(project_dir: &Path) -> Result<()> {
    // Create requirements.txt
    let requirements = "# Add your Python dependencies here\n";
    fs::write(project_dir.join("requirements.txt"), requirements)?;
    
    // Create main.py
    let main_content = r#"#!/usr/bin/env python3
"""
Collaborative Intelligence Project
"""

def main():
    print("Hello from Collaborative Intelligence!")

if __name__ == "__main__":
    main()
"#;
    
    fs::write(project_dir.join("src").join("main.py"), main_content)?;
    
    println!("{} Created Python project files", "✓".green());
    Ok(())
}

fn create_web_template(project_dir: &Path) -> Result<()> {
    // Create package.json
    let package_json = serde_json::json!({
        "name": project_dir.file_name().unwrap().to_str().unwrap(),
        "version": "0.1.0",
        "description": "Collaborative Intelligence Web Project",
        "main": "src/index.js",
        "scripts": {
            "start": "node src/index.js",
            "dev": "nodemon src/index.js"
        },
        "dependencies": {},
        "devDependencies": {
            "nodemon": "^3.0.0"
        }
    });
    
    fs::write(
        project_dir.join("package.json"),
        serde_json::to_string_pretty(&package_json)?
    )?;
    
    // Create index.js
    let index_content = r#"// Collaborative Intelligence Web Project
console.log("Hello from Collaborative Intelligence!");

// Your web application code here
"#;
    
    fs::write(project_dir.join("src").join("index.js"), index_content)?;
    
    // Create index.html
    let html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Collaborative Intelligence Project</title>
</head>
<body>
    <h1>Welcome to Collaborative Intelligence</h1>
    <p>Your web project is ready!</p>
</body>
</html>
"#;
    
    fs::write(project_dir.join("src").join("index.html"), html_content)?;
    
    println!("{} Created web project files", "✓".green());
    Ok(())
}

fn create_cli_template(project_dir: &Path) -> Result<()> {
    // Similar to Rust template but with CLI focus
    create_rust_template(project_dir)?;
    
    // Overwrite main.rs with CLI-specific content
    let main_content = r#"use anyhow::Result;
use clap::{Arg, Command};

fn main() -> Result<()> {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Collaborative Intelligence CLI Tool")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose output")
        )
        .get_matches();

    if matches.get_flag("verbose") {
        println!("Verbose mode enabled");
    }

    println!("Hello from Collaborative Intelligence CLI!");
    
    Ok(())
}
"#;
    
    fs::write(project_dir.join("src").join("main.rs"), main_content)?;
    
    println!("{} Created CLI project files", "✓".green());
    Ok(())
}

fn enable_project_agents(project_dir: &Path, agent_list: &str) -> Result<()> {
    let agents: Vec<&str> = agent_list.split(',').map(|s| s.trim()).collect();
    
    // For now, just record the agents in CLAUDE.md
    let claude_path = project_dir.join("CLAUDE.md");
    if claude_path.exists() {
        let mut content = fs::read_to_string(&claude_path)?;
        
        content.push_str("\n## Enabled Agents\n");
        for agent in &agents {
            content.push_str(&format!("- {}\n", agent));
        }
        
        fs::write(&claude_path, content)?;
    }
    
    println!("{} Enabled agents: {}", "✓".green(), agent_list);
    Ok(())
}

fn create_basic_files(project_dir: &Path, project_name: &str) -> Result<()> {
    // Create README.md
    let readme_content = format!(
        r#"# {}

A Collaborative Intelligence enabled project.

## Getting Started

This project is integrated with the Collaborative Intelligence system. You can use AI agents to help with development, analysis, and other tasks.

## Available Commands

- `ci agent list` - List available agents
- `ci agent activate <agent>` - Activate an agent
- `claude` - Start Claude Code session

## Agents

Use agents by typing their name in a Claude Code session:
- `Athena` - Knowledge architect and memory systems specialist
- `Architect` - System design specialist
- `Developer` - Implementation specialist

## Documentation

Documentation is stored in the `docs/` directory.

## Contributing

This project follows Collaborative Intelligence best practices for development.
"#,
        project_name
    );
    
    fs::write(project_dir.join("README.md"), readme_content)?;
    println!("{} Created README.md", "✓".green());
    
    // Create docs/README.md
    let docs_readme = r#"# Documentation

This directory contains project documentation.

## Structure

- `api/` - API documentation
- `guides/` - User and developer guides
- `architecture/` - System architecture documents

## Collaborative Intelligence

This project uses Collaborative Intelligence agents for documentation generation and maintenance.
"#;
    
    fs::write(project_dir.join("docs").join("README.md"), docs_readme)?;
    
    // Create tests/README.md
    let tests_readme = r#"# Tests

This directory contains project tests.

## Running Tests

Instructions for running tests will be added based on your project type.

## Collaborative Intelligence Testing

Use the `Tester` agent for test generation and validation:
- `Tester` - Comprehensive testing and quality validation specialist
"#;
    
    fs::write(project_dir.join("tests").join("README.md"), tests_readme)?;
    
    println!("{} Created documentation files", "✓".green());
    
    Ok(())
}