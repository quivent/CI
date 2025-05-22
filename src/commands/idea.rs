//! Idea Management commands for CI
//!
//! This module provides commands for managing ideas, concepts, and inspirations,
//! allowing users to capture, organize, and track creative thoughts.

use anyhow::{Result, Context, anyhow};
use chrono::{DateTime, Utc, Local};
use colored::*;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};
use uuid::Uuid;

use crate::config::Config;

/// Structure to represent an idea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idea {
    /// Unique identifier for the idea
    id: String,
    
    /// Title of the idea
    title: String,
    
    /// Detailed description of the idea
    description: String,
    
    /// Category or domain of the idea
    category: String,
    
    /// Tags or keywords associated with the idea
    tags: Vec<String>,
    
    /// Current status of the idea
    status: IdeaStatus,
    
    /// Priority level of the idea
    priority: IdeaPriority,
    
    /// Creation timestamp
    created_at: DateTime<Utc>,
    
    /// Last update timestamp
    updated_at: DateTime<Utc>,
    
    /// Related ideas (by ID)
    related_ideas: Vec<String>,
    
    /// Notes or additional information
    notes: String,
}

/// Enumeration of possible idea statuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdeaStatus {
    /// New idea, not yet evaluated
    New,
    
    /// Idea is being explored
    Exploring,
    
    /// Idea is in development
    InDevelopment,
    
    /// Idea is implemented
    Implemented,
    
    /// Idea is on hold
    OnHold,
    
    /// Idea is archived
    Archived,
    
    /// Idea is rejected
    Rejected,
}

/// Enumeration of possible idea priorities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdeaPriority {
    /// Low priority
    Low,
    
    /// Medium priority
    Medium,
    
    /// High priority
    High,
    
    /// Critical priority
    Critical,
}

/// Implementation for idea management
struct IdeaManager {
    /// Path to the ideas storage file
    ideas_file: PathBuf,
}

impl IdeaManager {
    /// Create a new idea manager
    fn new(_config: &Config) -> Result<Self> {
        // Determine ideas file location - prefer project-specific location if in a project
        let current_dir = std::env::current_dir()?;
        
        // Check if we're in a git repository
        let is_git_repo = std::process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(&current_dir)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
            
        let ideas_file = if is_git_repo {
            // Use project-specific ideas file
            current_dir.join(".ci").join("ideas.json")
        } else {
            // Use global ideas file
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("ci")
                .join("ideas.json")
        };
        
        // Ensure parent directory exists
        if let Some(parent) = ideas_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        Ok(Self { ideas_file })
    }
    
    /// List all ideas
    fn list_ideas(&self, filter: Option<&str>, category: Option<&str>, status: Option<IdeaStatus>) -> Result<Vec<Idea>> {
        let ideas = self.load_ideas()?;
        
        let filtered_ideas = ideas.into_iter()
            .filter(|idea| {
                // Apply filters if provided
                let matches_filter = filter.map_or(true, |f| {
                    idea.title.to_lowercase().contains(&f.to_lowercase()) ||
                    idea.description.to_lowercase().contains(&f.to_lowercase()) ||
                    idea.tags.iter().any(|tag| tag.to_lowercase().contains(&f.to_lowercase()))
                });
                
                let matches_category = category.map_or(true, |c| {
                    idea.category.to_lowercase() == c.to_lowercase()
                });
                
                let matches_status = status.map_or(true, |s| {
                    idea.status == s
                });
                
                matches_filter && matches_category && matches_status
            })
            .collect();
            
        Ok(filtered_ideas)
    }
    
    /// Add a new idea
    fn add_idea(&self, title: &str, description: &str, category: &str, tags: Vec<String>) -> Result<Idea> {
        let mut ideas = self.load_ideas()?;
        
        // Create new idea
        let now = Utc::now();
        let new_idea = Idea {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            tags,
            status: IdeaStatus::New,
            priority: IdeaPriority::Medium,
            created_at: now,
            updated_at: now,
            related_ideas: Vec::new(),
            notes: String::new(),
        };
        
        // Add to list
        ideas.push(new_idea.clone());
        
        // Save ideas
        self.save_ideas(&ideas)?;
        
        Ok(new_idea)
    }
    
