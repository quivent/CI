use anyhow::{Context, Result};
use colored::*;
use crate::config::Config;
use crate::{VisualizationFormat, VisualizationTheme};
use super::ascii_art::AsciiArtGenerator;
use super::web_export::WebExporter;

/// Display CI ecosystem architecture overview
pub async fn show_overview(
    format: VisualizationFormat,
    theme: VisualizationTheme,
    interactive: bool,
    export: Option<&str>,
    save: bool,
    config: &Config,
) -> Result<()> {
    match format {
        VisualizationFormat::Terminal => {
            show_terminal_overview(theme, interactive, config).await
        },
        VisualizationFormat::Web => {
            show_web_overview(theme, export, save, config).await
        },
        VisualizationFormat::Svg => {
            show_svg_overview(theme, export, save, config).await
        },
        VisualizationFormat::Mermaid => {
            println!("{} Mermaid format not yet implemented", "⚠️".yellow());
            Ok(())
        },
        VisualizationFormat::Auto => {
            // Default to terminal for now
            show_terminal_overview(theme, interactive, config).await
        },
    }
}

async fn show_terminal_overview(
    theme: VisualizationTheme,
    interactive: bool,
    config: &Config,
) -> Result<()> {
    let art_gen = AsciiArtGenerator::new(theme);
    
    // Header
    println!("{}", art_gen.create_header("CI Ecosystem Architecture"));
    println!();
    
    // Main architecture diagram
    show_architecture_diagram(&art_gen)?;
    println!();
    
    // Component overview
    show_component_overview(&art_gen, config).await?;
    println!();
    
    // Command categories
    show_command_categories(&art_gen)?;
    println!();
    
    // Agent ecosystem
    show_agent_ecosystem(&art_gen, config).await?;
    
    if interactive {
        println!();
        println!("{}", "Interactive Mode:".bold().cyan());
        println!("{} Use 'ci visualize commands' to explore command structure", "→".blue());
        println!("{} Use 'ci visualize agents' to browse agent capabilities", "→".blue());
        println!("{} Use 'ci visualize workflows' to see process flows", "→".blue());
        println!("{} Use 'ci visualize project' to analyze current project", "→".blue());
    }
    
    Ok(())
}

fn show_architecture_diagram(art_gen: &AsciiArtGenerator) -> Result<()> {
    println!("{}", art_gen.create_section_header("System Architecture"));
    
    let architecture = format!(r#"
    ┌─────────────────────────────────────────────────────────────────┐
    │                     {} User Interface                     │
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
    │  │   {} CLI   │  │  {} Web UI  │  │  {} Agents  │  │ {} API │ │
    │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘ │
    └─────────────────┬───────────────────────────────────────────────┘
                      │
    ┌─────────────────┴───────────────────────────────────────────────┐
    │                      {} Core Engine                      │
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
    │  │ {} Command │  │ {} Project │  │ {} Session │  │ {} Config│ │
    │  │ {} Processor│  │ {} Manager │  │ {} Manager │  │ {} System│ │
    │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘ │
    └─────────────────┬───────────────────────────────────────────────┘
                      │
    ┌─────────────────┴───────────────────────────────────────────────┐
    │                   {} Intelligence Layer                   │
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
    │  │   {} Agent  │  │ {} Memory  │  │ {} Learning │  │ {} Tasks│ │
    │  │ {} Registry │  │ {} System  │  │ {} Engine  │  │ {} Queue│ │
    │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘ │
    └─────────────────┬───────────────────────────────────────────────┘
                      │
    ┌─────────────────┴───────────────────────────────────────────────┐
    │                     {} Data Layer                       │
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
    │  │ {} Project │  │   {} Agent  │  │ {} Session │  │ {} Cache│ │
    │  │  {} Files  │  │ {} Configs │  │  {} Data   │  │ {} Store│ │
    │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘ │
    └─────────────────────────────────────────────────────────────────┘
    "#,
        "CI".bold().cyan(),
        "Interactive".green(), "Visual".blue(), "Collaborative".magenta(), "RESTful".yellow(),
        "Intelligence".bold().magenta(),
        "Smart".green(), "Adaptive".blue(), "Workflow".cyan(), "Dynamic".yellow(),
        "Processing".green(), "Control".blue(), "Tracking".cyan(), "Management".yellow(),
        "Distributed".bold().blue(),
        "Specialized".green(), "Persistent".blue(), "Continuous".cyan(), "Priority".yellow(),
        "Discovery".green(), "Optimization".blue(), "Improvement".cyan(), "Processing".yellow(),
        "Persistent".bold().green(),
        "Source".green(), "Behavioral".blue(), "Conversation".cyan(), "Performance".yellow(),
        "Control".green(), "Profiles".blue(), "Archives".cyan(), "Layer".yellow()
    );
    
    println!("{}", architecture);
    Ok(())
}

