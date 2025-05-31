use anyhow::{Result, Context};
use colored::Colorize;
use std::process::Command;

use crate::config::Config;
use crate::KeyCommands;
use crate::helpers::api_keys::ApiKeyCommands;
use crate::helpers::command::CommandHelpers;
use crate::helpers::SystemHelpers;
use crate::tools::DocumentationGenerator;

pub async fn evolve(__config: &Config) -> Result<()> {
    println!("{}", "🌱 Evolving CI Tool".green().bold());
    println!("{}", "================".green());
    println!();
    println!("🧠 {}", "Using Claude Code to improve CI...".yellow());
    println!();
    println!("⏳ {}", "Functionality not yet implemented".cyan().italic());
    println!();
    println!("Future versions will automatically evolve the CI tool using Claude Code assistance.");
    Ok(())
}

pub async fn key(command: &Option<KeyCommands>, __config: &Config) -> Result<()> {
    // Direct formatting to match CI format exactly
    println!("{}", "⚙️ API Key Management".blue().bold());
    println!("{}", "==================".blue());
    println!();
    
    match command {
        Some(KeyCommands::List) => {
            ApiKeyCommands::list_keys_cli()?;
        },
        Some(KeyCommands::Add { service, key_name, key_value, env, project }) => {
            ApiKeyCommands::set_key_cli(
                service,
                key_name,
                key_value,
                env.as_deref(),
                *project
            )?;
        },
        Some(KeyCommands::Remove { service, key_name, env }) => {
            ApiKeyCommands::remove_key_cli(
                service,
                key_name,
                env.as_deref()
            )?;
        },
        Some(KeyCommands::Export) => {
            ApiKeyCommands::export_keys_cli()?;
        },
        None => {
            println!("{} {}:", "ℹ️".blue(), "API key management commands".bold());
            println!();
            println!("  {} {}", "ci key list".blue(), "- List all stored API keys");
            println!("  {} {}", "ci key add SERVICE KEY_NAME KEY_VALUE".blue(), "- Add a new API key");
            println!("  {} {}", "ci key add SERVICE KEY_NAME KEY_VALUE --env ENV".blue(), "- Add an environment-specific API key");
            println!("  {} {}", "ci key add SERVICE KEY_NAME KEY_VALUE --project".blue(), "- Add a project-specific API key");
            println!("  {} {}", "ci key remove SERVICE KEY_NAME".blue(), "- Remove an API key");
            println!("  {} {}", "ci key export".blue(), "- Export API keys for shell environment");
            println!();
            println!("{} {}:", "📝".yellow(), "Examples".bold());
            println!("  {} {}", "ci key add openai api_key sk-abcdef123456".blue(), "- Add OpenAI API key");
            println!("  {} {}", "ci key add anthropic api_key sk-ant-abcdef123456 --env development".blue(), "- Add Anthropic API key for development");
            println!();
            println!("{} {}:", "💡".green(), "Shell Integration".bold());
            println!("  {}", "eval \"$(ci key export)\"".blue());
        },
    }
    
    Ok(())
}

pub async fn rebuild(__config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Rebuilding CI Binary", 
        "🔨", 
        "System Management", 
        "blue"
    );
    
    println!("🚀 {}", "Starting rebuild process...".blue());
    
    // Check for cargo
    if !SystemHelpers::command_exists("cargo") {
        CommandHelpers::print_error("Rust and Cargo are required but not installed.");
        println!("Please install Rust from https://rustup.rs/ and try again.");
        return Err(anyhow::anyhow!("Cargo not found"));
    }
    
    // Run cargo check
    CommandHelpers::with_progress("Checking code", || {
        let _output = Command::new("cargo")
            .arg("check")
            .output()
            .context("Failed to execute cargo check")?;
            
        if !_output.status.success() {
            CommandHelpers::print_error("Code check failed.");
            println!("{}", String::from_utf8_lossy(&_output.stderr));
            return Err(anyhow::anyhow!("Cargo check failed"));
        }
        
        Ok(())
    })?;
    
    // Run cargo build
    CommandHelpers::with_progress("Rebuilding binary", || {
        let _output = Command::new("cargo")
            .arg("build")
            .output()
            .context("Failed to execute cargo build")?;
            
        if !_output.status.success() {
            CommandHelpers::print_error("Rebuild failed.");
            println!("{}", String::from_utf8_lossy(&_output.stderr));
            return Err(anyhow::anyhow!("Cargo build failed"));
        }
        
        Ok(())
    })?;
    
    CommandHelpers::print_success("Rebuild successful!");
    println!();
    println!("🚀 {}", "CI rebuilt successfully!".green());
    println!("You can run the binary with: {}", "./target/debug/CI".yellow());
    println!("Or install it with: {}", "ci install".yellow());
    
    Ok(())
}

