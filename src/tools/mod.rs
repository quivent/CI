//! Tools module for CI
//!
//! This module provides tools for developing and extending CI.

pub mod command_generator;
pub mod documentation_generator;
pub mod directive_processor;
pub mod ci_migration;

// Re-export commonly used tools
pub use command_generator::{generate_command, process_instant_command};
pub use documentation_generator::DocumentationGenerator;
pub use directive_processor::{DirectiveProcessor, process_file_standalone, process_content_standalone};
pub use ci_migration::{detect_ci_integration, migrate_to_cir};