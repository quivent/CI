// Shared Metadata Utilities for CI CLI
// Common metadata operations used across multiple modules

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub project_name: String,
    pub integration_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub agents: Vec<String>,
    pub topology_enabled: bool,
}

pub fn load_project_metadata(project_path: &Path) -> Result<Option<ProjectMetadata>, Box<dyn std::error::Error>> {
    let metadata_file = project_path.join(".ci").join("project.json");
    
    if !metadata_file.exists() {
        return Ok(None);
    }

    let metadata_content = fs::read_to_string(metadata_file)?;
    let metadata: ProjectMetadata = serde_json::from_str(&metadata_content)?;
    
    Ok(Some(metadata))
}

pub fn save_project_metadata(project_path: &Path, metadata: &ProjectMetadata) -> Result<(), Box<dyn std::error::Error>> {
    let ci_dir = project_path.join(".ci");
    if !ci_dir.exists() {
        fs::create_dir_all(&ci_dir)?;
    }

    let metadata_file = ci_dir.join("project.json");
    let metadata_json = serde_json::to_string_pretty(metadata)?;
    fs::write(metadata_file, metadata_json)?;
    
    Ok(())
}