use anyhow::{Context, Result};
use std::fs;
use crate::config::Config;
use crate::VisualizationTheme;

pub struct WebExporter {
    theme: VisualizationTheme,
}

impl WebExporter {
    pub fn new(theme: VisualizationTheme) -> Self {
        Self { theme }
    }
    
    pub async fn export_overview_svg(&self, path: &str, config: &Config) -> Result<()> {
        let svg_content = self.generate_overview_svg(config).await?;
        fs::write(path, svg_content)
            .context(format!("Failed to write SVG to {}", path))?;
        Ok(())
    }
    
    pub async fn export_overview_html(&self, path: &str, config: &Config) -> Result<()> {
        let html_content = self.generate_overview_html(config).await?;
        fs::write(path, html_content)
            .context(format!("Failed to write HTML to {}", path))?;
        Ok(())
    }
    
    pub async fn export_commands_html(&self, path: &str, group: Option<&str>, tree: bool, config: &Config) -> Result<()> {
        let html_content = self.generate_commands_html(group, tree, config).await?;
        fs::write(path, html_content)
            .context(format!("Failed to write commands HTML to {}", path))?;
        Ok(())
    }
    
    pub async fn export_commands_svg(&self, path: &str, group: Option<&str>, tree: bool, config: &Config) -> Result<()> {
        let svg_content = self.generate_commands_svg(group, tree, config).await?;
        fs::write(path, svg_content)
            .context(format!("Failed to write commands SVG to {}", path))?;
        Ok(())
    }
    
    pub async fn export_agents_html(&self, path: &str, category: Option<&str>, network: bool, config: &Config) -> Result<()> {
        let html_content = self.generate_agents_html(category, network, config).await?;
        fs::write(path, html_content)
            .context(format!("Failed to write agents HTML to {}", path))?;
        Ok(())
    }
    
    pub async fn export_agents_svg(&self, path: &str, category: Option<&str>, network: bool, config: &Config) -> Result<()> {
        let svg_content = self.generate_agents_svg(category, network, config).await?;
        fs::write(path, svg_content)
            .context(format!("Failed to write agents SVG to {}", path))?;
        Ok(())
    }
    
    pub async fn export_workflows_html(&self, path: &str, category: Option<&str>, beginner: bool, config: &Config) -> Result<()> {
        let html_content = self.generate_workflows_html(category, beginner, config).await?;
        fs::write(path, html_content)
            .context(format!("Failed to write workflows HTML to {}", path))?;
        Ok(())
    }
    
    pub async fn export_workflows_svg(&self, path: &str, category: Option<&str>, beginner: bool, config: &Config) -> Result<()> {
        let svg_content = self.generate_workflows_svg(category, beginner, config).await?;
        fs::write(path, svg_content)
            .context(format!("Failed to write workflows SVG to {}", path))?;
        Ok(())
    }
    
    pub async fn export_project_html(&self, path: &str, name: Option<&str>, detailed: bool, config: &Config) -> Result<()> {
        let html_content = self.generate_project_html(name, detailed, config).await?;
        fs::write(path, html_content)
            .context(format!("Failed to write project HTML to {}", path))?;
        Ok(())
    }
    
    pub async fn export_project_svg(&self, path: &str, name: Option<&str>, detailed: bool, config: &Config) -> Result<()> {
        let svg_content = self.generate_project_svg(name, detailed, config).await?;
        fs::write(path, svg_content)
            .context(format!("Failed to write project SVG to {}", path))?;
        Ok(())
    }
    
    async fn generate_overview_svg(&self, _config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"800\" viewBox=\"0 0 1200 800\">\n");
        svg.push_str("  <defs>\n");
        svg.push_str("    <style>\n");
        svg.push_str(&format!("      .title {{ font-family: 'Courier New', monospace; font-size: 28px; font-weight: bold; fill: {}; }}\n", primary_color));
        svg.push_str(&format!("      .subtitle {{ font-family: 'Courier New', monospace; font-size: 16px; fill: {}; }}\n", secondary_color));
        svg.push_str("      .component { font-family: 'Courier New', monospace; font-size: 14px; fill: #333; }\n");
        svg.push_str(&format!("      .connection {{ stroke: {}; stroke-width: 2; fill: none; }}\n", accent_color));
        svg.push_str(&format!("      .box {{ fill: #f8f9fa; stroke: {}; stroke-width: 2; rx: 8; }}\n", primary_color));
        svg.push_str(&format!("      .agent-box {{ fill: #e3f2fd; stroke: {}; stroke-width: 1; rx: 4; }}\n", accent_color));
        svg.push_str("    </style>\n");
        svg.push_str("  </defs>\n");
        svg.push_str("  \n");
        svg.push_str("  <!-- Background -->\n");
        svg.push_str("  <rect width=\"1200\" height=\"800\" fill=\"#ffffff\"/>\n");
        svg.push_str("  \n");
        svg.push_str("  <!-- Title -->\n");
        svg.push_str("  <text x=\"600\" y=\"40\" text-anchor=\"middle\" class=\"title\">CI Ecosystem Architecture</text>\n");
        svg.push_str("  <text x=\"600\" y=\"65\" text-anchor=\"middle\" class=\"subtitle\">Collaborative Intelligence System Overview</text>\n");
        svg.push_str("  \n");
        svg.push_str("  <!-- Simple architecture representation -->\n");
        svg.push_str("  <rect x=\"100\" y=\"150\" width=\"1000\" height=\"500\" fill=\"none\" stroke=\"#333\" stroke-width=\"2\"/>\n");
        svg.push_str("  <text x=\"600\" y=\"180\" text-anchor=\"middle\" class=\"subtitle\">CI Visualization System</text>\n");
        svg.push_str("  <text x=\"600\" y=\"400\" text-anchor=\"middle\" class=\"component\">Use 'ci visualize' commands to explore the system</text>\n");
        svg.push_str("</svg>");
        
