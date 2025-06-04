/// Agent Color Management System
/// 
/// Provides terminal background color management for visual agent identification.
/// This module ensures consistent color application across all CI operations.

use anyhow::Result;
use std::io::Write;
use std::collections::HashMap;

/// Get the background color for a specific agent
pub fn get_agent_color(agent_name: &str) -> Option<&'static str> {
    match agent_name {
        "Athena" => Some("#003300"),           // Dark Green 1
        "Master" => Some("#001a00"),           // Dark Green 2  
        "Debugger" => Some("#001133"),         // Dark Blue 1
        "Architect" => Some("#000d26"),        // Dark Blue 2
        "SwiftSpecialist" => Some("#191970"),  // Navy Blue
        "UI" => Some("#2d1b69"),               // Dark Purple 1
        "UX" => Some("#2d1b69"),               // Dark Purple 1
        "Designer" => Some("#1a0d33"),         // Dark Purple 2
        "Optimizer" => Some("#301934"),        // Deep Violet
        "Topologist" => Some("#2d1a0d"),       // Dark Brown 1
        "Planner" => Some("#331a00"),          // Dark Brown 2
        "Memory" => Some("#3c2415"),           // Coffee Brown
        "Fixer" => Some("#330d0d"),            // Dark Red 1
        "Tester" => Some("#2d0a0a"),           // Dark Red 2
        "Analyst" => Some("#722f37"),          // Burgundy
        "Developer" => Some("#1a1a1a"),        // Dark Gray 1
        "Documenter" => Some("#262626"),       // Charcoal
        "Writer" => Some("#262626"),           // Charcoal
        "Refactorer" => Some("#301934"),       // Deep Violet
        "Engineer" => Some("#1a1a1a"),         // Dark Gray 1
        "Verifier" => Some("#330d0d"),         // Dark Red 1
        "Researcher" => Some("#722f37"),       // Burgundy
        "Visionary" => Some("#191970"),        // Navy Blue
        "Recommender" => Some("#3c2415"),      // Coffee Brown
        "Scholar" => Some("#262626"),          // Charcoal
        "Sage" => Some("#003300"),             // Dark Green 1
        "Gaia" => Some("#001a00"),             // Dark Green 2
        "Mnemosyne" => Some("#3c2415"),        // Coffee Brown
        "SageKeeper" => Some("#003300"),       // Dark Green 1
        "Cartographer" => Some("#2d1a0d"),     // Dark Brown 1
        "Consolidator" => Some("#331a00"),     // Dark Brown 2
        "Streamliner" => Some("#301934"),      // Deep Violet
        "Enforcer" => Some("#330d0d"),         // Dark Red 1
        "Wellness" => Some("#001a00"),         // Dark Green 2
        "Hermes" => Some("#191970"),           // Navy Blue
        "Reactor" => Some("#722f37"),          // Burgundy
        "Pause" => Some("#1a1a1a"),            // Dark Gray 1
        "Human" => Some("#262626"),            // Charcoal
        "User" => Some("#262626"),             // Charcoal
        _ => None,
    }
}

/// Apply terminal background color using ANSI escape sequences
pub fn apply_terminal_background_color(color: &str) -> Result<()> {
    print!("\x1b]11;{}\x07", color);
    std::io::stdout().flush()
        .map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;
    Ok(())
}

/// Reset terminal background color to default
pub fn reset_terminal_color() -> Result<()> {
    print!("\x1b]111\x07");
    std::io::stdout().flush()
        .map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;
    Ok(())
}

/// Apply agent-specific background color with logging
pub fn apply_agent_color(agent_name: &str) -> Result<()> {
    if let Some(color) = get_agent_color(agent_name) {
        apply_terminal_background_color(color)?;
        println!("🎨 Applied {} background color: {}", agent_name, color);
        
        // Update current agent state
        let _ = update_current_agent_state(agent_name);
        
        Ok(())
    } else {
        // Don't treat missing color as an error, just log it
        println!("⚠️  No color defined for agent: {}", agent_name);
        Ok(())
    }
}

/// Update current agent state in session
fn update_current_agent_state(agent_name: &str) -> Result<()> {
    let session_file = std::env::temp_dir().join("ci_current_agent.txt");
    std::fs::write(&session_file, agent_name)
        .map_err(|e| anyhow::anyhow!("Failed to update current agent state: {}", e))?;
    Ok(())
}

/// Get current active agent from session state
pub fn get_current_agent() -> Option<String> {
    let session_file = std::env::temp_dir().join("ci_current_agent.txt");
    std::fs::read_to_string(&session_file).ok()
}

/// Get color name for display purposes
pub fn get_color_name(agent_name: &str) -> &'static str {
    match agent_name {
        "Athena" | "Sage" | "SageKeeper" => "Dark Green 1",
        "Master" | "Gaia" | "Wellness" => "Dark Green 2",
        "Debugger" => "Dark Blue 1", 
        "Architect" => "Dark Blue 2",
        "SwiftSpecialist" | "Visionary" | "Hermes" => "Navy Blue",
        "UI" | "UX" => "Dark Purple 1",
        "Designer" => "Dark Purple 2",
        "Optimizer" | "Refactorer" | "Streamliner" => "Deep Violet",
        "Topologist" | "Cartographer" => "Dark Brown 1",
        "Planner" | "Consolidator" => "Dark Brown 2",
        "Memory" | "Recommender" | "Mnemosyne" => "Coffee Brown",
        "Fixer" | "Verifier" | "Enforcer" => "Dark Red 1",
        "Tester" => "Dark Red 2",
        "Analyst" | "Researcher" | "Reactor" => "Burgundy",
        "Developer" | "Engineer" | "Pause" => "Dark Gray 1",
        "Documenter" | "Writer" | "Scholar" | "Human" | "User" => "Charcoal",
        _ => "Default",
    }
}

/// Load color configuration from repository file
pub fn load_color_config() -> Result<HashMap<String, String>> {
    let config_path = std::env::current_dir()?.join("AGENTS").join("agent_colors.json");
    
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;
        
        let mut colors = HashMap::new();
        if let Some(agent_colors) = config.get("agent_colors") {
            if let Some(obj) = agent_colors.as_object() {
                for (agent, data) in obj {
                    if let Some(color_obj) = data.as_object() {
                        if let Some(color) = color_obj.get("color").and_then(|c| c.as_str()) {
                            colors.insert(agent.clone(), color.to_string());
                        }
                    }
                }
            }
        }
        Ok(colors)
    } else {
        // Return hardcoded defaults if config file doesn't exist
        Ok(HashMap::new())
    }
}

/// Get agent color from config or fallback to hardcoded
pub fn get_agent_color_with_config(agent_name: &str) -> Option<String> {
    // Try loading from config file first
    if let Ok(config) = load_color_config() {
        if let Some(color) = config.get(agent_name) {
            return Some(color.clone());
        }
    }
    
    // Fallback to hardcoded colors
    get_agent_color(agent_name).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_color_mapping() {
        assert_eq!(get_agent_color("Athena"), Some("#003300"));
        assert_eq!(get_agent_color("Debugger"), Some("#001133"));
        assert_eq!(get_agent_color("NonExistentAgent"), None);
    }

    #[test]
    fn test_color_names() {
        assert_eq!(get_color_name("Athena"), "Dark Green 1");
        assert_eq!(get_color_name("Debugger"), "Dark Blue 1");
        assert_eq!(get_color_name("UnknownAgent"), "Default");
    }
}