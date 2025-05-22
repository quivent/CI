// CI Topology Management Module
// Integrated repository topology analysis and commit planning

pub mod categorizer;
pub mod planner;
pub mod operations;
pub mod metadata;

// Re-export key types for easier usage
pub use categorizer::{FileCategorizer, FileCategory, CategorizedFile, CategoryAnalysis, CommitPhase};
pub use planner::CommitPlanner;
pub use operations::GitOperations;
pub use metadata::{MetadataManager, ProjectConfig, SessionHistory};

use crate::errors::CIError;

#[derive(Debug)]
pub struct TopologyAnalysis {
    pub category_analysis: CategoryAnalysis,
    pub commit_phases: Vec<CommitPhase>,
    pub repository_stats: RepositoryStats,
}

#[derive(Debug)]
pub struct RepositoryStats {
    pub total_files: usize,
    pub untracked_files: usize,
    pub modified_files: usize,
    pub estimated_total_size: usize,
    pub suggested_phases: usize,
}

pub struct Topologist {
    categorizer: FileCategorizer,
    git_ops: GitOperations,
    metadata: MetadataManager,
}

impl Topologist {
    pub fn new() -> Self {
        Self {
            categorizer: FileCategorizer::new(),
            git_ops: GitOperations::new(),
            metadata: MetadataManager::new(),
        }
    }

    /// Analyze repository without creating any files
    pub fn analyze_repository(&self) -> Result<TopologyAnalysis, CIError> {
        let (untracked, modified) = self.git_ops.get_repository_status()
            .map_err(|e| CIError::GitOperationError(e.to_string()))?;
        let all_files: Vec<String> = untracked.into_iter().chain(modified.into_iter()).collect();
        
        let category_analysis = self.categorizer.analyze_files(all_files.clone());
        let commit_phases = self.categorizer.generate_commit_plan(&category_analysis);
        
        let repository_stats = RepositoryStats {
            total_files: all_files.len(),
            untracked_files: category_analysis.files.iter()
                .filter(|f| self.git_ops.is_untracked(&f.path).unwrap_or(false))
                .count(),
            modified_files: all_files.len() - category_analysis.files.iter()
                .filter(|f| self.git_ops.is_untracked(&f.path).unwrap_or(false))
                .count(),
            estimated_total_size: category_analysis.estimated_total_size,
            suggested_phases: category_analysis.suggested_phases,
        };

        Ok(TopologyAnalysis {
            category_analysis,
            commit_phases,
            repository_stats,
        })
    }

    /// Initialize topology management for current repository
    pub fn initialize(&mut self) -> Result<(), CIError> {
        self.metadata.initialize_project()
            .map_err(|e| CIError::MetadataError(e.to_string()))?;
        Ok(())
    }

    /// Execute a specific commit phase
    pub fn execute_phase(&mut self, phase_number: usize, commit_phases: &[CommitPhase]) -> Result<String, CIError> {
        if let Some(phase) = commit_phases.get(phase_number - 1) {
            let file_paths: Vec<&str> = phase.files.iter().map(|f| f.path.as_str()).collect();
            let commit_hash = self.git_ops.stage_and_commit_files(&file_paths, &phase.commit_message)
                .map_err(|e| CIError::GitOperationError(e.to_string()))?;
            
            // Record the phase execution
            self.metadata.record_phase_execution(phase_number, &commit_hash, phase.files.len(), phase.estimated_size)
                .map_err(|e| CIError::MetadataError(e.to_string()))?;
            
            Ok(commit_hash)
        } else {
            Err(CIError::TopologyError(format!("Phase {} not found", phase_number)))
        }
    }

    /// Clean all topology metadata
    pub fn clean(&mut self) -> Result<(), CIError> {
        self.metadata.clean_all_metadata()
            .map_err(|e| CIError::MetadataError(e.to_string()))
    }

    /// Get current session status
    pub fn get_status(&self) -> Result<Option<SessionHistory>, CIError> {
        self.metadata.get_current_session()
            .map_err(|e| CIError::MetadataError(e.to_string()))
    }
}

impl Default for Topologist {
    fn default() -> Self {
        Self::new()
    }
}