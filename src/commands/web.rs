//! Web portal management commands
//!
//! This module provides commands to manage the Collaborative Intelligence web portal,
//! including opening it, running development server, and other web-related operations.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::helpers::CommandHelpers;

/// Handle web portal commands
pub async fn web_command(subcommand: Option<&str>, dev: bool, config: &Config) -> Result<()> {
    match subcommand {
        Some("open") => open_web_portal(dev, config).await,
        Some("deploy") => deploy_web_portal(config).await,
        Some(cmd) => {
            CommandHelpers::print_error(&format!("Unknown web command: {}", cmd));
            display_web_help();
            Ok(())
        },
        None => {
            // Show help by default when no subcommand is provided
            display_web_help();
            Ok(())
        }
    }
}

/// Open the web portal
async fn open_web_portal(dev: bool, config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Open Web Portal", 
        "🌐", 
        "Web Portal Management", 
        "purple"
    );

    // Find the CollaborativeIntelligence web directory
    let web_path = find_web_directory()?;
    
    if !web_path.exists() {
        CommandHelpers::print_error("Web directory not found in CollaborativeIntelligence project");
        return Ok(());
    }

    // Check if package.json exists
    let package_json = web_path.join("package.json");
    if !package_json.exists() {
        CommandHelpers::print_error("package.json not found in web directory");
        return Ok(());
    }

    println!("📂 Web directory: {}", web_path.display().to_string().cyan());

    if dev {
        // Run development server (npm start)
        run_dev_server(&web_path).await
    } else {
        // Default behavior is also to run dev server
        run_dev_server(&web_path).await
    }
}

/// Run the development server
async fn run_dev_server(web_path: &PathBuf) -> Result<()> {
    println!("🚀 Starting development server...\n");
    
    // Change to web directory and run npm start
    let mut cmd = Command::new("npm");
    cmd.arg("start")
       .current_dir(web_path);

    // Execute the command and stream output
    let status = cmd.status()
        .context("Failed to execute npm start")?;

    if status.success() {
        CommandHelpers::print_success("Development server started successfully");
    } else {
        CommandHelpers::print_error("Failed to start development server");
    }

    Ok(())
}

/// Deploy the web portal
async fn deploy_web_portal(config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Deploy Web Portal", 
        "🚀", 
        "Web Portal Management", 
        "purple"
    );

    // Find the CollaborativeIntelligence web directory
    let web_path = find_web_directory()?;
    
    if !web_path.exists() {
        CommandHelpers::print_error("Web directory not found in CollaborativeIntelligence project");
        return Ok(());
    }

    // Check if package.json exists
    let package_json = web_path.join("package.json");
    if !package_json.exists() {
        CommandHelpers::print_error("package.json not found in web directory");
        return Ok(());
    }

    println!("📂 Web directory: {}", web_path.display().to_string().cyan());

    // Run npm build first (production build)
    println!("📦 Building project for production...\n");
    let build_status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(&web_path)
        .status()
        .context("Failed to execute npm run build")?;

    if !build_status.success() {
        CommandHelpers::print_error("Build failed. Cannot proceed with deployment");
        return Ok(());
    }

    println!("\n✅ Build completed successfully");

    // Check for Vercel configuration first
    let vercel_config = web_path.join(".vercel");
    if vercel_config.exists() {
        println!("🚀 Deploying to Vercel...\n");
        
        let deploy_status = Command::new("vercel")
            .arg("--prod")
            .arg("--yes")  // Skip confirmation prompts
            .current_dir(&web_path)
            .status()
            .context("Failed to execute vercel deploy")?;

        if deploy_status.success() {
            CommandHelpers::print_success("Web portal deployed to Vercel successfully");
            println!("🌐 Your web portal is now live!");
        } else {
            CommandHelpers::print_error("Vercel deployment failed");
        }
    } else if has_deploy_script(&web_path)? {
        println!("🚀 Running deployment script...\n");
        
        let deploy_status = Command::new("npm")
            .arg("run")
            .arg("deploy")
            .current_dir(&web_path)
            .status()
            .context("Failed to execute npm run deploy")?;

        if deploy_status.success() {
            CommandHelpers::print_success("Web portal deployed successfully");
        } else {
            CommandHelpers::print_error("Deployment script failed");
        }
    } else {
        println!("ℹ️  Build completed. Manual deployment required.");
        println!("   Build files are available in the project's build/dist directory");
        
        // Check for common build output directories
        let build_dirs = ["build", "dist", "out"];
        for dir in &build_dirs {
            let build_path = web_path.join(dir);
            if build_path.exists() {
                println!("   📁 Build output: {}", build_path.display().to_string().cyan());
                break;
            }
        }
    }

    Ok(())
}

/// Check if the project has a deploy script in package.json
fn has_deploy_script(web_path: &PathBuf) -> Result<bool> {
    let package_json = web_path.join("package.json");
    let content = std::fs::read_to_string(&package_json)
        .context("Failed to read package.json")?;
    
    // Simple check for deploy script - could be enhanced with proper JSON parsing
    Ok(content.contains("\"deploy\""))
}

/// Find the CollaborativeIntelligence web directory
fn find_web_directory() -> Result<PathBuf> {
    // Try to find the CollaborativeIntelligence directory
    // Start from current directory and work up
    let mut current = std::env::current_dir()
        .context("Failed to get current directory")?;

    // Look for CollaborativeIntelligence directory at various levels
    loop {
        let ci_path = current.join("CollaborativeIntelligence");
        if ci_path.exists() {
            let web_path = ci_path.join("web");
            if web_path.exists() {
                return Ok(web_path);
            }
        }

        // Also check parent directories
        match current.parent() {
            Some(parent) => {
                current = parent.to_path_buf();
                // Don't go beyond the user's home directory
                if let Ok(home) = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir")) {
                    if current == home {
                        break;
                    }
                }
            },
            None => break,
        }
    }

    // Try absolute path to user's home directory
    if let Some(home) = dirs::home_dir() {
        let ci_web = home.join("Documents/Projects/CollaborativeIntelligence/web");
        if ci_web.exists() {
            return Ok(ci_web);
        }
    }

    Err(anyhow::anyhow!("Could not find CollaborativeIntelligence/web directory"))
}

/// Display help for web commands
fn display_web_help() {
    CommandHelpers::print_command_header(
        "Web Portal Management", 
        "🌐", 
        "Collaborative Intelligence", 
        "purple"
    );

    println!("\n{}", "Available Web Commands:".bold());
    println!("  {:<12} {}", "open".purple(), "Open the web portal (runs development server)");
    println!("  {:<12} {}", "deploy".purple(), "Deploy the web portal (build + deploy)");
    println!("\n{}", "Options:".bold());
    println!("  {:<12} {}", "--dev".purple(), "Run in development mode (for open command)");
    println!("\n{}", "Examples:".bold());
    println!("  ci web               # Show this help");
    println!("  ci web open          # Start development server");
    println!("  ci web open --dev    # Explicitly start development server");
    println!("  ci web deploy        # Build and deploy the web portal");
}