    /// View a specific idea
    fn view_idea(&self, id: &str) -> Result<Idea> {
        let ideas = self.load_ideas()?;
        
        // Find the idea
        let idea = ideas.iter()
            .find(|i| i.id == id)
            .cloned()
            .ok_or_else(|| anyhow!("Idea not found with ID: {}", id))?;
            
        Ok(idea)
    }
    
    /// Update an existing idea
    fn update_idea(&self, id: &str, updates: IdeaUpdates) -> Result<Idea> {
        let mut ideas = self.load_ideas()?;
        
        // Find the idea
        let idea_index = ideas.iter()
            .position(|i| i.id == id)
            .ok_or_else(|| anyhow!("Idea not found with ID: {}", id))?;
            
        // Update the idea
        let mut idea = ideas[idea_index].clone();
        
        if let Some(title) = updates.title {
            idea.title = title;
        }
        
        if let Some(description) = updates.description {
            idea.description = description;
        }
        
        if let Some(category) = updates.category {
            idea.category = category;
        }
        
        if let Some(tags) = updates.tags {
            idea.tags = tags;
        }
        
        if let Some(status) = updates.status {
            idea.status = status;
        }
        
        if let Some(priority) = updates.priority {
            idea.priority = priority;
        }
        
        if let Some(related_ideas) = updates.related_ideas {
            idea.related_ideas = related_ideas;
        }
        
        if let Some(notes) = updates.notes {
            idea.notes = notes;
        }
        
        // Update timestamps
        idea.updated_at = Utc::now();
        
        // Replace in list
        ideas[idea_index] = idea.clone();
        
        // Save ideas
        self.save_ideas(&ideas)?;
        
        Ok(idea)
    }
    
    /// Delete an idea
    fn delete_idea(&self, id: &str) -> Result<()> {
        let mut ideas = self.load_ideas()?;
        
        // Find the idea
        let initial_count = ideas.len();
        ideas.retain(|i| i.id != id);
        
        if ideas.len() == initial_count {
            return Err(anyhow!("Idea not found with ID: {}", id));
        }
        
        // Save ideas
        self.save_ideas(&ideas)?;
        
        Ok(())
    }
    
    /// Load ideas from storage
    fn load_ideas(&self) -> Result<Vec<Idea>> {
        if !self.ideas_file.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.ideas_file)
            .with_context(|| format!("Failed to read ideas file: {}", self.ideas_file.display()))?;
            
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let ideas: Vec<Idea> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse ideas file")?;
            
        Ok(ideas)
    }
    
    /// Save ideas to storage
    fn save_ideas(&self, ideas: &[Idea]) -> Result<()> {
        let content = serde_json::to_string_pretty(ideas)
            .with_context(|| "Failed to serialize ideas")?;
            
        fs::write(&self.ideas_file, content)
            .with_context(|| format!("Failed to write ideas file: {}", self.ideas_file.display()))?;
            
        Ok(())
    }
    
    /// Get categories from existing ideas
    fn get_categories(&self) -> Result<Vec<String>> {
        let ideas = self.load_ideas()?;
        
        let mut categories: Vec<String> = ideas.iter()
            .map(|i| i.category.clone())
            .collect();
            
        // Remove duplicates
        categories.sort();
        categories.dedup();
        
        Ok(categories)
    }
    
    /// Get tags from existing ideas
    fn get_tags(&self) -> Result<Vec<String>> {
        let ideas = self.load_ideas()?;
        
        let mut tags: Vec<String> = ideas.iter()
            .flat_map(|i| i.tags.clone())
            .collect();
            
        // Remove duplicates
        tags.sort();
        tags.dedup();
        
        Ok(tags)
    }
}

