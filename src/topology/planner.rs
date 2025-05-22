// Commit Planning Module - Strategic commit sequence generation
// Adapted from standalone topologist for CI integration

use crate::topology::categorizer::{CategoryAnalysis, CategorizedFile, FileCategory, CommitPhase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitPlan {
    pub phases: Vec<CommitPhase>,
    pub strategy: CommitStrategy,
    pub estimated_duration: EstimatedDuration,
    pub optimization_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommitStrategy {
    Sequential,     // Standard sequential commits
    Parallel,       // Where possible, prepare parallel commits
    SizeOptimized,  // Optimize for commit size balance
    CategoryFirst,  // Group by category priority
    DependencyAware, // Consider file dependencies
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedDuration {
    pub total_phases: usize,
    pub estimated_minutes: u32,
    pub complexity_score: u8, // 1-10 scale
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseOptimization {
    pub recommended_batch_size: usize,
    pub risk_assessment: RiskLevel,
    pub review_complexity: ReviewComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,    // Safe, routine files
    Medium, // Some complexity or size
    High,   // Large changes or critical files
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewComplexity {
    Trivial,   // Auto-generated or config files
    Simple,    // Straightforward changes
    Moderate,  // Requires some review
    Complex,   // Needs careful examination
}

pub struct CommitPlanner {
    // Configuration for commit planning
    max_phase_size: usize,
    preferred_files_per_phase: usize,
    category_priorities: HashMap<FileCategory, u8>,
}

impl CommitPlanner {
    pub fn new() -> Self {
        let mut category_priorities = HashMap::new();
        category_priorities.insert(FileCategory::Configuration, 10);  // Highest priority
        category_priorities.insert(FileCategory::Documentation, 9);
        category_priorities.insert(FileCategory::DevelopmentTools, 8);
        category_priorities.insert(FileCategory::SourceCode, 7);
        category_priorities.insert(FileCategory::MediaAssets, 6);
        category_priorities.insert(FileCategory::BuildArtifacts, 1); // Lowest priority
        category_priorities.insert(FileCategory::Unknown, 5);

        Self {
            max_phase_size: 2000,           // Max estimated lines per phase
            preferred_files_per_phase: 15,  // Ideal number of files per phase
            category_priorities,
        }
    }

    /// Generate an optimized commit plan from category analysis
    pub fn generate_plan(&self, analysis: &CategoryAnalysis) -> CommitPlan {
        let strategy = self.determine_strategy(analysis);
        let phases = match strategy {
            CommitStrategy::Sequential => self.generate_sequential_phases(analysis),
            CommitStrategy::SizeOptimized => self.generate_size_optimized_phases(analysis),
            CommitStrategy::CategoryFirst => self.generate_category_first_phases(analysis),
            CommitStrategy::DependencyAware => self.generate_dependency_aware_phases(analysis),
            CommitStrategy::Parallel => self.generate_parallel_phases(analysis),
        };

        let estimated_duration = self.calculate_estimated_duration(&phases);
        let optimization_notes = self.generate_optimization_notes(&phases, &strategy);

        CommitPlan {
            phases,
            strategy,
            estimated_duration,
            optimization_notes,
        }
    }

    /// Determine the best strategy based on analysis
    fn determine_strategy(&self, analysis: &CategoryAnalysis) -> CommitStrategy {
        let total_files = analysis.files.len();
        let total_size = analysis.estimated_total_size;
        let category_count = analysis.category_counts.len();

        // Decision logic based on repository characteristics
        if total_files > 50 && total_size > 5000 {
            CommitStrategy::SizeOptimized
        } else if category_count > 4 {
            CommitStrategy::CategoryFirst
        } else if total_files > 30 {
            CommitStrategy::DependencyAware
        } else {
            CommitStrategy::Sequential
        }
    }

    /// Generate sequential phases (basic approach)
    fn generate_sequential_phases(&self, analysis: &CategoryAnalysis) -> Vec<CommitPhase> {
        let mut phases = Vec::new();
        let mut current_phase_files = Vec::new();
        let mut current_phase_size = 0;
        let mut current_category = None;

        for file in &analysis.files {
            // Start new phase if category changes or size limit reached
            if current_category != Some(&file.category) || 
               current_phase_size + file.estimated_size > self.max_phase_size {
                
                if !current_phase_files.is_empty() {
                    phases.push(self.create_commit_phase(
                        phases.len() + 1,
                        current_phase_files.clone(),
                        current_category.clone().unwrap().clone(),
                        current_phase_size,
                    ));
                    
                    current_phase_files.clear();
                    current_phase_size = 0;
                }
                
                current_category = Some(&file.category);
            }

            current_phase_files.push(file.clone());
            current_phase_size += file.estimated_size;
        }

        // Add final phase
        if !current_phase_files.is_empty() {
            phases.push(self.create_commit_phase(
                phases.len() + 1,
                current_phase_files,
                current_category.unwrap().clone(),
                current_phase_size,
            ));
        }

        phases
    }

    /// Generate size-optimized phases
    fn generate_size_optimized_phases(&self, analysis: &CategoryAnalysis) -> Vec<CommitPhase> {
        let mut phases = Vec::new();
        let mut files_by_size: Vec<&CategorizedFile> = analysis.files.iter().collect();
        
        // Sort by size (largest first) to better balance phases
        files_by_size.sort_by(|a, b| b.estimated_size.cmp(&a.estimated_size));

        let mut current_phase_files = Vec::new();
        let mut current_phase_size = 0;
        let target_phase_size = self.max_phase_size / 2; // Use smaller target for better balance

        for file in files_by_size {
            if current_phase_size + file.estimated_size > self.max_phase_size && !current_phase_files.is_empty() {
                // Create phase with current files
                let dominant_category = self.get_dominant_category(&current_phase_files);
                phases.push(self.create_commit_phase(
                    phases.len() + 1,
                    current_phase_files.clone(),
                    dominant_category,
                    current_phase_size,
                ));
                
                current_phase_files.clear();
                current_phase_size = 0;
            }

            current_phase_files.push(file.clone());
            current_phase_size += file.estimated_size;

            // Create phase early if we've reached a good balance
            if current_phase_size >= target_phase_size && current_phase_files.len() >= 5 {
                let dominant_category = self.get_dominant_category(&current_phase_files);
                phases.push(self.create_commit_phase(
                    phases.len() + 1,
                    current_phase_files.clone(),
                    dominant_category,
                    current_phase_size,
                ));
                
                current_phase_files.clear();
                current_phase_size = 0;
            }
        }

        // Add final phase
        if !current_phase_files.is_empty() {
            let dominant_category = self.get_dominant_category(&current_phase_files);
            phases.push(self.create_commit_phase(
                phases.len() + 1,
                current_phase_files,
                dominant_category,
                current_phase_size,
            ));
        }

        phases
    }

    /// Generate category-first phases
    fn generate_category_first_phases(&self, analysis: &CategoryAnalysis) -> Vec<CommitPhase> {
        let mut phases = Vec::new();
        
        // Group files by category with priority ordering
        let mut files_by_category: HashMap<FileCategory, Vec<CategorizedFile>> = HashMap::new();
        
        for file in &analysis.files {
            files_by_category
                .entry(file.category.clone())
                .or_insert_with(Vec::new)
                .push(file.clone());
        }

        // Process categories in priority order
        let mut sorted_categories: Vec<_> = files_by_category.keys().cloned().collect();
        sorted_categories.sort_by(|a, b| {
            let priority_a = self.category_priorities.get(a).unwrap_or(&5);
            let priority_b = self.category_priorities.get(b).unwrap_or(&5);
            priority_b.cmp(priority_a) // Higher priority first
        });

        for category in sorted_categories {
            let files = files_by_category.get(&category).unwrap();
            
            // Split large categories into multiple phases
            if files.len() > self.preferred_files_per_phase {
                let chunks = files.chunks(self.preferred_files_per_phase);
                for chunk in chunks {
                    let phase_size = chunk.iter().map(|f| f.estimated_size).sum();
                    phases.push(self.create_commit_phase(
                        phases.len() + 1,
                        chunk.to_vec(),
                        category.clone(),
                        phase_size,
                    ));
                }
            } else {
                let phase_size = files.iter().map(|f| f.estimated_size).sum();
                phases.push(self.create_commit_phase(
                    phases.len() + 1,
                    files.clone(),
                    category.clone(),
                    phase_size,
                ));
            }
        }

        phases
    }

    /// Generate dependency-aware phases
    fn generate_dependency_aware_phases(&self, analysis: &CategoryAnalysis) -> Vec<CommitPhase> {
        // For now, use sequential with some intelligent grouping
        // Future: Implement actual dependency analysis
        let mut phases = self.generate_sequential_phases(analysis);
        
        // Post-process to optimize for dependencies
        self.optimize_for_dependencies(&mut phases);
        
        phases
    }

    /// Generate parallel-ready phases
    fn generate_parallel_phases(&self, analysis: &CategoryAnalysis) -> Vec<CommitPhase> {
        // Similar to category-first but with parallel execution hints
        let mut phases = self.generate_category_first_phases(analysis);
        
        // Add parallel execution metadata
        for phase in &mut phases {
            // Mark phases that could potentially be executed in parallel
            // This is a simplified heuristic
            phase.estimated_size = phase.estimated_size; // Placeholder for parallel metadata
        }
        
        phases
    }

    /// Create a commit phase with proper message generation
    fn create_commit_phase(
        &self,
        phase_number: usize,
        files: Vec<CategorizedFile>,
        category: FileCategory,
        estimated_size: usize,
    ) -> CommitPhase {
        let commit_message = self.generate_commit_message(&files, &category);
        
        CommitPhase {
            phase_number,
            files,
            category,
            estimated_size,
            commit_message,
        }
    }

    /// Generate optimized commit message
    fn generate_commit_message(&self, files: &[CategorizedFile], category: &FileCategory) -> String {
        let file_count = files.len();
        
        let (prefix, description) = match category {
            FileCategory::Configuration => {
                if files.iter().any(|f| f.path.contains("package.json") || f.path.contains("Cargo.toml")) {
                    ("deps:", format!("Add {} dependency configuration files", file_count))
                } else if files.iter().any(|f| f.path.contains("docker") || f.path.contains("Docker")) {
                    ("docker:", format!("Add {} containerization configuration files", file_count))
                } else {
                    ("config:", format!("Add {} configuration files", file_count))
                }
            },
            FileCategory::Documentation => {
                if files.iter().any(|f| f.path.to_lowercase().contains("readme")) {
                    ("docs:", format!("Add {} core documentation files", file_count))
                } else {
                    ("docs:", format!("Add {} documentation and guides", file_count))
                }
            },
            FileCategory::SourceCode => {
                let languages: Vec<&str> = files.iter()
                    .filter_map(|f| {
                        let path = &f.path;
                        if path.ends_with(".rs") { Some("Rust") }
                        else if path.ends_with(".js") || path.ends_with(".ts") { Some("JavaScript/TypeScript") }
                        else if path.ends_with(".py") { Some("Python") }
                        else if path.ends_with(".go") { Some("Go") }
                        else { None }
                    })
                    .collect();
                
                if languages.len() == 1 {
                    ("feat:", format!("Add {} {} source files", file_count, languages[0]))
                } else {
                    ("feat:", format!("Add {} source code files", file_count))
                }
            },
            FileCategory::DevelopmentTools => {
                ("tools:", format!("Add {} development tools and scripts", file_count))
            },
            FileCategory::MediaAssets => {
                ("assets:", format!("Add {} media files and static assets", file_count))
            },
            FileCategory::BuildArtifacts => {
                ("build:", format!("Add {} build artifacts", file_count))
            },
            FileCategory::Unknown => {
                ("add:", format!("Add {} miscellaneous files", file_count))
            },
        };

        format!("{} {}", prefix, description)
    }

    /// Get the dominant category in a mixed file list
    fn get_dominant_category(&self, files: &[CategorizedFile]) -> FileCategory {
        let mut category_counts: HashMap<FileCategory, usize> = HashMap::new();
        
        for file in files {
            *category_counts.entry(file.category.clone()).or_insert(0) += 1;
        }

        // Return the category with the highest count, with priority as tiebreaker
        category_counts
            .into_iter()
            .max_by(|(cat_a, count_a), (cat_b, count_b)| {
                count_a.cmp(count_b).then_with(|| {
                    let priority_a = self.category_priorities.get(cat_a).unwrap_or(&5);
                    let priority_b = self.category_priorities.get(cat_b).unwrap_or(&5);
                    priority_a.cmp(priority_b)
                })
            })
            .map(|(category, _)| category)
            .unwrap_or(FileCategory::Unknown)
    }

    /// Calculate estimated duration for the commit plan
    fn calculate_estimated_duration(&self, phases: &[CommitPhase]) -> EstimatedDuration {
        let total_phases = phases.len();
        let total_files: usize = phases.iter().map(|p| p.files.len()).sum();
        let total_size: usize = phases.iter().map(|p| p.estimated_size).sum();

        // Estimate time based on various factors
        let base_time_per_phase = 2; // 2 minutes baseline per phase
        let time_per_file = if total_files > 50 { 0 } else { 1 }; // Less time per file for large batches
        let size_factor = total_size / 1000; // Additional time for large changes

        let estimated_minutes = (total_phases * base_time_per_phase) + 
                                (total_files * time_per_file) + 
                                size_factor;

        // Complexity score based on various factors
        let complexity_score = ((total_phases.min(10) * 2) + 
                                (total_files.min(100) / 10) + 
                                (total_size.min(10000) / 1000)).min(10);

        EstimatedDuration {
            total_phases,
            estimated_minutes: estimated_minutes as u32,
            complexity_score: complexity_score as u8,
        }
    }

    /// Generate optimization notes for the plan
    fn generate_optimization_notes(&self, phases: &[CommitPhase], strategy: &CommitStrategy) -> Vec<String> {
        let mut notes = Vec::new();
        
        // Strategy-specific notes
        match strategy {
            CommitStrategy::SizeOptimized => {
                notes.push("Phases optimized for balanced commit sizes".to_string());
            },
            CommitStrategy::CategoryFirst => {
                notes.push("Files grouped by category for logical organization".to_string());
            },
            CommitStrategy::DependencyAware => {
                notes.push("Commit order considers file dependencies".to_string());
            },
            _ => {}
        }

        // Size-based recommendations
        let large_phases = phases.iter().filter(|p| p.estimated_size > 1500).count();
        if large_phases > 0 {
            notes.push(format!("{} phases are large (>1500 lines) - consider reviewing carefully", large_phases));
        }

        // File count recommendations
        let busy_phases = phases.iter().filter(|p| p.files.len() > 20).count();
        if busy_phases > 0 {
            notes.push(format!("{} phases have many files (>20) - consider splitting if needed", busy_phases));
        }

        // General recommendations
        if phases.len() > 10 {
            notes.push("Consider using 'execute all' for efficient batch processing".to_string());
        }

        notes
    }

    /// Optimize phases for dependencies (simplified implementation)
    fn optimize_for_dependencies(&self, phases: &mut [CommitPhase]) {
        // Simple optimization: Move configuration files to earlier phases
        phases.sort_by(|a, b| {
            let priority_a = self.category_priorities.get(&a.category).unwrap_or(&5);
            let priority_b = self.category_priorities.get(&b.category).unwrap_or(&5);
            priority_b.cmp(priority_a) // Higher priority first
        });

        // Renumber phases after sorting
        for (i, phase) in phases.iter_mut().enumerate() {
            phase.phase_number = i + 1;
        }
    }
}

impl Default for CommitPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::categorizer::{CategorizedFile, FileCategory};

    #[test]
    fn test_commit_planner_creation() {
        let planner = CommitPlanner::new();
        assert_eq!(planner.max_phase_size, 2000);
        assert_eq!(planner.preferred_files_per_phase, 15);
    }

    #[test]
    fn test_strategy_determination() {
        let planner = CommitPlanner::new();
        
        // Test small repository
        let small_analysis = create_test_analysis(5, 500);
        let strategy = planner.determine_strategy(&small_analysis);
        assert!(matches!(strategy, CommitStrategy::Sequential));
        
        // Test large repository
        let large_analysis = create_test_analysis(60, 6000);
        let strategy = planner.determine_strategy(&large_analysis);
        assert!(matches!(strategy, CommitStrategy::SizeOptimized));
    }

    fn create_test_analysis(file_count: usize, total_size: usize) -> CategoryAnalysis {
        let files = (0..file_count)
            .map(|i| CategorizedFile {
                path: format!("file_{}.txt", i),
                category: FileCategory::Documentation,
                estimated_size: total_size / file_count,
                priority: 5,
                grouping_hint: "test".to_string(),
            })
            .collect();

        let mut category_counts = HashMap::new();
        category_counts.insert(FileCategory::Documentation, file_count);

        CategoryAnalysis {
            files,
            category_counts,
            estimated_total_size: total_size,
            suggested_phases: 2,
        }
    }
}