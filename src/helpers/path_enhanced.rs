//! Path resolution and file operation helpers for CI
//!
//! This module provides helper functions for working with file system paths,
//! resolving project paths, and performing common file operations.

use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use crate::errors::{CIError, Result, path_not_found, ErrorExt};

/// Helper functions for path operations
pub struct PathHelpers;

impl PathHelpers {
    /// Resolve a path with standard error handling
    pub fn resolve_project_path(path: &Option<String>) -> Result<PathBuf> {
        let project_dir = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            env::current_dir()
                .map_err(|e| CIError::IO(e))?
        };
        
        if !project_dir.exists() {
            return Err(path_not_found(project_dir.clone()));
        }
        
        if !project_dir.is_dir() {
            return Err(CIError::InvalidArgument(
                format!("Path is not a directory: {}", project_dir.display())
            ));
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
                .ok_or_else(|| CIError::Environment("Could not determine home directory".to_string()))?;
                
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
            
            found_path.ok_or_else(|| CIError::ResourceNotFound("CI repository path not found".to_string()))?
        };
        
        if !ci_repo.exists() {
            return Err(path_not_found(ci_repo.clone()));
        }
        
        if !ci_repo.is_dir() {
            return Err(CIError::InvalidArgument(
                format!("CI repository path is not a directory: {}", ci_repo.display())
            ));
        }
        
        Ok(ci_repo)
    }
    
    /// Ensure a directory exists, creating it if necessary
    pub fn ensure_directory_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| CIError::IO(e))?;
        } else if !path.is_dir() {
            return Err(CIError::InvalidArgument(
                format!("Path exists but is not a directory: {}", path.display())
            ));
        }
        
        Ok(())
    }
    
    /// Create a file with content, ensuring the parent directory exists
    pub fn create_file_with_content(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            Self::ensure_directory_exists(parent)?;
        }
        
        fs::write(path, content)
            .map_err(|e| CIError::IO(e))?;
            
        Ok(())
    }
    
    /// Read file content with error handling
    pub fn read_file_content(path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(path_not_found(path.to_path_buf()));
        }
        
        fs::read_to_string(path)
            .map_err(|e| CIError::IO(e))
    }
    
    /// Check if a directory exists and is readable
    pub fn directory_exists_and_readable(path: &Path) -> bool {
        if path.exists() && path.is_dir() {
            // Try to read the directory to confirm access
            return fs::read_dir(path).is_ok();
        }
        false
    }
    
    /// Verify directory exists and return Result instead of boolean
    pub fn verify_directory(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(path_not_found(path.to_path_buf()));
        }
        
        if !path.is_dir() {
            return Err(CIError::InvalidArgument(
                format!("Path is not a directory: {}", path.display())
            ));
        }
        
        if !Self::directory_exists_and_readable(path) {
            return Err(CIError::PermissionDenied(path.to_path_buf()));
        }
        
        Ok(())
    }
    
    /// Checks if running in an agent context with its own toolkit
    pub fn is_agent_with_toolkit() -> bool {
        // Check for environment variable that indicates this is an agent context
        if let Ok(value) = env::var("CI_AGENT_CONTEXT") {
            if value == "true" {
                return Self::get_agent_toolkit_path().exists();
            }
        }
        
        // Check if we're in an agent directory structure
        if let Ok(current_dir) = env::current_dir() {
            let toolkit_path = current_dir.join("toolkit");
            if toolkit_path.exists() && toolkit_path.is_dir() {
                return true;
            }
        }
        
        false
    }
    
    /// Gets the path to an agent's own toolkit, if applicable
    pub fn get_agent_toolkit_path() -> PathBuf {
        // First check for explicit environment variable
        if let Ok(path) = env::var("CI_AGENT_TOOLKIT_PATH") {
            return PathBuf::from(path);
        }
        
        // Fall back to standard location
        if let Ok(current_dir) = env::current_dir() {
            return current_dir.join("toolkit");
        }
        
        // Default to a subdirectory of current directory
        PathBuf::from("./toolkit")
    }
    
    /// Gets the appropriate path for a resource with agent-aware behavior
    /// When in agent context, prioritizes the agent's own toolkit
    pub fn get_agent_aware_path(resource_path: &Path, resource_name: &str) -> PathBuf {
        // When in agent context, first check the agent's own toolkit
        if Self::is_agent_with_toolkit() {
            let agent_toolkit = Self::get_agent_toolkit_path();
            let agent_resource = agent_toolkit.join(resource_name);
            
            if agent_resource.exists() {
                return agent_resource;
            }
            
            // If specified resource_path is provided, check if it exists there
            if resource_path.exists() {
                let resource_path_file = resource_path.join(resource_name);
                if resource_path_file.exists() {
                    return resource_path_file;
                }
            }
            
            // Return the agent's toolkit path even if the resource doesn't exist yet
            // This way, new resources will be created in the agent's own toolkit
            return agent_toolkit.join(resource_name);
        }
        
        // When not in agent context, use the standard resource path
        resource_path.join(resource_name)
    }
    
    /// Get the relative path between two absolute paths
    pub fn get_relative_path(from: &Path, to: &Path) -> PathBuf {
        // Simple implementation that handles common cases
        if let (Ok(from_canon), Ok(to_canon)) = (fs::canonicalize(from), fs::canonicalize(to)) {
            if let (Some(from_str), Some(to_str)) = (from_canon.to_str(), to_canon.to_str()) {
                if to_str.starts_with(from_str) {
                    if let Some(rel) = to_str.strip_prefix(from_str) {
                        let rel = rel.trim_start_matches('/');
                        if !rel.is_empty() {
                            return PathBuf::from(rel);
                        }
                    }
                }
            }
        }
        
        // Fallback to absolute path if relative path cannot be determined
        to.to_path_buf()
    }
    
    /// Get the current working directory with error handling
    pub fn get_current_dir() -> Result<PathBuf> {
        env::current_dir()
            .map_err(|e| CIError::IO(e))
    }
}