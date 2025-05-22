//! Advanced commit message generation and analysis
//!
//! This module provides enhanced commit message generation with deeper analysis 
//! of changes and integration with AI services when available.

use std::path::Path;
use std::collections::HashMap;
use std::process::Command;
use anyhow::{Result, Context, anyhow};
use crate::helpers::command::CommandHelpers;
use colored::Colorize;

/// File change classification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Addition,
    Modification,
    Removal,
    Rename,
    Permission,
    Untracked,
}

/// File change information
#[derive(Debug)]
pub struct FileChange {
    pub file_path: String,
    pub change_type: ChangeType,
    pub additions: usize,
    pub deletions: usize,
    pub language: Option<String>,
    pub component: Option<String>,
}

/// Commit analysis with detailed breakdown
#[derive(Debug)]
pub struct CommitAnalysis {
    pub files_changed: Vec<FileChange>,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub languages: HashMap<String, usize>,
    pub components: HashMap<String, usize>,
    pub suggested_message: String,
    pub suggested_details: String,
    pub change_summary: String,
}

/// Advanced commit message analyzer
pub struct CommitAnalyzer;

impl CommitAnalyzer {
    /// Analyze staged changes and generate a detailed commit message
    pub async fn analyze_staged_changes(repo_path: &Path) -> Result<CommitAnalysis> {
        // Get staged files list
        let output = Command::new("git")
            .args(["diff", "--name-status", "--staged"])
            .current_dir(repo_path)
            .output()
            .with_context(|| "Failed to get staged files")?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to get staged files"));
        }
        
        let status_output = String::from_utf8_lossy(&output.stdout);
        if status_output.trim().is_empty() {
            return Err(anyhow!("No staged changes found"));
        }
        
        // Parse file changes with status (A: added, M: modified, D: deleted, R: renamed)
        let mut file_changes = Vec::new();
        
        for line in status_output.lines() {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }
            
            // Split into status and file(s)
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.is_empty() {
                continue;
            }
            
            let status = parts[0];
            
            if parts.len() < 2 {
                continue;
            }
            
            let file_path = parts[1];
            
            // Determine change type from status code
            let change_type = match status {
                "A" => ChangeType::Addition,
                "M" => ChangeType::Modification,
                "D" => ChangeType::Removal,
                "R" => ChangeType::Rename,
                _ => ChangeType::Modification, // default case
            };
            
            // For renames, handle both old and new paths
            let (file_path, _old_path) = if change_type == ChangeType::Rename && parts.len() >= 3 {
                (parts[2], Some(parts[1]))
            } else {
                (file_path, None)
            };
            
            // Get file stats (additions/deletions) for non-deleted files
            let (additions, deletions) = if change_type != ChangeType::Removal {
                Self::get_file_diff_stats(repo_path, file_path)?
            } else {
                (0, 0) // Deleted files don't have additions/deletions in the staged diff
            };
            
            // Determine language from file extension
            let language = Self::detect_language(file_path);
            
            // Determine component from file path
            let component = Self::detect_component(file_path);
            
