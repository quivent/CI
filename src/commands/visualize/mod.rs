use anyhow::Result;
use colored::*;
use crate::config::Config;
use crate::{VisualizationCommands, VisualizationFormat, VisualizationTheme};

mod overview;
mod commands;
mod agents;
mod workflows;
mod project;
mod ascii_art;
mod web_export;

pub use overview::*;
pub use commands::*;
pub use agents::*;
pub use workflows::*;
pub use project::*;
use ascii_art::AsciiArtGenerator;
use web_export::WebExporter;

/// Handle visualization commands
pub async fn handle_visualization_command(
    view: &VisualizationCommands,
    config: &Config,
) -> Result<()> {
    match view {
        VisualizationCommands::Overview { 
            format, 
            theme, 
            interactive, 
            export,
            save,
            web,
            svg,
            dark,
            light,
        } => {
            let final_format = resolve_format(format, *web, *svg);
            let final_theme = resolve_theme(theme, *dark, *light);
            
            show_overview(
                final_format,
                final_theme,
                *interactive,
                export.as_deref(),
                *save,
                config,
            ).await
        },
        VisualizationCommands::Commands { 
            format, 
            group, 
            tree, 
            interactive,
            save,
            web,
            svg,
            dark,
            light,
        } => {
            let final_format = resolve_format(format, *web, *svg);
            let final_theme = resolve_theme(&None, *dark, *light);
            
            show_commands(
                group.as_deref(),
                final_format,
                final_theme,
                *tree,
                *interactive,
                *save,
                config,
            ).await
        },
        VisualizationCommands::Agents { 
            format, 
            category, 
            network, 
            interactive,
            save,
            web,
            svg,
            dark,
            light,
        } => {
            let final_format = resolve_format(format, *web, *svg);
            let final_theme = resolve_theme(&None, *dark, *light);
            
            show_agents(
                category.as_deref(),
                final_format,
                final_theme,
                *network,
                *interactive,
                *save,
                config,
            ).await
        },
        VisualizationCommands::Workflows { 
            format, 
            beginner, 
            category,
            save,
            web,
            svg,
            dark,
            light,
        } => {
            let final_format = resolve_format(format, *web, *svg);
            let final_theme = resolve_theme(&None, *dark, *light);
            
            show_workflows(
                category.as_deref(),
                final_format,
                final_theme,
                *beginner,
                *save,
                config,
            ).await
        },
        VisualizationCommands::Project { 
            name, 
            format, 
            detailed,
            save,
            web,
            svg,
            dark,
            light,
        } => {
            let final_format = resolve_format(format, *web, *svg);
            let final_theme = resolve_theme(&None, *dark, *light);
            
            show_project(
                name.as_deref(),
                final_format,
                final_theme,
                *detailed,
                *save,
                config,
            ).await
        },
    }
}

/// Resolve the final format from explicit format flag and shortcut flags
fn resolve_format(
    format: &Option<VisualizationFormat>, 
    web: bool, 
    svg: bool
) -> VisualizationFormat {
    // Shortcut flags take precedence
    if web {
        VisualizationFormat::Web
    } else if svg {
        VisualizationFormat::Svg
    } else {
        // Use explicit format or default to Terminal
        format.clone().unwrap_or(VisualizationFormat::Terminal)
    }
}

/// Resolve the final theme from explicit theme flag and shortcut flags
fn resolve_theme(
    theme: &Option<VisualizationTheme>, 
    dark: bool, 
    light: bool
) -> VisualizationTheme {
    // Shortcut flags take precedence
    if dark {
        VisualizationTheme::Dark
    } else if light {
        VisualizationTheme::Light
    } else {
        // Use explicit theme or default to Dark
        theme.clone().unwrap_or(VisualizationTheme::Dark)
    }
}

/// Open a file in the default application (browser for HTML/SVG)
pub fn open_file(file_path: &str, file_type: &str, save: bool) {
    let open_result = std::process::Command::new("open")
        .arg(file_path)
        .output();
        
    match open_result {
        Ok(_) => {
            println!("{} Opening {} in default viewer", "✅".green(), file_type);
            if save {
                println!("{} File saved to: {}", "📄".blue(), file_path.cyan());
            } else {
                println!("{} Temporary file will be cleaned up after viewing", "🧹".blue());
            }
        },
        Err(_) => {
            println!("{} {} saved to: {}", "📄".blue(), file_type, file_path.cyan());
            println!("{} Open this file in your browser to view", "💡".yellow());
        }
    }
    
    // Auto-cleanup for temp files if not saving
    if !save && file_path.starts_with("/tmp/") {
        // Give some time for the file to open, then clean up
        let file_path_owned = file_path.to_string();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let _ = std::fs::remove_file(file_path_owned);
        });
    }
}