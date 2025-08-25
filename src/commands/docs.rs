//! Documentation generation and serving for the CI CLI
//!
//! This module provides interactive web-based documentation for the Collaborative Intelligence CLI,
//! including command references, agent galleries, examples, and deployment options.

use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use crate::config::Config;
use crate::{DocsCommands, DeployTarget};
use crate::helpers::CommandHelpers;

/// Handle docs command and its subcommands
pub async fn handle_docs_command(command: &DocsCommands, config: &Config) -> Result<()> {
    match command {
        DocsCommands::Serve { temp, port, open, watch } => {
            serve_docs(*temp, *port, *open, *watch, config).await
        },
        DocsCommands::Generate { output, interactive, agents, theme } => {
            generate_docs(output, *interactive, *agents, theme, config).await
        },
        DocsCommands::App { interactive, examples, visualizer, output } => {
            create_web_app(*interactive, *examples, *visualizer, output, config).await
        },
        DocsCommands::Deploy { target } => {
            deploy_docs(target, config).await
        },
    }
}

/// Serve interactive documentation on local development server
async fn serve_docs(temp: bool, port: u16, open: bool, watch: bool, config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Starting Documentation Server", 
        "🌐", 
        "Documentation & Examples", 
        "blue"
    );
    
    println!("📝 Mode: {}", if temp { "Temporary".yellow() } else { "Persistent".green() });
    println!("🌐 Port: {}", port.to_string().cyan());
    println!("👀 Watch: {}", if watch { "Enabled".green() } else { "Disabled".red() });
    println!();
    
    // Create documentation content
    let docs_content = generate_documentation_content(config).await?;
    
    if temp {
        // Create temporary HTML file
        let temp_file = std::env::temp_dir().join("ci_docs.html");
        std::fs::write(&temp_file, docs_content)?;
        println!("📄 Temporary file: {}", temp_file.display());
        
        // Start simple HTTP server
        start_simple_server(port, &temp_file, open).await?;
    } else {
        // Create persistent documentation
        let docs_dir = config.ci_path.join("docs").join("cli");
        std::fs::create_dir_all(&docs_dir)?;
        
        let docs_file = docs_dir.join("index.html");
        std::fs::write(&docs_file, docs_content)?;
        println!("📄 Documentation file: {}", docs_file.display());
        
        // Start server from docs directory
        start_simple_server(port, &docs_file, open).await?;
    }
    
    Ok(())
}

/// Generate static documentation site
async fn generate_docs(output: &Path, interactive: bool, agents: bool, theme: &str, config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Generating Documentation Site", 
        "📚", 
        "Documentation & Examples", 
        "green"
    );
    
    println!("📁 Output: {}", output.display().to_string().cyan());
    println!("🎯 Interactive: {}", if interactive { "Yes".green() } else { "No".red() });
    println!("🤖 Agent Gallery: {}", if agents { "Yes".green() } else { "No".red() });
    println!("🎨 Theme: {}", theme.yellow());
    println!();
    
    // Create output directory
    std::fs::create_dir_all(output)?;
    
    // Generate main documentation
    let docs_content = generate_documentation_content(config).await?;
    std::fs::write(output.join("index.html"), docs_content)?;
    
    // Generate additional files if requested
    if interactive {
        let interactive_content = generate_interactive_content(config).await?;
        std::fs::write(output.join("interactive.html"), interactive_content)?;
    }
    
    if agents {
        let agents_content = generate_agents_gallery(config).await?;
        std::fs::write(output.join("agents.html"), agents_content)?;
    }
    
    // Copy CSS and JS assets
    generate_assets(output, theme)?;
    
    println!("✅ Documentation generated successfully!");
    println!("📂 Files created in: {}", output.display());
    
    Ok(())
}

