//! System operation helpers for CI
//!
//! This module provides helper functions for system operations and environment handling.
//! It includes functions for managing system paths, opening files, and retrieving system information.

use std::path::{Path, PathBuf};
use std::env;
use std::process::Command;
use anyhow::{Context, Result, anyhow};

/// Helper functions for system operations
pub struct SystemHelpers;

impl SystemHelpers {
    /// Check if a command exists in PATH
    pub fn command_exists(command: &str) -> bool {
        #[cfg(unix)]
        {
            Command::new("which")
                .arg(command)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
        
        #[cfg(windows)]
        {
            Command::new("where")
                .arg(command)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    }
    
    /// Get path to the CI binary
    pub fn get_ci_binary_path() -> Result<PathBuf> {
        // Try the current executable
        if let Ok(current_exe) = env::current_exe() {
            return Ok(current_exe);
        }
        
        // Try PATH
        if Self::command_exists("ci") {
            let output = Command::new("which")
                .arg("ci")
                .output()
                .with_context(|| "Failed to locate ci binary")?;
                
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(path_str));
            }
        }
        
        // Try common installation paths
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
            
        let common_paths = vec![
            home_dir.join(".cargo/bin/CI"),
            home_dir.join(".cargo/bin/ci"),
            home_dir.join(".local/bin/CI"),
            home_dir.join(".local/bin/ci"),
            home_dir.join("bin/CI"),
            home_dir.join("bin/ci"),
            PathBuf::from("/usr/local/bin/CI"),
            PathBuf::from("/usr/local/bin/ci"),
        ];
        
        for path in common_paths {
            if path.exists() {
                return Ok(path);
            }
        }
        
        // For backward compatibility, check for CI/ci
        if Self::command_exists("ci") {
            let output = Command::new("which")
                .arg("ci")
                .output()
                .with_context(|| "Failed to locate ci binary")?;
                
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(path_str));
            }
        }
        
        let legacy_paths = vec![
            home_dir.join(".cargo/bin/CI"),
            home_dir.join(".cargo/bin/ci"),
            home_dir.join(".local/bin/CI"),
            home_dir.join(".local/bin/ci"),
            home_dir.join("bin/CI"),
            home_dir.join("bin/ci"),
            PathBuf::from("/usr/local/bin/CI"),
            PathBuf::from("/usr/local/bin/ci"),
        ];
        
        for path in legacy_paths {
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err(anyhow!("Could not locate CI binary"))
    }
    
    /// Create a symlink to the CI binary
    pub fn create_symlink(source: &Path, destination: &Path) -> Result<()> {
        if !source.exists() {
            return Err(anyhow!("Source file does not exist: {}", source.display()));
        }
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = destination.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory at {}", parent.display()))?;
            }
        }
        
        // Remove existing destination if it exists
        if destination.exists() {
            std::fs::remove_file(destination)
                .with_context(|| format!("Failed to remove existing file at {}", destination.display()))?;
        }
        
        // Create symlink
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, destination)
                .with_context(|| format!("Failed to create symlink from {} to {}", 
                    source.display(), destination.display()))?;
        }
        
        #[cfg(windows)]
        {
            if source.is_dir() {
                std::os::windows::fs::symlink_dir(source, destination)
                    .with_context(|| format!("Failed to create directory symlink from {} to {}", 
                        source.display(), destination.display()))?;
            } else {
                std::os::windows::fs::symlink_file(source, destination)
                    .with_context(|| format!("Failed to create file symlink from {} to {}", 
                        source.display(), destination.display()))?;
            }
        }
        
        Ok(())
    }
    
    /// Get all symlinks to the CI binary
    pub fn get_ci_symlinks() -> Result<Vec<PathBuf>> {
        let mut symlinks = Vec::new();
        let ci_path = Self::get_ci_binary_path()?;
        
        // Check common bin directories
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
            
        let common_paths = vec![
            home_dir.join(".local/bin/CI"),
            home_dir.join(".local/bin/ci"),
            home_dir.join("bin/CI"),
            home_dir.join("bin/ci"),
            PathBuf::from("/usr/local/bin/CI"),
            PathBuf::from("/usr/local/bin/ci"),
            // Legacy paths
            home_dir.join(".local/bin/CI"),
            home_dir.join(".local/bin/ci"),
            home_dir.join("bin/CI"),
            home_dir.join("bin/ci"),
            PathBuf::from("/usr/local/bin/CI"),
            PathBuf::from("/usr/local/bin/ci"),
        ];
        
        for path in common_paths {
            if path.exists() {
                // Check if it's a symlink pointing to our binary
                #[cfg(unix)]
                {
                    if let Ok(target) = std::fs::read_link(&path) {
                        if target == ci_path {
                            symlinks.push(path);
                        }
                    }
                }
                
                #[cfg(windows)]
                {
                    // On Windows, just check if the file exists and has the same size
                    // as symlinks work differently
                    if let (Ok(path_meta), Ok(ci_meta)) = (std::fs::metadata(&path), std::fs::metadata(&ci_path)) {
                        if path_meta.len() == ci_meta.len() {
                            symlinks.push(path);
                        }
                    }
                }
            }
        }
        
        Ok(symlinks)
    }
    
    /// Open a URL in the default browser
    pub fn open_url(url: &str) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(url)
                .output()
                .with_context(|| format!("Failed to open URL: {}", url))?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(url)
                .output()
                .with_context(|| format!("Failed to open URL: {}", url))?;
        }
        
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/c", "start", url])
                .output()
                .with_context(|| format!("Failed to open URL: {}", url))?;
        }
        
        Ok(())
    }
    
    /// Open a file with the default application
    pub fn open_file(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", path.display()));
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(path)
                .output()
                .with_context(|| format!("Failed to open file: {}", path.display()))?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(path)
                .output()
                .with_context(|| format!("Failed to open file: {}", path.display()))?;
        }
        
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/c", "start", "", &path.to_string_lossy()])
                .output()
                .with_context(|| format!("Failed to open file: {}", path.display()))?;
        }
        
        Ok(())
    }
    
    /// Get system information
    pub fn get_system_info() -> Result<SystemInfo> {
        let mut info = SystemInfo {
            os: String::new(),
            arch: String::new(),
            version: String::new(),
            hostname: String::new(),
            username: String::new(),
            shell: String::new(),
        };
        
        // Get OS info
        #[cfg(target_os = "linux")]
        {
            info.os = "Linux".to_string();
        }
        
        #[cfg(target_os = "macos")]
        {
            info.os = "macOS".to_string();
        }
        
        #[cfg(target_os = "windows")]
        {
            info.os = "Windows".to_string();
        }
        
        // Get architecture
        #[cfg(target_arch = "x86_64")]
        {
            info.arch = "x86_64".to_string();
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            info.arch = "aarch64".to_string();
        }
        
        // Get OS version
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
                if output.status.success() {
                    info.version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("lsb_release").arg("-r").arg("-s").output() {
                if output.status.success() {
                    info.version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("cmd").args(["/c", "ver"]).output() {
                if output.status.success() {
                    let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    info.version = ver;
                }
            }
        }
        
        // Get hostname
        if let Ok(output) = Command::new("hostname").output() {
            if output.status.success() {
                info.hostname = String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
        
        // Get username
        if let Ok(username) = env::var("USER") {
            info.username = username;
        } else if let Ok(username) = env::var("USERNAME") {
            info.username = username;
        }
        
        // Get shell
        if let Ok(shell) = env::var("SHELL") {
            info.shell = shell;
        }
        
        Ok(info)
    }
}

/// Structure to hold system information
#[derive(Debug)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
    pub hostname: String,
    pub username: String,
    pub shell: String,
}

/// Helper functions for logging
pub struct LoggingHelpers;

impl LoggingHelpers {
    /// Initialize logging with the specified level
    pub fn init(level: &str) -> Result<()> {
        let level = match level.to_lowercase().as_str() {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => log::LevelFilter::Info, // Default to info
        };
        
        env_logger::Builder::new()
            .filter_level(level)
            .format_timestamp_secs()
            .init();
            
        Ok(())
    }
    
    /// Log an operation to a file
    pub fn log_operation(operation: &str, details: &str, log_file: &Path) -> Result<()> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("{} - {} - {}\n", timestamp, operation, details);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = log_file.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create log directory at {}", parent.display()))?;
            }
        }
        
        // Append to log file
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .with_context(|| format!("Failed to open log file at {}", log_file.display()))?;
            
        file.write_all(log_entry.as_bytes())
            .with_context(|| format!("Failed to write to log file at {}", log_file.display()))?;
            
        Ok(())
    }
}