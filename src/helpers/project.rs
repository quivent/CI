//! Project management and analysis helpers for CI
//!
//! This module provides helper functions for working with CI-integrated projects,
//! including project information retrieval, statistics gathering, and registration.

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use anyhow::{Context, Result, anyhow};

/// Helper functions for project management
pub struct ProjectHelpers;

impl ProjectHelpers {
    /// Check if a directory is a CI-integrated project
    pub fn is_ci_project(path: &Path) -> bool {
        let claude_md_path = path.join("CLAUDE.md");
        claude_md_path.exists()
    }
    
    /// Get project info from CLAUDE.md
    pub fn get_project_info(path: &Path) -> Result<ProjectInfo> {
        let claude_md_path = path.join("CLAUDE.md");
        
        if !claude_md_path.exists() {
            return Err(anyhow!("Not a CI project: CLAUDE.md not found"));
        }
        
        let content = fs::read_to_string(&claude_md_path)
            .with_context(|| format!("Failed to read CLAUDE.md at {}", claude_md_path.display()))?;
            
        let mut info = ProjectInfo {
            name: String::new(),
            integration_type: String::new(),
            agents: Vec::new(),
            created: None,
            config_path: claude_md_path,
        };
        
        // Parse project name (from "# Project: Name" line)
        for line in content.lines() {
            if line.starts_with("# Project:") {
                info.name = line.trim_start_matches("# Project:").trim().to_string();
                break;
            }
        }
        
        // Parse configuration section
        let config_section = Self::extract_section(&content, "Configuration");
        if let Some(section) = config_section {
            for line in section.lines() {
                if line.contains("Integration:") {
                    info.integration_type = line.split_once("Integration:")
                        .map(|(_, v)| v.trim().to_string())
                        .unwrap_or_default()
                        .trim_end_matches(" integration")
                        .to_string();
                } else if line.contains("Created:") {
                    info.created = line.split_once("Created:")
                        .map(|(_, v)| v.trim().to_string());
                }
            }
        }
        
        // Parse agents section
        let agents_section = Self::extract_section(&content, "Active Agents");
        if let Some(section) = agents_section {
            for line in section.lines() {
                if line.starts_with("-") {
                    let agent = line.trim_start_matches("-").trim().to_string();
                    if !agent.is_empty() {
                        info.agents.push(agent);
                    }
                }
            }
        }
        
        Ok(info)
    }
    
    /// Extract a markdown section by name
    fn extract_section(content: &str, section_name: &str) -> Option<String> {
        let section_header = format!("## {}", section_name);
        let lines: Vec<&str> = content.lines().collect();
        let mut section_content = Vec::new();
        let mut in_section = false;
        
        for line in lines {
            if line == section_header {
                in_section = true;
                continue;
            } else if in_section && line.starts_with("##") {
                break;
            } else if in_section {
                section_content.push(line);
            }
        }
        
        if section_content.is_empty() {
            None
        } else {
            Some(section_content.join("\n"))
        }
    }
    
    /// Get project statistics
    pub fn get_project_stats(path: &Path) -> Result<ProjectStats> {
        let mut stats = ProjectStats {
            file_count: 0,
            file_types: HashMap::new(),
            total_size: 0,
            // Default other fields
            ..Default::default()
        };
        
        // Check if directory exists
        if !path.exists() || !path.is_dir() {
            return Err(anyhow!("Path does not exist or is not a directory"));
        }
        
        // Walk directory recursively
        Self::collect_stats(path, &mut stats)?;
        
        Ok(stats)
    }
    
