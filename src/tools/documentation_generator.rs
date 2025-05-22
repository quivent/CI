//! Documentation generation tool for CI
//!
//! This module provides comprehensive documentation generation capabilities,
//! including command references, helper function documentation, and
//! documentation index creation.

use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::{Result, Context};
use colored::*;
use regex::Regex;

/// Structure for documentation related operations
pub struct DocumentationGenerator;

impl DocumentationGenerator {
    /// Generate comprehensive documentation for the entire project
    pub fn generate_project_documentation() -> Result<()> {
        println!("{} Generating comprehensive project documentation...", "📚".cyan());
        
        // Ensure docs directory exists
        let docs_dir = Path::new("docs");
        if !docs_dir.exists() {
            fs::create_dir_all(docs_dir)
                .with_context(|| "Failed to create docs directory")?;
        }
        
        // Create documentation index
        Self::create_documentation_index()?;
        
        // Generate command documentation
        Self::generate_command_documentation()?;
        
        // Generate helper documentation
        Self::generate_helper_documentation()?;
        
        // Generate templates
        Self::create_documentation_templates()?;
        
        println!("{} Documentation generation completed successfully!", "✓".green());
        
        Ok(())
    }
    
    /// Create documentation index file
    pub fn create_documentation_index() -> Result<()> {
        let index_path = Path::new("docs/documentation_index.md");
        
        println!("{} Generating documentation index at {}...", "📝".cyan(), index_path.display());
        
        // Collect all markdown files in the docs directory
        let mut doc_files = HashMap::new();
        let docs_dir = Path::new("docs");
        
        if docs_dir.exists() {
            Self::collect_documentation_files(docs_dir, &mut doc_files)?;
        }
        
        // Generate index content
        let mut index_content = String::new();
        index_content.push_str("# CI Documentation Index\n\n");
        index_content.push_str("## Overview\n\n");
        index_content.push_str("This index provides links to all documentation for the CI (Collaborative Intelligence in Rust) tool.\n\n");
        
        // Add Command Reference section
        index_content.push_str("## Command Reference\n\n");
        
        // Intelligence & Discovery Commands
        index_content.push_str("### 🧠 Intelligence & Discovery Commands\n\n");
        Self::add_category_files_to_index(&mut index_content, &doc_files, "commands/intelligence");
        
        // Source Control Commands
        index_content.push_str("### 📊 Source Control Commands\n\n");
        Self::add_category_files_to_index(&mut index_content, &doc_files, "commands/source_control");
        
        // Project Lifecycle Commands
        index_content.push_str("### 🚀 Project Lifecycle Commands\n\n");
        Self::add_category_files_to_index(&mut index_content, &doc_files, "commands/lifecycle");
        
        // System Management Commands
        index_content.push_str("### ⚙️ System Management Commands\n\n");
        Self::add_category_files_to_index(&mut index_content, &doc_files, "commands/system");
        
        // Add Helper Reference section
        index_content.push_str("## Helper Function Reference\n\n");
        Self::add_category_files_to_index(&mut index_content, &doc_files, "helpers");
        
        // Add Guides section
        index_content.push_str("## Guides & Tutorials\n\n");
        Self::add_category_files_to_index(&mut index_content, &doc_files, "guides");
        
        // Add Other Documentation section
        index_content.push_str("## Other Documentation\n\n");
        
        // Add remaining files that don't fit into categories
        let mut other_files = doc_files.keys()
            .filter(|&k| !k.starts_with("commands/") && !k.starts_with("helpers/") && !k.starts_with("guides/") && *k != "documentation_index.md")
            .collect::<Vec<_>>();
        
        other_files.sort();
        
        for key in other_files {
            let (title, path) = doc_files.get(key).unwrap();
            index_content.push_str(&format!("- [{}]({})\n", title, path.strip_prefix("docs/").unwrap_or(path).to_string_lossy()));
        }
        
        // Write index file
        fs::write(index_path, index_content)
            .with_context(|| format!("Failed to write documentation index: {}", index_path.display()))?;
            
        println!("{} Documentation index created successfully!", "✓".green());
        
        Ok(())
    }
    
