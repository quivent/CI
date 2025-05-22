//! CI Directive Processor for CI
//!
//! This module provides functionality to process CI directives embedded in markdown files.
//! It allows for backward compatibility with the original CI tool while providing
//! standalone functionality in CI.

use anyhow::{Result, Context, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use colored::*;

/// CI directive marker patterns
#[allow(dead_code)]
const CI_DIRECTIVE_START: &str = "_CI.";
#[allow(dead_code)]
const CI_DIRECTIVE_END: &str = "_";

/// DirectiveProcessor handles the parsing and execution of CI directives in markdown files
pub struct DirectiveProcessor {
    /// Base path for resolving relative file references
    base_path: PathBuf,
    
    /// Environment variables available to directives
    env_vars: HashMap<String, String>,
    
    /// Stack of files being processed to prevent circular references
    file_stack: Vec<PathBuf>,
    
    /// Flag to enable verbose output
    verbose: bool,
}

/// Represents a CI directive with its command and arguments
#[derive(Debug, Clone)]
struct Directive {
    /// The command name (e.g., 'load', 'return_to', 'exec')
    command: String,
    
    /// The arguments for the command
    args: Vec<String>,
    
    /// The original text of the directive
    original: String,
}

impl DirectiveProcessor {
    /// Create a new DirectiveProcessor with the specified base path
    pub fn new(base_path: &Path) -> Self {
        DirectiveProcessor {
            base_path: base_path.to_path_buf(),
            env_vars: HashMap::new(),
            file_stack: Vec::new(),
            verbose: false,
        }
    }
    
    /// Enable or disable verbose output
    pub fn set_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }
    
    /// Set an environment variable for directive processing
    pub fn set_env(&mut self, key: &str, value: &str) -> &mut Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Process a markdown file and execute any CI directives
    pub fn process_file(&mut self, file_path: &Path) -> Result<String> {
        // Check for circular references
        if self.file_stack.contains(&file_path.to_path_buf()) {
            return Err(anyhow!("Circular reference detected: {}", file_path.display()));
        }
        
        // Push file onto stack
        self.file_stack.push(file_path.to_path_buf());
        
        // Read file content
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        
        // Process the content
        let processed = self.process_content(&content, file_path)?;
        
        // Pop file from stack
        self.file_stack.pop();
        
        Ok(processed)
    }
    
    /// Process a string containing markdown content with potential CI directives
    pub fn process_content(&mut self, content: &str, current_file: &Path) -> Result<String> {
        let mut result = String::new();
        let mut current_pos = 0;
        
        // Find all directives in the content
        for directive in self.find_directives(content) {
            // Add text before the directive
            let directive_start = content.find(&directive.original).unwrap_or(current_pos);
            result.push_str(&content[current_pos..directive_start]);
            
            // Process the directive
            match self.execute_directive(&directive, current_file) {
                Ok(processed) => {
                    // Add the processed directive result
                    result.push_str(&processed);
                },
                Err(e) => {
                    // On error, add a comment about the error
                    result.push_str(&format!("<!-- CI Directive Error: {} -->", e));
                    // Also include the original directive
                    result.push_str(&directive.original);
                    
                    if self.verbose {
                        eprintln!("{} Error processing directive '{}': {}", 
                            "WARNING:".yellow().bold(), 
                            directive.original, 
                            e);
                    }
                }
            }
            
            // Update current position
            current_pos = directive_start + directive.original.len();
        }
        
        // Add any remaining text
        result.push_str(&content[current_pos..]);
        
        Ok(result)
    }
    
    /// Find all CI directives in a content string
    fn find_directives(&self, content: &str) -> Vec<Directive> {
        lazy_static! {
            // Regex to match CI directives: _CI.command('arg1', 'arg2', ...)_
            static ref DIRECTIVE_REGEX: Regex = Regex::new(
                r"_CI\.([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*?)\)_"
            ).unwrap();
            
            // Regex to match arguments in single or double quotes
            static ref ARG_REGEX: Regex = Regex::new(
                r#"'([^']*)'|"([^"]*)""#
            ).unwrap();
        }
        
        let mut directives = Vec::new();
        
        for cap in DIRECTIVE_REGEX.captures_iter(content) {
            let command = cap.get(1).map_or("", |m| m.as_str()).to_string();
            let args_str = cap.get(2).map_or("", |m| m.as_str());
            let original = cap.get(0).map_or("", |m| m.as_str()).to_string();
            
            // Parse arguments
            let mut args = Vec::new();
            for arg_cap in ARG_REGEX.captures_iter(args_str) {
                if let Some(arg) = arg_cap.get(1).or_else(|| arg_cap.get(2)) {
                    args.push(arg.as_str().to_string());
                }
            }
            
            directives.push(Directive {
                command,
                args,
                original,
            });
        }
        
        directives
    }
    
    /// Execute a CI directive and return the processed result
    fn execute_directive(&mut self, directive: &Directive, current_file: &Path) -> Result<String> {
        match directive.command.as_str() {
            "load" => self.execute_load_directive(directive, current_file),
            "return_to" => self.execute_return_directive(directive, current_file),
            "include" => self.execute_include_directive(directive, current_file),
            "env" => self.execute_env_directive(directive, current_file),
            "exec" => self.execute_exec_directive(directive, current_file),
            "agent" => self.execute_agent_directive(directive, current_file),
            _ => Err(anyhow!("Unknown directive command: {}", directive.command)),
        }
    }
    
    /// Execute a 'load' directive to load and process another file
    fn execute_load_directive(&mut self, directive: &Directive, current_file: &Path) -> Result<String> {
        if directive.args.is_empty() {
            return Err(anyhow!("load directive requires a file path argument"));
        }
        
        let file_path = self.resolve_file_path(&directive.args[0], current_file)?;
        
        if self.verbose {
            println!("Loading file: {}", file_path.display());
        }
        
        // Process the file
        self.process_file(&file_path)
    }
    
    /// Execute a 'return_to' directive to return to a previous file
    fn execute_return_directive(&mut self, directive: &Directive, _current_file: &Path) -> Result<String> {
        if directive.args.is_empty() {
            return Err(anyhow!("return_to directive requires a file path argument"));
        }
        
        // This is mostly a no-op directive that just provides a marker
        // We'll replace it with a comment to indicate the return point
        Ok(format!("<!-- Returned to {} -->", directive.args[0]))
    }
    
    /// Execute an 'include' directive to include content from another file
    fn execute_include_directive(&mut self, directive: &Directive, current_file: &Path) -> Result<String> {
        if directive.args.is_empty() {
            return Err(anyhow!("include directive requires a file path argument"));
        }
        
        let file_path = self.resolve_file_path(&directive.args[0], current_file)?;
        
        if self.verbose {
            println!("Including file: {}", file_path.display());
        }
        
        // Read file but don't process directives in it
        fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read included file: {}", file_path.display()))
    }
    
    /// Execute an 'env' directive to set or get environment variables
    fn execute_env_directive(&mut self, directive: &Directive, _current_file: &Path) -> Result<String> {
        if directive.args.is_empty() {
            return Err(anyhow!("env directive requires at least one argument"));
        }
        
        // If two arguments, set an environment variable
        if directive.args.len() >= 2 {
            let key = &directive.args[0];
            let value = &directive.args[1];
            
            self.env_vars.insert(key.clone(), value.clone());
            
            // Return empty string for set operations
            return Ok(String::new());
        }
        
        // If one argument, get environment variable
        let key = &directive.args[0];
        
        // Look up the variable in our map, then fallback to environment
        let value = self.env_vars.get(key)
            .cloned()
            .or_else(|| std::env::var(key).ok())
            .unwrap_or_else(|| String::new());
            
        Ok(value)
    }
    
    /// Execute an 'exec' directive to run a shell command
    fn execute_exec_directive(&mut self, directive: &Directive, _current_file: &Path) -> Result<String> {
        if directive.args.is_empty() {
            return Err(anyhow!("exec directive requires a command argument"));
        }
        
        let command = &directive.args[0];
        
        // For security, we limit which commands can be executed
        // This implementation allows only specific commands
        let allowed_commands = vec!["echo", "date", "hostname", "uname", "whoami"];
        
        // Extract the command name (before any space)
        let cmd_name = command.split_whitespace().next().unwrap_or("");
        
        if !allowed_commands.contains(&cmd_name) {
            return Err(anyhow!("Command not allowed: {}", cmd_name));
        }
        
        if self.verbose {
            println!("Executing command: {}", command);
        }
        
        // Run the command and capture output
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .with_context(|| format!("Failed to execute command: {}", command))?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Command failed: {}", error));
        }
        
        // Return the command output
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(result)
    }
    
    /// Execute an 'agent' directive to load agent configuration
    fn execute_agent_directive(&mut self, directive: &Directive, _current_file: &Path) -> Result<String> {
        if directive.args.is_empty() {
            return Err(anyhow!("agent directive requires an agent name argument"));
        }
        
        let agent_name = &directive.args[0];
        
        // Look for agent file in AGENTS directory
        let agents_dir = self.base_path.join("AGENTS");
        let agent_file = agents_dir.join(format!("{}.md", agent_name));
        
        if !agent_file.exists() {
            return Err(anyhow!("Agent file not found: {}", agent_file.display()));
        }
        
        if self.verbose {
            println!("Loading agent: {}", agent_name);
        }
        
        // Process the agent file
        self.process_file(&agent_file)
    }
    
    /// Resolve a file path relative to the current file or base path
    fn resolve_file_path(&self, file_path: &str, current_file: &Path) -> Result<PathBuf> {
        // If the path is absolute, use it directly
        let path = PathBuf::from(file_path);
        if path.is_absolute() {
            return Ok(path);
        }
        
        // If starts with ./ or ../, resolve relative to current file
        if file_path.starts_with("./") || file_path.starts_with("../") {
            let parent = current_file.parent()
                .ok_or_else(|| anyhow!("Failed to get parent directory of {}", current_file.display()))?;
            return Ok(parent.join(file_path));
        }
        
        // Otherwise, resolve relative to base path
        Ok(self.base_path.join(file_path))
    }
}

