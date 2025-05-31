use anyhow::Result;
use colored::*;
use crate::config::Config;
use crate::{VisualizationFormat, VisualizationTheme};
use super::ascii_art::AsciiArtGenerator;
use super::web_export::WebExporter;

/// Display CI workflow visualization
pub async fn show_workflows(
    category: Option<&str>,
    format: VisualizationFormat,
    theme: VisualizationTheme,
    beginner: bool,
    save: bool,
    config: &Config,
) -> Result<()> {
    match format {
        VisualizationFormat::Web => {
            let exporter = WebExporter::new(theme);
            let file_path = if save { "ci_workflows.html" } else { "/tmp/ci_workflows.html" };
            println!("{} Creating interactive workflows visualization...", "🌐".blue());
            exporter.export_workflows_html(file_path, category, beginner, config).await?;
            super::open_file(file_path, "HTML visualization", save);
        },
        VisualizationFormat::Svg => {
            let exporter = WebExporter::new(theme);
            let file_path = if save { "ci_workflows.svg" } else { "/tmp/ci_workflows.svg" };
            println!("{} Creating workflows SVG diagram...", "📊".blue());
            exporter.export_workflows_svg(file_path, category, beginner, config).await?;
            super::open_file(file_path, "SVG diagram", save);
        },
        _ => {
            let art_gen = AsciiArtGenerator::new(theme);
            
            let title = if beginner {
                "CI Workflows - Beginner Friendly".to_string()
            } else if let Some(cat) = category {
                format!("CI Workflows - {} Category", cat)
            } else {
                "CI Workflows Overview".to_string()
            };
            
            println!("{}", art_gen.create_header(&title));
            println!();
            
            if beginner {
                show_beginner_workflows(&art_gen)?;
            } else {
                show_workflow_categories(&art_gen, category)?;
            }
        }
    }
    
    Ok(())
}

fn show_beginner_workflows(_art_gen: &AsciiArtGenerator) -> Result<()> {
    println!("🌱 {}", "Beginner-Friendly Workflows".bold().green());
    println!();
    
    let workflows = vec![
        ("Getting Started", vec!["ci init <project>", "ci agents", "ci load Athena"]),
        ("Basic Development", vec!["ci status", "ci fix", "ci commit"]),
        ("Agent Exploration", vec!["ci visualize agents", "ci load <agent>", "ci intent"]),
    ];
    
    for (workflow, steps) in workflows {
        println!("  {} {}", "📋".blue(), workflow.bold().cyan());
        for (i, step) in steps.iter().enumerate() {
            let connector = if i == steps.len() - 1 { "└─→" } else { "├─→" };
            println!("    {} {}", connector.green(), step.white());
        }
        println!();
    }
    
    Ok(())
}

fn show_workflow_categories(_art_gen: &AsciiArtGenerator, category: Option<&str>) -> Result<()> {
    println!("🔄 {}", "Workflow Categories".bold().cyan());
    
    let categories = vec![
        ("Development", "🔧", vec!["Project setup", "Code management", "Issue resolution"]),
        ("Intelligence", "🧠", vec!["Agent activation", "Context management", "Collaborative work"]),
        ("Visualization", "👁️", vec!["System exploration", "Architecture mapping", "Process flow"]),
    ];
    
    for (cat_name, icon, workflows) in categories {
        if let Some(filter) = category {
            if cat_name.to_lowercase() != filter.to_lowercase() {
                continue;
            }
        }
        
        println!("  {} {}", icon.blue(), cat_name.bold().cyan());
        for workflow in workflows {
            println!("    {} {}", "•".green(), workflow.white());
        }
        println!();
    }
    
    Ok(())
}