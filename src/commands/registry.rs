use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::fs;

use crate::config::Config;
use crate::helpers::CommandHelpers;
use crate::helpers::path::PathHelpers;
use crate::helpers::project::{ProjectHelpers, ProjectInfo};
use crate::helpers::repository::RepositoryHelpers;

/// Registry subcommands
#[derive(Debug, Clone)]
pub enum RegistryCommand {
    /// List all registered projects
    List,
    /// Register a project with CI
    Register { path: Option<String> },
    /// Unregister a project from CI
    Unregister { name: String },
    /// Show details about a specific project
    Show { name: String },
    /// Display project registry statistics
    Stats,
}

/// Command to manage the project registry
pub async fn registry(command: &Option<RegistryCommand>, __config: &Config) -> Result<()> {
    println!("{}", "🧠 Project Registry".blue().bold());
    println!("{}", "================".blue());
    println!();
    
    // Get CI repository path
    let cir_repo_path = PathHelpers::get_cir_repository_path(&None)?;
    
    match command {
        Some(RegistryCommand::List) => {
            println!("📋 {}", "Fetching project list...".cyan());
            println!();
            list_projects(&cir_repo_path).await?;
        },
        Some(RegistryCommand::Register { path }) => {
            println!("📝 {}", "Registering project...".cyan());
            println!();
            register_project(path, &cir_repo_path).await?;
        },
        Some(RegistryCommand::Unregister { name }) => {
            println!("🗑️  {}", format!("Preparing to unregister project: {}", name.yellow()).cyan());
            println!();
            unregister_project(name, &cir_repo_path).await?;
        },
        Some(RegistryCommand::Show { name }) => {
            println!("🔍 {}", format!("Retrieving details for project: {}", name.yellow()).cyan());
            println!();
            show_project_details(name, &cir_repo_path).await?;
        },
        Some(RegistryCommand::Stats) => {
            println!("📊 {}", "Analyzing registry statistics...".cyan());
            println!();
            show_registry_stats(&cir_repo_path).await?;
        },
        None => {
            // Default to list
            println!("📋 {}", "Fetching project list (default)...".cyan());
            println!();
            list_projects(&cir_repo_path).await?;
        }
    }
    
    Ok(())
}

/// List all registered projects
async fn list_projects(cir_repo_path: &Path) -> Result<()> {
    println!("{}", "📚 Registered Projects".blue().bold());
    println!("{}", "-------------------".blue());
    println!();
    
    let projects = ProjectHelpers::list_registered_projects(cir_repo_path)?;
    
    if projects.is_empty() {
        println!("{} {}", "ℹ️".blue(), "No projects are currently registered".italic());
        println!();
        println!("{} {}", "💡".yellow(), "To register a project, use:".bold());
        println!("   {}", "ci registry register".cyan());
        return Ok(());
    }
    
    // Sort projects by name
    let mut sorted_projects = projects;
    sorted_projects.sort_by(|a, b| a.name.cmp(&b.name));
    
    let project_count = sorted_projects.len();
    
    for project in sorted_projects {
        println!("{} {}", "📁".green(), project.name.green().bold());
        println!("   {}: {}", "Integration".yellow(), project.integration_type);
        println!("   {}: {}", "Agents".yellow(), project.agents.join(", "));
        println!();
    }
    
    println!("{} {}", "✅".green(), format!("Found {} registered projects", project_count).green().bold());
    
    Ok(())
}