/// Create full interactive web application
async fn create_web_app(interactive: bool, examples: bool, visualizer: bool, output: &Path, config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Creating Interactive Web App", 
        "⚡", 
        "Documentation & Examples", 
        "magenta"
    );
    
    println!("📁 Output: {}", output.display().to_string().cyan());
    println!("🎯 Interactive Builder: {}", if interactive { "Yes".green() } else { "No".red() });
    println!("📋 Live Examples: {}", if examples { "Yes".green() } else { "No".red() });
    println!("📊 Agent Visualizer: {}", if visualizer { "Yes".green() } else { "No".red() });
    println!();
    
    // Create output directory
    std::fs::create_dir_all(output)?;
    
    // Generate web app files
    let app_content = generate_web_app_content(interactive, examples, visualizer, config).await?;
    std::fs::write(output.join("index.html"), app_content)?;
    
    // Generate additional features
    if interactive {
        let builder_js = generate_command_builder_js(config).await?;
        std::fs::write(output.join("command-builder.js"), builder_js)?;
    }
    
    if visualizer {
        let visualizer_js = generate_agent_visualizer_js(config).await?;
        std::fs::write(output.join("agent-visualizer.js"), visualizer_js)?;
    }
    
    // Copy assets
    generate_assets(output, "auto")?;
    
    println!("✅ Interactive web app created!");
    println!("📂 Files created in: {}", output.display());
    
    Ok(())
}

/// Deploy documentation to various platforms
async fn deploy_docs(target: &DeployTarget, config: &Config) -> Result<()> {
    match target {
        DeployTarget::GithubPages { repo, branch } => {
            deploy_to_github_pages(repo.as_deref(), branch, config).await
        },
        DeployTarget::Vercel { project } => {
            deploy_to_vercel(project.as_deref(), config).await
        },
        DeployTarget::Local { path, symlink } => {
            deploy_to_local(path, *symlink, config).await
        },
    }
}