/// Process a file with CI directives in standalone mode
pub fn process_file_standalone(file_path: &Path) -> Result<String> {
    let base_path = file_path.parent()
        .ok_or_else(|| anyhow!("Failed to get parent directory of {}", file_path.display()))?
        .to_path_buf();
        
    let mut processor = DirectiveProcessor::new(&base_path);
    processor.process_file(file_path)
}

/// Process content with CI directives in standalone mode
pub fn process_content_standalone(content: &str, base_path: &Path) -> Result<String> {
    let mut processor = DirectiveProcessor::new(base_path);
    processor.process_content(content, &base_path.join("CLAUDE.md"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_find_directives() {
        let processor = DirectiveProcessor::new(Path::new("."));
        
        let content = r#"# Test File
        
This is a test file with some CI directives:

_CI.load('other_file.md')_

And another one:

_CI.env('PROJECT_NAME')_

And a third one with multiple arguments:

_CI.exec('echo "Hello, World!"')_
"#;

        let directives = processor.find_directives(content);
        
        assert_eq!(directives.len(), 3);
        assert_eq!(directives[0].command, "load");
        assert_eq!(directives[0].args, vec!["other_file.md"]);
        assert_eq!(directives[1].command, "env");
        assert_eq!(directives[1].args, vec!["PROJECT_NAME"]);
        assert_eq!(directives[2].command, "exec");
        assert_eq!(directives[2].args, vec!["echo \"Hello, World!\""]);
    }
    
    #[test]
    fn test_env_directive() {
        let temp_dir = tempdir().unwrap();
        let mut processor = DirectiveProcessor::new(temp_dir.path());
        
        // Set an environment variable
        let set_directive = Directive {
            command: "env".to_string(),
            args: vec!["TEST_VAR".to_string(), "test_value".to_string()],
            original: "_CI.env('TEST_VAR', 'test_value')_".to_string(),
        };
        
        let result = processor.execute_env_directive(&set_directive, temp_dir.path().join("test.md").as_path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
        
        // Get the environment variable
        let get_directive = Directive {
            command: "env".to_string(),
            args: vec!["TEST_VAR".to_string()],
            original: "_CI.env('TEST_VAR')_".to_string(),
        };
        
        let result = processor.execute_env_directive(&get_directive, temp_dir.path().join("test.md").as_path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_value");
    }
    
    #[test]
    fn test_circular_reference_detection() {
        let temp_dir = tempdir().unwrap();
        let file_a = temp_dir.path().join("a.md");
        let file_b = temp_dir.path().join("b.md");
        
        // Create file A that loads file B
        fs::write(&file_a, "_CI.load('b.md')_").unwrap();
        
        // Create file B that loads file A
        fs::write(&file_b, "_CI.load('a.md')_").unwrap();
        
        let mut processor = DirectiveProcessor::new(temp_dir.path());
        let result = processor.process_file(&file_a);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular reference detected"));
    }
}