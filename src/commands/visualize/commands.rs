use anyhow::Result;
use colored::*;
use crate::config::Config;
use crate::{VisualizationFormat, VisualizationTheme};
use super::ascii_art::AsciiArtGenerator;
use super::web_export::WebExporter;

/// Display CI command structure visualization
pub async fn show_commands(
    group: Option<&str>,
    format: VisualizationFormat,
    theme: VisualizationTheme,
    tree: bool,
    interactive: bool,
    save: bool,
    config: &Config,
) -> Result<()> {
    match format {
        VisualizationFormat::Web => {
            let exporter = WebExporter::new(theme);
            let file_path = if save { "ci_commands.html" } else { "/tmp/ci_commands.html" };
            println!("{} Creating interactive commands visualization...", "🌐".blue());
            exporter.export_commands_html(file_path, group, tree, config).await?;
            super::open_file(file_path, "HTML visualization", save);
        },
        VisualizationFormat::Svg => {
            let exporter = WebExporter::new(theme);
            let file_path = if save { "ci_commands.svg" } else { "/tmp/ci_commands.svg" };
            println!("{} Creating commands SVG diagram...", "📊".blue());
            exporter.export_commands_svg(file_path, group, tree, config).await?;
            super::open_file(file_path, "SVG diagram", save);
        },
        _ => {
            let art_gen = AsciiArtGenerator::new(theme);
            
            let title = if let Some(g) = group {
                format!("CI Commands - {} Group", g)
            } else {
                "CI Commands Overview".to_string()
            };
            
            println!("{}", art_gen.create_header(&title));
            println!();
            
            if tree {
                println!("📊 {}", "Command Tree View".bold().cyan());
                show_command_tree(&art_gen)?;
            } else {
                println!("📋 {}", "Command Categories".bold().cyan());
                show_command_categories(&art_gen)?;
            }
            
            if interactive {
                println!();
                println!("{}", "💡 Use --tree flag to see hierarchical structure".dimmed());
            }
        }
    }
    
    Ok(())
}

fn show_command_tree(art_gen: &AsciiArtGenerator) -> Result<()> {
    println!("{}", art_gen.create_tree_node(0, false, "CI Commands"));
    println!("{}", art_gen.create_tree_node(1, false, "Intelligence"));
    println!("{}", art_gen.create_tree_node(2, false, "agents - List available agents"));
    println!("{}", art_gen.create_tree_node(2, false, "load - Load specific agent"));
    println!("{}", art_gen.create_tree_node(2, true, "intent - Analyze project"));
    println!("{}", art_gen.create_tree_node(1, false, "Development"));
    println!("{}", art_gen.create_tree_node(2, false, "init - Initialize project"));
    println!("{}", art_gen.create_tree_node(2, true, "fix - Fix issues"));
    println!("{}", art_gen.create_tree_node(1, true, "Visualization"));
    println!("{}", art_gen.create_tree_node(2, false, "overview - System overview"));
    println!("{}", art_gen.create_tree_node(2, true, "agents - Agent ecosystem"));
    Ok(())
}

fn show_command_categories(art_gen: &AsciiArtGenerator) -> Result<()> {
    let categories = vec![
        ("Intelligence", "🧠"),
        ("Development", "🔧"),
        ("Workflow", "🔄"),
        ("Visualization", "👁️"),
    ];
    
    for (category, icon) in categories {
        println!("  {} {}", icon.blue(), category.bold().cyan());
    }
    
    Ok(())
}