pub async fn install(__config: &Config) -> Result<()> {
    
    use std::fs;
    use std::env;
    
    CommandHelpers::print_command_header(
        "Installing CI Tool", 
        "📦", 
        "System Management", 
        "blue"
    );
    
    println!("🔧 {}", "Installing to system path...".blue());
    
    // Check for cargo
    if !SystemHelpers::command_exists("cargo") {
        CommandHelpers::print_error("Rust and Cargo are required but not installed.");
        println!("Please install Rust from https://rustup.rs/ and try again.");
        return Err(anyhow::anyhow!("Cargo not found"));
    }
    
    // Build release version
    CommandHelpers::with_progress("Building release version", || {
        let _output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .output()
            .context("Failed to execute cargo build --release")?;
            
        if !_output.status.success() {
            CommandHelpers::print_error("Release build failed.");
            println!("{}", String::from_utf8_lossy(&_output.stderr));
            return Err(anyhow::anyhow!("Cargo build --release failed"));
        }
        
        Ok(())
    })?;
    
    // Determine installation directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let install_dir = home_dir.join(".local/bin");
    
    // Create installation directory if it doesn't exist
    CommandHelpers::with_progress(&format!("Creating installation directory: {}", install_dir.display()), || {
        fs::create_dir_all(&install_dir)
            .context(format!("Failed to create directory: {}", install_dir.display()))?;
        Ok(())
    })?;
    
    // Get the binary path
    let current_dir = env::current_dir()
        .context("Failed to get current directory")?;
    let binary_path = current_dir.join("target/release/CI");
    
    if !binary_path.exists() {
        CommandHelpers::print_error(&format!("Binary not found at: {}", binary_path.display()));
        return Err(anyhow::anyhow!("Binary not found"));
    }
    
    // Copy binary to installation directory
    let destination = install_dir.join("CI");
    CommandHelpers::with_progress(&format!("Installing binary to {}", destination.display()), || {
        fs::copy(&binary_path, &destination)
            .context(format!("Failed to copy binary to {}", destination.display()))?;
        
        // Make binary executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&destination)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&destination, perms)?;
        }
        
        Ok(())
    })?;
    
    // Check if installation directory is in PATH
    let path_env = env::var("PATH").unwrap_or_default();
    let install_dir_str = install_dir.to_string_lossy().to_string();
    
    if !path_env.split(':').any(|p| p == install_dir_str) {
        CommandHelpers::print_warning(&format!("{} is not in your PATH", install_dir.display()));
        
        // Determine shell configuration file
        let shell_config = if env::var("SHELL").unwrap_or_default().contains("zsh") {
            home_dir.join(".zshrc")
        } else if env::var("SHELL").unwrap_or_default().contains("bash") {
            if cfg!(target_os = "macos") {
                home_dir.join(".bash_profile")
            } else {
                home_dir.join(".bashrc")
            }
        } else {
            home_dir.join(".profile")
        };
        
        if shell_config.exists() {
            println!("You may want to add the following line to your {}:", shell_config.display());
            println!("export PATH=\"$PATH:{}\"", install_dir.display());
        } else {
            println!("Please add {} to your PATH.", install_dir.display());
        }
    }
    
    CommandHelpers::print_success("CI installed successfully!");
    println!();
    println!("You can now run '{}'", "CI".blue());
    
    // Create symlinks
    let lowercase_dest = install_dir.join("ci");
    if !lowercase_dest.exists() {
        CommandHelpers::with_progress("Creating lowercase symlink", || {
            SystemHelpers::create_symlink(&destination, &lowercase_dest)?;
            Ok(())
        })?;
    }
    
    Ok(())
}

