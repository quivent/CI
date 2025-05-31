use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Standardization protocol for CI implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardizationProtocol {
    pub version: String,
    pub enforcement_level: EnforcementLevel,
    pub standards: HashMap<String, Standard>,
    pub global_policies: Vec<GlobalPolicy>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Advisory,
    Warning,
    Error,
    Blocking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Standard {
    pub category: String,
    pub description: String,
    pub required_pattern: String,
    pub examples: Vec<String>,
    pub violations: Vec<String>,
    pub enforcement: EnforcementLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPolicy {
    pub name: String,
    pub description: String,
    pub applies_to: Vec<String>,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub pattern: String,
    pub scope: ValidationScope,
    pub action: ValidationAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationScope {
    FunctionNames,
    ErrorHandling,
    ImportStatements,
    ConfigGeneration,
    AgentActivation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationAction {
    Report,
    Fix,
    Block,
}

/// Standardization engine for CI implementations
pub struct StandardizationEngine {
    protocol: StandardizationProtocol,
    project_root: PathBuf,
}

impl StandardizationEngine {
    /// Initialize standardization engine with automatic protocol detection
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let protocol = Self::create_default_protocol();
        
        let mut engine = Self {
            protocol,
            project_root,
        };
        
        // Immediately scan for similar implementations
        engine.scan_for_similar_implementations()?;
        
        Ok(engine)
    }
    
    /// Create default standardization protocol
    fn create_default_protocol() -> StandardizationProtocol {
        let mut standards = HashMap::new();
        
        // Agent Function Naming Standard
        standards.insert("agent_function_naming".to_string(), Standard {
            category: "Naming Conventions".to_string(),
            description: "All agent-related functions must use agent_ prefix".to_string(),
            required_pattern: "^agent_[a-z_]+$".to_string(),
            examples: vec![
                "agent_activate()".to_string(),
                "agent_load()".to_string(),
                "agent_configure()".to_string(),
            ],
            violations: vec![
                "enable_agent()".to_string(),
                "activate_agent()".to_string(),
                "loadAgent()".to_string(),
            ],
            enforcement: EnforcementLevel::Error,
        });
        
        // Error Handling Standard
        standards.insert("error_handling".to_string(), Standard {
            category: "Error Management".to_string(),
            description: "Use CIError for all agent-related errors with context".to_string(),
            required_pattern: "CIError::[A-Z][a-zA-Z]*\\(.*\\)\\.into\\(\\)".to_string(),
            examples: vec![
                "CIError::AgentNotFound(name.clone()).into()".to_string(),
                "CIError::ActivationFailed(msg).into()".to_string(),
            ],
            violations: vec![
                "anyhow::anyhow!()".to_string(),
                "panic!()".to_string(),
            ],
            enforcement: EnforcementLevel::Warning,
        });
        
        // CLAUDE.md Generation Standard
        standards.insert("claude_md_generation".to_string(), Standard {
            category: "Configuration".to_string(),
            description: "Use unified CLAUDE.md template with agent activation protocol".to_string(),
            required_pattern: "agent_activation_protocol_template".to_string(),
            examples: vec![
                "StandardizationEngine::generate_claude_md()".to_string(),
            ],
            violations: vec![
                "Multiple different CLAUDE.md formats".to_string(),
            ],
            enforcement: EnforcementLevel::Blocking,
        });
        
        // Agent Activation Standard
        standards.insert("agent_activation".to_string(), Standard {
            category: "Agent Management".to_string(),
            description: "Use signature protocol detection for agent activation".to_string(),
            required_pattern: "\\[AGENT_NAME\\].*--\\s*\\[AGENT_NAME\\]".to_string(),
            examples: vec![
                "[ATHENA]: content -- [ATHENA]".to_string(),
            ],
            violations: vec![
                "@[AGENT_ACTIVATION:{}]".to_string(),
            ],
            enforcement: EnforcementLevel::Error,
        });
        
        let global_policies = vec![
            GlobalPolicy {
                name: "Agent Loading Policy".to_string(),
                description: "All agents must follow standardization protocols".to_string(),
                applies_to: vec!["Athena".to_string(), "ProjectArchitect".to_string(), "Standardist".to_string()],
                requirements: vec![
                    "Read standardization protocols on initialization".to_string(),
                    "Validate implementations against standards".to_string(),
                    "Report violations immediately".to_string(),
                ],
            },
        ];
        
        let validation_rules = vec![
            ValidationRule {
                name: "Function Name Validation".to_string(),
                pattern: "fn\\s+(\\w+)\\(".to_string(),
                scope: ValidationScope::FunctionNames,
                action: ValidationAction::Report,
            },
            ValidationRule {
                name: "Error Pattern Validation".to_string(),
                pattern: "(anyhow::anyhow!|CIError::|panic!)".to_string(),
                scope: ValidationScope::ErrorHandling,
                action: ValidationAction::Fix,
            },
        ];
        
        StandardizationProtocol {
            version: "1.0.0".to_string(),
            enforcement_level: EnforcementLevel::Error,
            standards,
            global_policies,
            validation_rules,
        }
    }
    
    /// Scan for similar implementations with different mechanisms
    pub fn scan_for_similar_implementations(&mut self) -> Result<()> {
        println!("{}", "🔍 Scanning for implementation inconsistencies...".cyan().bold());
        
        let agent_files = self.find_agent_related_files()?;
        let mut violations = Vec::new();
        
        for file_path in &agent_files {
            let content = fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;
            
            violations.extend(self.analyze_file_violations(file_path, &content)?);
        }
        
        if !violations.is_empty() {
            println!("{} Found {} violations:", "⚠".yellow().bold(), violations.len());
            for violation in &violations {
                println!("  {} {}: {}", "•".red(), violation.file, violation.description);
            }
            
            self.generate_standardization_report(&violations)?;
        }
        
        Ok(())
    }
    
    /// Find all agent-related files
    fn find_agent_related_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        let search_dirs = vec![
            self.project_root.join("src/commands"),
            self.project_root.join("src/helpers"),
            self.project_root.join("src/tools"),
        ];
        
        for dir in search_dirs {
            if dir.exists() {
                self.find_rust_files_recursive(&dir, &mut files)?;
            }
        }
        
        // Filter for agent-related files
        files.retain(|f| {
            let filename = f.file_name().unwrap_or_default().to_string_lossy();
            filename.contains("agent") || 
            filename.contains("claude") ||
            filename.contains("config")
        });
        
        Ok(files)
    }
    
    /// Recursively find Rust files
    fn find_rust_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.find_rust_files_recursive(&path, files)?;
                } else if path.extension().map_or(false, |ext| ext == "rs") {
                    files.push(path);
                }
            }
        }
        Ok(())
    }
    
    /// Analyze file for standard violations
    fn analyze_file_violations(&self, file_path: &Path, content: &str) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        
        // Check function naming
        for (line_num, line) in content.lines().enumerate() {
            if let Some(func_name) = self.extract_function_name(line) {
                if self.is_agent_function(&func_name) && !self.follows_naming_standard(&func_name) {
                    violations.push(Violation {
                        file: file_path.display().to_string(),
                        line: line_num + 1,
                        standard: "agent_function_naming".to_string(),
                        description: format!("Function '{}' violates naming standard", func_name),
                        severity: ViolationSeverity::Error,
                    });
                }
            }
            
            // Check error handling
            if line.contains("anyhow::anyhow!") && self.is_agent_context(line) {
                violations.push(Violation {
                    file: file_path.display().to_string(),
                    line: line_num + 1,
                    standard: "error_handling".to_string(),
                    description: "Use CIError instead of anyhow::anyhow! for agent errors".to_string(),
                    severity: ViolationSeverity::Warning,
                });
            }
        }
        
        Ok(violations)
    }
    
    /// Extract function name from line
    fn extract_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("fn ") {
            let after_fn = &line[start + 3..];
            if let Some(end) = after_fn.find('(') {
                return Some(after_fn[..end].trim().to_string());
            }
        }
        None
    }
    
    /// Check if function is agent-related
    fn is_agent_function(&self, name: &str) -> bool {
        name.contains("agent") || 
        name.contains("activate") ||
        name.contains("claude") ||
        name.contains("config")
    }
    
    /// Check if function follows naming standard
    fn follows_naming_standard(&self, name: &str) -> bool {
        if name.contains("agent") {
            name.starts_with("agent_") || name == "agent" || name.starts_with("create_") || name.starts_with("handle_")
        } else {
            true // Non-agent functions can use any convention
        }
    }
    
    /// Check if line is in agent context
    fn is_agent_context(&self, line: &str) -> bool {
        line.to_lowercase().contains("agent") ||
        line.contains("CLAUDE") ||
        line.contains("activation")
    }
    
    /// Generate standardization report
    fn generate_standardization_report(&self, violations: &[Violation]) -> Result<()> {
        let report_path = self.project_root.join("STANDARDIZATION_REPORT.md");
        
        let mut report = String::new();
        report.push_str("# CI Standardization Report\n\n");
        report.push_str(&format!("Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("Protocol Version: {}\n\n", self.protocol.version));
        
        report.push_str("## Violations Found\n\n");
        
        let mut violations_by_standard: HashMap<String, Vec<&Violation>> = HashMap::new();
        for violation in violations {
            violations_by_standard.entry(violation.standard.clone())
                .or_insert_with(Vec::new)
                .push(violation);
        }
        
        for (standard, viols) in violations_by_standard {
            report.push_str(&format!("### {}\n\n", standard));
            for violation in viols {
                report.push_str(&format!("- **{}:{}**: {}\n", violation.file, violation.line, violation.description));
            }
            report.push_str("\n");
        }
        
        report.push_str("## Standardization Protocol\n\n");
        report.push_str("```json\n");
        report.push_str(&serde_json::to_string_pretty(&self.protocol)?);
        report.push_str("\n```\n");
        
        fs::write(&report_path, report)
            .with_context(|| format!("Failed to write report to {}", report_path.display()))?;
        
        println!("{} Generated standardization report: {}", "✓".green().bold(), report_path.display());
        
        Ok(())
    }
    
    /// Apply standardization fixes
    pub fn apply_standardization_fixes(&self) -> Result<()> {
        println!("{}", "🔧 Applying standardization fixes...".cyan().bold());
        
        // This would implement automatic fixes for violations
        // For now, we'll create the framework for manual fixes
        
        Ok(())
    }
    
    /// Save standardization protocol to global location
    pub fn save_to_global_policy(&self) -> Result<()> {
        let ci_root = crate::helpers::path::get_ci_root()?;
        let protocols_dir = ci_root.join("protocols");
        fs::create_dir_all(&protocols_dir)?;
        
        let protocol_file = protocols_dir.join("standardization.json");
        let protocol_json = serde_json::to_string_pretty(&self.protocol)?;
        
        fs::write(&protocol_file, protocol_json)
            .with_context(|| format!("Failed to save protocol to {}", protocol_file.display()))?;
        
        println!("{} Saved standardization protocol to: {}", "✓".green().bold(), protocol_file.display());
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Violation {
    file: String,
    line: usize,
    standard: String,
    description: String,
    severity: ViolationSeverity,
}

#[derive(Debug, Clone)]
enum ViolationSeverity {
    Warning,
    Error,
    Critical,
}

/// Initialize standardization engine for CI project
pub fn initialize_standardization() -> Result<StandardizationEngine> {
    let project_root = std::env::current_dir()?;
    StandardizationEngine::new(project_root)
}

/// Quick standardization check for current project
pub fn quick_standardization_check() -> Result<()> {
    let engine = initialize_standardization()?;
    engine.save_to_global_policy()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_function_name_extraction() {
        let engine = StandardizationEngine::new(TempDir::new().unwrap().path().to_path_buf()).unwrap();
        
        assert_eq!(engine.extract_function_name("fn test_function() {"), Some("test_function".to_string()));
        assert_eq!(engine.extract_function_name("pub fn agent_activate(name: &str) -> Result<()> {"), Some("agent_activate".to_string()));
        assert_eq!(engine.extract_function_name("    fn helper() {"), Some("helper".to_string()));
    }
    
    #[test]
    fn test_naming_standard_validation() {
        let engine = StandardizationEngine::new(TempDir::new().unwrap().path().to_path_buf()).unwrap();
        
        assert!(engine.follows_naming_standard("agent_activate"));
        assert!(engine.follows_naming_standard("agent_load"));
        assert!(!engine.follows_naming_standard("enable_agent"));
        assert!(!engine.follows_naming_standard("activate_agent"));
    }
}