/// Structure for idea updates
struct IdeaUpdates {
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    status: Option<IdeaStatus>,
    priority: Option<IdeaPriority>,
    related_ideas: Option<Vec<String>>,
    notes: Option<String>,
}

/// Helper functions to format idea data for display
mod formatters {
    use super::*;
    
    pub fn format_status(status: IdeaStatus) -> String {
        match status {
            IdeaStatus::New => "New".blue().to_string(),
            IdeaStatus::Exploring => "Exploring".cyan().to_string(),
            IdeaStatus::InDevelopment => "In Development".yellow().to_string(),
            IdeaStatus::Implemented => "Implemented".green().to_string(),
            IdeaStatus::OnHold => "On Hold".magenta().to_string(),
            IdeaStatus::Archived => "Archived".white().to_string(),
            IdeaStatus::Rejected => "Rejected".red().to_string(),
        }
    }
    
    pub fn format_priority(priority: IdeaPriority) -> String {
        match priority {
            IdeaPriority::Low => "Low".normal().to_string(),
            IdeaPriority::Medium => "Medium".yellow().to_string(),
            IdeaPriority::High => "High".red().to_string(),
            IdeaPriority::Critical => "Critical".red().bold().to_string(),
        }
    }
    
    pub fn format_date(date: DateTime<Utc>) -> String {
        let local_time: DateTime<Local> = date.into();
        local_time.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    pub fn format_idea_short(idea: &Idea) -> String {
        format!("{} [{}] {} - {}",
            idea.id[..8].blue(),
            format_status(idea.status),
            idea.title.white().bold(),
            idea.description.chars().take(50).collect::<String>()
        )
    }
    
    pub fn format_idea_detail(idea: &Idea) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("\n{} {}\n", "ID:".blue().bold(), idea.id));
        output.push_str(&format!("{} {}\n", "Title:".blue().bold(), idea.title.white().bold()));
        output.push_str(&format!("{} {}\n", "Category:".blue().bold(), idea.category.green()));
        output.push_str(&format!("{} {}\n", "Status:".blue().bold(), format_status(idea.status)));
        output.push_str(&format!("{} {}\n", "Priority:".blue().bold(), format_priority(idea.priority)));
        output.push_str(&format!("{} {}\n", "Created:".blue().bold(), format_date(idea.created_at)));
        output.push_str(&format!("{} {}\n", "Updated:".blue().bold(), format_date(idea.updated_at)));
        
        if !idea.tags.is_empty() {
            output.push_str(&format!("{} {}\n", "Tags:".blue().bold(), 
                idea.tags.iter().map(|t| t.yellow().to_string()).collect::<Vec<_>>().join(", ")
            ));
        }
        
        output.push_str(&format!("\n{}\n{}\n", "Description:".blue().bold(), idea.description));
        
        if !idea.notes.is_empty() {
            output.push_str(&format!("\n{}\n{}\n", "Notes:".blue().bold(), idea.notes));
        }
        
        if !idea.related_ideas.is_empty() {
            output.push_str(&format!("\n{}\n", "Related Ideas:".blue().bold()));
            for related in &idea.related_ideas {
                output.push_str(&format!("  - {}\n", related[..8].blue()));
            }
        }
        
        output
    }
}

/// Parse idea status from string
fn parse_status(status: &str) -> Result<IdeaStatus> {
    match status.to_lowercase().as_str() {
        "new" => Ok(IdeaStatus::New),
        "exploring" => Ok(IdeaStatus::Exploring),
        "development" | "indevelopment" | "in-development" | "in_development" => Ok(IdeaStatus::InDevelopment),
        "implemented" | "complete" | "completed" | "done" => Ok(IdeaStatus::Implemented),
        "onhold" | "on-hold" | "on_hold" | "hold" => Ok(IdeaStatus::OnHold),
        "archived" | "archive" => Ok(IdeaStatus::Archived),
        "rejected" | "reject" => Ok(IdeaStatus::Rejected),
        _ => Err(anyhow!("Invalid status: {}. Valid options: new, exploring, development, implemented, onhold, archived, rejected", status)),
    }
}