pub async fn link(__config: &Config) -> Result<()> {
    
    use std::env;
    use crate::helpers::CommandHelpers;
    
    CommandHelpers::print_command_header(
        "Creating Symlinks", 
        "🔗", 
        "System Management", 
        "blue"
    );
    
    println!("📌 {}", "Creating symbolic links to CI binary...".yellow());
    
    // Get the CI binary path
    let ci_path = match SystemHelpers::get_ci_binary_path() {
        Ok(path) => path,
        Err(e) => {
            CommandHelpers::print_error(&format!("Failed to locate CI binary: {}", e));
            return Err(anyhow::anyhow!("CI binary not found"));
        }
    };
    
    // Use local bin directory for user-level installation
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let bin_dir = home_dir.join(".local/bin");
    
    // Create the directory if it doesn't exist
    if !bin_dir.exists() {
        CommandHelpers::with_progress(&format!("Creating bin directory: {}", bin_dir.display()), || {
            std::fs::create_dir_all(&bin_dir)?;
            Ok(())
        })?;
    }
    
    // Create lowercase symlink
    let lowercase_dest = bin_dir.join("ci");
    CommandHelpers::with_progress(&format!("Creating lowercase symlink: {}", lowercase_dest.display()), || {
        if lowercase_dest.exists() && lowercase_dest.is_symlink() {
            std::fs::remove_file(&lowercase_dest)?;
        }
        SystemHelpers::create_symlink(&ci_path, &lowercase_dest)?;
        Ok(())
    })?;
    
    // Check if .local/bin is in PATH
    let path_env = env::var("PATH").unwrap_or_default();
    let bin_dir_str = bin_dir.to_string_lossy();
    if !path_env.split(':').any(|p| p == bin_dir_str) {
        CommandHelpers::print_warning(&format!("{} is not in your PATH", bin_dir.display()));
        
        // Determine shell configuration file
        let shell_config = if env::var("SHELL").unwrap_or_default().contains("zsh") {
            home_dir.join(".zshrc")
        } else if env::var("SHELL").unwrap_or_default().contains("bash") {
            if cfg!(target_os = "macos") {
                home_dir.join(".bash_profile")
            } else {
                home_dir.join(".bashrc")
            }
        } else {
            home_dir.join(".profile")
        };
        
        if shell_config.exists() {
            println!("You may want to add the following line to your {}:", shell_config.display());
            println!("export PATH=\"$PATH:{}\"", bin_dir.display());
        } else {
            println!("Please add {} to your PATH.", bin_dir.display());
        }
    }
    
    // Check if you want to create legacy symlinks
    if CommandHelpers::prompt_confirmation("Create legacy CI symlinks?") {
        let legacy_uppercase = bin_dir.join("CI");
        let legacy_lowercase = bin_dir.join("ci");
        
        CommandHelpers::with_progress("Creating legacy symlinks", || {
            if legacy_uppercase.exists() && legacy_uppercase.is_symlink() {
                std::fs::remove_file(&legacy_uppercase)?;
            }
            if legacy_lowercase.exists() && legacy_lowercase.is_symlink() {
                std::fs::remove_file(&legacy_lowercase)?;
            }
            
            SystemHelpers::create_symlink(&ci_path, &legacy_uppercase)?;
            SystemHelpers::create_symlink(&ci_path, &legacy_lowercase)?;
            Ok(())
        })?;
    }
    
    CommandHelpers::print_success("Symlinks created successfully");
    Ok(())
}

pub async fn unlink(__config: &Config) -> Result<()> {
    use std::path::PathBuf;
    use std::fs;
    use crate::helpers::CommandHelpers;
    
    CommandHelpers::print_command_header(
        "Removing Symlinks", 
        "❌", 
        "System Management", 
        "blue"
    );
    
    println!("🧹 {}", "Cleaning up symbolic links to CI binary...".yellow());
    
    // Get the CI binary path
    let ci_path = match SystemHelpers::get_ci_binary_path() {
        Ok(path) => path,
        Err(e) => {
            CommandHelpers::print_error(&format!("Failed to locate CI binary: {}", e));
            return Err(anyhow::anyhow!("CI binary not found"));
        }
    };
    
    // Directory to check for symlinks
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let bin_dirs = vec![
        home_dir.join(".local/bin"),
        home_dir.join("bin"),
        PathBuf::from("/usr/local/bin"),
    ];
    
    let mut removed_count = 0;
    
    // Possible symlink names
    let symlink_names = vec!["ci", "CI", "ci", "CI"];
    
    for bin_dir in bin_dirs {
        if !bin_dir.exists() {
            continue;
        }
        
        for name in &symlink_names {
            let symlink_path = bin_dir.join(name);
            
            // Check if it's a symlink to our binary
            if symlink_path.exists() && symlink_path.is_symlink() {
                if let Ok(target) = fs::read_link(&symlink_path) {
                    // Only remove if it points to our binary
                    if target == ci_path {
                        CommandHelpers::with_progress(&format!("Removing symlink: {}", symlink_path.display()), || {
                            fs::remove_file(&symlink_path)?;
                            Ok(())
                        })?;
                        removed_count += 1;
                    }
                }
            }
        }
    }
    
    if removed_count > 0 {
        CommandHelpers::print_success(&format!("Removed {} symlinks", removed_count));
    } else {
        CommandHelpers::print_info("No symlinks to CI were found");
    }
    
    Ok(())
}

pub async fn version(__config: &Config) -> Result<()> {
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
    
    Ok(())
}

/// Generate comprehensive documentation for the CI tool
pub async fn docs(__config: &Config) -> Result<()> {
    println!("{}", "📚 Generating Documentation".blue().bold());
    println!("{}", "========================".blue());
    println!();
    
    println!("📝 {}", "Analyzing project structure...".yellow());
    
    // Generate comprehensive documentation
    DocumentationGenerator::generate_project_documentation()?;
    
    println!();
    println!("✅ {}", "Documentation generated successfully".green().bold());
    println!();
    println!("📋 Documentation files are available in the project's docs directory");
    
    Ok(())
}