        Ok(svg)
    }
    
    async fn generate_overview_html(&self, _config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CI Ecosystem Architecture</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            color: #333;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        
        .header {{
            background: linear-gradient(135deg, {primary_color} 0%, {accent_color} 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: bold;
        }}
        
        .header p {{
            margin: 10px 0 0 0;
            font-size: 1.2em;
            opacity: 0.9;
        }}
        
        .section {{
            padding: 30px 40px;
            border-bottom: 1px solid #eee;
        }}
        
        .section:last-child {{
            border-bottom: none;
        }}
        
        .section h2 {{
            color: {primary_color};
            margin: 0 0 20px 0;
            font-size: 1.8em;
            border-bottom: 3px solid {accent_color};
            padding-bottom: 10px;
        }}
        
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-top: 20px;
        }}
        
        .card {{
            background: #f8f9fa;
            border: 2px solid {secondary_color};
            border-radius: 8px;
            padding: 20px;
            transition: all 0.3s ease;
        }}
        
        .card:hover {{
            transform: translateY(-5px);
            box-shadow: 0 10px 25px rgba(0,0,0,0.1);
            border-color: {accent_color};
        }}
        
        .card h3 {{
            color: {primary_color};
            margin: 0 0 10px 0;
            font-size: 1.3em;
        }}
        
        .card p {{
            margin: 0;
            line-height: 1.6;
            color: #666;
        }}
        
        .stats {{
            display: flex;
            justify-content: space-around;
            flex-wrap: wrap;
            gap: 20px;
            margin: 20px 0;
        }}
        
        .stat {{
            text-align: center;
            background: linear-gradient(135deg, {primary_color} 0%, {accent_color} 100%);
            color: white;
            padding: 20px;
            border-radius: 10px;
            min-width: 120px;
        }}
        
        .stat-number {{
            font-size: 2.5em;
            font-weight: bold;
            display: block;
        }}
        
        .stat-label {{
            font-size: 0.9em;
            opacity: 0.9;
        }}
        
        .architecture-flow {{
            display: flex;
            flex-direction: column;
            gap: 20px;
            margin: 20px 0;
        }}
        
        .layer {{
            background: #f8f9fa;
            border: 2px solid {secondary_color};
            border-radius: 10px;
            padding: 20px;
            position: relative;
        }}
        
        .layer h4 {{
            color: {primary_color};
            margin: 0 0 15px 0;
            text-align: center;
            font-size: 1.2em;
        }}
        
        .layer-components {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
        }}
        
        .component {{
            background: white;
            border: 1px solid {accent_color};
            border-radius: 6px;
            padding: 15px;
            text-align: center;
            font-weight: bold;
            color: {primary_color};
        }}
        
        .arrow-down {{
            text-align: center;
            font-size: 2em;
            color: {accent_color};
            margin: 10px 0;
        }}
        
        @media (max-width: 768px) {{
            .container {{
                margin: 10px;
                border-radius: 8px;
            }}
            
            .header {{
                padding: 20px;
            }}
            
            .header h1 {{
                font-size: 2em;
            }}
            
            .section {{
                padding: 20px;
            }}
            
            .stats {{
                justify-content: center;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>CI Ecosystem Architecture</h1>
            <p>Collaborative Intelligence System Overview</p>
        </div>
        
        <div class="section">
            <h2>System Statistics</h2>
            <div class="stats">
                <div class="stat">
                    <span class="stat-number">50+</span>
                    <span class="stat-label">Specialized Agents</span>
                </div>
                <div class="stat">
                    <span class="stat-number">30+</span>
                    <span class="stat-label">Command Categories</span>
                </div>
                <div class="stat">
                    <span class="stat-number">4</span>
                    <span class="stat-label">System Layers</span>
                </div>
                <div class="stat">
                    <span class="stat-number">∞</span>
                    <span class="stat-label">Possibilities</span>
                </div>
            </div>
        </div>
        
        <div class="section">
            <h2>Architecture Overview</h2>
            <div class="architecture-flow">
                <div class="layer">
                    <h4>User Interface Layer</h4>
                    <div class="layer-components">
                        <div class="component">CLI Interface</div>
                        <div class="component">Web Interface</div>
                        <div class="component">Agent Network</div>
                        <div class="component">API Gateway</div>
                    </div>
                </div>
                
                <div class="arrow-down">↓</div>
                
                <div class="layer">
                    <h4>Core Engine Layer</h4>
                    <div class="layer-components">
                        <div class="component">Command Processor</div>
                        <div class="component">Project Manager</div>
                        <div class="component">Session Manager</div>
                        <div class="component">Config System</div>
                    </div>
                </div>
                
                <div class="arrow-down">↓</div>
                
                <div class="layer">
                    <h4>Intelligence Layer</h4>
                    <div class="layer-components">
                        <div class="component">Agent Registry</div>
                        <div class="component">Memory System</div>
                        <div class="component">Learning Engine</div>
                        <div class="component">Task Queue</div>
                    </div>
                </div>
                
                <div class="arrow-down">↓</div>
                
                <div class="layer">
                    <h4>Data Layer</h4>
                    <div class="layer-components">
                        <div class="component">Project Files</div>
                        <div class="component">Agent Configs</div>
                        <div class="component">Session Data</div>
                        <div class="component">Cache Store</div>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="section">
            <h2>Core Components</h2>
            <div class="grid">
                <div class="card">
                    <h3>🧠 Intelligence</h3>
                    <p>50+ specialized AI agents for different domains and tasks, each with unique capabilities and expertise.</p>
                </div>
                <div class="card">
                    <h3>⚡ Commands</h3>
                    <p>Comprehensive CLI with 30+ command categories covering development, deployment, and management.</p>
                </div>
                <div class="card">
                    <h3>📦 Projects</h3>
                    <p>Smart project initialization and management with automatic configuration and dependency handling.</p>
                </div>
                <div class="card">
                    <h3>💬 Sessions</h3>
                    <p>Persistent conversation and workflow tracking that maintains context across interactions.</p>
                </div>
                <div class="card">
                    <h3>🧭 Memory</h3>
                    <p>Advanced context preservation and learning system that improves over time.</p>
                </div>
                <div class="card">
                    <h3>⚙️ Configuration</h3>
                    <p>Flexible configuration and customization system adaptable to any project or workflow.</p>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#,
            primary_color = primary_color,
            secondary_color = secondary_color,
            accent_color = accent_color
        );
        
        Ok(html)
    }
    
    async fn generate_commands_html(&self, group: Option<&str>, tree: bool, _config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let title = if let Some(g) = group {
            format!("CI Commands - {} Group", g)
        } else {
            "CI Commands Overview".to_string()
        };
        
        let commands_content = if tree {
            self.generate_command_tree_html()
        } else {
            self.generate_command_categories_html()
        };
        
        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            color: #333;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        
        .header {{
            background: linear-gradient(135deg, {primary_color} 0%, {accent_color} 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: bold;
        }}
        
        .content {{
            padding: 40px;
        }}
        
        .tree-node {{
            margin: 5px 0;
            padding: 8px 0;
            font-family: 'Courier New', monospace;
            white-space: pre;
            color: {primary_color};
        }}
        
        .category {{
            background: #f8f9fa;
            border: 2px solid {secondary_color};
            border-radius: 8px;
            padding: 20px;
            margin: 15px 0;
            transition: all 0.3s ease;
        }}
        
        .category:hover {{
            transform: translateY(-2px);
            box-shadow: 0 5px 15px rgba(0,0,0,0.1);
            border-color: {accent_color};
        }}
        
        .category h3 {{
            color: {primary_color};
            margin: 0 0 10px 0;
            font-size: 1.4em;
        }}
        
        .command-list {{
            list-style: none;
            padding: 0;
            margin: 10px 0 0 0;
        }}
        
        .command-list li {{
            padding: 8px 0;
            border-bottom: 1px solid #eee;
            color: #666;
        }}
        
        .command-list li:last-child {{
            border-bottom: none;
        }}
        
        .command-name {{
            font-weight: bold;
            color: {accent_color};
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{title}</h1>
        </div>
        
        <div class="content">
            {commands_content}
        </div>
    </div>
</body>
</html>"#,
            title = title,
            primary_color = primary_color,
            secondary_color = secondary_color,
            accent_color = accent_color,
            commands_content = commands_content
        );
        
        Ok(html)
    }
    
    fn generate_command_tree_html(&self) -> String {
        r#"<h2>📊 Command Tree View</h2>
            <div class="tree-node">CI Commands</div>
            <div class="tree-node">├── Intelligence</div>
            <div class="tree-node">│   ├── agents - List available agents</div>
            <div class="tree-node">│   ├── load - Load specific agent</div>
            <div class="tree-node">│   └── intent - Analyze project</div>
            <div class="tree-node">├── Development</div>
            <div class="tree-node">│   ├── init - Initialize project</div>
            <div class="tree-node">│   └── fix - Fix issues</div>
            <div class="tree-node">└── Visualization</div>
            <div class="tree-node">    ├── overview - System overview</div>
            <div class="tree-node">    └── agents - Agent ecosystem</div>"#.to_string()
    }
    
    fn generate_command_categories_html(&self) -> String {
        r#"<h2>📋 Command Categories</h2>
            <div class="category">
                <h3>🧠 Intelligence</h3>
                <ul class="command-list">
                    <li><span class="command-name">agents</span> - List and manage available AI agents</li>
                    <li><span class="command-name">load</span> - Load specific agent for current session</li>
                    <li><span class="command-name">intent</span> - Analyze project and suggest actions</li>
                </ul>
            </div>
            
            <div class="category">
                <h3>🔧 Development</h3>
                <ul class="command-list">
                    <li><span class="command-name">init</span> - Initialize new CI project</li>
                    <li><span class="command-name">fix</span> - Auto-fix common issues</li>
                    <li><span class="command-name">test</span> - Run project tests</li>
                </ul>
            </div>
            
            <div class="category">
                <h3>🔄 Workflow</h3>
                <ul class="command-list">
                    <li><span class="command-name">deploy</span> - Deploy to environments</li>
                    <li><span class="command-name">sync</span> - Synchronize project state</li>
                    <li><span class="command-name">backup</span> - Create project backups</li>
                </ul>
            </div>
            
            <div class="category">
                <h3>👁️ Visualization</h3>
                <ul class="command-list">
                    <li><span class="command-name">overview</span> - System architecture overview</li>
                    <li><span class="command-name">agents</span> - Agent ecosystem visualization</li>
                    <li><span class="command-name">workflows</span> - Common usage patterns</li>
                </ul>
            </div>"#.to_string()
    }
    
    async fn generate_commands_svg(&self, group: Option<&str>, tree: bool, _config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let title = if let Some(g) = group {
            format!("CI Commands - {} Group", g)
        } else {
            "CI Commands Overview".to_string()
        };
        
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1000\" height=\"600\" viewBox=\"0 0 1000 600\">\n");
        svg.push_str("  <defs>\n");
        svg.push_str("    <style>\n");
        svg.push_str(&format!("      .title {{ font-family: 'Courier New', monospace; font-size: 24px; font-weight: bold; fill: {}; }}\n", primary_color));
        svg.push_str(&format!("      .category {{ font-family: 'Courier New', monospace; font-size: 18px; font-weight: bold; fill: {}; }}\n", accent_color));
        svg.push_str("      .command { font-family: 'Courier New', monospace; font-size: 14px; fill: #333; }\n");
        svg.push_str(&format!("      .box {{ fill: #f8f9fa; stroke: {}; stroke-width: 2; rx: 8; }}\n", primary_color));
        svg.push_str("    </style>\n");
        svg.push_str("  </defs>\n");
        svg.push_str("  \n");
        svg.push_str("  <rect width=\"1000\" height=\"600\" fill=\"#ffffff\"/>\n");
        svg.push_str(&format!("  <text x=\"500\" y=\"40\" text-anchor=\"middle\" class=\"title\">{}</text>\n", title));
        
        if tree {
            svg.push_str("  <text x=\"100\" y=\"100\" class=\"category\">CI Commands Tree</text>\n");
            svg.push_str("  <text x=\"120\" y=\"130\" class=\"command\">├── Intelligence</text>\n");
            svg.push_str("  <text x=\"140\" y=\"150\" class=\"command\">│   ├── agents</text>\n");
            svg.push_str("  <text x=\"140\" y=\"170\" class=\"command\">│   ├── load</text>\n");
            svg.push_str("  <text x=\"140\" y=\"190\" class=\"command\">│   └── intent</text>\n");
            svg.push_str("  <text x=\"120\" y=\"220\" class=\"command\">├── Development</text>\n");
            svg.push_str("  <text x=\"140\" y=\"240\" class=\"command\">│   ├── init</text>\n");
            svg.push_str("  <text x=\"140\" y=\"260\" class=\"command\">│   └── fix</text>\n");
            svg.push_str("  <text x=\"120\" y=\"290\" class=\"command\">└── Visualization</text>\n");
            svg.push_str("  <text x=\"140\" y=\"310\" class=\"command\">    ├── overview</text>\n");
            svg.push_str("  <text x=\"140\" y=\"330\" class=\"command\">    └── agents</text>\n");
        } else {
            let categories = [
                ("🧠 Intelligence", 100),
                ("🔧 Development", 200),
                ("🔄 Workflow", 300),
                ("👁️ Visualization", 400),
            ];
            
            for (i, (category, y)) in categories.iter().enumerate() {
                svg.push_str(&format!("  <rect x=\"100\" y=\"{}\" width=\"800\" height=\"80\" class=\"box\"/>\n", y));
                svg.push_str(&format!("  <text x=\"500\" y=\"{}\" text-anchor=\"middle\" class=\"category\">{}</text>\n", y + 50, category));
            }
        }
        
        svg.push_str("</svg>");
        
        Ok(svg)
    }
    
    async fn generate_agents_html(&self, category: Option<&str>, network: bool, _config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let title = match category {
            Some(cat) => format!("CI Agents - {} Category", cat),
            None => "CI Agents Ecosystem".to_string(),
        };
        
        let agents_content = if network {
            self.generate_agent_network_html()
        } else {
            self.generate_agent_categories_html()
        };
        
        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            color: #333;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        
        .header {{
            background: linear-gradient(135deg, {primary_color} 0%, {accent_color} 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: bold;
        }}
        
        .content {{
            padding: 40px;
        }}
        
        .agent-network {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        
        .agent-node {{
            background: #f8f9fa;
            border: 2px solid {secondary_color};
            border-radius: 10px;
            padding: 20px;
            text-align: center;
            transition: all 0.3s ease;
        }}
        
        .agent-node:hover {{
            transform: scale(1.05);
            border-color: {accent_color};
            box-shadow: 0 5px 15px rgba(0,0,0,0.2);
        }}
        
        .agent-name {{
            font-size: 1.4em;
            font-weight: bold;
            color: {primary_color};
            margin-bottom: 10px;
        }}
        
        .agent-connections {{
            list-style: none;
            padding: 0;
            margin: 0;
        }}
        
        .agent-connections li {{
            padding: 5px 0;
            color: #666;
            font-size: 0.9em;
        }}
        
        .category-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        
        .category-card {{
            background: #f8f9fa;
            border: 2px solid {secondary_color};
            border-radius: 8px;
            padding: 25px;
            text-align: center;
            transition: all 0.3s ease;
        }}
        
        .category-card:hover {{
            transform: translateY(-3px);
            border-color: {accent_color};
            box-shadow: 0 8px 20px rgba(0,0,0,0.15);
        }}
        
        .category-icon {{
            font-size: 3em;
            margin-bottom: 15px;
        }}
        
        .category-name {{
            font-size: 1.3em;
            font-weight: bold;
            color: {primary_color};
            margin-bottom: 10px;
        }}
        
        .category-count {{
            font-size: 1.1em;
            color: {accent_color};
            font-weight: bold;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{title}</h1>
        </div>
        
        <div class="content">
            {agents_content}
        </div>
    </div>
</body>
</html>"#,
            title = title,
            primary_color = primary_color,
            secondary_color = secondary_color,
            accent_color = accent_color,
            agents_content = agents_content
        );
        
        Ok(html)
    }
    
    fn generate_agent_network_html(&self) -> String {
        r#"<h2>🕸️ Agent Network View</h2>
            <div class="agent-network">
                <div class="agent-node">
                    <div class="agent-name">Athena</div>
                    <ul class="agent-connections">
                        <li>• Coordinates all agents</li>
                        <li>• Memory management</li>
                        <li>• Strategic planning</li>
                    </ul>
                </div>
                
                <div class="agent-node">
                    <div class="agent-name">Developer</div>
                    <ul class="agent-connections">
                        <li>• Code creation</li>
                        <li>• Bug fixing</li>
                        <li>• Code review</li>
                    </ul>
                </div>
                
                <div class="agent-node">
                    <div class="agent-name">Visualist</div>
                    <ul class="agent-connections">
                        <li>• Terminal graphics</li>
                        <li>• System mapping</li>
                        <li>• Data visualization</li>
                    </ul>
                </div>
                
                <div class="agent-node">
                    <div class="agent-name">Architect</div>
                    <ul class="agent-connections">
                        <li>• System design</li>
                        <li>• Architecture planning</li>
                        <li>• Technology selection</li>
                    </ul>
                </div>
            </div>"#.to_string()
    }
    
    fn generate_agent_categories_html(&self) -> String {
        r#"<h2>📊 Agent Categories</h2>
            <div class="category-grid">
                <div class="category-card">
                    <div class="category-icon">💻</div>
                    <div class="category-name">Development</div>
                    <div class="category-count">12 agents</div>
                </div>
                
                <div class="category-card">
                    <div class="category-icon">🏗️</div>
                    <div class="category-name">Architecture</div>
                    <div class="category-count">8 agents</div>
                </div>
                
                <div class="category-card">
                    <div class="category-icon">🔍</div>
                    <div class="category-name">Analysis</div>
                    <div class="category-count">6 agents</div>
                </div>
                
                <div class="category-card">
                    <div class="category-icon">🧪</div>
                    <div class="category-name">Testing</div>
                    <div class="category-count">5 agents</div>
                </div>
                
                <div class="category-card">
                    <div class="category-icon">⚙️</div>
                    <div class="category-name">Operations</div>
                    <div class="category-count">15 agents</div>
                </div>
                
                <div class="category-card">
                    <div class="category-icon">🎨</div>
                    <div class="category-name">Creative</div>
                    <div class="category-count">4 agents</div>
                </div>
            </div>"#.to_string()
    }
    
    async fn generate_agents_svg(&self, category: Option<&str>, network: bool, _config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let title = match category {
            Some(cat) => format!("CI Agents - {} Category", cat),
            None => "CI Agents Ecosystem".to_string(),
        };
        
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"800\" viewBox=\"0 0 1200 800\">\n");
        svg.push_str("  <defs>\n");
        svg.push_str("    <style>\n");
        svg.push_str(&format!("      .title {{ font-family: 'Courier New', monospace; font-size: 24px; font-weight: bold; fill: {}; }}\n", primary_color));
        svg.push_str(&format!("      .agent {{ font-family: 'Courier New', monospace; font-size: 16px; font-weight: bold; fill: {}; }}\n", accent_color));
        svg.push_str("      .connection { font-family: 'Courier New', monospace; font-size: 12px; fill: #666; }\n");
        svg.push_str(&format!("      .node {{ fill: #f8f9fa; stroke: {}; stroke-width: 2; rx: 10; }}\n", primary_color));
        svg.push_str(&format!("      .line {{ stroke: {}; stroke-width: 2; }}\n", secondary_color));
        svg.push_str("    </style>\n");
        svg.push_str("  </defs>\n");
        svg.push_str("  \n");
        svg.push_str("  <rect width=\"1200\" height=\"800\" fill=\"#ffffff\"/>\n");
        svg.push_str(&format!("  <text x=\"600\" y=\"40\" text-anchor=\"middle\" class=\"title\">{}</text>\n", title));
        
        if network {
            // Agent network diagram
            let agents = [
                ("Athena", 300, 150),
                ("Developer", 150, 300),
                ("Visualist", 450, 300),
                ("Architect", 300, 450),
            ];
            
            // Draw connections
            svg.push_str("  <line x1=\"300\" y1=\"150\" x2=\"150\" y2=\"300\" class=\"line\"/>\n");
            svg.push_str("  <line x1=\"300\" y1=\"150\" x2=\"450\" y2=\"300\" class=\"line\"/>\n");
            svg.push_str("  <line x1=\"300\" y1=\"150\" x2=\"300\" y2=\"450\" class=\"line\"/>\n");
            
            // Draw agent nodes
            for (name, x, y) in &agents {
                svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"120\" height=\"80\" class=\"node\"/>\n", x - 60, y - 40));
                svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" class=\"agent\">{}</text>\n", x, y, name));
            }
        } else {
            // Category view
            let categories = [
                ("💻 Development", 200, 150),
                ("🏗️ Architecture", 600, 150),
                ("🔍 Analysis", 1000, 150),
                ("🧪 Testing", 200, 350),
                ("⚙️ Operations", 600, 350),
                ("🎨 Creative", 1000, 350),
            ];
            
            for (category, x, y) in &categories {
                svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"180\" height=\"100\" class=\"node\"/>\n", x - 90, y - 50));
                svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" class=\"agent\">{}</text>\n", x, y, category));
            }
        }
        
        svg.push_str("</svg>");
        
        Ok(svg)
    }
    
    async fn generate_workflows_html(&self, category: Option<&str>, beginner: bool, _config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let title = if beginner {
            "CI Workflows - Beginner Friendly".to_string()
        } else if let Some(cat) = category {
            format!("CI Workflows - {} Category", cat)
        } else {
            "CI Workflows Overview".to_string()
        };
        
        let workflows_content = if beginner {
            self.generate_beginner_workflows_html()
        } else {
            self.generate_workflow_categories_html()
        };
        
        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            color: #333;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        
        .header {{
            background: linear-gradient(135deg, {primary_color} 0%, {accent_color} 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: bold;
        }}
        
        .content {{
            padding: 40px;
        }}
        
        .workflow-section {{
            background: #f8f9fa;
            border: 2px solid {secondary_color};
            border-radius: 10px;
            padding: 25px;
            margin: 20px 0;
            transition: all 0.3s ease;
        }}
        
        .workflow-section:hover {{
            border-color: {accent_color};
            box-shadow: 0 5px 15px rgba(0,0,0,0.1);
        }}
        
        .workflow-title {{
            font-size: 1.4em;
            font-weight: bold;
            color: {primary_color};
            margin-bottom: 15px;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        
        .workflow-steps {{
            list-style: none;
            padding: 0;
            margin: 0;
        }}
        
        .workflow-steps li {{
            padding: 10px 0;
            border-left: 3px solid {accent_color};
            padding-left: 20px;
            margin: 8px 0;
            background: white;
            border-radius: 0 5px 5px 0;
            transition: all 0.2s ease;
        }}
        
        .workflow-steps li:hover {{
            background: #f0f8ff;
            transform: translateX(5px);
        }}
        
        .step-command {{
            font-weight: bold;
            color: {accent_color};
            font-family: 'Courier New', monospace;
        }}
        
        .category-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
            gap: 25px;
            margin: 20px 0;
        }}
        
        .category-card {{
            background: #f8f9fa;
            border: 2px solid {secondary_color};
            border-radius: 10px;
            padding: 25px;
            transition: all 0.3s ease;
        }}
        
        .category-card:hover {{
            transform: translateY(-3px);
            border-color: {accent_color};
            box-shadow: 0 8px 20px rgba(0,0,0,0.15);
        }}
        
        .category-header {{
            display: flex;
            align-items: center;
            gap: 15px;
            margin-bottom: 15px;
        }}
        
        .category-icon {{
            font-size: 2.5em;
        }}
        
        .category-name {{
            font-size: 1.3em;
            font-weight: bold;
            color: {primary_color};
        }}
        
        .workflow-list {{
            list-style: none;
            padding: 0;
            margin: 0;
        }}
        
        .workflow-list li {{
            padding: 8px 0;
            color: #666;
            border-bottom: 1px solid #eee;
        }}
        
        .workflow-list li:last-child {{
            border-bottom: none;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{title}</h1>
        </div>
        
        <div class="content">
            {workflows_content}
        </div>
    </div>
</body>
</html>"#,
            title = title,
            primary_color = primary_color,
            secondary_color = secondary_color,
            accent_color = accent_color,
            workflows_content = workflows_content
        );
        
        Ok(html)
    }
    
    fn generate_beginner_workflows_html(&self) -> String {
        r#"<h2>🌱 Beginner-Friendly Workflows</h2>
            <div class="workflow-section">
                <div class="workflow-title">
                    <span>📋</span>
                    <span>Getting Started</span>
                </div>
                <ul class="workflow-steps">
                    <li><span class="step-command">ci init &lt;project&gt;</span> - Initialize new project</li>
                    <li><span class="step-command">ci agents</span> - List available agents</li>
                    <li><span class="step-command">ci load Athena</span> - Load coordinator agent</li>
                </ul>
            </div>
            
            <div class="workflow-section">
                <div class="workflow-title">
                    <span>📋</span>
                    <span>Basic Development</span>
                </div>
                <ul class="workflow-steps">
                    <li><span class="step-command">ci status</span> - Check project status</li>
                    <li><span class="step-command">ci fix</span> - Auto-fix common issues</li>
                    <li><span class="step-command">ci commit</span> - Commit changes</li>
                </ul>
            </div>
            
            <div class="workflow-section">
                <div class="workflow-title">
                    <span>📋</span>
                    <span>Agent Exploration</span>
                </div>
                <ul class="workflow-steps">
                    <li><span class="step-command">ci visualize agents</span> - Explore agent ecosystem</li>
                    <li><span class="step-command">ci load &lt;agent&gt;</span> - Load specific agent</li>
                    <li><span class="step-command">ci intent</span> - Analyze project requirements</li>
                </ul>
            </div>"#.to_string()
    }
    
    fn generate_workflow_categories_html(&self) -> String {
        r#"<h2>🔄 Workflow Categories</h2>
            <div class="category-grid">
                <div class="category-card">
                    <div class="category-header">
                        <div class="category-icon">🔧</div>
                        <div class="category-name">Development</div>
                    </div>
                    <ul class="workflow-list">
                        <li>• Project setup and initialization</li>
                        <li>• Code management and versioning</li>
                        <li>• Issue resolution and debugging</li>
                        <li>• Testing and quality assurance</li>
                    </ul>
                </div>
                
                <div class="category-card">
                    <div class="category-header">
                        <div class="category-icon">🧠</div>
                        <div class="category-name">Intelligence</div>
                    </div>
                    <ul class="workflow-list">
                        <li>• Agent activation and management</li>
                        <li>• Context management and memory</li>
                        <li>• Collaborative agent workflows</li>
                        <li>• Intent analysis and planning</li>
                    </ul>
                </div>
                
                <div class="category-card">
                    <div class="category-header">
                        <div class="category-icon">👁️</div>
                        <div class="category-name">Visualization</div>
                    </div>
                    <ul class="workflow-list">
                        <li>• System exploration and mapping</li>
                        <li>• Architecture visualization</li>
                        <li>• Process flow documentation</li>
                        <li>• Interactive system navigation</li>
                    </ul>
                </div>
            </div>"#.to_string()
    }
    
    async fn generate_workflows_svg(&self, category: Option<&str>, beginner: bool, _config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let title = if beginner {
            "CI Workflows - Beginner Friendly".to_string()
        } else if let Some(cat) = category {
            format!("CI Workflows - {} Category", cat)
        } else {
            "CI Workflows Overview".to_string()
        };
        
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"800\" viewBox=\"0 0 1200 800\">\n");
        svg.push_str("  <defs>\n");
        svg.push_str("    <style>\n");
        svg.push_str(&format!("      .title {{ font-family: 'Courier New', monospace; font-size: 24px; font-weight: bold; fill: {}; }}\n", primary_color));
        svg.push_str(&format!("      .workflow {{ font-family: 'Courier New', monospace; font-size: 16px; font-weight: bold; fill: {}; }}\n", accent_color));
        svg.push_str("      .step { font-family: 'Courier New', monospace; font-size: 12px; fill: #666; }\n");
        svg.push_str(&format!("      .box {{ fill: #f8f9fa; stroke: {}; stroke-width: 2; rx: 8; }}\n", primary_color));
        svg.push_str(&format!("      .arrow {{ stroke: {}; stroke-width: 2; marker-end: url(#arrowhead); }}\n", secondary_color));
        svg.push_str("    </style>\n");
        svg.push_str("    <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\n");
        svg.push_str(&format!("      <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"{}\"/>\n", secondary_color));
        svg.push_str("    </marker>\n");
        svg.push_str("  </defs>\n");
        svg.push_str("  \n");
        svg.push_str("  <rect width=\"1200\" height=\"800\" fill=\"#ffffff\"/>\n");
        svg.push_str(&format!("  <text x=\"600\" y=\"40\" text-anchor=\"middle\" class=\"title\">{}</text>\n", title));
        
        if beginner {
            // Workflow sequence diagram
            let workflows = [
                ("Getting Started", 150, 100),
                ("Basic Development", 150, 300),
                ("Agent Exploration", 150, 500),
            ];
            
            for (i, (name, x, y)) in workflows.iter().enumerate() {
                svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"200\" height=\"120\" class=\"box\"/>\n", x, y));
                svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" class=\"workflow\">{}</text>\n", x + 100, y + 60, name));
                
                if i < workflows.len() - 1 {
                    svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"arrow\"/>\n", 
                        x + 100, y + 120, x + 100, workflows[i + 1].2));
                }
            }
        } else {
            // Category layout
            let categories = [
                ("🔧 Development", 200, 150),
                ("🧠 Intelligence", 600, 150),
                ("👁️ Visualization", 1000, 150),
            ];
            
            for (category, x, y) in &categories {
                svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"180\" height=\"120\" class=\"box\"/>\n", x - 90, y - 60));
                svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" class=\"workflow\">{}</text>\n", x, y, category));
            }
        }
        
        svg.push_str("</svg>");
        
        Ok(svg)
    }
    
    async fn generate_project_html(&self, name: Option<&str>, detailed: bool, config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let project_name = name.unwrap_or("Current Project");
        let title = format!("Project Analysis - {}", project_name);
        
        let project_content = if detailed {
            self.generate_detailed_project_html(project_name, config)
        } else {
            self.generate_basic_project_html(project_name, config)
        };
        
        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            color: #333;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        
        .header {{
            background: linear-gradient(135deg, {primary_color} 0%, {accent_color} 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: bold;
        }}
        
        .content {{
            padding: 40px;
        }}
        
        .overview-section {{
            background: #f8f9fa;
            border: 2px solid {secondary_color};
            border-radius: 10px;
            padding: 25px;
            margin: 20px 0;
        }}
        
        .section-title {{
            font-size: 1.4em;
            font-weight: bold;
            color: {primary_color};
            margin-bottom: 20px;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        
        .info-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        
        .info-item {{
            display: flex;
            align-items: center;
            gap: 15px;
            padding: 15px;
            background: white;
            border-radius: 8px;
            border-left: 4px solid {accent_color};
        }}
        
        .info-icon {{
            font-size: 1.5em;
        }}
        
        .info-content {{
            flex: 1;
        }}
        
        .info-label {{
            font-weight: bold;
            color: {primary_color};
            margin-bottom: 5px;
        }}
        
        .info-value {{
            color: #666;
        }}
        
        .status-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px;
            margin: 20px 0;
        }}
        
        .status-item {{
            display: flex;
            align-items: center;
            gap: 15px;
            padding: 15px;
            background: white;
            border-radius: 8px;
            transition: all 0.3s ease;
        }}
        
        .status-item:hover {{
            transform: translateY(-2px);
            box-shadow: 0 5px 15px rgba(0,0,0,0.1);
        }}
        
        .status-icon {{
            font-size: 1.5em;
        }}
        
        .status-content {{
            flex: 1;
        }}
        
        .status-name {{
            font-weight: bold;
            color: {primary_color};
            margin-bottom: 5px;
        }}
        
        .status-description {{
            font-size: 0.9em;
            color: #666;
        }}
        
        .activity-list {{
            list-style: none;
            padding: 0;
            margin: 20px 0;
        }}
        
        .activity-list li {{
            padding: 12px 0;
            border-left: 3px solid {accent_color};
            padding-left: 20px;
            margin: 8px 0;
            background: white;
            border-radius: 0 5px 5px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{title}</h1>
        </div>
        
        <div class="content">
            {project_content}
        </div>
    </div>
</body>
</html>"#,
            title = title,
            primary_color = primary_color,
            secondary_color = secondary_color,
            accent_color = accent_color,
            project_content = project_content
        );
        
        Ok(html)
    }
    
    fn generate_basic_project_html(&self, project_name: &str, config: &Config) -> String {
        format!(r#"<div class="overview-section">
                <div class="section-title">
                    <span>📦</span>
                    <span>Project Overview</span>
                </div>
                <div class="info-grid">
                    <div class="info-item">
                        <div class="info-icon">📛</div>
                        <div class="info-content">
                            <div class="info-label">Project Name</div>
                            <div class="info-value">{}</div>
                        </div>
                    </div>
                    <div class="info-item">
                        <div class="info-icon">🏷️</div>
                        <div class="info-content">
                            <div class="info-label">Project Type</div>
                            <div class="info-value">Rust CLI</div>
                        </div>
                    </div>
                    <div class="info-item">
                        <div class="info-icon">🔗</div>
                        <div class="info-content">
                            <div class="info-label">CI Integration</div>
                            <div class="info-value">✅ Active</div>
                        </div>
                    </div>
                    <div class="info-item">
                        <div class="info-icon">📍</div>
                        <div class="info-content">
                            <div class="info-label">Location</div>
                            <div class="info-value">{}</div>
                        </div>
                    </div>
                </div>
            </div>"#, project_name, config.ci_path.to_string_lossy())
    }
    
    fn generate_detailed_project_html(&self, project_name: &str, config: &Config) -> String {
        format!(r#"{basic_content}
            
            <div class="overview-section">
                <div class="section-title">
                    <span>🔍</span>
                    <span>Detailed Analysis</span>
                </div>
                <div class="status-grid">
                    <div class="status-item">
                        <div class="status-icon">✅</div>
                        <div class="status-content">
                            <div class="status-name">Configuration Files</div>
                            <div class="status-description">CLAUDE.md present</div>
                        </div>
                    </div>
                    <div class="status-item">
                        <div class="status-icon">✅</div>
                        <div class="status-content">
                            <div class="status-name">CI Integration</div>
                            <div class="status-description">Active and configured</div>
                        </div>
                    </div>
                    <div class="status-item">
                        <div class="status-icon">✅</div>
                        <div class="status-content">
                            <div class="status-name">Agent Registry</div>
                            <div class="status-description">Accessible</div>
                        </div>
                    </div>
                    <div class="status-item">
                        <div class="status-icon">⚠️</div>
                        <div class="status-content">
                            <div class="status-name">Session Tracking</div>
                            <div class="status-description">Not configured</div>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="overview-section">
                <div class="section-title">
                    <span>📈</span>
                    <span>Recent Activity</span>
                </div>
                <ul class="activity-list">
                    <li>• Visualization system implemented</li>
                    <li>• Command structure enhanced</li>
                    <li>• Terminal interface optimized</li>
                    <li>• Web export functionality added</li>
                </ul>
            </div>"#, basic_content = self.generate_basic_project_html(project_name, config))
    }
    
    async fn generate_project_svg(&self, name: Option<&str>, detailed: bool, config: &Config) -> Result<String> {
        let (primary_color, secondary_color, accent_color) = self.get_theme_colors();
        
        let project_name = name.unwrap_or("Current Project");
        let title = format!("Project Analysis - {}", project_name);
        
        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1000\" height=\"600\" viewBox=\"0 0 1000 600\">\n");
        svg.push_str("  <defs>\n");
        svg.push_str("    <style>\n");
        svg.push_str(&format!("      .title {{ font-family: 'Courier New', monospace; font-size: 24px; font-weight: bold; fill: {}; }}\n", primary_color));
        svg.push_str(&format!("      .section {{ font-family: 'Courier New', monospace; font-size: 18px; font-weight: bold; fill: {}; }}\n", accent_color));
        svg.push_str("      .info { font-family: 'Courier New', monospace; font-size: 14px; fill: #333; }\n");
        svg.push_str(&format!("      .box {{ fill: #f8f9fa; stroke: {}; stroke-width: 2; rx: 8; }}\n", primary_color));
        svg.push_str("    </style>\n");
        svg.push_str("  </defs>\n");
        svg.push_str("  \n");
        svg.push_str("  <rect width=\"1000\" height=\"600\" fill=\"#ffffff\"/>\n");
        svg.push_str(&format!("  <text x=\"500\" y=\"40\" text-anchor=\"middle\" class=\"title\">{}</text>\n", title));
        
        // Project overview box
        svg.push_str("  <rect x=\"100\" y=\"100\" width=\"800\" height=\"200\" class=\"box\"/>\n");
        svg.push_str("  <text x=\"500\" y=\"130\" text-anchor=\"middle\" class=\"section\">📦 Project Overview</text>\n");
        svg.push_str(&format!("  <text x=\"150\" y=\"160\" class=\"info\">📛 Project Name: {}</text>\n", project_name));
        svg.push_str("  <text x=\"150\" y=\"180\" class=\"info\">🏷️ Project Type: Rust CLI</text>\n");
        svg.push_str("  <text x=\"150\" y=\"200\" class=\"info\">🔗 CI Integration: ✅ Active</text>\n");
        svg.push_str(&format!("  <text x=\"150\" y=\"220\" class=\"info\">📍 Location: {}</text>\n", config.ci_path.to_string_lossy()));
        
        if detailed {
            // Status checks
            svg.push_str("  <rect x=\"100\" y=\"350\" width=\"800\" height=\"200\" class=\"box\"/>\n");
            svg.push_str("  <text x=\"500\" y=\"380\" text-anchor=\"middle\" class=\"section\">🔍 System Status</text>\n");
            svg.push_str("  <text x=\"150\" y=\"410\" class=\"info\">✅ Configuration Files: Ready</text>\n");
            svg.push_str("  <text x=\"150\" y=\"430\" class=\"info\">✅ CI Integration: Active</text>\n");
            svg.push_str("  <text x=\"150\" y=\"450\" class=\"info\">✅ Agent Registry: Available</text>\n");
            svg.push_str("  <text x=\"150\" y=\"470\" class=\"info\">⚠️ Session Tracking: Not configured</text>\n");
        }
        
        svg.push_str("</svg>");
        
        Ok(svg)
    }
    
    fn get_theme_colors(&self) -> (&str, &str, &str) {
        match self.theme {
            VisualizationTheme::Dark => ("#2196F3", "#64B5F6", "#1976D2"),
            VisualizationTheme::Light => ("#4CAF50", "#81C784", "#388E3C"),
            VisualizationTheme::Contrast => ("#9C27B0", "#BA68C8", "#7B1FA2"),
            VisualizationTheme::Terminal => ("#424242", "#757575", "#212121"),
        }
    }
}