    /// Collect all documentation files in a directory and its subdirectories
    fn collect_documentation_files(dir: &Path, files: &mut HashMap<String, (String, PathBuf)>) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }
        
        for entry in fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))? {
            
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively process subdirectories
                Self::collect_documentation_files(&path, files)?;
            } else if let Some(ext) = path.extension() {
                if ext == "md" {
                    // Process markdown file
                    let key = path.strip_prefix("docs/")
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                        
                    // Extract title from markdown file
                    let content = fs::read_to_string(&path)
                        .with_context(|| format!("Failed to read file: {}", path.display()))?;
                        
                    let title = Self::extract_title_from_markdown(&content)
                        .unwrap_or_else(|| {
                            path.file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        });
                        
                    files.insert(key, (title, path));
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract title from markdown content (first # heading)
    fn extract_title_from_markdown(content: &str) -> Option<String> {
        let re = Regex::new(r"^#\s+(.+)$").unwrap();
        
        for line in content.lines() {
            if let Some(captures) = re.captures(line) {
                if let Some(title) = captures.get(1) {
                    return Some(title.as_str().trim().to_string());
                }
            }
        }
        
        None
    }
    
    /// Add category files to the index content
    fn add_category_files_to_index(index_content: &mut String, doc_files: &HashMap<String, (String, PathBuf)>, category: &str) {
        let mut category_files = doc_files.iter()
            .filter(|(k, _)| k.starts_with(category))
            .collect::<Vec<_>>();
            
        category_files.sort_by(|a, b| a.0.cmp(b.0));
        
        for (_key, (title, path)) in category_files {
            index_content.push_str(&format!("- [{}]({})\n", title, path.strip_prefix("docs/").unwrap_or(path).to_string_lossy()));
        }
        
        index_content.push_str("\n");
    }
    
    /// Generate documentation for all commands
    pub fn generate_command_documentation() -> Result<()> {
        println!("{} Generating command documentation...", "📝".cyan());
        
        // Ensure command docs directories exist
        let categories = [
            "intelligence",
            "source_control",
            "lifecycle",
            "system",
        ];
        
        for category in &categories {
            let category_dir = Path::new("docs/commands").join(category);
            if !category_dir.exists() {
                fs::create_dir_all(&category_dir)
                    .with_context(|| format!("Failed to create command docs directory: {}", category_dir.display()))?;
            }
        }
        
        // Process each command file
        for category in &categories {
            let command_file = format!("src/commands/{}.rs", category);
            if Path::new(&command_file).exists() {
                Self::extract_and_document_commands(&command_file, category)?;
            }
        }
        
        println!("{} Command documentation completed successfully!", "✓".green());
        
        Ok(())
    }
    
    /// Extract and document commands from a command file
    fn extract_and_document_commands(file_path: &str, category: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read command file: {}", file_path))?;
            
        // Use regex to find command functions
        let re = Regex::new(r"/// (.+?)\s*\npub async fn ([a-z_]+)").unwrap();
        
        for captures in re.captures_iter(&content) {
            let description = captures.get(1).unwrap().as_str().trim();
            let command_name = captures.get(2).unwrap().as_str();
            
            // Generate enhanced documentation for this command
            Self::generate_enhanced_command_doc(command_name, description, category)?;
        }
        
        Ok(())
    }
    
    /// Generate enhanced documentation for a command
    fn generate_enhanced_command_doc(command_name: &str, description: &str, category: &str) -> Result<()> {
        let doc_path = format!("docs/commands/{}/{}.md", category, command_name);
        
        // Check if documentation already exists
        if Path::new(&doc_path).exists() {
            // Just update existing documentation
            return Ok(());
        }
        
        let emoji = match category {
            "intelligence" => "🧠",
            "source_control" => "📊",
            "lifecycle" => "🚀",
            "system" => "⚙️",
            _ => "📌",
        };
        
        let category_name = match category {
            "intelligence" => "Intelligence & Discovery",
            "source_control" => "Source Control",
            "lifecycle" => "Project Lifecycle",
            "system" => "System Management",
            _ => "Other",
        };
        
        // Get related commands based on category
        let related_commands = Self::get_related_commands(command_name, category);
        
        let related_commands_section = if related_commands.is_empty() {
            "*No related commands*".to_string()
        } else {
            related_commands.iter()
                .map(|cmd| format!("- [{}](../{})", cmd.0, cmd.1))
                .collect::<Vec<_>>()
                .join("\n")
        };
        
        let doc_content = format!(r#"# {} {}

{}

*Category: {}*

## Description

{}

## Usage

```bash
ci {} [OPTIONS]
```

## Options

*Command-specific options are listed below. Run `ci help {}` for more information.*

| Option | Description |
|--------|-------------|
| *To be documented* | |

## Examples

```bash
# Basic usage
ci {}

# Example with options
ci {} --option value
```

## Notes

- Add any implementation notes or important information here
- Document any limitations or special considerations

## Related Commands

{}
"#, 
            emoji,
            command_name.to_uppercase(),
            description,
            category_name,
            description,
            command_name,
            command_name,
            command_name,
            command_name,
            related_commands_section
        );
        
        fs::write(&doc_path, doc_content)
            .with_context(|| format!("Failed to write command documentation: {}", doc_path))?;
            
        Ok(())
    }
    
    /// Get related commands for a given command and category
    fn get_related_commands(command_name: &str, category: &str) -> Vec<(String, String)> {
        let command_file = format!("src/commands/{}.rs", category);
        if !Path::new(&command_file).exists() {
            return Vec::new();
        }
        
        let content = match fs::read_to_string(&command_file) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        
        // Use regex to find command functions
        let re = Regex::new(r"pub async fn ([a-z_]+)").unwrap();
        
        let mut related = Vec::new();
        for captures in re.captures_iter(&content) {
            let related_command = captures.get(1).unwrap().as_str();
            if related_command != command_name {
                related.push((
                    related_command.to_string(), 
                    format!("{}/{}.md", category, related_command)
                ));
            }
        }
        
        related
    }
    
    /// Generate documentation for all helpers
    pub fn generate_helper_documentation() -> Result<()> {
        println!("{} Generating helper documentation...", "📝".cyan());
        
        // Ensure helpers docs directory exists
        let helpers_dir = Path::new("docs/helpers");
        if !helpers_dir.exists() {
            fs::create_dir_all(helpers_dir)
                .with_context(|| "Failed to create helpers docs directory")?;
        }
        
        // Process helper files
        let helpers_glob = glob::glob("src/helpers/*.rs")
            .with_context(|| "Failed to glob helper files")?;
            
        for entry in helpers_glob {
            let path = entry.with_context(|| "Failed to get helper file path")?;
            
            if path.file_name().unwrap_or_default() != "mod.rs" {
                Self::extract_and_document_helper(&path)?;
            }
        }
        
        println!("{} Helper documentation completed successfully!", "✓".green());
        
        Ok(())
    }
    
    /// Extract and document a helper file
    fn extract_and_document_helper(path: &Path) -> Result<()> {
        let file_name = path.file_stem().unwrap_or_default().to_string_lossy();
        let doc_path = format!("docs/helpers/{}.md", file_name);
        
        // Check if documentation already exists
        if Path::new(&doc_path).exists() {
            // Just update existing documentation
            return Ok(());
        }
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read helper file: {}", path.display()))?;
            
        // Extract module documentation
        let module_doc = Self::extract_module_doc(&content);
        
        // Extract structs and their methods
        let structs = Self::extract_structs_and_methods(&content);
        
        // Generate documentation content
        let mut doc_content = String::new();
        doc_content.push_str(&format!("# {} Helper\n\n", to_title_case(&file_name)));
        
        if let Some(doc) = module_doc {
            doc_content.push_str(&format!("{}\n\n", doc.trim()));
        } else {
            doc_content.push_str(&format!("Helper functions for {}.\n\n", file_name));
        }
        
        // Add table of contents
        doc_content.push_str("## Table of Contents\n\n");
        
        for (struct_name, _) in &structs {
            doc_content.push_str(&format!("- [{}](#{})\n", struct_name, struct_name.to_lowercase()));
        }
        
        doc_content.push_str("\n");
        
        // Add struct documentation
        for (struct_name, methods) in structs {
            doc_content.push_str(&format!("## {}\n\n", struct_name));
            
            // Add methods table
            doc_content.push_str("| Method | Description |\n");
            doc_content.push_str("|--------|-------------|\n");
            
            for (method_name, description) in &methods {
                let short_desc = description.lines().next().unwrap_or("").trim();
                doc_content.push_str(&format!("| `{}` | {} |\n", method_name, short_desc));
            }
            
            doc_content.push_str("\n");
            
            // Add detailed method documentation
            for (method_name, description) in &methods {
                doc_content.push_str(&format!("### {}\n\n", method_name));
                doc_content.push_str(&format!("{}\n\n", description.trim()));
            }
        }
        
        // Add usage examples section
        doc_content.push_str("## Usage Examples\n\n");
        doc_content.push_str("```rust\n// Example usage of this helper\n```\n\n");
        
        fs::write(&doc_path, doc_content)
            .with_context(|| format!("Failed to write helper documentation: {}", doc_path))?;
            
        Ok(())
    }
    
    /// Extract module documentation from file content
    fn extract_module_doc(content: &str) -> Option<String> {
        let re = Regex::new(r"(?s)//!(.+?)(?:\n\n|\n[^/])").ok()?;
        
        re.captures(content).map(|caps| {
            caps.get(1).unwrap().as_str()
                .lines()
                .map(|line| line.trim_start_matches("//!").trim())
                .collect::<Vec<_>>()
                .join("\n")
        })
    }
    
    /// Extract structs and their methods from file content
    fn extract_structs_and_methods(content: &str) -> Vec<(String, Vec<(String, String)>)> {
        let mut result = Vec::new();
        
        // Extract struct names
        let struct_re = Regex::new(r"pub struct (\w+)").unwrap();
        let impl_re = Regex::new(r"impl (\w+)").unwrap();
        let method_re = Regex::new(r"(?s)/// (.+?)(?:\n    pub fn |\n    pub async fn )(\w+)").unwrap();
        
        // First find all structs
        for captures in struct_re.captures_iter(content) {
            let struct_name = captures.get(1).unwrap().as_str();
            result.push((struct_name.to_string(), Vec::new()));
        }
        
        // Then find all impls
        for (i, captures) in impl_re.captures_iter(content).enumerate() {
            let impl_name = captures.get(1).unwrap().as_str();
            
            // Find the matching struct
            let struct_index = result.iter().position(|(name, _)| name == impl_name);
            
            // Skip if no matching struct found (might be for a trait)
            if struct_index.is_none() {
                continue;
            }
            
            // Find the impl block
            let impl_start = captures.get(0).unwrap().start();
            let impl_end = if let Some(next_captures) = impl_re.captures_iter(content).nth(i + 1) {
                next_captures.get(0).unwrap().start()
            } else {
                content.len()
            };
            
            let impl_block = &content[impl_start..impl_end];
            
            // Extract methods with docs
            let mut methods = Vec::new();
            for method_captures in method_re.captures_iter(impl_block) {
                let method_doc = method_captures.get(1).unwrap().as_str()
                    .lines()
                    .map(|line| line.trim_start_matches("///").trim())
                    .collect::<Vec<_>>()
                    .join("\n");
                    
                let method_name = method_captures.get(2).unwrap().as_str();
                
                methods.push((method_name.to_string(), method_doc));
            }
            
            // Add methods to the struct
            if let Some(struct_index) = struct_index {
                result[struct_index].1 = methods;
            }
        }
        
        result
    }
    
    /// Create documentation templates
    pub fn create_documentation_templates() -> Result<()> {
        println!("{} Creating documentation templates...", "📝".cyan());
        
        // Ensure template directory exists
        let templates_dir = Path::new("docs/templates");
        if !templates_dir.exists() {
            fs::create_dir_all(templates_dir)
                .with_context(|| "Failed to create templates directory")?;
        }
        
        // Command documentation template
        let command_template = r#"# 📌 COMMAND_NAME

Command description

*Category: Category Name*

## Description

Detailed description of the command and its purpose.

## Usage

```bash
ci command_name [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--option1` | Description of option 1 |
| `--option2` | Description of option 2 |

## Examples

```bash
# Basic usage
ci command_name

# Example with options
ci command_name --option1 value
```

## Notes

- Add any implementation notes or important information here
- Document any limitations or special considerations

## Related Commands

- [related_command](path/to/related_command.md)
"#;

        // Helper documentation template
        let helper_template = r#"# Helper Name

Helper description

## Table of Contents

- [HelperStruct](#helperstruct)

## HelperStruct

| Method | Description |
|--------|-------------|
| `method_name` | Brief description of the method |

### method_name

Detailed description of the method.

## Usage Examples

```rust
// Example usage of this helper
```
"#;

        // Guide documentation template
        let guide_template = r#"# Guide Title

Guide description

## Overview

Explain what this guide covers and why it's important.

## Prerequisites

- List any prerequisites or requirements
- Knowledge/tools needed

## Steps

### 1. First Step

Detailed instructions for the first step.

```bash
# Example code or commands
```

### 2. Second Step

Detailed instructions for the second step.

## Common Issues

- Issue 1: Solution 1
- Issue 2: Solution 2

## Further Reading

- [Related Resource 1](url)
- [Related Resource 2](url)
"#;

        // Write templates
        fs::write(templates_dir.join("command.md"), command_template)
            .with_context(|| "Failed to write command template")?;
            
        fs::write(templates_dir.join("helper.md"), helper_template)
            .with_context(|| "Failed to write helper template")?;
            
        fs::write(templates_dir.join("guide.md"), guide_template)
            .with_context(|| "Failed to write guide template")?;
            
        println!("{} Documentation templates created successfully!", "✓".green());
        
        Ok(())
    }
}

/// Convert a string to title case (first letter of each word capitalized)
fn to_title_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in input.chars() {
        if c == '_' || c == ' ' || c == '-' {
            result.push(' ');
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}