//! BRAIN management commands for Collaborative Intelligence
//!
//! This module provides commands to register, check, and manage the CI BRAIN system.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use dirs;
use colored::Colorize;
use crate::config::Config;
use crate::helpers::CommandHelpers;

/// Configuration file for storing BRAIN location
const BRAIN_CONFIG_FILE: &str = ".ci_brain_config";

/// Get the path to the BRAIN configuration file
fn get_brain_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(BRAIN_CONFIG_FILE))
}

/// Read the registered BRAIN path from config
fn read_brain_path() -> Result<PathBuf> {
    let config_path = get_brain_config_path()?;
    if !config_path.exists() {
        return Err(anyhow::anyhow!("BRAIN not registered. Use 'ci brain register <path>' first."));
    }
    
    let content = fs::read_to_string(&config_path)
        .context("Failed to read BRAIN configuration")?;
    let path = content.trim();
    Ok(PathBuf::from(path))
}

/// Write the BRAIN path to config
fn write_brain_path(path: &Path) -> Result<()> {
    let config_path = get_brain_config_path()?;
    fs::write(&config_path, path.to_string_lossy().as_bytes())
        .context("Failed to write BRAIN configuration")?;
    Ok(())
}

/// Register a new BRAIN location
pub async fn register_brain(path: &str) -> Result<()> {
    CommandHelpers::print_command_header(
        "Register BRAIN Location", 
        "🧠", 
        "BRAIN Management", 
        "cyan"
    );

    let brain_path = Path::new(path);
    
    // Validate the path exists
    if !brain_path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path));
    }
    
    // Check if it looks like a CollaborativeIntelligence directory
    let brain_dir = brain_path.join("BRAIN");
    if !brain_dir.exists() {
        return Err(anyhow::anyhow!(
            "Path does not contain a BRAIN directory: {}\nExpected: {}",
            path,
            brain_dir.display()
        ));
    }
    
    // Count BRAIN files
    let brain_files = count_brain_files(&brain_dir)?;
    if brain_files == 0 {
        return Err(anyhow::anyhow!(
            "BRAIN directory contains no markdown files: {}",
            brain_dir.display()
        ));
    }
    
    // Write the configuration
    write_brain_path(brain_path)?;
    
    println!("✅ {}", "BRAIN registered successfully!".green().bold());
    println!("   📍 Location: {}", brain_path.display().to_string().cyan());
    println!("   📚 Found {} BRAIN files", brain_files.to_string().yellow());
    println!("   🔧 Config: {}", get_brain_config_path()?.display().to_string().dimmed());
    
    Ok(())
}

/// Count markdown files in BRAIN directory
fn count_brain_files(brain_dir: &Path) -> Result<usize> {
    let mut count = 0;
    
    if let Ok(entries) = fs::read_dir(brain_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                count += 1;
            } else if path.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file() && sub_path.extension().map_or(false, |ext| ext == "md") {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    
    Ok(count)
}

/// Check BRAIN health and status
pub async fn health_check() -> Result<()> {
    CommandHelpers::print_command_header(
        "BRAIN Health Check", 
        "🏥", 
        "BRAIN Management", 
        "green"
    );

    match read_brain_path() {
        Ok(brain_path) => {
            let brain_dir = brain_path.join("BRAIN");
            
            println!("🔍 {}", "Checking BRAIN health...".cyan());
            println!();
            
            // Check path accessibility
            if brain_path.exists() {
                println!("✅ BRAIN path accessible: {}", brain_path.display().to_string().green());
            } else {
                println!("❌ BRAIN path not accessible: {}", brain_path.display().to_string().red());
                return Err(anyhow::anyhow!("BRAIN path not accessible"));
            }
            
            // Check BRAIN directory
            if brain_dir.exists() {
                println!("✅ BRAIN directory found: {}", brain_dir.display().to_string().green());
            } else {
                println!("❌ BRAIN directory missing: {}", brain_dir.display().to_string().red());
                return Err(anyhow::anyhow!("BRAIN directory missing"));
            }
            
            // Count and validate files
            let brain_files = count_brain_files(&brain_dir)?;
            if brain_files > 0 {
                println!("✅ BRAIN files found: {} markdown files", brain_files.to_string().green());
            } else {
                println!("❌ No BRAIN files found in directory");
                return Err(anyhow::anyhow!("No BRAIN files found"));
            }
            
            // Test file reading
            if let Ok(entries) = fs::read_dir(&brain_dir) {
                let mut readable_files = 0;
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if !content.trim().is_empty() {
                                readable_files += 1;
                            }
                        }
                        if readable_files >= 3 { break; } // Test first few files
                    }
                }
                
                if readable_files > 0 {
                    println!("✅ BRAIN files readable: {} files tested", readable_files.to_string().green());
                } else {
                    println!("❌ BRAIN files not readable");
                    return Err(anyhow::anyhow!("BRAIN files not readable"));
                }
            }
            
            println!();
            println!("🎉 {}", "BRAIN is healthy and ready!".green().bold());
        }
        Err(e) => {
            println!("❌ BRAIN not registered");
            println!("   Use: {} to register BRAIN location", "ci brain register <path>".yellow());
            return Err(e);
        }
    }
    
    Ok(())
}

