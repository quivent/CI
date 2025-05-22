//! Command generation tool for instant command creation
//!
//! This module provides tooling for rapid command generation using templates,
//! supporting the `CI:[command-name]` pattern for command creation.

use std::fs;
use std::path::Path;
use anyhow::{Result, Context, anyhow};
use colored::Colorize;

/// Generates a new command implementation from a template
pub fn generate_command(name: &str, description: &str, category: &str) -> Result<()> {
    // Normalize the command name
    let command_name = name.to_lowercase().replace("-", "_");
    
    // Determine target file based on category
    let target_file = match category.to_lowercase().as_str() {
        "intelligence" | "intelligence & discovery" => "src/commands/intelligence.rs",
        "source" | "source control" => "src/commands/source_control.rs",
        "lifecycle" | "project lifecycle" => "src/commands/lifecycle.rs",
        "system" | "system management" => "src/commands/system.rs",
        _ => return Err(anyhow!("Invalid category: {}", category)),
    };
    
    // Check if the file exists
    if !Path::new(target_file).exists() {
        return Err(anyhow!("Target file not found: {}", target_file));
    }
    
    // Read current file content
    let current_content = fs::read_to_string(target_file)
        .with_context(|| format!("Failed to read {}", target_file))?;
    
    // Check if command already exists
    if current_content.contains(&format!("pub async fn {}(", command_name)) {
        return Err(anyhow!("Command '{}' already exists", command_name));
    }
    
    // Generate new function implementation from template
    let template_path = format!("src/tools/templates/{}.rs.tpl", 
        target_file.strip_prefix("src/commands/").unwrap().strip_suffix(".rs").unwrap());
    
    let template = fs::read_to_string(&template_path).unwrap_or_else(|_| get_default_template(category));
    
    let function_implementation = template
        .replace("{{name}}", &command_name)
        .replace("{{description}}", description)
        .replace("{{category}}", get_category_name(category))
        .replace("{{emoji}}", get_category_emoji(category))
        .replace("{{color}}", get_category_color(category));
    
    // Add function to file before the last closing brace
    let mut content = current_content.clone();
    let insert_position = content.rfind('}').unwrap_or(content.len());
    content.insert_str(insert_position, &format!("\n\n{}\n", function_implementation));
    
    // Write updated file
    fs::write(target_file, content)
        .with_context(|| format!("Failed to write {}", target_file))?;
    
    // Update main.rs to add command to the Commands enum
    update_main_rs(name, description, category)?;
    
    // Create documentation file if docs directory exists
    if Path::new("docs").exists() {
        create_documentation(name, description, category)?;
    }
    
    println!("{} Command '{}' created successfully in category '{}'", "✓".green(), name.cyan(), category.green());
    println!("  Added to {}", target_file.yellow());
    println!("  Updated main.rs with new command");
    
    Ok(())
}

