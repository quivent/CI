//! Path resolution and file operation helpers for CI
//!
//! This module provides helper functions for working with file system paths,
//! resolving project paths, and performing common file operations.

use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use anyhow::{Context, Result, anyhow};

/// Helper functions for path operations
pub struct PathHelpers;

impl PathHelpers {
    /// Resolve a path with standard error handling
    pub fn resolve_project_path(path: &Option<String>) -> Result<PathBuf> {
        let project_dir = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            env::current_dir()
                .with_context(|| "Failed to get current directory")?
        };
        
        if !project_dir.exists() {
            return Err(anyhow!("Path does not exist: {}", project_dir.display()));
        }
        
        if !project_dir.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", project_dir.display()));
        }
        
        Ok(project_dir)
    }
    
    /// Get CI repository path with standard resolution
    pub fn get_ci_repository_path(ci_path: &Option<String>) -> Result<PathBuf> {
        let ci_repo = if let Some(path) = ci_path {
            PathBuf::from(path)
        } else if let Ok(path) = env::var("CI_PATH") {
            PathBuf::from(path)
        } else {
            // Try to find in common locations
            let home_dir = dirs::home_dir()
                .ok_or_else(|| anyhow!("Could not determine home directory"))?;
                
            let common_paths = vec![
                home_dir.join("Documents/Projects/CollaborativeIntelligence"),
                home_dir.join("Projects/CollaborativeIntelligence"),
                home_dir.join("CollaborativeIntelligence"),
                PathBuf::from("/usr/local/share/CollaborativeIntelligence"),
            ];
            
            let mut found_path = None;
            for path in common_paths {
                if path.exists() && path.join("CLAUDE.md").exists() {
                    found_path = Some(path);
                    break;
                }
            }
            
            found_path.ok_or_else(|| anyhow!("CI repository path not found"))?
        };
        
        if !ci_repo.exists() {
            return Err(anyhow!("CI repository path does not exist: {}", ci_repo.display()));
        }
        
        if !ci_repo.is_dir() {
            return Err(anyhow!("CI repository path is not a directory: {}", ci_repo.display()));
        }
        
        Ok(ci_repo)
    }
    
    /// Ensure a directory exists, creating it if necessary
    pub fn ensure_directory_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .with_context(|| format!("Failed to create directory at {}", path.display()))?;
        } else if !path.is_dir() {
            return Err(anyhow!("Path exists but is not a directory: {}", path.display()));
        }
        
        Ok(())
    }
    
    /// Create a file with content, ensuring the parent directory exists
    pub fn create_file_with_content(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            Self::ensure_directory_exists(parent)?;
        }
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write file at {}", path.display()))?;
            
        Ok(())
    }
    
    /// Read file content with error handling
    pub fn read_file_content(path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", path.display()));
        }
        
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file at {}", path.display()))
    }
    
    /// Copy a file, ensuring the target directory exists
    pub fn copy_file(source: &Path, destination: &Path) -> Result<()> {
        if !source.exists() {
            return Err(anyhow!("Source file does not exist: {}", source.display()));
        }
        
        if let Some(parent) = destination.parent() {
            Self::ensure_directory_exists(parent)?;
        }
        
        fs::copy(source, destination)
            .with_context(|| format!("Failed to copy from {} to {}", 
                source.display(), destination.display()))?;
                
        Ok(())
    }
    
    /// Recursively copy a directory
    pub fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
        if !source.exists() {
            return Err(anyhow!("Source directory does not exist: {}", source.display()));
        }
        
        if !source.is_dir() {
            return Err(anyhow!("Source is not a directory: {}", source.display()));
        }
        
        // Create destination directory
        Self::ensure_directory_exists(destination)?;
        
        // Read source directory
        let entries = fs::read_dir(source)
            .with_context(|| format!("Failed to read directory at {}", source.display()))?;
            
        // Copy each entry
        for entry in entries {
            let entry = entry
                .with_context(|| format!("Failed to read directory entry in {}", source.display()))?;
            let source_path = entry.path();
            let file_name = source_path.file_name()
                .ok_or_else(|| anyhow!("Failed to get file name for {}", source_path.display()))?;
            let destination_path = destination.join(file_name);
            
            if source_path.is_dir() {
                // Recursively copy subdirectory
                Self::copy_directory(&source_path, &destination_path)?;
            } else {
                // Copy file
                Self::copy_file(&source_path, &destination_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Get the bin directory for the user (for installing binaries)
    pub fn get_user_bin_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
            
        // Try common user bin directories
        let bin_paths = vec![
            home_dir.join(".local/bin"),      // Linux standard
            home_dir.join("bin"),             // Unix traditional
            home_dir.join(".bin"),            // Alternative
        ];
        
        // Check PATH for existing bin directories
        if let Ok(path_var) = env::var("PATH") {
            for path_str in path_var.split(':') {
                let path = PathBuf::from(path_str);
                if path.exists() && path.is_dir() && path.starts_with(&home_dir) {
                    return Ok(path);
                }
            }
        }
        
        // Fall back to the first common path that exists or can be created
        for bin_path in bin_paths {
            if bin_path.exists() && bin_path.is_dir() {
                return Ok(bin_path);
            }
            
            // Try to create the directory
            if let Ok(()) = Self::ensure_directory_exists(&bin_path) {
                return Ok(bin_path);
            }
        }
        
        // Last resort: ~/.local/bin
        let default_bin = home_dir.join(".local/bin");
        Self::ensure_directory_exists(&default_bin)?;
        Ok(default_bin)
    }
    
    /// Check if a path is in PATH
    pub fn is_in_path(path: &Path) -> bool {
        if let Ok(path_var) = env::var("PATH") {
            for path_str in path_var.split(':') {
                if Path::new(path_str) == path {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Find files matching a pattern in a directory (non-recursive)
    pub fn find_files(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
        if !dir.exists() || !dir.is_dir() {
            return Err(anyhow!("Directory does not exist: {}", dir.display()));
        }
        
        let mut files = Vec::new();
        
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory at {}", dir.display()))?;
            
        for entry in entries {
            let entry = entry
                .with_context(|| format!("Failed to read directory entry in {}", dir.display()))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        files.push(path);
                    }
                }
            }
        }
        
        Ok(files)
    }
    
    /// Find files recursively matching a pattern in a directory
    pub fn find_files_recursive(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        Self::find_files_recursive_internal(dir, extension, &mut files)?;
        Ok(files)
    }
    
    /// Helper for recursive file search
    fn find_files_recursive_internal(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Err(anyhow!("Directory does not exist: {}", dir.display()));
        }
        
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory at {}", dir.display()))?;
            
        for entry in entries {
            let entry = entry
                .with_context(|| format!("Failed to read directory entry in {}", dir.display()))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        files.push(path);
                    }
                }
            } else if path.is_dir() {
                // Skip git, node_modules, target directories
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name != ".git" && dir_name != "node_modules" && dir_name != "target" {
                    Self::find_files_recursive_internal(&path, extension, files)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Find files matching a glob pattern
    pub fn find_files_with_glob(pattern: &str) -> Result<Vec<PathBuf>> {
        let paths = glob::glob(pattern)
            .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
            
        let result = paths
            .filter_map(Result::ok)
            .collect();
            
        Ok(result)
    }
    
    /// Get relative path from base to target
    pub fn get_relative_path(base: &Path, target: &Path) -> Result<PathBuf> {
        use path_absolutize::Absolutize;
        
        let abs_base = base.absolutize()
            .with_context(|| format!("Failed to get absolute path for {}", base.display()))?;
            
        let abs_target = target.absolutize()
            .with_context(|| format!("Failed to get absolute path for {}", target.display()))?;
            
        // Convert to PathBuf because the absolutize library returns &Path
        let abs_base = abs_base.to_path_buf();
        let abs_target = abs_target.to_path_buf();
        
        // Get components
        let base_components: Vec<_> = abs_base.components().collect();
        let target_components: Vec<_> = abs_target.components().collect();
        
        // Find common prefix
        let mut common_prefix = 0;
        for (a, b) in base_components.iter().zip(target_components.iter()) {
            if a == b {
                common_prefix += 1;
            } else {
                break;
            }
        }
        
        // Build relative path
        let mut result = PathBuf::new();
        
        // Add .. for each component in base after the common prefix
        for _ in common_prefix..base_components.len() {
            result.push("..");
        }
        
        // Add target components after the common prefix
        for component in target_components.iter().skip(common_prefix) {
            result.push(component.as_os_str());
        }
        
        Ok(result)
    }
    
    /// Create backup of a file
    pub fn backup_file(path: &Path) -> Result<PathBuf> {
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", path.display()));
        }
        
        // Generate backup filename with timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
        let mut backup_path = path.to_path_buf();
        
        // Use file_name.ext.timestamp format
        let file_name = path.file_name()
            .ok_or_else(|| anyhow!("Invalid file name: {}", path.display()))?
            .to_string_lossy();
            
        let backup_name = format!("{}.{}", file_name, timestamp);
        backup_path.set_file_name(backup_name);
        
        // Copy file to backup location
        fs::copy(path, &backup_path)
            .with_context(|| format!("Failed to create backup from {} to {}", 
                path.display(), backup_path.display()))?;
                
        Ok(backup_path)
    }
}

/// Get the CI root directory
pub fn get_ci_root() -> Result<PathBuf> {
    PathHelpers::get_ci_repository_path(&None)
}