/// Show BRAIN source information
pub async fn show_source() -> Result<()> {
    CommandHelpers::print_command_header(
        "BRAIN Source Information", 
        "📊", 
        "BRAIN Management", 
        "blue"
    );

    match read_brain_path() {
        Ok(brain_path) => {
            let brain_dir = brain_path.join("BRAIN");
            
            println!("📍 {}: {}", "BRAIN Location".cyan().bold(), brain_path.display());
            println!("📂 {}: {}", "BRAIN Directory".cyan().bold(), brain_dir.display());
            println!();
            
            // List BRAIN files
            if let Ok(entries) = fs::read_dir(&brain_dir) {
                let mut files = Vec::new();
                let mut dirs = Vec::new();
                
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    let name = path.file_name().unwrap().to_string_lossy();
                    
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                        let size = fs::metadata(&path)?.len();
                        files.push((name.to_string(), size));
                    } else if path.is_dir() {
                        let count = count_brain_files(&path).unwrap_or(0);
                        if count > 0 {
                            dirs.push((name.to_string(), count));
                        }
                    }
                }
                
                if !files.is_empty() {
                    println!("📄 {} ({} files):", "Root BRAIN Files".yellow().bold(), files.len());
                    files.sort_by(|a, b| a.0.cmp(&b.0));
                    for (name, size) in files {
                        let size_kb = size / 1024;
                        println!("   • {} ({} KB)", name.green(), size_kb.to_string().dimmed());
                    }
                    println!();
                }
                
                if !dirs.is_empty() {
                    println!("📁 {} ({} directories):", "BRAIN Subdirectories".yellow().bold(), dirs.len());
                    dirs.sort_by(|a, b| a.0.cmp(&b.0));
                    for (name, count) in dirs {
                        println!("   • {} ({} files)", name.cyan(), count.to_string().dimmed());
                    }
                    println!();
                }
                
                let total_files = count_brain_files(&brain_dir)?;
                println!("📊 {}: {}", "Total BRAIN Files".cyan().bold(), total_files.to_string().yellow().bold());
            }
        }
        Err(e) => {
            println!("❌ BRAIN not registered");
            println!("   Use: {} to register BRAIN location", "ci brain register <path>".yellow());
            return Err(e);
        }
    }
    
    Ok(())
}