async fn show_component_overview(art_gen: &AsciiArtGenerator, config: &Config) -> Result<()> {
    println!("{}", art_gen.create_section_header("Core Components"));
    
    let components = vec![
        ("Intelligence", "50+ specialized AI agents for different domains", "🧠"),
        ("Commands", "Comprehensive CLI with 30+ command categories", "⚡"),
        ("Projects", "Smart project initialization and management", "📦"),
        ("Sessions", "Persistent conversation and workflow tracking", "💬"),
        ("Memory", "Advanced context preservation and learning", "🧭"),
        ("Config", "Flexible configuration and customization", "⚙️"),
    ];
    
    for (component, description, icon) in components {
        println!("  {} {} {}", 
                 icon.blue(), 
                 component.bold().cyan(), 
                 description.white()
        );
    }
    
    Ok(())
}

fn show_command_categories(art_gen: &AsciiArtGenerator) -> Result<()> {
    println!("{}", art_gen.create_section_header("Command Categories"));
    
    let categories = vec![
        ("Intelligence", vec!["agents", "load", "intent", "projects"], "🧠"),
        ("Development", vec!["init", "integrate", "fix", "verify"], "🔧"),
        ("Workflow", vec!["session", "idea", "status", "deploy"], "🔄"),
        ("Source Control", vec!["stage", "commit", "remotes", "clean"], "📝"),
        ("Configuration", vec!["config", "key", "local", "detach"], "⚙️"),
        ("System", vec!["install", "rebuild", "legacy", "docs"], "🔨"),
        ("Visualization", vec!["overview", "commands", "agents", "workflows"], "👁️"),
    ];
    
    for (category, commands, icon) in categories {
        println!("  {} {} {}", 
                 icon.blue(), 
                 category.bold().cyan(),
                 format!("({})", commands.join(", ")).white()
        );
    }
    
    Ok(())
}

async fn show_agent_ecosystem(art_gen: &AsciiArtGenerator, config: &Config) -> Result<()> {
    println!("{}", art_gen.create_section_header("Agent Ecosystem"));
    
    // Try to get actual agent count from the CI API
    let agent_info = match get_agent_summary(config).await {
        Ok(info) => info,
        Err(_) => AgentSummary {
            total: 50,
            enabled: 45,
            categories: vec![
                ("Development".to_string(), 12),
                ("Analysis".to_string(), 8),
                ("Architecture".to_string(), 6),
                ("Testing".to_string(), 5),
                ("Documentation".to_string(), 4),
                ("Operations".to_string(), 15),
            ],
        }
    };
    
    println!("  {} Total Agents: {}", "📊".blue(), agent_info.total.to_string().bold().green());
    println!("  {} Active Agents: {}", "✅".blue(), agent_info.enabled.to_string().bold().cyan());
    println!();
    
    println!("  {} Agent Categories:", "🏷️".blue());
    for (category, count) in &agent_info.categories {
        let bar = "█".repeat((*count as f32 / 3.0) as usize);
        println!("    {} {} {}", 
                 format!("{:>12}", category).cyan(), 
                 format!("{:>2}", count).yellow(),
                 bar.green()
        );
    }
    
    Ok(())
}

async fn show_web_overview(
    theme: VisualizationTheme,
    export: Option<&str>,
    save: bool,
    config: &Config,
) -> Result<()> {
    let exporter = WebExporter::new(theme);
    
    if let Some(export_path) = export {
        // Explicit export mode
        if export_path.ends_with(".html") {
            println!("{} Exporting HTML to: {}", "🌐".blue(), export_path.cyan());
            exporter.export_overview_html(export_path, config).await?;
        } else if export_path.ends_with(".svg") {
            println!("{} Exporting SVG to: {}", "📊".blue(), export_path.cyan());
            exporter.export_overview_svg(export_path, config).await?;
        } else {
            println!("{} Unknown export format. Use .html or .svg extension", "⚠️".yellow());
        }
    } else {
        // Auto-export and open mode for shortcut flags
        let file_path = if save {
            "ci_overview.html"
        } else {
            "/tmp/ci_overview.html"
        };
        
        println!("{} Creating interactive overview...", "🌐".blue());
        exporter.export_overview_html(file_path, config).await?;
        super::open_file(file_path, "HTML visualization", save);
    }
    
    Ok(())
}

