use colored::Colorize;

/// Print version information 
pub fn version() {
    let version = env!("CARGO_PKG_VERSION");
    
    println!("{}", "📊 CI Version Information".green().bold());
    println!("{}", "======================".green());
    println!();
    println!("Collaborative Intelligence CLI {}", format!("v{}", version).yellow().bold());
    println!();
    println!("🛠️  Built with {}", "Rust".yellow().bold());
    println!("🧠 For the {} system", "Collaborative Intelligence".blue().bold());
    println!();
    println!("📝 {}", "Repository: https://github.com/joshkornreich/ci".cyan());
}