            file_changes.push(FileChange {
                file_path: file_path.to_string(),
                change_type,
                additions,
                deletions,
                language,
                component,
            });
        }
        
        // Calculate totals and analyze
        let (languages, components, total_additions, total_deletions) = Self::calculate_totals(&file_changes);
        
        // Generate suggested commit message
        let (message, details) = Self::generate_commit_message(&file_changes, &languages, &components)?;
        
        // Generate change summary
        let change_summary = Self::generate_change_summary(&file_changes, total_additions, total_deletions);
        
        Ok(CommitAnalysis {
            files_changed: file_changes,
            total_additions,
            total_deletions,
            languages,
            components,
            suggested_message: message,
            suggested_details: details,
            change_summary,
        })
    }
    
    /// Format and print the commit analysis in a user-friendly way
    pub fn display_analysis(analysis: &CommitAnalysis) {
        CommandHelpers::print_divider("green");
        println!("{}", "Commit Analysis".green().bold());
        CommandHelpers::print_divider("green");
        
        // Print suggested commit message
        println!("{}", "Suggested Commit Message:".bold());
        println!("{}", analysis.suggested_message.cyan());
        println!();
        
        // Print files changed
        println!("{}", "Files Changed:".bold());
        for file in &analysis.files_changed {
            let change_icon = match file.change_type {
                ChangeType::Addition => "+".green(),
                ChangeType::Modification => "M".yellow(),
                ChangeType::Removal => "-".red(),
                ChangeType::Rename => "R".cyan(),
                ChangeType::Permission => "P".purple(),
                ChangeType::Untracked => "?".blue(),
            };
            
            println!("{} {} {}{}", 
                change_icon,
                file.file_path, 
                if file.additions > 0 { format!(" +{}", file.additions).green() } else { "".normal() },
                if file.deletions > 0 { format!(" -{}", file.deletions).red() } else { "".normal() }
            );
        }
        println!();
        
        // Print language breakdown if we have multiple languages
        if analysis.languages.len() > 1 {
            println!("{}", "Language Breakdown:".bold());
            let total_files = analysis.files_changed.len();
            for (language, count) in &analysis.languages {
                let percentage = (*count as f64 / total_files as f64) * 100.0;
                println!("  {} - {} files ({:.1}%)", language, count, percentage);
            }
            println!();
        }
        
        // Print change summary
        println!("{}", "Change Summary:".bold());
        println!("{}", analysis.change_summary);
        println!();
        
        // Print suggested details
        if !analysis.suggested_details.is_empty() {
            println!("{}", "Additional Details:".bold());
            println!("{}", analysis.suggested_details);
        }
        
        CommandHelpers::print_divider("green");
    }
    
    /// Get additions and deletions for a file
    fn get_file_diff_stats(repo_path: &Path, file_path: &str) -> Result<(usize, usize)> {
        let output = Command::new("git")
            .args(["diff", "--staged", "--numstat", file_path])
            .current_dir(repo_path)
            .output()
            .with_context(|| format!("Failed to get stats for {}", file_path))?;
            
        if !output.status.success() {
            return Ok((0, 0)); // Default if we can't get stats
        }
        
        let stat_output = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stat_output.split_whitespace().collect();
        
        if parts.len() >= 2 {
            let additions = parts[0].parse().unwrap_or(0);
            let deletions = parts[1].parse().unwrap_or(0);
            Ok((additions, deletions))
        } else {
            Ok((0, 0))
        }
    }
    
    /// Detect language from file extension
    fn detect_language(file_path: &str) -> Option<String> {
        let lower_path = file_path.to_lowercase();
        
        if lower_path.ends_with(".rs") {
            Some("Rust".to_string())
        } else if lower_path.ends_with(".js") {
            Some("JavaScript".to_string())
        } else if lower_path.ends_with(".ts") {
            Some("TypeScript".to_string())
        } else if lower_path.ends_with(".jsx") || lower_path.ends_with(".tsx") {
            Some("React".to_string())
        } else if lower_path.ends_with(".py") {
            Some("Python".to_string())
        } else if lower_path.ends_with(".html") {
            Some("HTML".to_string())
        } else if lower_path.ends_with(".css") {
            Some("CSS".to_string())
        } else if lower_path.ends_with(".scss") || lower_path.ends_with(".sass") {
            Some("SASS".to_string())
        } else if lower_path.ends_with(".json") {
            Some("JSON".to_string())
        } else if lower_path.ends_with(".toml") {
            Some("TOML".to_string())
        } else if lower_path.ends_with(".yaml") || lower_path.ends_with(".yml") {
            Some("YAML".to_string())
        } else if lower_path.ends_with(".md") {
            Some("Markdown".to_string())
        } else if lower_path.ends_with(".sh") {
            Some("Shell".to_string())
        } else if lower_path.ends_with(".go") {
            Some("Go".to_string())
        } else if lower_path.ends_with(".java") {
            Some("Java".to_string())
        } else if lower_path.ends_with(".c") {
            Some("C".to_string())
        } else if lower_path.ends_with(".cpp") || lower_path.ends_with(".cc") || lower_path.ends_with(".cxx") {
            Some("C++".to_string())
        } else if lower_path.ends_with(".h") || lower_path.ends_with(".hpp") {
            Some("C/C++ Header".to_string())
        } else if lower_path.ends_with(".rb") {
            Some("Ruby".to_string())
        } else if lower_path.ends_with(".php") {
            Some("PHP".to_string())
        } else if lower_path.ends_with(".swift") {
            Some("Swift".to_string())
        } else if lower_path.ends_with(".kt") || lower_path.ends_with(".kts") {
            Some("Kotlin".to_string())
        } else if lower_path.ends_with(".dart") {
            Some("Dart".to_string())
        } else {
            None
        }
    }
    
    /// Detect component from file path
    fn detect_component(file_path: &str) -> Option<String> {
        let parts: Vec<&str> = file_path.split('/').collect();
        
        if parts.is_empty() {
            return None;
        }
        
        // Identify component based on common directory patterns
        if parts.len() >= 2 {
            match parts[0] {
                "src" => {
                    if parts.len() >= 3 && parts[1] == "commands" {
                        return Some("Commands".to_string());
                    } else if parts.len() >= 3 && parts[1] == "helpers" {
                        return Some("Helpers".to_string());
                    } else if parts.len() >= 3 && parts[1] == "components" {
                        return Some("UI Components".to_string());
                    } else if parts.len() >= 3 && parts[1] == "utils" {
                        return Some("Utilities".to_string());
                    } else if parts.len() >= 3 && parts[1] == "models" {
                        return Some("Models".to_string());
                    } else if parts.len() >= 3 && parts[1] == "services" {
                        return Some("Services".to_string());
                    } else if parts.len() >= 3 && parts[1] == "api" {
                        return Some("API".to_string());
                    } else {
                        return Some("Source".to_string());
                    }
                },
                "tests" => Some("Tests".to_string()),
                "docs" => Some("Documentation".to_string()),
                "scripts" => Some("Scripts".to_string()),
                "tools" => Some("Tools".to_string()),
                "config" => Some("Configuration".to_string()),
                "assets" => Some("Assets".to_string()),
                "public" => Some("Public Assets".to_string()),
                "client" => Some("Client".to_string()),
                "server" => Some("Server".to_string()),
                "bin" => Some("Binaries".to_string()),
                "lib" => Some("Libraries".to_string()),
                ".github" => Some("GitHub Config".to_string()),
                _ => None,
            }
        } else {
            // Check root files
            let file_name = parts[0].to_lowercase();
            
            if file_name == "readme.md" {
                Some("Documentation".to_string())
            } else if file_name == "cargo.toml" || file_name == "package.json" {
                Some("Dependencies".to_string())
            } else if file_name.ends_with(".md") {
                Some("Documentation".to_string())
            } else if file_name == ".gitignore" {
                Some("Git Config".to_string())
            } else if file_name == "license" || file_name == "license.md" || file_name == "license.txt" {
                Some("License".to_string())
            } else if file_name.starts_with("docker") {
                Some("Docker Config".to_string())
            } else if file_name.starts_with(".") {
                Some("Configuration".to_string())
            } else {
                None
            }
        }
    }
    
    /// Calculate totals for languages, components, and changes
    fn calculate_totals(
        files: &[FileChange]
    ) -> (HashMap<String, usize>, HashMap<String, usize>, usize, usize) {
        let mut languages = HashMap::new();
        let mut components = HashMap::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;
        
        for file in files {
            // Count languages
            if let Some(ref lang) = file.language {
                *languages.entry(lang.clone()).or_insert(0) += 1;
            }
            
            // Count components
            if let Some(ref comp) = file.component {
                *components.entry(comp.clone()).or_insert(0) += 1;
            }
            
            // Sum additions and deletions
            total_additions += file.additions;
            total_deletions += file.deletions;
        }
        
        (languages, components, total_additions, total_deletions)
    }
    
    /// Generate commit message based on files changed
    fn generate_commit_message(
        files: &[FileChange], 
        languages: &HashMap<String, usize>,
        components: &HashMap<String, usize>
    ) -> Result<(String, String)> {
        // Determine primary component (most changed)
        let primary_component = components.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "code".to_string());
            
        // Determine action based on change types
        let has_additions = files.iter().any(|f| f.change_type == ChangeType::Addition);
        let has_removals = files.iter().any(|f| f.change_type == ChangeType::Removal);
        let has_modifications = files.iter().any(|f| f.change_type == ChangeType::Modification);
        
        let action = if has_additions && !has_modifications && !has_removals {
            "Add"
        } else if has_removals && !has_additions && !has_modifications {
            "Remove"
        } else if has_modifications && !has_additions && !has_removals {
            "Update"
        } else if has_removals && has_additions {
            "Refactor"
        } else {
            "Update"
        };
        
        // Determine primary language
        let primary_language = languages.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(name, _)| name.clone());
            
        // Generate basic commit message
        let component_desc = if primary_component == "Documentation" {
            "documentation".to_string()
        } else if primary_component == "Tests" {
            "tests".to_string()
        } else if files.len() == 1 {
            // For single file changes, use the file name
            let file_path = &files[0].file_path;
            let file_name = file_path.split('/').last().unwrap_or(file_path);
            file_name.to_string()
        } else {
            primary_component.to_lowercase()
        };
        
        // More specific commit message for special cases
        let message = if files.len() == 1 {
            // Single file change
            let file = &files[0];
            match file.change_type {
                ChangeType::Addition => format!("Add {}", file.file_path),
                ChangeType::Removal => format!("Remove {}", file.file_path),
                ChangeType::Modification => format!("Update {}", file.file_path),
                ChangeType::Rename => format!("Rename {}", file.file_path),
                ChangeType::Permission => format!("Change permissions for {}", file.file_path),
                ChangeType::Untracked => format!("Add {}", file.file_path),
            }
        } else if languages.len() == 1 && primary_language.is_some() {
            // Multiple files, single language
            let lang = primary_language.unwrap();
            format!("{} {} in {}", action, component_desc, lang)
        } else {
            // Multiple files, multiple languages
            format!("{} {} ({} files)", action, component_desc, files.len())
        };
        
        // Generate more detailed description
        let mut details = format!("Files changed:\n");
        
        // Group files by component for better organization
        let mut by_component: HashMap<String, Vec<&FileChange>> = HashMap::new();
        
        for file in files {
            let component = file.component.clone().unwrap_or_else(|| "Other".to_string());
            by_component.entry(component).or_default().push(file);
        }
        
        // Format details by component
        for (component, files) in by_component {
            details.push_str(&format!("\n{}:\n", component));
            
            for file in files {
                let change_symbol = match file.change_type {
                    ChangeType::Addition => "+",
                    ChangeType::Modification => "~",
                    ChangeType::Removal => "-",
                    ChangeType::Rename => "→",
                    ChangeType::Permission => "p",
                    ChangeType::Untracked => "?",
                };
                
                details.push_str(&format!("  {} {}", change_symbol, file.file_path));
                
                // Add stat details if non-zero
                if file.additions > 0 || file.deletions > 0 {
                    details.push_str(&format!(" (+{}, -{})", file.additions, file.deletions));
                }
                
                details.push('\n');
            }
        }
        
        Ok((message, details))
    }
    
    /// Generate a summary of changes
    fn generate_change_summary(files: &[FileChange], total_additions: usize, total_deletions: usize) -> String {
        let mut summary = String::new();
        
        // Count by change type
        let additions_count = files.iter().filter(|f| f.change_type == ChangeType::Addition).count();
        let modifications_count = files.iter().filter(|f| f.change_type == ChangeType::Modification).count();
        let removals_count = files.iter().filter(|f| f.change_type == ChangeType::Removal).count();
        let renames_count = files.iter().filter(|f| f.change_type == ChangeType::Rename).count();
        
        // Format summary
        if additions_count > 0 {
            summary.push_str(&format!("{} file{} added", additions_count, if additions_count == 1 { "" } else { "s" }));
        }
        
        if modifications_count > 0 {
            if !summary.is_empty() {
                summary.push_str(", ");
            }
            summary.push_str(&format!("{} file{} modified", modifications_count, if modifications_count == 1 { "" } else { "s" }));
        }
        
        if removals_count > 0 {
            if !summary.is_empty() {
                summary.push_str(", ");
            }
            summary.push_str(&format!("{} file{} removed", removals_count, if removals_count == 1 { "" } else { "s" }));
        }
        
        if renames_count > 0 {
            if !summary.is_empty() {
                summary.push_str(", ");
            }
            summary.push_str(&format!("{} file{} renamed", renames_count, if renames_count == 1 { "" } else { "s" }));
        }
        
        // Add stats summary
        if total_additions > 0 || total_deletions > 0 {
            summary.push_str(&format!(" with {} addition{} and {} deletion{}", 
                total_additions, 
                if total_additions == 1 { "" } else { "s" },
                total_deletions,
                if total_deletions == 1 { "" } else { "s" }
            ));
        }
        
        summary
    }
}