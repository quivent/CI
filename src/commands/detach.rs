use anyhow::Result;

use crate::config::Config;
use crate::helpers::CommandHelpers;
use crate::helpers::path::PathHelpers;
use crate::helpers::integration_manager::IntegrationManager;

/// Detach CI integration from a project
pub async fn detach(
    path: &Option<String>,
    keep_override: bool,
    __config: &Config
) -> Result<()> {
    CommandHelpers::print_command_header(
        "Detach CI Integration", 
        "🔓", 
        "Project Management", 
        "yellow"
    );
    
    // Resolve project path
    let project_dir = match PathHelpers::resolve_project_path(path) {
        Ok(path) => path,
        Err(e) => {
            CommandHelpers::print_error(&format!("Invalid path: {}", e));
            return Err(e);
        }
    };
    
    CommandHelpers::print_info(&format!("Target directory: {}", project_dir.display()));
    
    // Check if CLAUDE.i.md exists to determine if we have override integration
    if !IntegrationManager::has_claude_i_md(&project_dir) {
        CommandHelpers::print_warning("Project doesn't appear to use override integration.");
        CommandHelpers::print_info("This command is meant for projects using the 'override' integration type.");
        CommandHelpers::print_info("If you want to completely remove CI integration, use 'ci uninstall' instead.");
        return Ok(());
    }
    
    // Check if CLAUDE.md exists
    if !IntegrationManager::has_claude_md(&project_dir) {
        CommandHelpers::print_warning("No CLAUDE.md file found. Nothing to detach from.");
        return Ok(());
    }
    
    // Detach the integration
    IntegrationManager::detach_integration(&project_dir)?;
    
    CommandHelpers::print_success("CI integration successfully detached.");
    CommandHelpers::print_info("The CLAUDE.md file no longer refers to the CI system.");
    
    if keep_override {
        CommandHelpers::print_info("The override file (CLAUDE.i.md) has been renamed to CLAUDE.i.md.bak.");
        CommandHelpers::print_info("You can restore the integration by renaming it back to CLAUDE.i.md");
        CommandHelpers::print_info("and adding the directive to CLAUDE.md.");
    } else {
        // If not keeping the override, suggest removing the backup file
        CommandHelpers::print_info("You may want to remove the backup file with:");
        CommandHelpers::print_status(&format!("rm {}", project_dir.join("CLAUDE.i.md.bak").display()));
    }
    
    Ok(())
}