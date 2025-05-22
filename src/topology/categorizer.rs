// File Categorization Engine for CI Topology Management
// Adapted from standalone topologist for CI integration

use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FileCategory {
    Configuration,
    Documentation, 
    SourceCode,
    BuildArtifacts,
    DevelopmentTools,
    MediaAssets,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CategorizedFile {
    pub path: String,
    pub category: FileCategory,
    pub estimated_size: usize,
    pub priority: u8, // 1-10, higher = commit earlier
    pub grouping_hint: String,
}

#[derive(Debug)]
pub struct CategoryAnalysis {
    pub files: Vec<CategorizedFile>,
    pub category_counts: HashMap<FileCategory, usize>,
    pub estimated_total_size: usize,
    pub suggested_phases: usize,
}

pub struct FileCategorizer {
    // Configuration patterns for different file types
    config_patterns: Vec<&'static str>,
    doc_patterns: Vec<&'static str>,
    code_patterns: Vec<&'static str>,
    build_patterns: Vec<&'static str>,
    tool_patterns: Vec<&'static str>,
    media_patterns: Vec<&'static str>,
}

impl FileCategorizer {
    pub fn new() -> Self {
        Self {
            config_patterns: vec![
                "*.json", "*.yaml", "*.yml", "*.toml", "*.ini", "*.cfg",
                ".env*", "config*", "settings*", "Makefile*", "*.lock",
                "package.json", "Cargo.toml", "requirements.txt", "composer.json"
            ],
            doc_patterns: vec![
                "*.md", "*.rst", "*.txt", "README*", "CHANGELOG*", "LICENSE*",
                "docs/*", "documentation/*", "*.adoc", "*.tex"
            ],
            code_patterns: vec![
                "*.rs", "*.js", "*.ts", "*.py", "*.go", "*.java", "*.cpp", "*.c",
                "*.rb", "*.php", "*.sh", "*.bash", "*.zsh", "*.ps1", "*.sql",
                "src/*", "lib/*", "app/*", "*.html", "*.css", "*.scss"
            ],
            build_patterns: vec![
                "target/*", "build/*", "dist/*", "out/*", "*.min.*", "*.bundle.*",
                "node_modules/*", "*.o", "*.a", "*.so", "*.dylib", "*.exe"
            ],
            tool_patterns: vec![
                "scripts/*", "tools/*", "bin/*", "*.sh", "Dockerfile*", 
                ".github/*", ".gitlab/*", "ci/*", "deploy/*", "*.template"
            ],
            media_patterns: vec![
                "*.png", "*.jpg", "*.jpeg", "*.gif", "*.svg", "*.ico",
                "*.mp4", "*.mov", "*.avi", "*.pdf", "*.ttf", "*.woff*",
                "assets/*", "images/*", "media/*", "static/*"
            ],
        }
    }

    pub fn analyze_files(&self, file_paths: Vec<String>) -> CategoryAnalysis {
        let mut categorized_files = Vec::new();
        let mut category_counts = HashMap::new();
        let mut total_size = 0;

        for file_path in file_paths {
            let categorized = self.categorize_file(&file_path);
            
            // Update counters
            *category_counts.entry(categorized.category.clone()).or_insert(0) += 1;
            total_size += categorized.estimated_size;
            
            categorized_files.push(categorized);
        }

        // Sort by priority (high priority first) then by category
        categorized_files.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(format!("{:?}", a.category).cmp(&format!("{:?}", b.category)))
        });

        let suggested_phases = self.calculate_suggested_phases(&categorized_files);

        CategoryAnalysis {
            files: categorized_files,
            category_counts,
            estimated_total_size: total_size,
            suggested_phases,
        }
    }

    fn categorize_file(&self, file_path: &str) -> CategorizedFile {
        let path = Path::new(file_path);
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // Categorization logic with priority assignment
        let (category, priority, grouping_hint) = if self.matches_patterns(file_path, &self.build_patterns) {
            (FileCategory::BuildArtifacts, 1, "build-artifacts".to_string())
        } else if self.matches_patterns(file_path, &self.config_patterns) {
            (FileCategory::Configuration, 9, self.get_config_group(file_path))
        } else if self.matches_patterns(file_path, &self.doc_patterns) {
            (FileCategory::Documentation, 8, self.get_doc_group(file_path))
        } else if self.matches_patterns(file_path, &self.tool_patterns) {
            (FileCategory::DevelopmentTools, 7, "dev-tools".to_string())
        } else if self.matches_patterns(file_path, &self.code_patterns) {
            (FileCategory::SourceCode, 6, self.get_code_group(file_path, extension))
        } else if self.matches_patterns(file_path, &self.media_patterns) {
            (FileCategory::MediaAssets, 5, "media".to_string())
        } else {
            (FileCategory::Unknown, 4, "misc".to_string())
        };

        CategorizedFile {
            path: file_path.to_string(),
            category,
            estimated_size: self.estimate_file_size(file_path, extension),
            priority,
            grouping_hint,
        }
    }

    fn matches_patterns(&self, file_path: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|pattern| {
            if pattern.contains('*') {
                self.glob_match(pattern, file_path)
            } else {
                file_path.contains(pattern)
            }
        })
    }

    fn glob_match(&self, pattern: &str, file_path: &str) -> bool {
        // Simple glob matching for basic patterns
        if pattern.starts_with("*.") {
            let extension = &pattern[2..];
            file_path.ends_with(&format!(".{}", extension))
        } else if pattern.ends_with("/*") {
            let dir = &pattern[..pattern.len()-2];
            file_path.starts_with(&format!("{}/", dir))
        } else if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len()-1];
            file_path.contains(prefix)
        } else {
            file_path == pattern
        }
    }

    fn get_config_group(&self, file_path: &str) -> String {
        if file_path.contains("package") || file_path.contains("requirements") {
            "dependencies".to_string()
        } else if file_path.contains("docker") || file_path.contains("Docker") {
            "containerization".to_string()
        } else if file_path.contains(".env") {
            "environment".to_string()
        } else {
            "configuration".to_string()
        }
    }

    fn get_doc_group(&self, file_path: &str) -> String {
        if file_path.to_lowercase().contains("readme") {
            "core-docs".to_string()
        } else if file_path.contains("docs/") || file_path.contains("documentation/") {
            "detailed-docs".to_string()
        } else {
            "documentation".to_string()
        }
    }

    fn get_code_group(&self, file_path: &str, extension: &str) -> String {
        match extension {
            "rs" => "rust-code".to_string(),
            "js" | "ts" => "javascript".to_string(),
            "py" => "python".to_string(),
            "go" => "golang".to_string(),
            "java" => "java".to_string(),
            "cpp" | "c" => "c-cpp".to_string(),
            "html" | "css" | "scss" => "web-frontend".to_string(),
            "sh" | "bash" | "zsh" => "shell-scripts".to_string(),
            _ => "source-code".to_string(),
        }
    }

    fn estimate_file_size(&self, file_path: &str, extension: &str) -> usize {
        // Rough size estimation based on file type and path depth
        let base_size = match extension {
            "md" | "txt" | "rst" => 200,  // Documentation files
            "json" | "yaml" | "yml" | "toml" => 100, // Config files
            "rs" | "js" | "ts" | "py" | "go" => 300, // Source code
            "sh" | "bash" => 150, // Scripts
            "lock" => 500, // Lock files can be large
            _ => 100, // Default
        };

        // Adjust based on path depth (deeper = likely more complex)
        let depth_multiplier = file_path.matches('/').count() as f32 * 0.2 + 1.0;
        
        (base_size as f32 * depth_multiplier) as usize
    }

    fn calculate_suggested_phases(&self, files: &[CategorizedFile]) -> usize {
        let total_estimated_size: usize = files.iter().map(|f| f.estimated_size).sum();
        let unique_categories = files.iter()
            .map(|f| &f.category)
            .collect::<std::collections::HashSet<_>>()
            .len();

        // Base phases on category diversity and total size
        let size_based_phases = (total_estimated_size / 1000).max(1); // ~1000 lines per phase
        let category_based_phases = unique_categories.max(1);
        
        // Take the smaller of the two, but at least 2 phases for organization
        size_based_phases.min(category_based_phases).max(2)
    }

    pub fn generate_commit_plan(&self, analysis: &CategoryAnalysis) -> Vec<CommitPhase> {
        let mut phases = Vec::new();
        let mut current_phase_files = Vec::new();
        let mut current_phase_size = 0;
        let mut current_category = None;

        const MAX_PHASE_SIZE: usize = 1500; // Target max lines per phase

        for file in &analysis.files {
            // Start new phase if category changes or size exceeds limit
            if current_category != Some(&file.category) || 
               current_phase_size + file.estimated_size > MAX_PHASE_SIZE {
                
                if !current_phase_files.is_empty() {
                    phases.push(CommitPhase {
                        phase_number: phases.len() + 1,
                        files: current_phase_files.clone(),
                        category: current_category.clone().unwrap().clone(),
                        estimated_size: current_phase_size,
                        commit_message: self.generate_commit_message(&current_phase_files),
                    });
                    
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
            let commit_message = self.generate_commit_message(&current_phase_files);
            phases.push(CommitPhase {
                phase_number: phases.len() + 1,
                files: current_phase_files,
                category: current_category.unwrap().clone(),
                estimated_size: current_phase_size,
                commit_message,
            });
        }

        phases
    }

    fn generate_commit_message(&self, files: &[CategorizedFile]) -> String {
        let category = &files[0].category;
        let file_count = files.len();
        
        let (prefix, description) = match category {
            FileCategory::Configuration => (
                "config:",
                format!("Add {} configuration files for system setup", file_count)
            ),
            FileCategory::Documentation => (
                "docs:",
                format!("Add {} documentation files and guides", file_count)
            ),
            FileCategory::SourceCode => (
                "feat:",
                format!("Add {} source code files and implementations", file_count)
            ),
            FileCategory::DevelopmentTools => (
                "tools:",
                format!("Add {} development tools and scripts", file_count)
            ),
            FileCategory::MediaAssets => (
                "assets:",
                format!("Add {} media files and static assets", file_count)
            ),
            _ => (
                "add:",
                format!("Add {} miscellaneous files", file_count)
            ),
        };

        format!("{} {}", prefix, description)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitPhase {
    pub phase_number: usize,
    pub files: Vec<CategorizedFile>,
    pub category: FileCategory,
    pub estimated_size: usize,
    pub commit_message: String,
}

impl Default for FileCategorizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_categorization() {
        let categorizer = FileCategorizer::new();
        
        let test_files = vec![
            "package.json".to_string(),
            "README.md".to_string(),
            "src/main.rs".to_string(),
            "target/release/app".to_string(),
            "docs/guide.md".to_string(),
        ];

        let analysis = categorizer.analyze_files(test_files);
        
        assert_eq!(analysis.files.len(), 5);
        assert!(analysis.category_counts.contains_key(&FileCategory::Configuration));
        assert!(analysis.category_counts.contains_key(&FileCategory::Documentation));
        assert!(analysis.category_counts.contains_key(&FileCategory::SourceCode));
        assert!(analysis.category_counts.contains_key(&FileCategory::BuildArtifacts));
    }

    #[test]
    fn test_commit_plan_generation() {
        let categorizer = FileCategorizer::new();
        
        let test_files = vec![
            "package.json".to_string(),
            "README.md".to_string(),
            "src/main.rs".to_string(),
        ];

        let analysis = categorizer.analyze_files(test_files);
        let plan = categorizer.generate_commit_plan(&analysis);
        
        assert!(!plan.is_empty());
        assert!(plan.iter().all(|phase| !phase.commit_message.is_empty()));
    }
}