// Export submodules
pub mod command;
pub mod repository;
pub mod config;
pub mod project;
pub mod path;
pub mod system;
pub mod commit_analyzer;
pub mod api_keys;
pub mod status_reporter;
pub mod integration_manager;
pub mod progress_indicator;
pub mod api_client;
pub mod agent_autoload;
pub mod agent_colors;

// Re-export commonly used helpers
pub use command::CommandHelpers;
pub use repository::{RepositoryHelpers, RepositoryStatus};
pub use config::{ConfigHelpers, ConfigStatus};
pub use project::{ProjectHelpers, ProjectInfo, ProjectStats};
pub use path::PathHelpers;
pub use system::{SystemHelpers, SystemInfo};
pub use commit_analyzer::{CommitAnalyzer, CommitAnalysis, FileChange, ChangeType};
pub use api_keys::{ApiKeyManager, ApiKeyCommands};
pub use status_reporter::{StatusReporter, StatusReport};
pub use progress_indicator::{Spinner, SpinnerResult, with_spinner, with_progress_updates};
pub use api_client::CIApiClient;
pub use agent_autoload::{AgentAutoload, AgentActivationConfig};
pub use agent_colors::{get_agent_color, apply_agent_color, reset_terminal_color, get_color_name};

use colored::*;
use anyhow::Context;

/// Helper functions for common CI command operations
pub struct SectionPrinter;

impl SectionPrinter {
    pub fn begin(title: &str) {
        println!("\n{}", title.bold());
        println!("{}", "-".repeat(title.len()));
    }
    
    pub fn end() {
        println!();
    }
}

/// Create a temporary file with automatic cleanup
pub fn create_temp_file(prefix: &str) -> anyhow::Result<tempfile::NamedTempFile> {
    tempfile::Builder::new()
        .prefix(prefix)
        .rand_bytes(5)
        .tempfile()
        .with_context(|| format!("Failed to create temporary file with prefix '{}'", prefix))
}

/// Create a temporary directory with automatic cleanup
pub fn create_temp_dir(prefix: &str) -> anyhow::Result<tempfile::TempDir> {
    tempfile::Builder::new()
        .prefix(prefix)
        .rand_bytes(5)
        .tempdir()
        .with_context(|| format!("Failed to create temporary directory with prefix '{}'", prefix))
}