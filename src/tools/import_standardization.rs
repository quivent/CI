use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::Path;

/// Import standardization protocol for CI implementations
pub struct ImportStandardization;

impl ImportStandardization {
    /// Standard import order for CI files
    pub const STANDARD_IMPORT_ORDER: &'static [&'static str] = &[
        "// Standard library imports (alphabetical)",
        "std::",
        "",
        "// External crate imports (alphabetical)", 
        "anyhow::",
        "chrono::",
        "clap::",
        "colored::",
        "serde::",
        "serde_json::",
        "tempfile::",
        "tokio::",
        "",
        "// Internal crate imports (alphabetical)",
        "crate::config::",
        "crate::errors::",
        "crate::helpers::",
        "crate::shared::",
        "crate::tools::",
        "crate::topology::",
    ];
    
    /// Standard import patterns for different file types
    pub fn get_standard_imports_for_command() -> Vec<String> {
        vec![
            "use anyhow::{Context, Result};".to_string(),
            "use clap::{Arg, ArgMatches, Command};".to_string(),
            "use colored::Colorize;".to_string(),
            "use std::fs;".to_string(),
            "use std::path::{Path, PathBuf};".to_string(),
            "".to_string(),
            "use crate::errors::CIError;".to_string(),
            "use crate::helpers::CommandHelpers;".to_string(),
        ]
    }
    
    pub fn get_standard_imports_for_helper() -> Vec<String> {
        vec![
            "use anyhow::{Context, Result};".to_string(),
            "use colored::Colorize;".to_string(),
            "use std::fs;".to_string(),
            "use std::path::{Path, PathBuf};".to_string(),
            "".to_string(),
            "use crate::errors::CIError;".to_string(),
        ]
    }
    
    pub fn get_standard_imports_for_config() -> Vec<String> {
        vec![
            "use anyhow::{Context, Result};".to_string(),
            "use serde::{Deserialize, Serialize};".to_string(),
            "use serde_json;".to_string(),
            "use std::fs;".to_string(),
            "use std::path::{Path, PathBuf};".to_string(),
            "".to_string(),
            "use crate::errors::CIError;".to_string(),
        ]
    }
    
    /// Analyze import patterns in a file
    pub fn analyze_imports(file_path: &Path, content: &str) -> Vec<ImportViolation> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut import_section_start = None;
        let mut import_section_end = None;
        let mut imports = Vec::new();
        
        // Find import section
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("use ") {
                if import_section_start.is_none() {
                    import_section_start = Some(i);
                }
                imports.push((i + 1, line.to_string()));
            } else if !line.trim().is_empty() && !line.starts_with("//") && import_section_start.is_some() {
                import_section_end = Some(i);
                break;
            }
        }
        
        // Check import ordering
        if imports.len() > 1 {
            violations.extend(Self::check_import_ordering(&imports, file_path));
        }
        
        // Check for missing standard imports based on file type
        violations.extend(Self::check_missing_standard_imports(file_path, &imports));
        
        violations
    }
    
    fn check_import_ordering(imports: &[(usize, String)], file_path: &Path) -> Vec<ImportViolation> {
        let mut violations = Vec::new();
        
        let mut std_imports = Vec::new();
        let mut external_imports = Vec::new();
        let mut internal_imports = Vec::new();
        
        for (line_num, import) in imports {
            if import.contains("std::") {
                std_imports.push((*line_num, import.clone()));
            } else if import.contains("crate::") {
                internal_imports.push((*line_num, import.clone()));
            } else {
                external_imports.push((*line_num, import.clone()));
            }
        }
        
        // Check if std imports come first
        if !std_imports.is_empty() && !external_imports.is_empty() {
            let first_std = std_imports.first().unwrap().0;
            let first_external = external_imports.first().unwrap().0;
            
            if first_std > first_external {
                violations.push(ImportViolation {
                    file: file_path.display().to_string(),
                    line: first_std,
                    violation_type: ImportViolationType::WrongOrder,
                    description: "std imports should come before external crate imports".to_string(),
                });
            }
        }
        
        // Check if internal imports come last
        if !internal_imports.is_empty() && !external_imports.is_empty() {
            let last_external = external_imports.last().unwrap().0;
            let first_internal = internal_imports.first().unwrap().0;
            
            if first_internal < last_external {
                violations.push(ImportViolation {
                    file: file_path.display().to_string(),
                    line: first_internal,
                    violation_type: ImportViolationType::WrongOrder,
                    description: "crate imports should come after external crate imports".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn check_missing_standard_imports(file_path: &Path, imports: &[(usize, String)]) -> Vec<ImportViolation> {
        let mut violations = Vec::new();
        let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
        
        let import_strings: Vec<String> = imports.iter().map(|(_, imp)| imp.clone()).collect();
        
        // Check for missing colored::Colorize in command files
        if filename.contains("command") || file_path.to_string_lossy().contains("/commands/") {
            if !import_strings.iter().any(|imp| imp.contains("colored::Colorize")) &&
               !import_strings.iter().any(|imp| imp.contains("colored::*")) {
                violations.push(ImportViolation {
                    file: file_path.display().to_string(),
                    line: 1,
                    violation_type: ImportViolationType::MissingStandard,
                    description: "Command files should import colored::Colorize for consistent output".to_string(),
                });
            }
        }
        
        // Check for missing CIError import in files that use Result
        if import_strings.iter().any(|imp| imp.contains("Result")) &&
           !import_strings.iter().any(|imp| imp.contains("CIError")) {
            violations.push(ImportViolation {
                file: file_path.display().to_string(),
                line: 1,
                violation_type: ImportViolationType::MissingStandard,
                description: "Files using Result should import CIError for consistent error handling".to_string(),
            });
        }
        
        violations
    }
    
    /// Generate standardized import section for a file type
    pub fn generate_standard_imports(file_type: FileType) -> String {
        let imports = match file_type {
            FileType::Command => Self::get_standard_imports_for_command(),
            FileType::Helper => Self::get_standard_imports_for_helper(),
            FileType::Config => Self::get_standard_imports_for_config(),
        };
        
        imports.join("\n")
    }
    
    /// Apply import standardization to a file
    pub fn standardize_file_imports(file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;
        
        let violations = Self::analyze_imports(file_path, &content);
        
        if violations.is_empty() {
            return Ok(());
        }
        
        println!("{} Standardizing imports in: {}", "🔧".cyan(), file_path.display());
        
        for violation in &violations {
            println!("  {} {}: {}", "•".yellow(), violation.line, violation.description);
        }
        
        // TODO: Implement automatic import reordering
        // For now, just report violations
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ImportViolation {
    pub file: String,
    pub line: usize,
    pub violation_type: ImportViolationType,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum ImportViolationType {
    WrongOrder,
    MissingStandard,
    Inconsistent,
    Unnecessary,
}

#[derive(Debug, Clone)]
pub enum FileType {
    Command,
    Helper,
    Config,
}

impl FileType {
    pub fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy();
        
        if path_str.contains("/commands/") {
            FileType::Command
        } else if path_str.contains("/helpers/") || path_str.contains("/config/") {
            if path_str.contains("config") {
                FileType::Config
            } else {
                FileType::Helper
            }
        } else {
            FileType::Helper // Default
        }
    }
}

/// Run import standardization across all CI files
pub fn standardize_all_imports() -> Result<()> {
    println!("{}", "🔍 Analyzing import patterns across CI codebase...".cyan().bold());
    
    let src_dir = std::env::current_dir()?.join("src");
    let mut total_violations = 0;
    
    fn scan_directory(dir: &Path, violations_count: &mut usize) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_directory(&path, violations_count)?;
                } else if path.extension().map_or(false, |ext| ext == "rs") {
                    let content = fs::read_to_string(&path)?;
                    let violations = ImportStandardization::analyze_imports(&path, &content);
                    
                    if !violations.is_empty() {
                        *violations_count += violations.len();
                        ImportStandardization::standardize_file_imports(&path)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    scan_directory(&src_dir, &mut total_violations)?;
    
    if total_violations > 0 {
        println!("{} Found {} import violations across codebase", "⚠".yellow().bold(), total_violations);
    } else {
        println!("{} All import patterns are standardized", "✓".green().bold());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_import_analysis() {
        let content = r#"use std::fs;
use anyhow::Result;
use crate::errors::CIError;
use colored::Colorize;

fn main() {}
"#;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, content).unwrap();
        
        let violations = ImportStandardization::analyze_imports(&file_path, content);
        
        // Should detect wrong order (anyhow before std completion)
        assert!(!violations.is_empty());
    }
}