/// Register a project with CI
async fn register_project(path: &Option<String>, cir_repo_path: &Path) -> Result<()> {
    // Resolve project path
    let project_dir = PathHelpers::resolve_project_path(path)?;
    
    println!("🔍 {}", format!("Analyzing project at: {}", project_dir.display().to_string().cyan()).yellow());
    println!();
    
    // Check if it's a CI project
    if !ProjectHelpers::is_cir_project(&project_dir) {
        println!("{}", "❌ Not a CI Project".red().bold());
        println!("{}", "=================".red());
        println!();
        println!("The directory does not contain a valid CI configuration.");
        println!();
        println!("{} {}", "💡".yellow(), "Run this command first:".bold());
        println!("   {}", "ci integrate".cyan());
        
        return Err(anyhow::anyhow!("Not a CI project"));
    }
    
    // Get project info
    let project_info = ProjectHelpers::get_project_info(&project_dir)?;
    
    println!("📋 {}", "Project details:".blue().bold());
    println!("   {}: {}", "Name".yellow(), project_info.name);
    println!("   {}: {}", "Path".yellow(), project_dir.display());
    println!("   {}: {}", "Integration".yellow(), project_info.integration_type);
    println!("   {}: {}", "Agents".yellow(), project_info.agents.join(", "));
    println!();
    
    println!("⏳ {}", format!("Registering project: {}", project_info.name).cyan());
    
    // Register the project
    ProjectHelpers::register_project(&project_dir, cir_repo_path)?;
    
    println!();
    println!("{} {}", "✅".green(), format!("Project '{}' registered successfully", project_info.name).green().bold());
    println!();
    println!("{} {}", "💡".yellow(), "You can now reference this project in any CI command".italic());
    
    Ok(())
}

/// Unregister a project from CI
async fn unregister_project(name: &str, cir_repo_path: &Path) -> Result<()> {
    let projects_dir = cir_repo_path.join("Projects");
    let project_path = projects_dir.join(name);
    
    println!("🔍 {}", format!("Checking registry for project: {}", name.cyan()).yellow());
    
    if !project_path.exists() {
        println!("{}", "❌ Project Not Found".red().bold());
        println!("{}", "=================".red());
        println!();
        println!("The project '{}' is not registered in the CI registry.", name.red());
        println!();
        println!("{} {}", "💡".yellow(), "To see registered projects, run:".bold());
        println!("   {}", "ci registry list".cyan());
        
        return Err(anyhow::anyhow!("Project not registered"));
    }
    
    println!("{}", "⚠️  Unregister Confirmation".yellow().bold());
    println!("{}", "=======================".yellow());
    println!();
    println!("This will unregister project '{}' from the CI registry.", name.yellow().bold());
    println!("The project files will not be affected, only the registry entry will be removed.");
    println!();
    
    print!("{}️ Are you sure you want to unregister this project? (y/N): ", "⚠️".yellow());
    std::io::stdout().flush()?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
        println!("{} {}", "ℹ️".blue(), "Operation cancelled".blue());
        return Ok(());
    }
    
    println!();
    println!("🗑️  {}", format!("Unregistering project: {}", name).cyan());
    
    // Remove the project
    if project_path.is_symlink() {
        fs::remove_file(&project_path)
    } else {
        fs::remove_dir_all(&project_path)
    }.with_context(|| format!("Failed to remove project registry entry at {}", project_path.display()))?;
    
    println!();
    println!("{} {}", "✅".green(), format!("Project '{}' unregistered successfully", name).green().bold());
    
    Ok(())
}

