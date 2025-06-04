use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

fn main() {
    // Skip "cargo-install-ci" from args
    let args: Vec<String> = env::args().skip(1).collect();
    
    // Check if we're in the right directory
    if !Path::new("Cargo.toml").exists() || !Path::new("src").exists() {
        eprintln!("❌ Error: This command must be run from the CI project root directory.");
        exit(1);
    }

    println!("🔨 Building CI binary...");
    let output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to execute cargo build");

    if !output.status.success() {
        eprintln!("❌ Build failed!");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }

    // Determine the installation directory
    let home = env::var("HOME").expect("HOME environment variable not set");
    let install_dir = format!("{}/.local/bin", home);
    
    // Create installation directory
    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("❌ Failed to create install directory {}: {}", install_dir, e);
        exit(1);
    }

    println!("📦 Installing CI binary to {}...", install_dir);
    
    // Copy binary
    let src_binary = "target/release/CI";
    let dest_binary = format!("{}/CI", install_dir);
    
    if let Err(e) = fs::copy(src_binary, &dest_binary) {
        eprintln!("❌ Failed to copy binary: {}", e);
        exit(1);
    }

    // Make executable (Unix systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_binary).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_binary, perms).unwrap();
    }

    // Create lowercase symlink
    let symlink_path = format!("{}/ci", install_dir);
    #[cfg(unix)]
    {
        use std::os::unix::fs;
        let _ = std::fs::remove_file(&symlink_path); // Remove if exists
        if let Err(e) = fs::symlink(&dest_binary, &symlink_path) {
            eprintln!("⚠️ Warning: Failed to create symlink: {}", e);
        }
    }

    // Check if install directory is in PATH
    let path = env::var("PATH").unwrap_or_default();
    if !path.split(':').any(|p| p == install_dir) {
        println!("⚠️  Warning: {} is not in your PATH", install_dir);
        
        // Determine shell config file
        let shell = env::var("SHELL").unwrap_or_default();
        let shell_config = if shell.contains("zsh") {
            format!("{}/.zshrc", home)
        } else if shell.contains("bash") {
            if cfg!(target_os = "macos") {
                format!("{}/.bash_profile", home)
            } else {
                format!("{}/.bashrc", home)
            }
        } else {
            format!("{}/.profile", home)
        };
        
        println!("Add the following line to your {}:", shell_config);
        println!("export PATH=\"$PATH:{}\"", install_dir);
    }

    // Install completions
    install_completions(&home, &install_dir);

    println!("✅ CI installation complete!");
    println!("You can now run 'ci' to use the Collaborative Intelligence CLI");
}

fn install_completions(home: &str, install_dir: &str) {
    // Bash completions
    if Path::new("completions/ci.bash").exists() {
        let completions_dir = format!("{}/.local/share/bash-completion/completions", home);
        if fs::create_dir_all(&completions_dir).is_ok() {
            if fs::copy("completions/ci.bash", format!("{}/ci", completions_dir)).is_ok() {
                println!("✅ Installed bash completion");
            }
        }
    }

    // Fish completions
    if Path::new("completions/ci.fish").exists() {
        let fish_dir = format!("{}/.config/fish/completions", home);
        if fs::create_dir_all(&fish_dir).is_ok() {
            if fs::copy("completions/ci.fish", format!("{}/ci.fish", fish_dir)).is_ok() {
                println!("✅ Installed fish completion");
            }
        }
    }

    // Zsh completions
    if Path::new("completions/ci.zsh").exists() {
        let zsh_dir = format!("{}/.zsh/completions", home);
        if fs::create_dir_all(&zsh_dir).is_ok() {
            if fs::copy("completions/ci.zsh", format!("{}/_ci", zsh_dir)).is_ok() {
                println!("✅ Installed zsh completion");
            }
        }
    }
}