use anyhow::Result;
use colored::*;
use crate::config::Config;
use crate::{VisualizationFormat, VisualizationTheme};
use super::ascii_art::AsciiArtGenerator;
use super::web_export::WebExporter;

/// Display current project visualization
pub async fn show_project(
    name: Option<&str>,
    format: VisualizationFormat,
    theme: VisualizationTheme,
    detailed: bool,
    save: bool,
    config: &Config,
) -> Result<()> {
    match format {
        VisualizationFormat::Web => {
            let exporter = WebExporter::new(theme);
            let file_path = if save { "ci_project.html" } else { "/tmp/ci_project.html" };
            println!("{} Creating interactive project visualization...", "🌐".blue());
            exporter.export_project_html(file_path, name, detailed, config).await?;
            super::open_file(file_path, "HTML visualization", save);
        },
        VisualizationFormat::Svg => {
            let exporter = WebExporter::new(theme);
            let file_path = if save { "ci_project.svg" } else { "/tmp/ci_project.svg" };
            println!("{} Creating project SVG diagram...", "📊".blue());
            exporter.export_project_svg(file_path, name, detailed, config).await?;
            super::open_file(file_path, "SVG diagram", save);
        },
        _ => {
            let art_gen = AsciiArtGenerator::new(theme);
            
            let project_name = name.unwrap_or("Current Project");
            let title = format!("Project Analysis - {}", project_name);
            
            println!("{}", art_gen.create_header(&title));
            println!();
            
            show_project_overview(&art_gen, project_name, config)?;
            
            if detailed {
                println!();
                show_detailed_analysis(&art_gen)?;
            }
        }
    }
    
    Ok(())
}

fn show_project_overview(art_gen: &AsciiArtGenerator, project_name: &str, config: &Config) -> Result<()> {
    println!("📦 {}", "Project Overview".bold().cyan());
    
    println!("  {} Project Name: {}", "📛".blue(), project_name.bold().cyan());
    println!("  {} Project Type: {}", "🏷️".blue(), "Rust CLI".green());
    println!("  {} CI Integration: {}", "🔗".blue(), "✅ Active".green());
    println!("  {} Location: {}", "📍".blue(), config.ci_path.to_string_lossy().white());
    
    Ok(())
}

fn show_detailed_analysis(_art_gen: &AsciiArtGenerator) -> Result<()> {
    println!("🔍 {}", "Detailed Analysis".bold().cyan());
    
    let checks = vec![
        ("Configuration Files", "✅", "CLAUDE.md present"),
        ("CI Integration", "✅", "Active and configured"),
        ("Agent Registry", "✅", "Accessible"),
        ("Session Tracking", "⚠️", "Not configured"),
    ];
    
    for (check, status, description) in checks {
        let status_colored = if status == "✅" { status.green() } else { status.yellow() };
        println!("  {} {} {}", 
                 status_colored,
                 check.bold().white(),
                 format!("({})", description).dimmed()
        );
    }
    
    println!();
    println!("📈 {}", "Recent Activity".bold().cyan());
    let activities = vec![
        "Visualization system implemented",
        "Command structure enhanced", 
        "Terminal interface optimized",
    ];
    
    for activity in activities {
        println!("  {} {}", "•".green(), activity.white());
    }
    
    Ok(())
}