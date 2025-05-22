// Metadata Management Module - Local state persistence and session tracking
// Adapted from standalone topologist for CI integration

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use chrono::{DateTime, Utc};

const METADATA_DIR: &str = ".ci-topology";
const CONFIG_FILE: &str = "config.json";
const HISTORY_FILE: &str = "commit_history.json";
const CACHE_FILE: &str = "analysis_cache.json";

pub struct MetadataManager {
    metadata_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub version: String,
    pub project_id: String,
    pub created: DateTime<Utc>,
    pub commit_strategy: String,
    pub size_tracking: bool,
    pub auto_gitignore: bool,
    pub phases_completed: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionHistory {
    pub session_id: String,
    pub started: DateTime<Utc>,
    pub completed: Option<DateTime<Utc>>,
    pub phases: Vec<PhaseExecution>,
    pub total_planned_phases: Option<usize>,
    pub total_impact: Option<SessionImpact>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PhaseExecution {
    pub phase: usize,
    pub commit_hash: String,
    pub files_count: usize,
    pub size_change: usize,
    pub category: String,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionImpact {
    pub commits: usize,
    pub files_added: usize,
    pub files_modified: usize,
    pub net_insertions: usize,
    pub net_deletions: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitHistoryData {
    pub sessions: Vec<SessionHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisCache {
    pub cached_at: DateTime<Utc>,
    pub repository_hash: String,
    pub file_count: usize,
    pub expires_at: DateTime<Utc>,
}

impl MetadataManager {
    pub fn new() -> Self {
        Self {
            metadata_dir: PathBuf::from(METADATA_DIR),
        }
    }

    /// Initialize project metadata directory and configuration
    pub fn initialize_project(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create metadata directory if it doesn't exist
        if !self.metadata_dir.exists() {
            fs::create_dir_all(&self.metadata_dir)?;
        }

        // Create or update project configuration
        let config_path = self.metadata_dir.join(CONFIG_FILE);
        
        let config = if config_path.exists() {
            // Load existing config and update timestamp
            let existing_config = self.load_project_config()?;
            existing_config
        } else {
            // Create new configuration
            ProjectConfig {
                version: "1.0".to_string(),
                project_id: Uuid::new_v4().to_string(),
                created: Utc::now(),
                commit_strategy: "phasal".to_string(),
                size_tracking: true,
                auto_gitignore: true,
                phases_completed: Vec::new(),
            }
        };

        self.save_project_config(&config)?;

        // Initialize empty commit history if it doesn't exist
        let history_path = self.metadata_dir.join(HISTORY_FILE);
        if !history_path.exists() {
            let empty_history = CommitHistoryData {
                sessions: Vec::new(),
            };
            self.save_commit_history(&empty_history)?;
        }

        // Add .ci-topology to .gitignore if auto_gitignore is enabled
        if config.auto_gitignore {
            self.add_to_gitignore()?;
        }

        Ok(())
    }

    /// Load project configuration
    pub fn load_project_config(&self) -> Result<ProjectConfig, Box<dyn std::error::Error>> {
        let config_path = self.metadata_dir.join(CONFIG_FILE);
        
        if !config_path.exists() {
            return Err("Project not initialized. Run 'ci topologist init' first.".into());
        }

        let config_data = fs::read_to_string(config_path)?;
        let config: ProjectConfig = serde_json::from_str(&config_data)?;
        
        Ok(config)
    }

    /// Save project configuration
    pub fn save_project_config(&self, config: &ProjectConfig) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = self.metadata_dir.join(CONFIG_FILE);
        let config_json = serde_json::to_string_pretty(config)?;
        fs::write(config_path, config_json)?;
        Ok(())
    }

    /// Record a phase execution
    pub fn record_phase_execution(
        &mut self, 
        phase_number: usize, 
        commit_hash: &str, 
        files_count: usize, 
        estimated_size: usize
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let mut history = self.load_commit_history().unwrap_or_else(|_| CommitHistoryData {
            sessions: Vec::new(),
        });

        // Get or create current session
        let session = if let Some(session) = history.sessions.last_mut() {
            if session.completed.is_none() {
                session
            } else {
                // Create new session if the last one is completed
                let new_session = SessionHistory {
                    session_id: Uuid::new_v4().to_string(),
                    started: Utc::now(),
                    completed: None,
                    phases: Vec::new(),
                    total_planned_phases: None,
                    total_impact: None,
                };
                history.sessions.push(new_session);
                history.sessions.last_mut().unwrap()
            }
        } else {
            // Create first session
            let new_session = SessionHistory {
                session_id: Uuid::new_v4().to_string(),
                started: Utc::now(),
                completed: None,
                phases: Vec::new(),
                total_planned_phases: None,
                total_impact: None,
            };
            history.sessions.push(new_session);
            history.sessions.last_mut().unwrap()
        };

        // Add phase execution
        let phase_execution = PhaseExecution {
            phase: phase_number,
            commit_hash: commit_hash.to_string(),
            files_count,
            size_change: estimated_size,
            category: "Unknown".to_string(), // Will be enhanced in future versions
            executed_at: Utc::now(),
        };

        session.phases.push(phase_execution);

        // Update config to track completed phases
        let mut config = self.load_project_config()?;
        if !config.phases_completed.contains(&phase_number) {
            config.phases_completed.push(phase_number);
            config.phases_completed.sort();
            self.save_project_config(&config)?;
        }

        self.save_commit_history(&history)?;
        Ok(())
    }

    /// Get current active session
    pub fn get_current_session(&self) -> Result<Option<SessionHistory>, Box<dyn std::error::Error>> {
        let history = match self.load_commit_history() {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };

        // Return the last uncompleted session
        let current_session = history.sessions
            .into_iter()
            .rev()
            .find(|session| session.completed.is_none());

        Ok(current_session)
    }

    /// Complete current session
    pub fn complete_current_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.load_commit_history()?;

        if let Some(session) = history.sessions.last_mut() {
            if session.completed.is_none() {
                session.completed = Some(Utc::now());
                
                // Calculate total impact
                let total_files = session.phases.iter().map(|p| p.files_count).sum();
                let total_size = session.phases.iter().map(|p| p.size_change).sum();
                
                session.total_impact = Some(SessionImpact {
                    commits: session.phases.len(),
                    files_added: total_files, // Simplified - in reality would distinguish added vs modified
                    files_modified: 0,
                    net_insertions: total_size,
                    net_deletions: 0,
                });

                self.save_commit_history(&history)?;
            }
        }

        Ok(())
    }

    /// Load commit history
    fn load_commit_history(&self) -> Result<CommitHistoryData, Box<dyn std::error::Error>> {
        let history_path = self.metadata_dir.join(HISTORY_FILE);
        
        if !history_path.exists() {
            return Ok(CommitHistoryData {
                sessions: Vec::new(),
            });
        }

        let history_data = fs::read_to_string(history_path)?;
        let history: CommitHistoryData = serde_json::from_str(&history_data)?;
        
        Ok(history)
    }

    /// Save commit history
    fn save_commit_history(&self, history: &CommitHistoryData) -> Result<(), Box<dyn std::error::Error>> {
        let history_path = self.metadata_dir.join(HISTORY_FILE);
        let history_json = serde_json::to_string_pretty(history)?;
        fs::write(history_path, history_json)?;
        Ok(())
    }

    /// Cache analysis results temporarily
    pub fn cache_analysis(&self, file_count: usize, repo_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cache = AnalysisCache {
            cached_at: Utc::now(),
            repository_hash: repo_hash.to_string(),
            file_count,
            expires_at: Utc::now() + chrono::Duration::hours(1), // Cache for 1 hour
        };

        let cache_path = self.metadata_dir.join(CACHE_FILE);
        let cache_json = serde_json::to_string_pretty(&cache)?;
        fs::write(cache_path, cache_json)?;
        
        Ok(())
    }

    /// Load cached analysis if valid
    pub fn load_cached_analysis(&self) -> Result<Option<AnalysisCache>, Box<dyn std::error::Error>> {
        let cache_path = self.metadata_dir.join(CACHE_FILE);
        
        if !cache_path.exists() {
            return Ok(None);
        }

        let cache_data = fs::read_to_string(&cache_path)?;
        let cache: AnalysisCache = serde_json::from_str(&cache_data)?;
        
        // Check if cache is still valid
        if Utc::now() > cache.expires_at {
            // Cache expired, remove it
            let _ = fs::remove_file(cache_path);
            return Ok(None);
        }

        Ok(Some(cache))
    }

    /// Add .ci-topology to .gitignore
    fn add_to_gitignore(&self) -> Result<(), Box<dyn std::error::Error>> {
        let gitignore_path = Path::new(".gitignore");
        
        let gitignore_content = if gitignore_path.exists() {
            fs::read_to_string(gitignore_path)?
        } else {
            String::new()
        };

        // Check if .ci-topology is already in .gitignore
        if gitignore_content.lines().any(|line| line.trim() == ".ci-topology/" || line.trim() == ".ci-topology") {
            return Ok(());
        }

        // Add .ci-topology to .gitignore
        let mut new_content = gitignore_content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str("\n# CI Topology Management\n.ci-topology/\n");

        fs::write(gitignore_path, new_content)?;
        Ok(())
    }

    /// Clean all metadata files and directories
    pub fn clean_all_metadata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.metadata_dir.exists() {
            fs::remove_dir_all(&self.metadata_dir)?;
        }

        // Optionally remove .ci-topology from .gitignore
        self.remove_from_gitignore()?;

        Ok(())
    }

    /// Remove .ci-topology from .gitignore
    fn remove_from_gitignore(&self) -> Result<(), Box<dyn std::error::Error>> {
        let gitignore_path = Path::new(".gitignore");
        
        if !gitignore_path.exists() {
            return Ok(());
        }

        let gitignore_content = fs::read_to_string(gitignore_path)?;
        let mut lines: Vec<&str> = gitignore_content.lines().collect();
        
        // Remove lines related to ci-topology
        lines.retain(|line| {
            let trimmed = line.trim();
            !matches!(trimmed, ".ci-topology" | ".ci-topology/" | "# CI Topology Management")
        });

        // Remove empty lines at the end that might have been left
        while lines.last() == Some(&"") {
            lines.pop();
        }

        let new_content = lines.join("\n");
        if !new_content.is_empty() {
            fs::write(gitignore_path, new_content + "\n")?;
        } else {
            fs::remove_file(gitignore_path)?;
        }

        Ok(())
    }

    /// Check if project is initialized
    pub fn is_initialized(&self) -> bool {
        self.metadata_dir.exists() && 
        self.metadata_dir.join(CONFIG_FILE).exists()
    }

    /// Get project statistics
    pub fn get_project_stats(&self) -> Result<ProjectStats, Box<dyn std::error::Error>> {
        if !self.is_initialized() {
            return Err("Project not initialized".into());
        }

        let config = self.load_project_config()?;
        let history = self.load_commit_history().unwrap_or_else(|_| CommitHistoryData {
            sessions: Vec::new(),
        });

        let total_sessions = history.sessions.len();
        let total_phases = history.sessions
            .iter()
            .map(|s| s.phases.len())
            .sum();

        let total_files_processed = history.sessions
            .iter()
            .flat_map(|s| &s.phases)
            .map(|p| p.files_count)
            .sum();

        Ok(ProjectStats {
            project_id: config.project_id,
            created: config.created,
            total_sessions,
            total_phases,
            total_files_processed,
            current_session_active: self.get_current_session()?.is_some(),
        })
    }
}

impl Default for MetadataManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ProjectStats {
    pub project_id: String,
    pub created: DateTime<Utc>,
    pub total_sessions: usize,
    pub total_phases: usize,
    pub total_files_processed: usize,
    pub current_session_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_manager_creation() {
        let metadata_manager = MetadataManager::new();
        assert_eq!(metadata_manager.metadata_dir, PathBuf::from(METADATA_DIR));
    }

    #[test]
    fn test_project_config_serialization() {
        let config = ProjectConfig {
            version: "1.0".to_string(),
            project_id: "test-id".to_string(),
            created: Utc::now(),
            commit_strategy: "phasal".to_string(),
            size_tracking: true,
            auto_gitignore: true,
            phases_completed: vec![1, 2, 3],
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProjectConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.version, deserialized.version);
        assert_eq!(config.project_id, deserialized.project_id);
        assert_eq!(config.phases_completed, deserialized.phases_completed);
    }
}