    /// Recursively collect directory statistics
    fn collect_stats(path: &Path, stats: &mut ProjectStats) -> Result<()> {
        if path.is_dir() {
            // Skip .git, node_modules, and target directories
            let dirname = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
                
            if dirname == ".git" || dirname == "node_modules" || dirname == "target" {
                return Ok(());
            }
            
            // Process directory contents
            let entries = fs::read_dir(path)
                .with_context(|| format!("Failed to read directory: {}", path.display()))?;
                
            for entry in entries {
                let entry = entry
                    .with_context(|| format!("Failed to read directory entry in {}", path.display()))?;
                let path = entry.path();
                
                if path.is_dir() {
                    Self::collect_stats(&path, stats)?;
                } else if path.is_file() {
                    // Get file extension
                    let extension = path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                        
                    // Get file size
                    let metadata = fs::metadata(&path)
                        .with_context(|| format!("Failed to get metadata for {}", path.display()))?;
                    let size = metadata.len() as usize;
                    
                    // Update stats
                    stats.file_count += 1;
                    stats.total_size += size;
                    
                    // Count file type
                    *stats.file_types.entry(extension.clone()).or_insert(0) += 1;
                    
                    // Check for specific file types
                    if extension == "rs" {
                        stats.rust_files += 1;
                    } else if extension == "js" || extension == "ts" {
                        stats.js_files += 1;
                    } else if extension == "py" {
                        stats.python_files += 1;
                    } else if extension == "md" || extension == "txt" {
                        stats.doc_files += 1;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Register a project with CI
    pub fn register_project(project_path: &Path, ci_repo_path: &Path) -> Result<()> {
        // Ensure we have a valid project with CLAUDE.md
        if !Self::is_ci_project(project_path) {
            return Err(anyhow!("Not a CI project: CLAUDE.md not found"));
        }
        
        // Get project name
        let project_info = Self::get_project_info(project_path)?;
        let project_name = project_info.name.clone();
        
        // Create project registry directory if it doesn't exist
        let projects_dir = ci_repo_path.join("Projects");
        if !projects_dir.exists() {
            fs::create_dir_all(&projects_dir)
                .with_context(|| format!("Failed to create Projects directory at {}", projects_dir.display()))?;
        }
        
        // Create project directory link
        let project_link_dir = projects_dir.join(&project_name);
        
        // Update if exists, otherwise create
        if project_link_dir.exists() {
            // Remove existing symlink or directory
            if project_link_dir.is_symlink() {
                fs::remove_file(&project_link_dir)
                    .with_context(|| format!("Failed to remove existing symlink at {}", project_link_dir.display()))?;
            } else {
                fs::remove_dir_all(&project_link_dir)
                    .with_context(|| format!("Failed to remove existing directory at {}", project_link_dir.display()))?;
            }
        }
        
        // Create symlink on Unix or directory with metadata on Windows
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(project_path, &project_link_dir)
                .with_context(|| format!("Failed to create symlink from {} to {}", 
                    project_path.display(), project_link_dir.display()))?;
        }
        
        #[cfg(windows)]
        {
            // On Windows, create a directory + metadata file instead of a symlink
            fs::create_dir(&project_link_dir)
                .with_context(|| format!("Failed to create project link directory at {}", project_link_dir.display()))?;
                
            // Create .project-path file with the actual path
            let path_file = project_link_dir.join(".project-path");
            fs::write(&path_file, project_path.to_string_lossy().as_ref())
                .with_context(|| format!("Failed to write project path file at {}", path_file.display()))?;
                
            // Copy CLAUDE.md
            let source_claude = project_path.join("CLAUDE.md");
            let dest_claude = project_link_dir.join("CLAUDE.md");
            fs::copy(&source_claude, &dest_claude)
                .with_context(|| format!("Failed to copy CLAUDE.md from {} to {}", 
                    source_claude.display(), dest_claude.display()))?;
        }
        
        Ok(())
    }
    
    /// List all registered projects
    pub fn list_registered_projects(ci_repo_path: &Path) -> Result<Vec<ProjectInfo>> {
        let projects_dir = ci_repo_path.join("Projects");
        if !projects_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut projects = Vec::new();
        
        // Read Projects directory
        let entries = fs::read_dir(&projects_dir)
            .with_context(|| format!("Failed to read Projects directory at {}", projects_dir.display()))?;
            
        for entry in entries {
            let entry = entry
                .with_context(|| format!("Failed to read directory entry in {}", projects_dir.display()))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Get project path (actual path, not the link)
                let project_path = if path.is_symlink() {
                    // Read symlink target on Unix
                    fs::read_link(&path)
                        .with_context(|| format!("Failed to read symlink at {}", path.display()))?
                } else {
                    // Read .project-path file on Windows
                    let path_file = path.join(".project-path");
                    if path_file.exists() {
                        let path_content = fs::read_to_string(&path_file)
                            .with_context(|| format!("Failed to read project path file at {}", path_file.display()))?;
                        PathBuf::from(path_content)
                    } else {
                        // Just use the directory itself if no .project-path file
                        path.clone()
                    }
                };
                
                // Try to get project info
                match Self::get_project_info(&project_path) {
                    Ok(info) => projects.push(info),
                    Err(_) => {
                        // Not a valid project, could add warning if desired
                    }
                }
            }
        }
        
        Ok(projects)
    }
    
    /// Detect project type based on files and structure
    pub fn detect_project_type(path: &Path) -> Result<ProjectType> {
        // Check for common project type identifiers
        if path.join("Cargo.toml").exists() {
            return Ok(ProjectType::Rust);
        } else if path.join("package.json").exists() {
            // Distinguish between Node, React, etc.
            let package_json = fs::read_to_string(path.join("package.json"))
                .with_context(|| "Failed to read package.json")?;
                
            let package_data: serde_json::Value = serde_json::from_str(&package_json)
                .with_context(|| "Failed to parse package.json")?;
                
            if let Some(deps) = package_data.get("dependencies") {
                if deps.get("react").is_some() {
                    return Ok(ProjectType::React);
                } else if deps.get("vue").is_some() {
                    return Ok(ProjectType::Vue);
                } else if deps.get("@angular/core").is_some() {
                    return Ok(ProjectType::Angular);
                }
            }
            
            return Ok(ProjectType::Node);
        } else if path.join("go.mod").exists() {
            return Ok(ProjectType::Go);
        } else if path.join("requirements.txt").exists() || path.join("setup.py").exists() || path.join("Pipfile").exists() {
            return Ok(ProjectType::Python);
        } else if path.join("composer.json").exists() {
            return Ok(ProjectType::PHP);
        } else if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            return Ok(ProjectType::Java);
        } else if path.join("Gemfile").exists() {
            return Ok(ProjectType::Ruby);
        } else if path.join("CMakeLists.txt").exists() {
            return Ok(ProjectType::CPP);
        }
        
        // Count file types to make an educated guess
        let extensions = Self::count_file_extensions(path)?;
        
        // Determine based on most common extension
        let most_common = extensions.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(ext, _)| ext.as_str());
            
        match most_common {
            Some("rs") => Ok(ProjectType::Rust),
            Some("js") | Some("ts") => Ok(ProjectType::Node),
            Some("py") => Ok(ProjectType::Python),
            Some("go") => Ok(ProjectType::Go),
            Some("java") => Ok(ProjectType::Java),
            Some("rb") => Ok(ProjectType::Ruby),
            Some("php") => Ok(ProjectType::PHP),
            Some("cpp") | Some("cc") | Some("h") | Some("c") => Ok(ProjectType::CPP),
            _ => Ok(ProjectType::Unknown),
        }
    }
    
    /// Count file extensions in a project
    fn count_file_extensions(dir: &Path) -> Result<HashMap<String, usize>> {
        let mut extensions = HashMap::new();
        Self::count_file_extensions_internal(dir, &mut extensions)?;
        Ok(extensions)
    }
    
    /// Helper function for recursive extension counting
    fn count_file_extensions_internal(dir: &Path, extensions: &mut HashMap<String, usize>) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }
        