/// Fix common compiler warnings
pub async fn fix_warnings(__config: &Config) -> Result<()> {
    use std::fs;
    
    CommandHelpers::print_command_header(
        "Fixing Compiler Warnings", 
        "🛠️", 
        "System Management", 
        "blue"
    );
    
    // Run cargo check to identify warnings
    CommandHelpers::with_progress("Running cargo check to identify warnings", || {
        let _output = Command::new("cargo")
            .arg("check")
            .output()
            .context("Failed to execute cargo check")?;
            
        Ok(())
    })?;
    
    let mut fixed_count = 0;
    
    // Fix unused imports
    CommandHelpers::with_progress("Fixing unused imports in main.rs", || {
        let main_path = std::env::current_dir()?.join("src/main.rs");
        
        if main_path.exists() {
            let content = fs::read_to_string(&main_path)?;
            
            // Look for and remove unused imports
            let new_content = content
                .lines()
                .filter(|line| {
                    // Skip lines that look like unused imports
                    !line.trim().starts_with("use ") || 
                    !line.contains("CommandHelpers") ||
                    !line.ends_with(";")
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            if new_content != content {
                fs::write(&main_path, new_content)?;
                fixed_count += 1;
            }
        }
        
        Ok(())
    })?;
    
    // Fix unused variables
    CommandHelpers::with_progress("Fixing unused variables", || {
        let files_to_check = vec![
            "src/commands/intelligence.rs",
            "src/commands/source_control.rs",
            "src/commands/lifecycle.rs",
            "src/commands/system.rs",
        ];
        
        for file in files_to_check {
            let file_path = std::env::current_dir()?.join(file);
            
            if file_path.exists() {
                let content = fs::read_to_string(&file_path)?;
                
                // Replace patterns like "function(_config: &Config)" with "function(__config: &Config)"
                // to mark unused variables
                let new_content = content
                    .replace("(_config: &Config)", "(__config: &Config)")
                    .replace(", _config: &Config)", ", __config: &Config)");
                
                if new_content != content {
                    fs::write(&file_path, new_content)?;
                    fixed_count += 1;
                }
            }
        }
        
        Ok(())
    })?;
    
    // Run cargo check again to verify fixes
    CommandHelpers::with_progress("Verifying fixes", || {
        let _output = Command::new("cargo")
            .arg("check")
            .output()
            .context("Failed to execute cargo check")?;
        
        Ok(())
    })?;
    
    if fixed_count > 0 {
        CommandHelpers::print_success(&format!("Fixed {} warnings", fixed_count));
    } else {
        CommandHelpers::print_info("No warnings needed fixing");
    }
    
    println!();
    println!("You can now rebuild the project with: {}", "ci rebuild".yellow());
    
    Ok(())
}

/// Manage CI commands
/// Add a new command to CI
pub async fn add_command(
    name: Option<&str>,
    description: Option<&str>,
    category: Option<&str>,
    __config: &Config
) -> Result<()> {
    use std::io::Write;
    CommandHelpers::print_command_header(
        "Add New Command", 
        "⚙️", 
        "System Management", 
        "blue"
    );
    
    // Get command details if not provided
    let name = match name {
        Some(n) => n.to_string(),
        None => {
            println!("Enter command name:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };
    
    let description = match description {
        Some(d) => d.to_string(),
        None => {
            println!("Enter command description:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };
    
    let category = match category {
        Some(c) => c.to_string(),
        None => {
            println!("Select category:");
            println!("1. Intelligence");
            println!("2. SourceControl");
            println!("3. Lifecycle");
            println!("4. System");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "1" => "Intelligence",
                "2" => "SourceControl",
                "3" => "Lifecycle",
                "4" => "System",
                _ => {
                    CommandHelpers::print_error("Invalid category selection");
                    return Err(anyhow::anyhow!("Invalid category"));
                }
            }.to_string()
        }
    };
    
    // Determine the file to modify based on category
    let file_name = match category.as_str() {
        "Intelligence" => "intelligence.rs",
        "SourceControl" => "source_control.rs",
        "Lifecycle" => "lifecycle.rs",
        "System" => "system.rs",
        _ => {
            CommandHelpers::print_error("Unknown category");
            return Err(anyhow::anyhow!("Unknown category"));
        }
    };
    
    // Generate command function template
    let command_func = format!(r#"
/// {}
pub async fn {}(__config: &Config) -> Result<()> {{
    CommandHelpers::print_command_header(
        "{}", 
        "{}", 
        "{}", 
        "{}"
    );
    
    // TODO: Implement command functionality
    println!("Command functionality not yet implemented");
    
    CommandHelpers::print_success("Command execution complete");
    Ok(())
}}
"#, 
        description,
        name.to_lowercase(),
        description,
        match category.as_str() {
            "Intelligence" => "⚙️",
            "SourceControl" => "📊",
            "Lifecycle" => "🚀",
            "System" => "🧠",
            _ => "📌",
        },
        match category.as_str() {
            "Intelligence" => "Intelligence & Discovery",
            "SourceControl" => "Source Control",
            "Lifecycle" => "Project Lifecycle",
            "System" => "System Management",
            _ => "Miscellaneous",
        },
        match category.as_str() {
            "Intelligence" => "cyan",
            "SourceControl" => "green",
            "Lifecycle" => "yellow",
            "System" => "blue",
            _ => "white",
        }
    );
    
    // Get current directory and find command file
    let current_dir = std::env::current_dir()?;
    let command_file_path = current_dir.join("src/commands").join(file_name);
    
    if !command_file_path.exists() {
        CommandHelpers::print_error(&format!("Command file not found: {}", command_file_path.display()));
        return Err(anyhow::anyhow!("Command file not found"));
    }
    
    // Append command function to the file
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&command_file_path)?;
        
    file.write_all(command_func.as_bytes())?;
    
    CommandHelpers::print_success(&format!("Command '{}' added to {}", name, file_name));
    
    // Recommend next steps
    println!();
    println!("Next steps:");
    println!("1. Add the command to the Commands enum in src/main.rs");
    println!("2. Add a match arm in the main function to call your command");
    println!("3. Implement the command functionality");
    println!("4. Rebuild the project with: cargo build");
    
    Ok(())
}

/// Build, install and create symlinks in one command
pub async fn setup(__config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Complete CI Setup", 
        "🚀", 
        "System Management", 
        "blue"
    );
    
    println!("🔄 {}", "Running complete CI setup (build + install + symlink)...".blue());
    println!();
    
    // Step 1: Build release version
    println!("📦 {} {}", "Step 1/3:".bold(), "Building release version...".yellow());
    
    // Check for cargo
    if !SystemHelpers::command_exists("cargo") {
        CommandHelpers::print_error("Rust and Cargo are required but not installed.");
        println!("Please install Rust from https://rustup.rs/ and try again.");
        return Err(anyhow::anyhow!("Cargo not found"));
    }
    
    // Build release version directly
    CommandHelpers::with_progress("Building release version", || {
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .output()
            .context("Failed to execute cargo build --release")?;
            
        if !output.status.success() {
            CommandHelpers::print_error("Release build failed.");
            println!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Cargo build --release failed"));
        }
        
        Ok(())
    })?;
    
    CommandHelpers::print_success("Release build completed successfully");
    
    println!();
    
    // Step 2: Install to system path
    println!("📦 {} {}", "Step 2/3:".bold(), "Installing to system path...".yellow());
    
    use std::fs;
    use std::env;
    
    // Determine installation directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let install_dir = home_dir.join(".local/bin");
    
    // Create installation directory if it doesn't exist
    CommandHelpers::with_progress(&format!("Creating installation directory: {}", install_dir.display()), || {
        fs::create_dir_all(&install_dir)
            .context(format!("Failed to create directory: {}", install_dir.display()))?;
        Ok(())
    })?;
    
    // Get the binary path
    let current_dir = env::current_dir()
        .context("Failed to get current directory")?;
    let binary_path = current_dir.join("target/release/CI");
    
    if !binary_path.exists() {
        CommandHelpers::print_error(&format!("Binary not found at: {}", binary_path.display()));
        return Err(anyhow::anyhow!("Binary not found"));
    }
    
    // Copy binary to installation directory
    let destination = install_dir.join("CI");
    CommandHelpers::with_progress(&format!("Installing binary to {}", destination.display()), || {
        fs::copy(&binary_path, &destination)
            .context(format!("Failed to copy binary to {}", destination.display()))?;
        
        // Make binary executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&destination)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&destination, perms)?;
        }
        
        Ok(())
    })?;
    
    CommandHelpers::print_success("Installation completed successfully");
    
    println!();
    
    // Step 3: Create symlinks
    println!("🔗 {} {}", "Step 3/3:".bold(), "Creating symlinks...".yellow());
    
    // Create lowercase symlink
    let lowercase_dest = install_dir.join("ci");
    CommandHelpers::with_progress("Creating lowercase symlink", || {
        if lowercase_dest.exists() {
            std::fs::remove_file(&lowercase_dest)?;
        }
        SystemHelpers::create_symlink(&destination, &lowercase_dest)?;
        Ok(())
    })?;
    
    CommandHelpers::print_success("Symlinks created successfully");
    
    // Check if installation directory is in PATH
    let path_env = env::var("PATH").unwrap_or_default();
    let install_dir_str = install_dir.to_string_lossy();
    if !path_env.split(':').any(|p| p == install_dir_str) {
        CommandHelpers::print_warning(&format!("{} is not in your PATH", install_dir.display()));
        
        // Determine shell configuration file
        let shell_config = if env::var("SHELL").unwrap_or_default().contains("zsh") {
            home_dir.join(".zshrc")
        } else if env::var("SHELL").unwrap_or_default().contains("bash") {
            if cfg!(target_os = "macos") {
                home_dir.join(".bash_profile")
            } else {
                home_dir.join(".bashrc")
            }
        } else {
            home_dir.join(".profile")
        };
        
        if shell_config.exists() {
            println!("You may want to add the following line to your {}:", shell_config.display());
            println!("export PATH=\"$PATH:{}\"", install_dir.display());
        } else {
            println!("Please add {} to your PATH.", install_dir.display());
        }
        println!();
    }
    
    CommandHelpers::print_success("🎉 Complete CI setup finished successfully!");
    println!();
    println!("You can now use {} or {} to run CI commands", "ci".green().bold(), "CI".green().bold());
    
    Ok(())
}

pub async fn command(subcommand: &str, __config: &Config) -> Result<()> {
    use crate::helpers::CommandHelpers;
    use std::process::Command;
    use std::env;
    use std::fs;
    
    
    
    match subcommand {
        "create" => {
            // Create a new command with Claude Code's assistance
            CommandHelpers::print_command_header(
                "Create New Command", 
                "⚡", 
                "System Management", 
                "blue"
            );
            
            // Get current directory to find the source code
            let current_dir = env::current_dir()
                .context("Failed to get current directory")?;
            let src_dir = current_dir.join("src");
            
            // Verify source directory exists
            if !src_dir.exists() {
                CommandHelpers::print_error("Could not find src directory");
                return Err(anyhow::anyhow!("Source directory not found"));
            }
            
            // Create temporary prompt file for Claude Code
            let temp_dir = env::temp_dir();
            let prompt_file = temp_dir.join("ci_command_create.md");
            
            // Build detailed prompt for creating a new command
            let prompt = r#"# CI Command Creation

You're tasked with creating a new command for the CI (Collaborative Intelligence) command-line tool. I'll provide details about the command I want to add, and you'll implement it according to the project's conventions.

## Project Structure

Commands are organized by category in these files:
- `src/commands/intelligence.rs` - Intelligence & Discovery commands (⚙️ Cyan)
- `src/commands/source_control.rs` - Source Control commands (📊 Green)
- `src/commands/lifecycle.rs` - Project Lifecycle commands (🚀 Yellow)
- `src/commands/system.rs` - System Management commands (🧠 Blue)

## Command Implementation Template

```rust
/// [Command description]
pub async fn command_name(__config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "[Header text]", 
        "[Category emoji]", 
        "[Category name]", 
        "[Category color]"
    );
    
    // Command implementation
    
    CommandHelpers::print_success("[Success message]");
    return Ok(());
}
```

## Adding to main.rs

Commands need to be added to:
1. The `Commands` enum
2. The match statement in `main.rs`

## Command Details

Please provide the following information about the command you want to create:
1. Command name
2. Command description
3. Category (Intelligence, Source Control, Project Lifecycle, System Management)
4. Specific functionality you want the command to implement

## Example

For a command that displays Git statistics:
- Name: `stats`
- Description: "Display repository statistics"
- Category: Source Control
- Functionality: Show commit count, contributor count, file count, etc.

# Next Steps

Once I have your command details, I'll generate:
1. The full command implementation
2. Code to add to main.rs
3. Documentation if needed

Let's begin creating your command!"#;

            // Write prompt to temporary file
            fs::write(&prompt_file, prompt)?;
            
            // Launch Claude Code with the prompt
            println!("🚀 {}", "Launching Claude Code to create a new command...".blue());
            println!();
            println!("{} {}", "💡".yellow(), "Provide the following details:".bold());
            println!("  1. Command name");
            println!("  2. Description of what the command should do");
            println!("  3. Category (Intelligence, Source Control, Lifecycle, System)");
            println!("  4. Specific functionality you want implemented");
            println!();
            
            // Execute Claude Code with our prompt file
            let claude_result = Command::new("claude")
                .arg("code")
                .arg("--local")
                .arg(src_dir)
                .arg(prompt_file)
                .status()
                .context("Failed to launch Claude Code")?;
                
            if claude_result.success() {
                CommandHelpers::print_success("Command creation session completed");
                println!();
                println!("{} {}", "📝".blue(), "Remember to:".italic());
                println!("  1. Build the project with: {}", "cargo build".yellow());
                println!("  2. Install the updated binary: {}", "cargo install --path .".yellow());
                println!("  3. Test your new command");
            } else {
                CommandHelpers::print_error("Command creation session failed or was cancelled");
            }
        },
        
        "edit" => {
            // Edit an existing command with Claude Code's assistance
            CommandHelpers::print_command_header(
                "Edit Existing Command", 
                "✏️", 
                "System Management", 
                "blue"
            );
            
            // Get current directory to find the source code
            let current_dir = env::current_dir()
                .context("Failed to get current directory")?;
            let src_dir = current_dir.join("src");
            let commands_dir = src_dir.join("commands");
            
            // Verify commands directory exists
            if !commands_dir.exists() {
                CommandHelpers::print_error("Could not find commands directory");
                return Err(anyhow::anyhow!("Commands directory not found"));
            }
            
            // Discover existing command files
            let command_files = vec![
                "intelligence.rs",
                "source_control.rs",
                "lifecycle.rs",
                "system.rs",
            ];
            
            // List all commands
            let mut all_commands = Vec::new();
            
            for file in &command_files {
                let file_path = commands_dir.join(file);
                if file_path.exists() {
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        // Look for command function declarations (pub async fn name)
                        for line in content.lines() {
                            if line.trim().starts_with("pub async fn ") && line.contains("(") {
                                let function_name = line
                                    .trim()
                                    .strip_prefix("pub async fn ")
                                    .unwrap_or("")
                                    .split('(')
                                    .next()
                                    .unwrap_or("")
                                    .trim()
                                    .to_string();
                                    
                                if !function_name.is_empty() {
                                    // Get description from comment above function
                                    let mut description = String::new();
                                    if let Some(idx) = content.find(&format!("pub async fn {}", function_name)) {
                                        if let Some(desc_idx) = content[..idx].rfind("///") {
                                            description = content[desc_idx..]
                                                .lines()
                                                .next()
                                                .unwrap_or("")
                                                .trim_start_matches("///")
                                                .trim()
                                                .to_string();
                                        }
                                    }
                                    
                                    // Add to commands list with category
                                    let category = match *file {
                                        "intelligence.rs" => "Intelligence",
                                        "source_control.rs" => "Source Control",
                                        "lifecycle.rs" => "Lifecycle",
                                        "system.rs" => "System",
                                        _ => "Unknown",
                                    };
                                    
                                    all_commands.push((function_name, description, category.to_string(), file));
                                }
                            }
                        }
                    }
                }
            }
            
            // Create temporary prompt file for Claude Code
            let temp_dir = env::temp_dir();
            let prompt_file = temp_dir.join("ci_command_edit.md");
            
            // Build detailed prompt for editing a command
            let mut prompt = String::from("# CI Command Editor\n\n");
            prompt.push_str("You're tasked with editing an existing command in the CI tool. Below is a list of all available commands:\n\n");
            
            // Add list of commands to the prompt
            prompt.push_str("## Available Commands\n\n");
            prompt.push_str("| Command | Description | Category | File |\n");
            prompt.push_str("|---------|-------------|----------|------|\n");
            
            for (name, desc, category, file) in &all_commands {
                prompt.push_str(&format!("| `{}` | {} | {} | `{}` |\n", name, desc, category, file));
            }
            
            prompt.push_str(r#"

## Instructions

1. Ask which command the user wants to edit
2. Locate the command implementation in the appropriate file
3. Show the current implementation
4. Ask what changes they want to make
5. Implement the requested changes
6. Show the updated implementation
7. Apply the changes to the file

## Tips

- Keep the command structure and formatting consistent with the project's style
- Make sure to update both the implementation and any enum entries or match statements if necessary
- Ask for confirmation before saving changes
- Ensure error handling is appropriate

Let's begin by selecting a command to edit.
"#);

            // Write prompt to temporary file
            fs::write(&prompt_file, prompt)?;
            
            // Launch Claude Code with the prompt
            println!("📋 {}", "Available commands:".blue().bold());
            println!();
            
            // Group commands by category
            let mut intelligence_commands = Vec::new();
            let mut source_control_commands = Vec::new();
            let mut lifecycle_commands = Vec::new();
            let mut system_commands = Vec::new();
            
            for (name, desc, category, _) in &all_commands {
                match category.as_str() {
                    "Intelligence" => intelligence_commands.push((name, desc)),
                    "Source Control" => source_control_commands.push((name, desc)),
                    "Lifecycle" => lifecycle_commands.push((name, desc)),
                    "System" => system_commands.push((name, desc)),
                    _ => {}
                }
            }
            
            // Print commands by category
            if !intelligence_commands.is_empty() {
                println!("{}", "⚙️ Intelligence & Discovery".cyan().bold());
                for (name, desc) in &intelligence_commands {
                    println!("  {:<15} - {}", name.cyan(), desc);
                }
                println!();
            }
            
            if !source_control_commands.is_empty() {
                println!("{}", "📊 Source Control".green().bold());
                for (name, desc) in &source_control_commands {
                    println!("  {:<15} - {}", name.green(), desc);
                }
                println!();
            }
            
            if !lifecycle_commands.is_empty() {
                println!("{}", "🚀 Project Lifecycle".yellow().bold());
                for (name, desc) in &lifecycle_commands {
                    println!("  {:<15} - {}", name.yellow(), desc);
                }
                println!();
            }
            
            if !system_commands.is_empty() {
                println!("{}", "🧠 System Management".blue().bold());
                for (name, desc) in &system_commands {
                    println!("  {:<15} - {}", name.blue(), desc);
                }
                println!();
            }
            
            println!("🚀 {}", "Launching Claude Code to edit a command...".blue());
            println!();
            
            // Execute Claude Code with our prompt file
            let claude_result = Command::new("claude")
                .arg("code")
                .arg("--local")
                .arg(src_dir)
                .arg(prompt_file)
                .status()
                .context("Failed to launch Claude Code")?;
                
            if claude_result.success() {
                CommandHelpers::print_success("Command editing session completed");
                println!();
                println!("{} {}", "📝".blue(), "Remember to:".italic());
                println!("  1. Build the project with: {}", "cargo build".yellow());
                println!("  2. Install the updated binary: {}", "cargo install --path .".yellow());
                println!("  3. Test your updated command");
            } else {
                CommandHelpers::print_error("Command editing session failed or was cancelled");
            }
        },
        
        "list" => {
            // List all available commands
            CommandHelpers::print_command_header(
                "Available Commands", 
                "📋", 
                "System Management", 
                "blue"
            );
            
            // Get current directory to find the source code
            let current_dir = env::current_dir()
                .context("Failed to get current directory")?;
            let src_dir = current_dir.join("src");
            let commands_dir = src_dir.join("commands");
            
            // Verify commands directory exists
            if !commands_dir.exists() {
                CommandHelpers::print_error("Could not find commands directory");
                return Err(anyhow::anyhow!("Commands directory not found"));
            }
            
            // Discover existing command files
            let command_files = vec![
                "intelligence.rs",
                "source_control.rs",
                "lifecycle.rs",
                "system.rs",
            ];
            
            // List all commands
            let mut all_commands = Vec::new();
            
            for file in &command_files {
                let file_path = commands_dir.join(file);
                if file_path.exists() {
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        // Look for command function declarations (pub async fn name)
                        for line in content.lines() {
                            if line.trim().starts_with("pub async fn ") && line.contains("(") {
                                let function_name = line
                                    .trim()
                                    .strip_prefix("pub async fn ")
                                    .unwrap_or("")
                                    .split('(')
                                    .next()
                                    .unwrap_or("")
                                    .trim()
                                    .to_string();
                                    
                                if !function_name.is_empty() {
                                    // Get description from comment above function
                                    let mut description = String::new();
                                    if let Some(idx) = content.find(&format!("pub async fn {}", function_name)) {
                                        if let Some(desc_idx) = content[..idx].rfind("///") {
                                            description = content[desc_idx..]
                                                .lines()
                                                .next()
                                                .unwrap_or("")
                                                .trim_start_matches("///")
                                                .trim()
                                                .to_string();
                                        }
                                    }
                                    
                                    // Add to commands list with category
                                    let category = match *file {
                                        "intelligence.rs" => "Intelligence",
                                        "source_control.rs" => "Source Control",
                                        "lifecycle.rs" => "Lifecycle",
                                        "system.rs" => "System",
                                        _ => "Unknown",
                                    };
                                    
                                    all_commands.push((function_name, description, category.to_string()));
                                }
                            }
                        }
                    }
                }
            }
            
            // Group commands by category
            let mut intelligence_commands = Vec::new();
            let mut source_control_commands = Vec::new();
            let mut lifecycle_commands = Vec::new();
            let mut system_commands = Vec::new();
            
            for (name, desc, category) in &all_commands {
                match category.as_str() {
                    "Intelligence" => intelligence_commands.push((name, desc)),
                    "Source Control" => source_control_commands.push((name, desc)),
                    "Lifecycle" => lifecycle_commands.push((name, desc)),
                    "System" => system_commands.push((name, desc)),
                    _ => {}
                }
            }
            
            // Print commands by category
            if !intelligence_commands.is_empty() {
                println!("{}", "⚙️ Intelligence & Discovery".cyan().bold());
                for (name, desc) in &intelligence_commands {
                    println!("  {:<15} - {}", name.cyan(), desc);
                }
                println!();
            }
            
            if !source_control_commands.is_empty() {
                println!("{}", "📊 Source Control".green().bold());
                for (name, desc) in &source_control_commands {
                    println!("  {:<15} - {}", name.green(), desc);
                }
                println!();
            }
            
            if !lifecycle_commands.is_empty() {
                println!("{}", "🚀 Project Lifecycle".yellow().bold());
                for (name, desc) in &lifecycle_commands {
                    println!("  {:<15} - {}", name.yellow(), desc);
                }
                println!();
            }
            
            if !system_commands.is_empty() {
                println!("{}", "🧠 System Management".blue().bold());
                for (name, desc) in &system_commands {
                    println!("  {:<15} - {}", name.blue(), desc);
                }
                println!();
            }
            
            CommandHelpers::print_success("Command listing complete");
        },
        
        _ => {
            // Help information if no valid subcommand is provided
            CommandHelpers::print_command_header(
                "Command Management", 
                "⚙️", 
                "System Management", 
                "blue"
            );
            
            println!("This command helps you create, edit, and manage commands in the CI tool.\n");
            
            println!("{}", "Available Subcommands:".blue().bold());
            println!("  {:<15} - {}", "create".blue(), "Create a new command with Claude Code assistance");
            println!("  {:<15} - {}", "edit".blue(), "Edit an existing command with Claude Code assistance");
            println!("  {:<15} - {}", "list".blue(), "List all available commands");
            
            println!("\n{}", "Examples:".yellow().bold());
            println!("  {}", "ci command create".yellow());
            println!("  {}", "ci command edit".yellow());
            println!("  {}", "ci command list".yellow());
        }
    }
    
    Ok(())
}