async fn show_svg_overview(
    theme: VisualizationTheme,
    export: Option<&str>,
    save: bool,
    config: &Config,
) -> Result<()> {
    let exporter = WebExporter::new(theme);
    
    if let Some(export_path) = export {
        // Explicit export mode
        println!("{} Exporting SVG to: {}", "📊".blue(), export_path.cyan());
        exporter.export_overview_svg(export_path, config).await?;
    } else {
        // Auto-export and open mode for shortcut flags
        let file_path = if save {
            "ci_overview.svg"
        } else {
            "/tmp/ci_overview.svg"
        };
        
        println!("{} Creating SVG diagram...", "📊".blue());
        exporter.export_overview_svg(file_path, config).await?;
        super::open_file(file_path, "SVG diagram", save);
    }
    
    Ok(())
}

#[derive(Debug)]
struct AgentSummary {
    total: usize,
    enabled: usize,
    categories: Vec<(String, usize)>,
}

async fn get_agent_summary(config: &Config) -> Result<AgentSummary> {
    // Try to scan the AGENTS directory for real data
    let agents_dir = config.ci_path.join("AGENTS");
    if agents_dir.exists() && agents_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&agents_dir) {
            let mut total = 0;
            let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(agent_name) = path.file_name().and_then(|n| n.to_str()) {
                            // Skip non-agent directories
                            if agent_name.starts_with('.') || agent_name == "Manager" {
                                continue;
                            }
                            
                            total += 1;
                            
                            // Categorize agents based on name patterns
                            let category = categorize_agent(agent_name);
                            *categories.entry(category).or_insert(0) += 1;
                        }
                    }
                }
            }
            
            let mut category_vec: Vec<(String, usize)> = categories.into_iter().collect();
            category_vec.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
            
            return Ok(AgentSummary {
                total,
                enabled: total, // Assume all are enabled for now
                categories: category_vec,
            });
        }
    }
    
    // Fallback to mock data if scanning fails
    match std::fs::read_to_string(config.ci_path.join("AGENTS.md")) {
        Ok(_content) => {
            // Parse the content to extract agent information
            // For now, return mock data - this can be enhanced later
            Ok(AgentSummary {
                total: 50,
                enabled: 45,
                categories: vec![
                    ("Development".to_string(), 12),
                    ("Analysis".to_string(), 8),
                    ("Architecture".to_string(), 6),
                    ("Testing".to_string(), 5),
                    ("Documentation".to_string(), 4),
                    ("Operations".to_string(), 15),
                ],
            })
        },
        Err(e) => Err(anyhow::anyhow!("Failed to get agent data: {}", e))
    }
}

fn categorize_agent(agent_name: &str) -> String {
    let name_lower = agent_name.to_lowercase();
    
    // Development-related agents
    if name_lower.contains("develop") || name_lower.contains("engineer") || name_lower.contains("coder") || 
       name_lower.contains("rustist") || name_lower.contains("basher") || name_lower.contains("linguist") {
        "Development".to_string()
    }
    // Architecture and design
    else if name_lower.contains("architect") || name_lower.contains("designer") || name_lower.contains("planner") {
        "Architecture".to_string()
    }
    // Testing and verification
    else if name_lower.contains("test") || name_lower.contains("debug") || 
            name_lower.contains("fixer") || name_lower.contains("verif") {
        "Testing".to_string()
    }
    // Analysis and research
    else if name_lower.contains("analy") || name_lower.contains("research") || name_lower.contains("inspector") ||
            name_lower.contains("athena") || name_lower.contains("scholar") {
        "Analysis".to_string()
    }
    // Documentation and knowledge
    else if name_lower.contains("document") || name_lower.contains("memory") || name_lower.contains("knowledge") ||
            name_lower.contains("sage") || name_lower.contains("mnemosyne") {
        "Documentation".to_string()
    }
    // Operations and automation
    else if name_lower.contains("automat") || name_lower.contains("deploy") || name_lower.contains("ops") ||
            name_lower.contains("system") || name_lower.contains("admin") {
        "Operations".to_string()
    }
    // Management and coordination
    else if name_lower.contains("manager") || name_lower.contains("coordinate") || name_lower.contains("hermes") {
        "Management".to_string()
    }
    // Optimization and performance
    else if name_lower.contains("optim") || name_lower.contains("performance") || name_lower.contains("benchmark") ||
            name_lower.contains("streamlin") {
        "Optimization".to_string()
    }
    // User experience and interface
    else if name_lower.contains("ui") || name_lower.contains("ux") || name_lower.contains("user") ||
            name_lower.contains("visual") {
        "User Experience".to_string()
    }
    // Specialized tools
    else {
        "Specialized".to_string()
    }
}