/// Test BRAIN functionality
pub async fn test_brain() -> Result<()> {
    CommandHelpers::print_command_header(
        "BRAIN Functionality Test", 
        "🧪", 
        "BRAIN Management", 
        "magenta"
    );

    match read_brain_path() {
        Ok(brain_path) => {
            let brain_dir = brain_path.join("BRAIN");
            
            println!("🔬 {}", "Running BRAIN functionality tests...".cyan());
            println!();
            
            let mut tests_passed = 0;
            let mut tests_total = 0;
            
            // Test 1: Path accessibility
            tests_total += 1;
            if brain_path.exists() {
                println!("✅ Test 1: Path accessibility");
                tests_passed += 1;
            } else {
                println!("❌ Test 1: Path accessibility");
            }
            
            // Test 2: BRAIN directory exists
            tests_total += 1;
            if brain_dir.exists() {
                println!("✅ Test 2: BRAIN directory exists");
                tests_passed += 1;
            } else {
                println!("❌ Test 2: BRAIN directory exists");
            }
            
            // Test 3: Files are readable
            tests_total += 1;
            let mut readable_content = false;
            if let Ok(entries) = fs::read_dir(&brain_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if !content.trim().is_empty() {
                                readable_content = true;
                                break;
                            }
                        }
                    }
                }
            }
            if readable_content {
                println!("✅ Test 3: Files are readable");
                tests_passed += 1;
            } else {
                println!("❌ Test 3: Files are readable");
            }
            
            // Test 4: Environment variables can be set
            tests_total += 1;
            std::env::set_var("CI_BRAIN_TEST", "test_value");
            if std::env::var("CI_BRAIN_TEST").unwrap_or_default() == "test_value" {
                println!("✅ Test 4: Environment variables work");
                tests_passed += 1;
                std::env::remove_var("CI_BRAIN_TEST");
            } else {
                println!("❌ Test 4: Environment variables work");
            }
            
            println!();
            
            let success_rate = (tests_passed as f32 / tests_total as f32) * 100.0;
            if tests_passed == tests_total {
                println!("🎉 {} ({}/{} tests passed)", "All tests passed!".green().bold(), tests_passed, tests_total);
            } else {
                println!("⚠️  {} ({}/{} tests passed, {:.1}%)", 
                    "Some tests failed".yellow().bold(), 
                    tests_passed, 
                    tests_total,
                    success_rate
                );
            }
        }
        Err(e) => {
            println!("❌ BRAIN not registered");
            println!("   Use: {} to register BRAIN location", "ci brain register <path>".yellow());
            return Err(e);
        }
    }
    
    Ok(())
}

/// Show current BRAIN status
pub async fn show_status() -> Result<()> {
    CommandHelpers::print_command_header(
        "BRAIN Status", 
        "📋", 
        "BRAIN Management", 
        "white"
    );

    match read_brain_path() {
        Ok(brain_path) => {
            let brain_dir = brain_path.join("BRAIN");
            let config_path = get_brain_config_path()?;
            
            println!("🔍 {}", "Current BRAIN Configuration:".cyan().bold());
            println!();
            println!("📍 Registered Path: {}", brain_path.display().to_string().green());
            println!("📂 BRAIN Directory: {}", brain_dir.display().to_string().cyan());
            println!("🔧 Config File: {}", config_path.display().to_string().dimmed());
            
            // Check if accessible
            let accessible = brain_path.exists();
            println!("🔓 Accessible: {}", if accessible { "Yes".green() } else { "No".red() });
            
            if accessible {
                let brain_files = count_brain_files(&brain_dir).unwrap_or(0);
                println!("📚 BRAIN Files: {}", brain_files.to_string().yellow());
                
                // Environment variables
                let env_path = std::env::var("CI_BRAIN_PATH").unwrap_or_default();
                let env_available = std::env::var("CI_BRAIN_AVAILABLE").unwrap_or_default();
                
                if !env_path.is_empty() || !env_available.is_empty() {
                    println!();
                    println!("🌐 {}", "Environment Variables:".cyan().bold());
                    if !env_path.is_empty() {
                        println!("   CI_BRAIN_PATH: {}", env_path.green());
                    }
                    if !env_available.is_empty() {
                        println!("   CI_BRAIN_AVAILABLE: {}", env_available.green());
                    }
                }
            }
        }
        Err(_) => {
            println!("❌ {}", "BRAIN not registered".red().bold());
            println!();
            println!("To register BRAIN location:");
            println!("  {}", "ci brain register /path/to/CollaborativeIntelligence".yellow());
            println!();
            println!("Available commands:");
            println!("  {} - Register BRAIN location", "ci brain register <path>".cyan());
            println!("  {} - Check BRAIN health", "ci brain health".cyan());
            println!("  {} - Show BRAIN information", "ci brain source".cyan());
            println!("  {} - Test BRAIN functionality", "ci brain test".cyan());
        }
    }
    
    Ok(())
}

/// Handle brain commands
pub async fn handle_brain_command(command: &crate::BrainCommands, _config: &Config) -> Result<()> {
    match command {
        crate::BrainCommands::Register { path } => {
            register_brain(path).await
        }
        crate::BrainCommands::Health => {
            health_check().await
        }
        crate::BrainCommands::Source => {
            show_source().await
        }
        crate::BrainCommands::Test => {
            test_brain().await
        }
        crate::BrainCommands::Status => {
            show_status().await
        }
    }
}