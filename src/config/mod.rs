//! Configuration module for CI
//!
//! This module provides structures and functions for managing CI configuration,
//! including global config and project-specific config.

pub mod ci_config;

use crate::error::Error;
use std::path::PathBuf;

/// Core global configuration for CI
#[derive(Clone, Debug)]
pub struct Config {
    /// Path to the CI repository
    pub ci_path: PathBuf,
}

impl Config {
    /// Load the global configuration
    pub fn load() -> Result<Self, Error> {
        // Try to get CI path from environment variable
        if let Ok(path) = std::env::var("CI_PATH") {
            let ci_path = PathBuf::from(path);
            if ci_path.exists() {
                return Ok(Config { ci_path });
            }
        }
        
        // Try to find in common locations
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let common_paths = vec![
            home_dir.join("Documents/Projects/CollaborativeIntelligence"),
            home_dir.join("Projects/CollaborativeIntelligence"),
            home_dir.join("CollaborativeIntelligence"),
            PathBuf::from("/usr/local/share/CollaborativeIntelligence"),
        ];
        
        for path in common_paths {
            if path.exists() && path.join("CLAUDE.md").exists() {
                return Ok(Config { ci_path: path });
            }
        }
        
        Err(Error::ConfigError("CI repository path not found".to_string()))
    }
}

// Re-export CI config types
pub use ci_config::{CIConfig, find_nearest_config};