/// Updates main.rs to add the new command
fn update_main_rs(name: &str, description: &str, category: &str) -> Result<()> {
    let main_path = "src/main.rs";
    let main_content = fs::read_to_string(main_path)
        .with_context(|| format!("Failed to read {}", main_path))?;
    
    // Convert command name to proper case formats
    let command_name = name.to_lowercase().replace("-", "_");
    let command_enum_name = to_pascal_case(&command_name);
    
    // Find the Commands enum
    let commands_enum_start = main_content.find("enum Commands {")
        .ok_or_else(|| anyhow!("Commands enum not found in main.rs"))?;
    
    // Find the appropriate category section
    let category_marker = match category.to_lowercase().as_str() {
        "intelligence" | "intelligence & discovery" => "// Intelligence & Discovery Commands",
        "source" | "source control" => "// Source Control Commands",
        "lifecycle" | "project lifecycle" => "// Project Lifecycle Commands",
        "system" | "system management" => "// System Management Commands",
        _ => "// Intelligence & Discovery Commands", // Default category
    };
    
    let category_start = main_content[commands_enum_start..].find(category_marker)
        .ok_or_else(|| anyhow!("Category section not found in Commands enum"))?;
    
    let next_category_markers = [
        "// Source Control Commands",
        "// Project Lifecycle Commands", 
        "// System Management Commands",
        "}", // End of enum
    ];
    
    // Find the end of the current category section
    let mut category_end = main_content.len();
    for marker in next_category_markers {
        if let Some(pos) = main_content[commands_enum_start + category_start..].find(marker) {
            if commands_enum_start + category_start + pos < category_end {
                category_end = commands_enum_start + category_start + pos;
            }
        }
    }
    
    // Create the new command entry
    let color_code = match get_category_color(category) {
        "blue" => "blue",
        "green" => "green",
        "yellow" => "yellow",
        "cyan" => "cyan",
        _ => "white",
    };
    
    let command_entry = format!(r#"
    /// {}
    #[command(about = format!("{{}}", "{}".{}()))]
    {},"#, 
        description,
        description,
        color_code,
        command_enum_name
    );
    
    // Insert the command entry
    let mut new_content = main_content.clone();
    new_content.insert_str(category_end, &command_entry);
    
    // Now find the match statement to add the command handler
    let match_start = new_content.find("match cli.command.unwrap() {")
        .ok_or_else(|| anyhow!("Match statement not found in main.rs"))?;
    
    let category_match_marker = match category.to_lowercase().as_str() {
        "intelligence" | "intelligence & discovery" => "// Intelligence & Discovery Commands",
        "source" | "source control" => "// Source Control Commands",
        "lifecycle" | "project lifecycle" => "// Project Lifecycle Commands",
        "system" | "system management" => "// System Management Commands",
        _ => "// Intelligence & Discovery Commands", // Default category
    };
    
    let category_match_start = new_content[match_start..].find(category_match_marker)
        .ok_or_else(|| anyhow!("Category section not found in match statement"))?;
    
    // Find the end of the current category section
    let mut category_match_end = new_content.len();
    for marker in next_category_markers {
        if let Some(pos) = new_content[match_start + category_match_start..].find(marker) {
            if match_start + category_match_start + pos < category_match_end {
                category_match_end = match_start + category_match_start + pos;
            }
        }
    }
    
    // Create the match handler
    let match_handler = format!(r#"
        Commands::{} => {{
            commands::{}::{}(&config).await
        }},"#,
        command_enum_name,
        get_module_name(category),
        command_name
    );
    
    // Insert the match handler
    new_content.insert_str(category_match_end, &match_handler);
    
    // Write the updated file
    fs::write(main_path, new_content)
        .with_context(|| format!("Failed to write {}", main_path))?;
    
    Ok(())
}

/// Creates documentation for the new command
fn create_documentation(name: &str, description: &str, category: &str) -> Result<()> {
    let docs_dir = match category.to_lowercase().as_str() {
        "intelligence" | "intelligence & discovery" => "docs/commands/intelligence",
        "source" | "source control" => "docs/commands/source_control",
        "lifecycle" | "project lifecycle" => "docs/commands/lifecycle",
        "system" | "system management" => "docs/commands/system",
        _ => "docs/commands/intelligence", // Default category
    };
    
    // Create directory if it doesn't exist
    if !Path::new(docs_dir).exists() {
        fs::create_dir_all(docs_dir)
            .with_context(|| format!("Failed to create documentation directory: {}", docs_dir))?;
    }
    
    let doc_path = format!("{}/{}.md", docs_dir, name.to_lowercase());
    let doc_content = format!(r#"# {} Command

{}

## Usage

```
ci {} [OPTIONS]
```

## Description

{}

## Examples

```bash
# Basic usage
ci {}
```

## Options

*No command-specific options*

## Related Commands

*Add related commands here*
"#, 
        name.to_uppercase(),
        description,
        name.to_lowercase(),
        description,
        name.to_lowercase()
    );
    
    fs::write(&doc_path, doc_content)
        .with_context(|| format!("Failed to write documentation file: {}", doc_path))?;
    
    Ok(())
}

/// Gets the canonical category name
fn get_category_name(category: &str) -> &'static str {
    match category.to_lowercase().as_str() {
        "intelligence" | "intelligence & discovery" => "Intelligence & Discovery",
        "source" | "source control" => "Source Control",
        "lifecycle" | "project lifecycle" => "Project Lifecycle",
        "system" | "system management" => "System Management",
        _ => "Intelligence & Discovery", // Default category
    }
}

/// Gets the module name for a category
fn get_module_name(category: &str) -> &'static str {
    match category.to_lowercase().as_str() {
        "intelligence" | "intelligence & discovery" => "intelligence",
        "source" | "source control" => "source_control",
        "lifecycle" | "project lifecycle" => "lifecycle",
        "system" | "system management" => "system",
        _ => "intelligence", // Default category
    }
}

/// Gets the emoji for a category
fn get_category_emoji(category: &str) -> &'static str {
    match category.to_lowercase().as_str() {
        "intelligence" | "intelligence & discovery" => "🧠",
        "source" | "source control" => "📊",
        "lifecycle" | "project lifecycle" => "🚀",
        "system" | "system management" => "⚙️",
        _ => "📌",
    }
}