/// Parse idea priority from string
fn parse_priority(priority: &str) -> Result<IdeaPriority> {
    match priority.to_lowercase().as_str() {
        "low" | "l" => Ok(IdeaPriority::Low),
        "medium" | "med" | "m" => Ok(IdeaPriority::Medium),
        "high" | "h" => Ok(IdeaPriority::High),
        "critical" | "crit" | "c" => Ok(IdeaPriority::Critical),
        _ => Err(anyhow!("Invalid priority: {}. Valid options: low, medium, high, critical", priority)),
    }
}

/// Idea management command entry point
pub async fn idea(
    subcmd: &str,
    title: Option<&str>,
    description: Option<&str>,
    category: Option<&str>,
    tags: Option<&str>,
    id: Option<&str>,
    status: Option<&str>,
    priority: Option<&str>,
    filter: Option<&str>,
    _config: &Config
) -> Result<()> {
    // Direct formatting to match CI format exactly
    println!("{}", "💡 Idea Management".blue().bold());
    println!("{}", "================".blue());
    println!();
    println!("🔍 {}", format!("Operation: {}", subcmd).cyan().bold());
    println!();
    
    let idea_manager = IdeaManager::new(_config)?;
    
    match subcmd {
        "list" => {
            // Parse status filter if provided
            let status_filter = if let Some(s) = status {
                Some(parse_status(s)?)
            } else {
                None
            };
            
            // List ideas with optional filters
            let ideas = idea_manager.list_ideas(filter, category, status_filter)?;
            
            if ideas.is_empty() {
                println!("{} {}", "ℹ️".blue(), "No ideas found");
                return Ok(());
            }
            
            // Group by category if requested
            println!("{}", "📋 Ideas Collection:".blue().bold());
            println!();
            
            let idea_count = ideas.len();
            
            for idea in ideas {
                println!("  {}", formatters::format_idea_short(&idea));
            }
            
            println!();
            println!("{} {}", "✅".green(), format!("Listed {} ideas", idea_count).green().bold());
        },
        "add" => {
            // Validate required parameters
            let title = title.ok_or_else(|| anyhow!("Title is required for adding an idea"))?;
            let description = description.unwrap_or("");
            let category = category.unwrap_or("Uncategorized");
            
            println!("📝 {}", "Adding new idea...".yellow());
            
            // Parse tags
            let tags_vec = if let Some(t) = tags {
                t.split(',')
                    .map(|tag| tag.trim().to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect()
            } else {
                Vec::new()
            };
            
            // Add the idea
            let new_idea = idea_manager.add_idea(title, description, category, tags_vec)?;
            
            println!();
            println!("{}", "📋 Idea Details:".blue().bold());
            println!("{}", formatters::format_idea_detail(&new_idea));
            println!();
            println!("{} {}", "✅".green(), format!("Added new idea: {}", title).green().bold());
        },
        "view" => {
            // Validate ID
            let id = id.ok_or_else(|| anyhow!("ID is required for viewing an idea"))?;
            
            println!("🔍 {}", format!("Retrieving idea {}...", id[..8].blue()).yellow());
            
            // View the idea
            let idea = idea_manager.view_idea(id)?;
            
            println!();
            println!("{}", "📋 Idea Details:".blue().bold());
            println!("{}", formatters::format_idea_detail(&idea));
            println!();
            println!("{} {}", "✅".green(), "Idea details displayed".green().bold());
        },
        "update" => {
            // Validate ID
            let id = id.ok_or_else(|| anyhow!("ID is required for updating an idea"))?;
            
            println!("🔄 {}", format!("Updating idea {}...", id[..8].blue()).yellow());
            
            // Build updates
            let mut updates = IdeaUpdates {
                title: title.map(|s| s.to_string()),
                description: description.map(|s| s.to_string()),
                category: category.map(|s| s.to_string()),
                tags: None,
                status: None,
                priority: None,
                related_ideas: None,
                notes: None,
            };
            
            // Parse tags if provided
            if let Some(t) = tags {
                updates.tags = Some(
                    t.split(',')
                        .map(|tag| tag.trim().to_string())
                        .filter(|tag| !tag.is_empty())
                        .collect()
                );
            }
            
            // Parse status if provided
            if let Some(s) = status {
                updates.status = Some(parse_status(s)?);
            }
            
            // Parse priority if provided
            if let Some(p) = priority {
                updates.priority = Some(parse_priority(p)?);
            }
            
            // Update the idea
            let updated_idea = idea_manager.update_idea(id, updates)?;
            
            println!();
            println!("{}", "📋 Updated Idea:".blue().bold());
            println!("{}", formatters::format_idea_detail(&updated_idea));
            println!();
            println!("{} {}", "✅".green(), format!("Updated idea: {}", updated_idea.title).green().bold());
        },
        "delete" => {
            // Validate ID
            let id = id.ok_or_else(|| anyhow!("ID is required for deleting an idea"))?;
            
            println!("⚠️  {}", format!("Preparing to delete idea {}...", id[..8].blue()).yellow());
            
            // Confirm deletion
            print!("{}️ Are you sure you want to delete this idea? (y/N): ", "⚠️".yellow());
            io::stdout().flush().ok();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
                println!("{} {}", "ℹ️".blue(), "Deletion cancelled".blue());
                return Ok(());
            }
            
            println!("🗑️  {}", "Deleting idea...".yellow());
            
            // Delete the idea
            idea_manager.delete_idea(id)?;
            
            println!();
            println!("{} {}", "✅".green(), format!("Deleted idea with ID: {}", id).green().bold());
        },
        "categories" => {
            // List all categories
            println!("🔍 {}", "Retrieving idea categories...".yellow());
            
            let categories = idea_manager.get_categories()?;
            
            if categories.is_empty() {
                println!("{} {}", "ℹ️".blue(), "No categories found");
                return Ok(());
            }
            
            println!();
            println!("{}", "📊 Categories:".blue().bold());
            println!();
            
            let category_count = categories.len();
            
            for category in categories {
                println!("  - {}", category.green());
            }
            
            println!();
            println!("{} {}", "✅".green(), format!("Listed {} categories", category_count).green().bold());
        },
        "tags" => {
            // List all tags
            println!("🔍 {}", "Retrieving idea tags...".yellow());
            
            let tags = idea_manager.get_tags()?;
            
            if tags.is_empty() {
                println!("{} {}", "ℹ️".blue(), "No tags found");
                return Ok(());
            }
            
            println!();
            println!("{}", "🏷️  Tags:".blue().bold());
            println!();
            
            let tag_count = tags.len();
            
            for tag in tags {
                println!("  - {}", tag.yellow());
            }
            
            println!();
            println!("{} {}", "✅".green(), format!("Listed {} tags", tag_count).green().bold());
        },
        _ => {
            println!("{}", "❌ Unknown Subcommand".red().bold());
            println!("{}", "===================".red());
            println!();
            println!("The subcommand '{}' is not recognized.", subcmd.red().bold());
            println!();
            println!("{} {}:", "💡".cyan(), "Available subcommands".bold());
            println!("  - {}: List all ideas", "list".cyan());
            println!("  - {}: Add a new idea", "add".cyan());
            println!("  - {}: View idea details", "view".cyan());
            println!("  - {}: Update an existing idea", "update".cyan());
            println!("  - {}: Delete an idea", "delete".cyan());
            println!("  - {}: List idea categories", "categories".cyan());
            println!("  - {}: List idea tags", "tags".cyan());
            
            return Err(anyhow!("Unknown idea subcommand: {}", subcmd));
        }
    }
    
    Ok(())
}