/// Show details about a specific project
async fn show_project_details(name: &str, cir_repo_path: &Path) -> Result<()> {
    let projects = ProjectHelpers::list_registered_projects(cir_repo_path)?;
    
    let project = projects.iter().find(|p| p.name == name);
    
    if project.is_none() {
        println!("{}", "❌ Project Not Found".red().bold());
        println!("{}", "=================".red());
        println!();
        println!("The project '{}' is not registered in the CI registry.", name.red());
        println!();
        println!("{} {}", "💡".yellow(), "To see registered projects, run:".bold());
        println!("   {}", "ci registry list".cyan());
        
        return Err(anyhow::anyhow!("Project not found"));
    }
    
    let project = project.unwrap();
    
    // Get project path from registry
    let projects_dir = cir_repo_path.join("Projects");
    let project_link = projects_dir.join(name);
    
    // Get actual project path
    let actual_path = if project_link.is_symlink() {
        fs::read_link(&project_link)
            .with_context(|| format!("Failed to read symlink at {}", project_link.display()))?
    } else {
        // For Windows, check for .project-path file
        let path_file = project_link.join(".project-path");
        if path_file.exists() {
            let path_content = fs::read_to_string(&path_file)
                .with_context(|| format!("Failed to read project path file at {}", path_file.display()))?;
            Path::new(&path_content).to_path_buf()
        } else {
            project_link.clone()
        }
    };
    
    // Print detailed project info
    println!("{}", "📊 Project Details".blue().bold());
    println!("{}", "================".blue());
    println!();
    println!("{}: {}", "Project".yellow().bold(), project.name.green().bold());
    println!("{}: {}", "Path".yellow(), actual_path.display().to_string().cyan());
    println!("{}: {}", "Integration".yellow(), project.integration_type);
    println!("{}: {}", "Config".yellow(), project.config_path.display().to_string().cyan());
    
    if let Some(created) = &project.created {
        println!("{}: {}", "Created".yellow(), created);
    }
    
    // Check git status
    if RepositoryHelpers::is_inside_git_repo(&actual_path) {
        let status = RepositoryHelpers::get_repository_status(&actual_path)?;
        println!();
        println!("{}", "🌿 Git Status".green().bold());
        println!("{}", "-----------".green());
        RepositoryHelpers::display_status(&status);
    }
    
    // Show agents
    println!();
    println!("{}", "🤖 Agents".blue().bold());
    println!("{}", "--------".blue());
    for agent in &project.agents {
        println!("  - {}", agent.cyan());
    }
    
    // Show project stats
    if let Ok(stats) = ProjectHelpers::get_project_stats(&actual_path) {
        println!();
        println!("{}", "📈 Project Statistics".yellow().bold());
        println!("{}", "------------------".yellow());
        println!("  {}: {}", "Total Files".cyan(), stats.file_count);
        println!("  {}: {} bytes", "Total Size".cyan(), stats.total_size);
        
        if stats.rust_files > 0 {
            println!("  {}: {}", "Rust Files".cyan(), stats.rust_files);
        }
        if stats.js_files > 0 {
            println!("  {}: {}", "JS/TS Files".cyan(), stats.js_files);
        }
        if stats.python_files > 0 {
            println!("  {}: {}", "Python Files".cyan(), stats.python_files);
        }
        if stats.doc_files > 0 {
            println!("  {}: {}", "Documentation Files".cyan(), stats.doc_files);
        }
    }
    
    println!();
    println!("{} {}", "✅".green(), format!("Project details for '{}' displayed successfully", name).green().bold());
    
    Ok(())
}

/// Display project registry statistics
async fn show_registry_stats(cir_repo_path: &Path) -> Result<()> {
    let projects = ProjectHelpers::list_registered_projects(cir_repo_path)?;
    
    println!("{}", "📊 Registry Statistics".blue().bold());
    println!("{}", "===================".blue());
    println!();
    
    // Basic stats
    println!("{}: {}", "Total Projects".yellow().bold(), projects.len().to_string().green());
    
    // Count projects by integration type
    let mut integration_counts = std::collections::HashMap::new();
    for project in &projects {
        *integration_counts.entry(&project.integration_type).or_insert(0) += 1;
    }
    
    println!();
    println!("{}", "🔄 Integration Types".cyan().bold());
    println!("{}", "------------------".cyan());
    for (integration, count) in integration_counts {
        println!("  {}: {}", integration.green(), count);
    }
    
    // Count agent usage
    let mut agent_counts = std::collections::HashMap::new();
    for project in &projects {
        for agent in &project.agents {
            *agent_counts.entry(agent).or_insert(0) += 1;
        }
    }
    
    println!();
    println!("{}", "🤖 Agent Usage".green().bold());
    println!("{}", "------------".green());
    let mut agent_counts: Vec<_> = agent_counts.into_iter().collect();
    agent_counts.sort_by(|a, b| b.1.cmp(&a.1));
    
    for (agent, count) in agent_counts {
        println!("  {}: {}", agent.cyan(), count);
    }
    
    // Add some extra stats for more visual appeal
    let total_agents: usize = projects.iter().map(|p| p.agents.len()).sum();
    
    println!();
    println!("{}", "📈 Additional Statistics".yellow().bold());
    println!("{}", "-----------------------".yellow());
    println!("  {}: {}", "Average agents per project".cyan(), 
        if projects.is_empty() { 
            "0".to_string() 
        } else { 
            format!("{:.2}", total_agents as f64 / projects.len() as f64) 
        }
    );
    
    println!();
    println!("{} {}", "✅".green(), "Registry statistics generated successfully".green().bold());
    
    Ok(())
}