/// Gets the color for a category
fn get_category_color(category: &str) -> &'static str {
    match category.to_lowercase().as_str() {
        "intelligence" | "intelligence & discovery" => "blue",
        "source" | "source control" => "green",
        "lifecycle" | "project lifecycle" => "yellow",
        "system" | "system management" => "cyan",
        _ => "white",
    }
}

/// Converts a snake_case string to PascalCase
fn to_pascal_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in input.chars() {
        if c == '_' {
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

/// Gets a default template for a category
fn get_default_template(category: &str) -> String {
    let color = get_category_color(category);
    let emoji = get_category_emoji(category);
    let category_name = get_category_name(category);
    
    format!(r#"/// {{{{description}}}}
pub async fn {{{{name}}}}(_config: &Config) -> Result<()> {{
    CommandHelpers::print_command_header(
        "{{{{description}}}}", 
        "{}", 
        "{}", 
        "{}"
    );
    
    // Implementation goes here
    CommandHelpers::print_info("Command implementation pending");
    
    CommandHelpers::print_success("Command completed successfully");
    
    Ok(())
}}"#, emoji, category_name, color)
}

/// Parses and processes the instant command pattern
pub fn process_instant_command(input: &str) -> Result<()> {
    // Check if the input matches the pattern CI:[command-name]
    if !input.starts_with("CI:") {
        return Err(anyhow!("Invalid instant command format. Expected format: CI:[command-name] or CI:[command-name] [description]"));
    }
    
    // Extract command name and description
    let parts: Vec<&str> = input.trim_start_matches("CI:").trim().splitn(2, ' ').collect();
    let command_name = parts[0].trim();
    
    if command_name.is_empty() {
        return Err(anyhow!("Command name cannot be empty"));
    }
    
    // Get or prompt for description
    let description = if parts.len() > 1 && !parts[1].trim().is_empty() {
        parts[1].trim().to_string()
    } else {
        // Prompt for description
        println!("Enter description for command '{}': ", command_name);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .with_context(|| "Failed to read description from stdin")?;
        input.trim().to_string()
    };
    
    if description.is_empty() {
        return Err(anyhow!("Command description cannot be empty"));
    }
    
    // Auto-categorize the command
    let category = auto_categorize_command(command_name, &description);
    
    // Generate the command
    generate_command(command_name, &description, &category)?;
    
    Ok(())
}

/// Automatically categorize a command based on name and description
fn auto_categorize_command(name: &str, description: &str) -> String {
    let combined = format!("{} {}", name, description).to_lowercase();
    
    // Source Control category keywords
    let source_keywords = [
        "git", "commit", "repo", "repository", "branch", "merge", "pull", "push",
        "stage", "unstage", "remote", "origin", "status", "diff", "log", "github",
        "clone", "fork", "tag", "rebase", "stash", "checkout", "fetch", "ignore",
    ];
    
    // Project Lifecycle keywords
    let lifecycle_keywords = [
        "project", "init", "initialize", "create", "new", "start", "scaffold",
        "integrate", "setup", "configure", "installation", "template", "boilerplate",
        "verify", "validation", "check", "fix", "repair", "transform", "migrate",
    ];
    
    // System Management keywords
    let system_keywords = [
        "system", "install", "uninstall", "link", "unlink", "build", "compile",
        "update", "upgrade", "downgrade", "config", "configuration", "settings",
        "environment", "env", "variable", "path", "version", "release", "binary",
        "library", "dependency", "package", "module", "plugin", "extension", "api",
        "key", "credential", "token", "secret", "certificate", "permission", "access",
    ];
    
    // Intelligence & Discovery keywords (default if no matches)
    let intelligence_keywords = [
        "ai", "agent", "intelligence", "discover", "explore", "search", "find",
        "analyze", "insight", "context", "learn", "train", "model", "predict",
        "recommend", "suggest", "display", "show", "list", "query", "info", "about",
        "help", "usage", "manual", "guide", "tutorial", "example", "detail", "document",
    ];
    
    // Check matches for each category
    let mut matches = [
        ("Source Control", source_keywords.iter().filter(|&&k| combined.contains(k)).count()),
        ("Project Lifecycle", lifecycle_keywords.iter().filter(|&&k| combined.contains(k)).count()),
        ("System Management", system_keywords.iter().filter(|&&k| combined.contains(k)).count()),
        ("Intelligence & Discovery", intelligence_keywords.iter().filter(|&&k| combined.contains(k)).count()),
    ];
    
    // Sort by number of matches
    matches.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Return the category with the most matches
    matches[0].0.to_string()
}