        // Skip common directories to exclude
        let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if dir_name == ".git" || dir_name == "node_modules" || dir_name == "target" || 
           dir_name == "dist" || dir_name == "build" || dir_name == ".idea" {
            return Ok(());
        }
        
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory at {}", dir.display()))?;
            
        for entry in entries {
            let entry = entry
                .with_context(|| format!("Failed to read directory entry in {}", dir.display()))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    *extensions.entry(ext_str.to_string()).or_insert(0) += 1;
                }
            } else if path.is_dir() {
                Self::count_file_extensions_internal(&path, extensions)?;
            }
        }
        
        Ok(())
    }
    
    /// Generate project scaffolding based on project type
    pub fn generate_scaffold(path: &Path, project_type: ProjectType, project_name: &str) -> Result<()> {
        // Create base directories
        fs::create_dir_all(path.join("src"))
            .with_context(|| "Failed to create src directory")?;
        fs::create_dir_all(path.join("docs"))
            .with_context(|| "Failed to create docs directory")?;
        
        // Create README.md
        let readme_content = format!("# {}\n\nA Collaborative Intelligence project.\n", project_name);
        fs::write(path.join("README.md"), readme_content)
            .with_context(|| "Failed to create README.md")?;
        
        // Create type-specific files
        match project_type {
            ProjectType::Rust => {
                // Create Cargo.toml
                let cargo_content = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_name);
                fs::write(path.join("Cargo.toml"), cargo_content)
                    .with_context(|| "Failed to create Cargo.toml")?;
                
                // Create src/main.rs
                let main_content = r#"fn main() {
    println!("Hello, world!");
}
"#;
                fs::write(path.join("src/main.rs"), main_content)
                    .with_context(|| "Failed to create src/main.rs")?;
            },
            ProjectType::Node => {
                // Create package.json
                let package_content = format!(r#"{{
  "name": "{}",
  "version": "0.1.0",
  "description": "A Collaborative Intelligence project",
  "main": "src/index.js",
  "scripts": {{
    "start": "node src/index.js",
    "test": "echo \"Error: no test specified\" && exit 1"
  }},
  "keywords": [],
  "author": "",
  "license": "MIT"
}}
"#, project_name.to_lowercase().replace(" ", "-"));
                fs::write(path.join("package.json"), package_content)
                    .with_context(|| "Failed to create package.json")?;
                
                // Create src/index.js
                let index_content = r#"console.log('Hello, world!');
