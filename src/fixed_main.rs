//! CI - Command-line interface for Collaborative Intelligence
//!
//! Modern implementation of the Collaborative Intelligence CLI with enhanced features and categorized commands.

use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
// use tempfile;

mod version;

mod commands {
    pub mod intelligence;
    pub mod source_control;
    pub mod lifecycle;
    pub mod system;
    pub mod legacy;
    pub mod idea;
    pub mod config;
    pub mod detach;
}

pub mod helpers;
pub mod tools;
pub mod errors;

// Legacy error module - kept for backward compatibility
// New code should use the errors module
#[deprecated(since = "1.1.0", note = "Use the errors module instead")]
mod error {
    use thiserror::Error;
    
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("Failed to load configuration: {0}")]
        ConfigError(String),
        
        #[error("IO error: {0}")]
        IoError(#[from] std::io::Error),
    }
    
    // Conversion from errors::CIError to legacy Error
    impl From<crate::errors::CIError> for Error {
        fn from(err: crate::errors::CIError) -> Self {
            match err {
                crate::errors::CIError::Config(s) => Error::ConfigError(s),
                crate::errors::CIError::IO(e) => Error::IoError(e),
                _ => Error::ConfigError(format!("Error: {}", err)),
            }
        }
    }
    
    // Conversion from legacy Error to errors::CIError
    impl From<Error> for crate::errors::CIError {
        fn from(err: Error) -> Self {
            match err {
                Error::ConfigError(s) => crate::errors::CIError::Config(s),
                Error::IoError(e) => crate::errors::CIError::IO(e),
            }
        }
    }
}