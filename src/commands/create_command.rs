use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::config::Config;
use crate::helpers::CommandHelpers;
use crate::commands::generator::CommandGenerator;

pub async fn create_command(name: &str, description: &str, category: Option<&str>, __config: &Config) -> Result<()> {
    println!("{}", "⚙️ Command Generator".cyan().bold());
    println!("{}", "==================".cyan());
    println!();
    println!("📝 {}", format!("Creating command: {}", name.yellow().bold()));
    println!();
    
    // Determine the category based on description if not provided
    let actual_category = if let Some(cat) = category {
        println!("🏷️  {}: {}", "Using provided category".cyan(), cat.green());
        cat
    } else {
        let detected = CommandGenerator::categorize_command(description);
        println!("🧠 {}: {}", "Auto-detected category".cyan(), detected.green());
        detected
    };
    
    println!();
    println!("📋 {}", "Command details:".yellow().bold());
    println!("   {}: {}", "Name".blue(), name);
    println!("   {}: {}", "Description".blue(), description);
    println!("   {}: {}", "Category".blue(), actual_category);
    println!();
    
    println!("🔨 {}", "Generating command implementation...".cyan());
    
    // Create the command
    CommandGenerator::create_command(name, description, actual_category)?;
    
    println!();
    println!("{} {}", "✅".green(), "Command created successfully".green().bold());
    println!();
    println!("{} {}", "💡".yellow(), "To use this new command:".bold());
    println!("   {}", format!("ci {}", name.to_lowercase()).cyan());
    println!();
    println!("{} {}", "📝".blue(), "You may need to run 'ci rebuild' for the new command to take effect".italic());
    
    Ok(())
}