"#;
                fs::write(path.join("src/index.js"), index_content)
                    .with_context(|| "Failed to create src/index.js")?;
            },
            ProjectType::Python => {
                // Create setup.py
                let setup_content = format!(r#"from setuptools import setup, find_packages

setup(
    name="{}",
    version="0.1.0",
    packages=find_packages(),
)
"#, project_name.to_lowercase().replace(" ", "_"));
                fs::write(path.join("setup.py"), setup_content)
                    .with_context(|| "Failed to create setup.py")?;
                
                // Create src/__init__.py
                fs::create_dir_all(path.join("src"))
                    .with_context(|| "Failed to create src directory")?;
                fs::write(path.join("src/__init__.py"), "")
                    .with_context(|| "Failed to create src/__init__.py")?;
                
                // Create src/main.py
                let main_content = r#"def main():
    print("Hello, world!")

if __name__ == "__main__":
    main()
"#;
                fs::write(path.join("src/main.py"), main_content)
                    .with_context(|| "Failed to create src/main.py")?;
            },
            _ => {
                // For other types, just create a minimal structure
                fs::write(path.join("src/main.txt"), "Hello, world!")
                    .with_context(|| "Failed to create src/main.txt")?;
            }
        }
        
        // Create .gitignore
        let gitignore_content = match project_type {
            ProjectType::Rust => r#"# Generated by Cargo
/target/
Cargo.lock

# Remove Cargo.lock from gitignore if creating an executable
# Instead of a library

# These are backup files generated by rustfmt
**/*.rs.bk

# MSVC Windows builds of rustc generate these
*.pdb
"#,
            ProjectType::Node => r#"# Logs
logs
*.log
npm-debug.log*

# Dependency directories
node_modules/

# Build output
dist/
build/

# Environment variables
.env
.env.local
"#,
            ProjectType::Python => r#"# Byte-compiled / optimized / DLL files
__pycache__/
*.py[cod]
*$py.class

# Distribution / packaging
dist/
build/
*.egg-info/

# Unit test / coverage reports
htmlcov/
.coverage
"#,
            _ => r#"# OS files
.DS_Store
Thumbs.db

# Editor directories and files
.idea/
.vscode/
*.suo
*.ntvs*
*.njsproj
*.sln
*.sw?

# Build output
dist/
build/
out/
"#,
        };
        
        fs::write(path.join(".gitignore"), gitignore_content)
            .with_context(|| "Failed to create .gitignore")?;
            
        Ok(())
    }
}

/// Structure to hold project information
#[derive(Clone, Debug)]
pub struct ProjectInfo {
    pub name: String,
    pub integration_type: String,
    pub agents: Vec<String>,
    pub created: Option<String>,
    pub config_path: PathBuf,
}

/// Structure to hold project statistics
#[derive(Default, Debug)]
pub struct ProjectStats {
    pub file_count: usize,
    pub total_size: usize,
    pub rust_files: usize,
    pub js_files: usize,
    pub python_files: usize,
    pub doc_files: usize,
    pub file_types: HashMap<String, usize>,
}

/// Enum representing different project types
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Rust,
    Node,
    React,
    Vue,
    Angular,
    Python,
    Go,
    Java,
    Ruby,
    PHP,
    CPP,
    Unknown,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::Node => write!(f, "Node.js"),
            ProjectType::React => write!(f, "React"),
            ProjectType::Vue => write!(f, "Vue.js"),
            ProjectType::Angular => write!(f, "Angular"),
            ProjectType::Python => write!(f, "Python"),
            ProjectType::Go => write!(f, "Go"),
            ProjectType::Java => write!(f, "Java"),
            ProjectType::Ruby => write!(f, "Ruby"),
            ProjectType::PHP => write!(f, "PHP"),
            ProjectType::CPP => write!(f, "C++"),
            ProjectType::Unknown => write!(f, "Unknown"),
        }
    }
}