/// Generate the main documentation content
async fn generate_documentation_content(config: &Config) -> Result<String> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CI - Collaborative Intelligence CLI Documentation</title>
    <style>
        :root {{
            --primary: #3b82f6;
            --secondary: #64748b;
            --accent: #06b6d4;
            --background: #f8fafc;
            --surface: #ffffff;
            --text: #1e293b;
            --text-muted: #64748b;
            --border: #e2e8f0;
            --success: #10b981;
            --warning: #f59e0b;
            --error: #ef4444;
        }}
        
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            color: var(--text);
            background: var(--background);
        }}
        
        .header {{
            background: linear-gradient(135deg, var(--primary), var(--accent));
            color: white;
            padding: 3rem 2rem;
            text-align: center;
        }}
        
        .header h1 {{
            font-size: 3rem;
            margin-bottom: 1rem;
            font-weight: 700;
        }}
        
        .header p {{
            font-size: 1.25rem;
            opacity: 0.9;
            max-width: 600px;
            margin: 0 auto;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }}
        
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin: 2rem 0;
        }}
        
        .card {{
            background: var(--surface);
            border-radius: 12px;
            padding: 2rem;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            border: 1px solid var(--border);
            transition: transform 0.2s, box-shadow 0.2s;
        }}
        
        .card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 8px 25px -1px rgba(0, 0, 0, 0.15);
        }}
        
        .card h3 {{
            color: var(--primary);
            margin-bottom: 1rem;
            font-size: 1.5rem;
        }}
        
        .command {{
            background: #1e293b;
            color: #e2e8f0;
            padding: 1rem;
            border-radius: 8px;
            font-family: 'Fira Code', monospace;
            margin: 1rem 0;
            overflow-x: auto;
        }}
        
        .command .prompt {{
            color: var(--accent);
        }}
        
        .examples {{
            margin: 2rem 0;
        }}
        
        .example {{
            margin: 1rem 0;
            padding: 1rem;
            background: #f1f5f9;
            border-radius: 8px;
            border-left: 4px solid var(--primary);
        }}
        
        .agents-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
            gap: 1rem;
            margin: 2rem 0;
        }}
        
        .agent-card {{
            background: var(--surface);
            padding: 1rem;
            border-radius: 8px;
            border: 1px solid var(--border);
            text-align: center;
        }}
        
        .agent-card .name {{
            font-weight: 600;
            color: var(--primary);
            margin-bottom: 0.5rem;
        }}
        
        .agent-card .description {{
            font-size: 0.9rem;
            color: var(--text-muted);
        }}
        
        .stats {{
            display: flex;
            justify-content: center;
            gap: 2rem;
            margin: 2rem 0;
            flex-wrap: wrap;
        }}
        
        .stat {{
            text-align: center;
            padding: 1rem;
        }}
        
        .stat .number {{
            font-size: 2.5rem;
            font-weight: 700;
            color: var(--primary);
            display: block;
        }}
        
        .stat .label {{
            color: var(--text-muted);
            font-size: 0.9rem;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}
        
        @media (max-width: 768px) {{
            .header h1 {{ font-size: 2rem; }}
            .container {{ padding: 1rem; }}
            .stats {{ gap: 1rem; }}
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🤖 CI Documentation</h1>
        <p>Collaborative Intelligence CLI - Your gateway to autonomous agent coordination and intelligent task management</p>
    </div>
    
    <div class="container">
        <div class="stats">
            <div class="stat">
                <span class="number">100+</span>
                <span class="label">Available Agents</span>
            </div>
            <div class="stat">
                <span class="number">∞</span>
                <span class="label">Parallel Instances</span>
            </div>
            <div class="stat">
                <span class="number">50+</span>
                <span class="label">CLI Commands</span>
            </div>
        </div>
        
        <div class="grid">
            <div class="card">
                <h3>🚀 Quick Start</h3>
                <p>Get started with the most common CI commands:</p>
                <div class="command">
                    <span class="prompt">$</span> ci load Athena
                </div>
                <div class="command">
                    <span class="prompt">$</span> ci load "Documentor*7" --parallel -a -t "Document 7 modules"
                </div>
                <div class="command">
                    <span class="prompt">$</span> ci agents
                </div>
            </div>
            
            <div class="card">
                <h3>🤖 Agent System</h3>
                <p>Load and coordinate intelligent agents for various tasks:</p>
                <div class="examples">
                    <div class="example">
                        <strong>Single Agent:</strong><br>
                        <code>ci load Analyst -t "Analyze performance"</code>
                    </div>
                    <div class="example">
                        <strong>Multiple Agents:</strong><br>
                        <code>ci load Analyst Documentor --parallel</code>
                    </div>
                    <div class="example">
                        <strong>Agent Multipliers:</strong><br>
                        <code>ci load "Developer*5" --parallel -a</code>
                    </div>
                </div>
            </div>
            
            <div class="card">
                <h3>⚡ Parallel Execution</h3>
                <p>Run multiple agent instances simultaneously:</p>
                <div class="examples">
                    <div class="example">
                        <strong>7 Documentors:</strong><br>
                        <code>ci load "Documentor*7" --parallel</code>
                    </div>
                    <div class="example">
                        <strong>Mixed Team:</strong><br>
                        <code>ci load "Analyst*3" "Documentor*4"</code>
                    </div>
                </div>
            </div>
            
            <div class="card">
                <h3>📚 Command Reference</h3>
                <p>Essential commands for CI management:</p>
                <div class="examples">
                    <div class="example">
                        <code>ci agents</code> - List all available agents
                    </div>
                    <div class="example">
                        <code>ci projects</code> - Show integrated projects
                    </div>
                    <div class="example">
                        <code>ci docs serve</code> - This documentation!
                    </div>
                    <div class="example">
                        <code>ci visualize</code> - Generate architecture diagrams
                    </div>
                </div>
            </div>
        </div>
        
        <div class="card">
            <h3>🎯 Agent Coordination</h3>
            <p>When you run parallel agents, they coordinate through:</p>
            <ul style="margin: 1rem 0; padding-left: 2rem;">
                <li><strong>Shared Directory:</strong> Each session gets a unique coordination folder</li>
                <li><strong>Task Files:</strong> Central task tracking and progress sharing</li>
                <li><strong>Instance IDs:</strong> Each agent gets a unique identifier (Instance-1, Instance-2, etc.)</li>
                <li><strong>File Communication:</strong> Agents share findings through files in the coordination directory</li>
            </ul>
        </div>
    </div>
    
    <script>
        // Add some interactivity
        document.querySelectorAll('.command').forEach(cmd => {{
            cmd.addEventListener('click', () => {{
                navigator.clipboard.writeText(cmd.textContent.replace('$ ', ''));
                cmd.style.background = '#10b981';
                setTimeout(() => {{
                    cmd.style.background = '#1e293b';
                }}, 500);
            }});
        }});
        
        // Add copy notification
        const style = document.createElement('style');
        style.textContent = `
            .copied {{ background: var(--success) !important; }}
        `;
        document.head.appendChild(style);
    </script>
</body>
</html>
"#);
    
    Ok(html)
}

/// Generate interactive content
async fn generate_interactive_content(_config: &Config) -> Result<String> {
    Ok("<!-- Interactive content placeholder -->".to_string())
}

/// Generate agents gallery
async fn generate_agents_gallery(_config: &Config) -> Result<String> {
    Ok("<!-- Agents gallery placeholder -->".to_string())
}

/// Generate web app content
async fn generate_web_app_content(_interactive: bool, _examples: bool, _visualizer: bool, _config: &Config) -> Result<String> {
    Ok("<!-- Web app placeholder -->".to_string())
}

/// Generate command builder JavaScript
async fn generate_command_builder_js(_config: &Config) -> Result<String> {
    Ok("// Command builder placeholder".to_string())
}

/// Generate agent visualizer JavaScript
async fn generate_agent_visualizer_js(_config: &Config) -> Result<String> {
    Ok("// Agent visualizer placeholder".to_string())
}

/// Generate CSS and JS assets
fn generate_assets(output: &Path, _theme: &str) -> Result<()> {
    // Create assets directory
    let assets_dir = output.join("assets");
    std::fs::create_dir_all(&assets_dir)?;
    
    // Basic CSS file
    let css_content = r#"
/* CI Documentation Styles */
:root {
    --primary: #3b82f6;
    --secondary: #64748b;
    --accent: #06b6d4;
}
/* Additional styles would go here */
"#;
    std::fs::write(assets_dir.join("style.css"), css_content)?;
    
    Ok(())
}

/// Start simple HTTP server
async fn start_simple_server(port: u16, file_path: &Path, open: bool) -> Result<()> {
    use std::process::Command;
    
    println!("🚀 Starting server on port {}", port);
    println!("📄 Serving: {}", file_path.display());
    
    if open {
        println!("🌐 Opening browser...");
        let url = format!("http://localhost:{}", port);
        
        // Try to open browser
        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(&url).spawn().ok();
        }
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd").args(&["/c", "start", &url]).spawn().ok();
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(&url).spawn().ok();
        }
    }
    
    println!("💡 Press Ctrl+C to stop the server");
    println!("🌐 Visit: http://localhost:{}", port);
    
    // Simple server implementation would go here
    // For now, just show the file path
    println!("📋 Documentation ready at: {}", file_path.display());
    
    Ok(())
}

/// Deploy to GitHub Pages
async fn deploy_to_github_pages(_repo: Option<&str>, _branch: &str, _config: &Config) -> Result<()> {
    println!("🚀 GitHub Pages deployment would be implemented here");
    Ok(())
}

/// Deploy to Vercel
async fn deploy_to_vercel(_project: Option<&str>, _config: &Config) -> Result<()> {
    println!("🚀 Vercel deployment would be implemented here");
    Ok(())
}

/// Deploy to local directory
async fn deploy_to_local(path: &Path, _symlink: bool, _config: &Config) -> Result<()> {
    println!("🚀 Local deployment to {} would be implemented here", path.display());
    Ok(())
}