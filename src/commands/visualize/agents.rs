use anyhow::Result;
use colored::*;
use crate::config::Config;
use crate::{VisualizationFormat, VisualizationTheme};
use super::ascii_art::AsciiArtGenerator;
use super::web_export::WebExporter;

/// Display CI agent ecosystem visualization
pub async fn show_agents(
    category: Option<&str>,
    format: VisualizationFormat,
    theme: VisualizationTheme,
    network: bool,
    interactive: bool,
    save: bool,
    config: &Config,
) -> Result<()> {
    match format {
        VisualizationFormat::Web => {
            let exporter = WebExporter::new(theme);
            let file_path = if save { "ci_agents.html" } else { "/tmp/ci_agents.html" };
            println!("{} Creating interactive agents visualization...", "🌐".blue());
            exporter.export_agents_html(file_path, category, network, config).await?;
            super::open_file(file_path, "HTML visualization", save);
        },
        VisualizationFormat::Svg => {
            let exporter = WebExporter::new(theme);
            let file_path = if save { "ci_agents.svg" } else { "/tmp/ci_agents.svg" };
            println!("{} Creating agents SVG diagram...", "📊".blue());
            exporter.export_agents_svg(file_path, category, network, config).await?;
            super::open_file(file_path, "SVG diagram", save);
        },
        _ => {
            let art_gen = AsciiArtGenerator::new(theme);
            
            let title = match category {
                Some(cat) => format!("CI Agents - {} Category", cat),
                None => "CI Agents Ecosystem".to_string(),
            };
            
            println!("{}", art_gen.create_header(&title));
            println!();
            
            if network {
                show_agent_network(&art_gen)?;
            } else {
                show_agent_categories(&art_gen)?;
            }
            
            if interactive {
                println!();
                println!("{}", "💡 Use --network flag to see agent relationships".dimmed());
            }
        }
    }
    
    Ok(())
}

fn show_agent_network(art_gen: &AsciiArtGenerator) -> Result<()> {
    println!("🕸️ {}", "Agent Network View".bold().cyan());
    
    let connections = vec![
        ("Athena".to_string(), vec!["Coordinates all agents".to_string(), "Memory management".to_string()]),
        ("Developer".to_string(), vec!["Code creation".to_string(), "Bug fixing".to_string()]),
        ("Visualist".to_string(), vec!["Terminal graphics".to_string(), "System mapping".to_string()]),
    ];
    
    println!("{}", art_gen.create_connection_diagram(&connections));
    Ok(())
}

fn show_agent_categories(art_gen: &AsciiArtGenerator) -> Result<()> {
    println!("📊 {}", "Agent Categories".bold().cyan());
    
    let categories = vec![
        ("Development", "💻", 12),
        ("Architecture", "🏗️", 8),
        ("Analysis", "🔍", 6),
        ("Testing", "🧪", 5),
        ("Operations", "⚙️", 15),
    ];
    
    for (category, icon, count) in categories {
        println!("  {} {} {}", 
                 icon.blue(), 
                 category.bold().cyan(),
                 format!("({} agents)", count).yellow()
        );
    }
    
    Ok(())
}