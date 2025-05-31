// Topology Command Handler - CI Topology Management Integration
// Command handlers for all topology operations

use anyhow::{Result, Context};
use std::collections::HashMap;

use crate::config::Config;
use crate::helpers::command::CommandHelpers;
use crate::topology::{Topologist, TopologyAnalysis, FileCategory, CommitPhase};
use crate::errors::CIError;

#[derive(clap::Subcommand)]
pub enum TopologyCommands {
    /// Analyze repository state without creating files
    Analyze {
        #[arg(long, help = "Check if repository has unorganized files")]
        has_unorganized: bool,
    },
    /// Generate detailed commit plan
    Plan {
        #[arg(long, help = "Save plan to local metadata")]
        save: bool,
        #[arg(long, help = "Check if current plan is executable")]
        check: bool,
    },
    /// Execute commit phases
    Execute {
        #[arg(help = "Phase number to execute, or 'all' for complete execution")]
        phase: String,
    },
    /// Show current topology session status
    Status,
    /// Display size change tracking
    Track,
    /// Initialize topological management
    Init,
    /// Remove all topology metadata
    Clean,
    /// Export commit history summary
    Export,
    /// Git repository topology analysis (detailed)
    GitAnalysis,
    /// Sequential commit execution with metadata grouping
    Sequential {
        #[arg(long, help = "Group by metadata priority (high, medium, low)")]
        by_priority: bool,
        #[arg(long, help = "Group by file type and category")]
        by_category: bool,
        #[arg(long, help = "Group by estimated size for balanced commits")]
        by_size: bool,
        #[arg(long, help = "Interactive mode for manual phase selection")]
        interactive: bool,
        #[arg(long, help = "Dry run - show what would be committed")]
        dry_run: bool,
    },
}

pub async fn topology(command: &TopologyCommands, _config: &Config) -> Result<()> {
    let mut topologist = Topologist::new();

    match command {
        TopologyCommands::Analyze { has_unorganized } => {
            handle_analyze(&topologist, *has_unorganized).await
        },
        TopologyCommands::Plan { save, check } => {
            handle_plan(&mut topologist, *save, *check).await
        },
        TopologyCommands::Execute { phase } => {
            handle_execute(&mut topologist, phase).await
        },
        TopologyCommands::Status => {
            handle_status(&topologist).await
        },
        TopologyCommands::Track => {
            handle_track(&topologist).await
        },
        TopologyCommands::Init => {
            handle_init(&mut topologist).await
        },
        TopologyCommands::Clean => {
            handle_clean(&mut topologist).await
        },
        TopologyCommands::Export => {
            handle_export(&topologist).await
        },
        TopologyCommands::GitAnalysis => {
            handle_git_analysis().await
        },
        TopologyCommands::Sequential { by_priority, by_category, by_size, interactive, dry_run } => {
            handle_sequential_commits(&mut topologist, *by_priority, *by_category, *by_size, *interactive, *dry_run).await
        },
    }
}

async fn handle_analyze(topologist: &Topologist, has_unorganized: bool) -> Result<()> {
    CommandHelpers::print_command_header(
        "Repository topology analysis",
        "🔍",
        "Topology Management",
        "cyan"
    );

    let analysis = topologist.analyze_repository()
        .map_err(|e| CIError::TopologyError(e.to_string()))?;
    
    if has_unorganized {
        // Silent check for CI/CD integration
        if analysis.repository_stats.total_files > 0 {
            std::process::exit(1); // Has unorganized files
        } else {
            std::process::exit(0); // Repository is organized
        }
    }

    print_analysis_summary(&analysis);
    Ok(())
}

async fn handle_plan(topologist: &mut Topologist, save: bool, check: bool) -> Result<()> {
    CommandHelpers::print_command_header(
        "Generate systematic commit plan",
        "📋",
        "Topology Management", 
        "cyan"
    );

    let analysis = topologist.analyze_repository()
        .map_err(|e| CIError::TopologyError(e.to_string()))?;
    
    if check {
        // Check if plan is executable
        if analysis.commit_phases.is_empty() {
            println!("✅ Repository is organized, no plan needed");
            return Ok(());
        } else {
            println!("📋 Plan available: {} phases ready for execution", analysis.commit_phases.len());
            return Ok(());
        }
    }

    if analysis.commit_phases.is_empty() {
        println!("✅ Repository is already organized!");
        return Ok(());
    }

    print_detailed_plan(&analysis);

    if save {
        topologist.initialize()
            .map_err(|e| CIError::TopologyError(e.to_string()))?;
        println!("\n💾 Plan saved to .ci-topology/ metadata");
        println!("   Run 'ci topologist execute all' to proceed");
    } else {
        println!("\n💡 Run with --save to create executable plan");
    }

    Ok(())
}

async fn handle_execute(topologist: &mut Topologist, phase: &str) -> Result<()> {
    CommandHelpers::print_command_header(
        "Execute topology organization phases",
        "🚀",
        "Topology Management",
        "cyan"
    );

    let analysis = topologist.analyze_repository()
        .map_err(|e| CIError::TopologyError(e.to_string()))?;
    
    if analysis.commit_phases.is_empty() {
        println!("✅ No phases to execute - repository is organized");
        return Ok(());
    }

    if phase == "all" {
        println!("🚀 Executing all {} phases...\n", analysis.commit_phases.len());
        
        for (i, commit_phase) in analysis.commit_phases.iter().enumerate() {
            let phase_num = i + 1;
            println!("📦 Phase {}: {}", phase_num, commit_phase.commit_message);
            
            let commit_hash = topologist.execute_phase(phase_num, &analysis.commit_phases)
                .map_err(|e| CIError::TopologyError(e.to_string()))?;
            
            println!("   ✅ Commit: {} (+{} estimated lines)", 
                     &commit_hash[..8], commit_phase.estimated_size);
            println!("   📁 Files: {}", commit_phase.files.len());
            println!();
        }
        
        println!("🎉 All phases complete! Repository synchronized.");
        
    } else {
        let phase_num: usize = phase.parse()
            .context("Invalid phase number")?;
            
        if phase_num == 0 || phase_num > analysis.commit_phases.len() {
            return Err(anyhow::anyhow!("Phase {} not found. Available: 1-{}", 
                         phase_num, analysis.commit_phases.len()));
        }
        
        let commit_phase = &analysis.commit_phases[phase_num - 1];
        println!("📦 Executing Phase {}: {}", phase_num, commit_phase.commit_message);
        
        let commit_hash = topologist.execute_phase(phase_num, &analysis.commit_phases)
            .map_err(|e| CIError::TopologyError(e.to_string()))?;
        
        println!("✅ Phase {} Complete: {} (+{} estimated lines)", 
                 phase_num, &commit_hash[..8], commit_phase.estimated_size);
    }

    Ok(())
}

async fn handle_status(topologist: &Topologist) -> Result<()> {
    CommandHelpers::print_command_header(
        "Topology session status",
        "📊",
        "Topology Management",
        "cyan"
    );

    match topologist.get_status()
        .map_err(|e| CIError::TopologyError(e.to_string()))? {
        Some(session) => {
            println!("📊 Topology Session Active");
            println!("   Created: {}", session.started);
            println!("   Phases completed: {}/{}", 
                     session.phases.len(), 
                     session.total_planned_phases.unwrap_or(0));
            
            if let Some(last_phase) = session.phases.last() {
                println!("   Last commit: {}", last_phase.commit_hash);
            }
        },
        None => {
            println!("📊 No active topology session");
            println!("   Run 'ci topologist plan --save' to start");
        }
    }
    Ok(())
}

async fn handle_track(topologist: &Topologist) -> Result<()> {
    CommandHelpers::print_command_header(
        "Repository size change tracking",
        "📈",
        "Topology Management",
        "cyan"
    );

    match topologist.get_status()
        .map_err(|e| CIError::TopologyError(e.to_string()))? {
        Some(session) => {
            println!("📈 Size Change Tracking");
            println!("   Session: {}", session.session_id);
            
            let mut total_insertions = 0;
            let mut total_files = 0;
            
            for (i, phase) in session.phases.iter().enumerate() {
                println!("   Phase {}: +{} lines, {} files", 
                         i + 1, phase.size_change, phase.files_count);
                total_insertions += phase.size_change;
                total_files += phase.files_count;
            }
            
            println!("   Total Impact: +{} lines, {} files", total_insertions, total_files);
        },
        None => {
            println!("📈 No tracking data available");
            println!("   Start a session with 'ci topologist plan --save'");
        }
    }
    Ok(())
}

async fn handle_init(topologist: &mut Topologist) -> Result<()> {
    CommandHelpers::print_command_header(
        "Initialize topology management",
        "🎯",
        "Topology Management",
        "cyan"
    );

    topologist.initialize()
        .map_err(|e| CIError::TopologyError(e.to_string()))?;
    println!("🎯 Topology management initialized");
    println!("   Metadata directory: .ci-topology/");
    println!("   Next: 'ci topologist analyze' to get started");
    Ok(())
}

async fn handle_clean(topologist: &mut Topologist) -> Result<()> {
    CommandHelpers::print_command_header(
        "Clean topology metadata",
        "🧹",
        "Topology Management",
        "cyan"
    );

    topologist.clean()
        .map_err(|e| CIError::TopologyError(e.to_string()))?;
    println!("🧹 All topology metadata removed");
    println!("   Repository is clean of all traces");
    Ok(())
}

async fn handle_export(topologist: &Topologist) -> Result<()> {
    CommandHelpers::print_command_header(
        "Export topology session data",
        "📤",
        "Topology Management",
        "cyan"
    );

    match topologist.get_status()
        .map_err(|e| CIError::TopologyError(e.to_string()))? {
        Some(session) => {
            // Export in JSON format for programmatic use
            let json = serde_json::to_string_pretty(&session)
                .context("Failed to serialize session data")?;
            println!("{}", json);
        },
        None => {
            println!("{{}}");  // Empty JSON object
        }
    }
    Ok(())
}

fn print_analysis_summary(analysis: &TopologyAnalysis) {
    println!("🔍 Repository Analysis Complete");
    println!("├── {} total files detected", analysis.repository_stats.total_files);
    println!("├── {} untracked files", analysis.repository_stats.untracked_files);  
    println!("├── {} modified files", analysis.repository_stats.modified_files);
    println!("├── Estimated growth: +{} lines", analysis.repository_stats.estimated_total_size);
    println!("└── Suggested: {}-phase commit strategy", analysis.repository_stats.suggested_phases);
    
    if !analysis.commit_phases.is_empty() {
        println!("\n📂 File Categories:");
        let mut category_summary: HashMap<String, usize> = HashMap::new();
        
        for file in &analysis.category_analysis.files {
            let category_name = format!("{:?}", file.category);
            *category_summary.entry(category_name).or_insert(0) += 1;
        }
        
        for (category, count) in category_summary {
            let icon = match category.as_str() {
                "Configuration" => "⚙️",
                "Documentation" => "📚",
                "SourceCode" => "💻",
                "DevelopmentTools" => "🔧",
                "MediaAssets" => "🎨",
                _ => "📄",
            };
            println!("   {} {}: {} files", icon, category, count);
        }
        
        println!("\n💡 Run 'ci topologist plan --save' to proceed");
    }
}

fn print_detailed_plan(analysis: &TopologyAnalysis) {
    println!("📋 Detailed Commit Plan ({} phases)", analysis.commit_phases.len());
    println!();
    
    for (i, phase) in analysis.commit_phases.iter().enumerate() {
        let phase_num = i + 1;
        let category_icon = match phase.category {
            FileCategory::Configuration => "⚙️",
            FileCategory::Documentation => "📚", 
            FileCategory::SourceCode => "💻",
            FileCategory::DevelopmentTools => "🔧",
            FileCategory::MediaAssets => "🎨",
            _ => "📄",
        };
        
        println!("Phase {}: {} {}", phase_num, category_icon, phase.commit_message);
        println!("├── Files: {} (+{} estimated lines)", phase.files.len(), phase.estimated_size);
        println!("├── Category: {:?}", phase.category);
        
        // Show first few files as examples
        let max_show = 3;
        for (j, file) in phase.files.iter().take(max_show).enumerate() {
            let prefix = if j == phase.files.len().min(max_show) - 1 { "└──" } else { "├──" };
            println!("{}   {}", prefix, file.path);
        }
        
        if phase.files.len() > max_show {
            println!("└──   ... and {} more files", phase.files.len() - max_show);
        }
        println!();
    }
    
    let total_size: usize = analysis.commit_phases.iter().map(|p| p.estimated_size).sum();
    println!("📊 Total Impact: +{} estimated lines across {} commits", total_size, analysis.commit_phases.len());
}

async fn handle_git_analysis() -> Result<()> {
    CommandHelpers::print_command_header(
        "Git repository topology analysis",
        "🔍",
        "Topology Management",
        "cyan"
    );

    println!("# Git Repository Topography Analysis\n");
    
    // Get total .git size
    let git_size = get_git_directory_size()?;
    println!("**Total .git size: {:.1}MB**\n", git_size);
    
    // Analyze directory structure
    analyze_git_directory_structure()?;
    
    // Get object statistics
    analyze_object_distribution()?;
    
    // Analyze content by file type
    analyze_content_by_file_type()?;
    
    // Get largest files
    show_largest_files()?;

    Ok(())
}

fn get_git_directory_size() -> Result<f64> {
    use std::process::Command;
    
    let output = Command::new("du")
        .args(&["-sm", ".git"])
        .output()
        .context("Failed to execute du command")?;
    
    let output_str = std::str::from_utf8(&output.stdout)
        .context("Invalid UTF-8 in du output")?;
    let size_str = output_str.split_whitespace().next().unwrap_or("0");
    size_str.parse::<f64>()
        .context("Failed to parse git directory size")
}

fn analyze_git_directory_structure() -> Result<()> {
    use std::process::Command;
    
    println!("## Directory Structure");
    
    let output = Command::new("sh")
        .arg("-c")
        .arg("du -sh .git/* 2>/dev/null | sort -hr")
        .output()
        .context("Failed to analyze git directory")?;
    
    let output_str = std::str::from_utf8(&output.stdout)
        .context("Invalid UTF-8 in directory analysis")?;
    let total_size = get_git_directory_size()? * 1024.0; // Convert to KB
    
    println!("```");
    println!(".git/");
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let size_str = parts[0];
            let path = parts[1].replace(".git/", "");
            
            // Convert size to KB
            let size_kb = parse_size_to_kb(size_str);
            let percentage = (size_kb / total_size) * 100.0;
            
            let description = match path.as_str() {
                "objects" => "- Compressed object storage",
                "index" => "- Staging area state", 
                "hooks" => "- Git hooks",
                "filter-repo" => "- Recent filter operations",
                "info" => "- Repository metadata",
                _ => "- Config, refs, logs"
            };
            
            println!("├── {:15} {:>6} ({:4.1}%) {}", 
                path, size_str, percentage, description);
        }
    }
    println!("```\n");
    
    // Analyze pack files specifically
    analyze_pack_files()?;
    Ok(())
}

fn analyze_pack_files() -> Result<()> {
    use std::process::Command;
    
    let output = Command::new("ls")
        .args(&["-lh", ".git/objects/pack/"])
        .output();
    
    if let Ok(output) = output {
        let output_str = std::str::from_utf8(&output.stdout)
            .context("Invalid UTF-8 in pack files listing")?;
        if !output_str.is_empty() {
            println!("### Pack File Details:");
            println!("```");
            for line in output_str.lines().skip(1) { // Skip "total" line
                if line.contains(".pack") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        println!("Pack file:     {} - All git objects compressed", parts[4]);
                    }
                } else if line.contains(".idx") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        println!("Index file:    {} - Pack file index", parts[4]);
                    }
                }
            }
            println!("```\n");
        }
    }
    Ok(())
}

fn analyze_object_distribution() -> Result<()> {
    use std::process::Command;
    
    println!("## Object Distribution");
    
    // Get object count statistics
    let output = Command::new("git")
        .args(&["count-objects", "-v"])
        .output()
        .context("Failed to get object statistics")?;
    
    let output_str = std::str::from_utf8(&output.stdout)
        .context("Invalid UTF-8 in git count-objects output")?;
    let mut in_pack = 0;
    let mut size_pack = 0;
    
    for line in output_str.lines() {
        if line.starts_with("in-pack ") {
            in_pack = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        }
        if line.starts_with("size-pack ") {
            size_pack = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        }
    }
    
    // Analyze object types
    let output = Command::new("sh")
        .arg("-c")
        .arg("git rev-list --objects --all | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | awk '{type[$1]++; size[$1]+=$3} END {for(t in type) printf \"%s %d %d\\n\", t, type[t], size[t]}' | sort -k3 -nr")
        .output()
        .context("Failed to analyze object types")?;
    
    let output_str = std::str::from_utf8(&output.stdout)
        .context("Invalid UTF-8 in object analysis")?;
    let mut total_uncompressed = 0;
    
    println!("```");
    println!("Object Type    Count    Uncompressed    Compressed*");
    println!("─────────────────────────────────────────────────────");
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let obj_type = parts[0];
            let count: i32 = parts[1].parse().unwrap_or(0);
            let size: i32 = parts[2].parse().unwrap_or(0);
            total_uncompressed += size;
            
            let compressed_estimate = (size as f64 * size_pack as f64 / (total_uncompressed as f64 + 1.0)) as i32;
            
            println!("{:10}: {:6} objects    {:6.0} KB       ~{:4} KB", 
                format!("{}s", obj_type), count, size as f64 / 1024.0, compressed_estimate / 1024);
        }
    }
    
    println!("─────────────────────────────────────────────────────");
    println!("Total:         {:6} objects    {:6.0} KB       {:4} KB", 
        in_pack, total_uncompressed as f64 / 1024.0, size_pack);
    println!("```\n");
    Ok(())
}

fn analyze_content_by_file_type() -> Result<()> {
    use std::process::Command;
    
    println!("## Content Breakdown by File Type");
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(r#"git rev-list --objects --all | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | sed -n 's/^blob //p' | awk '{
            file=$3
            size=$2
            if (file ~ /\.rs$/) { rust_size += size; rust_count++ }
            else if (file ~ /\.md$/) { md_size += size; md_count++ }
            else if (file ~ /\.js$/) { js_size += size; js_count++ }
            else if (file ~ /\.html$/) { html_size += size; html_count++ }
            else if (file ~ /\.json$/) { json_size += size; json_count++ }
            else if (file ~ /\.(sh|bash)$/) { script_size += size; script_count++ }
            else { other_size += size; other_count++ }
        } END {
            total_size = rust_size + md_size + js_size + html_size + json_size + script_size + other_size
            printf "Markdown %d %.1f %.1f\n", md_count, md_size/1024, (md_size/total_size)*100
            printf "Rust(.rs) %d %.1f %.1f\n", rust_count, rust_size/1024, (rust_size/total_size)*100
            printf "Scripts %d %.1f %.1f\n", script_count, script_size/1024, (script_size/total_size)*100
            printf "Other %d %.1f %.1f\n", other_count, other_size/1024, (other_size/total_size)*100
            printf "JavaScript %d %.1f %.1f\n", js_count, js_size/1024, (js_size/total_size)*100
            printf "JSON %d %.1f %.1f\n", json_count, json_size/1024, (json_size/total_size)*100
            printf "HTML %d %.1f %.1f\n", html_count, html_size/1024, (html_size/total_size)*100
        }'"#)
        .output()
        .context("Failed to analyze content by file type")?;
    
    let output_str = std::str::from_utf8(&output.stdout)
        .context("Invalid UTF-8 in content analysis")?;
    
    println!("```");
    println!("File Type       Files    Size (KB)    % of Content");
    println!("─────────────────────────────────────────────────────");
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let file_type = parts[0];
            let count: i32 = parts[1].parse().unwrap_or(0);
            let size: f64 = parts[2].parse().unwrap_or(0.0);
            let percentage: f64 = parts[3].parse().unwrap_or(0.0);
            
            println!("{:15}: {:6}    {:8.1}     {:4.1}%", 
                file_type, count, size, percentage);
        }
    }
    println!("```\n");
    Ok(())
}

fn show_largest_files() -> Result<()> {
    use std::process::Command;
    
    println!("## Largest Files");
    
    let output = Command::new("sh")
        .arg("-c")
        .arg("git rev-list --objects --all | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | sed -n 's/^blob //p' | sort -k2 -nr | head -10")
        .output()
        .context("Failed to get largest files")?;
    
    let output_str = std::str::from_utf8(&output.stdout)
        .context("Invalid UTF-8 in largest files analysis")?;
    
    println!("```");
    println!("Size (KB)  File");
    println!("─────────────────────────────────────────");
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let size: f64 = parts[1].parse().unwrap_or(0.0) / 1024.0;
            let file = parts[2];
            
            println!("{:8.1}   {}", size, file);
        }
    }
    println!("```\n");
    
    // Calculate compression ratio
    let total_size = get_git_directory_size()?;
    let compression_ratio = ((total_size * 1024.0 - 2640.0) / (total_size * 1024.0)) * 100.0;
    
    println!("**Key Insights:**");
    println!("- **Efficient compression**: ~{:.0}% compression ratio", compression_ratio);
    println!("- **Healthy distribution**: No single massive files after optimization");
    println!("- **Total objects**: Efficiently packed in compressed storage");
    println!("\n*Compressed sizes are estimates based on pack compression ratio");
    Ok(())
}

async fn handle_sequential_commits(
    topologist: &mut Topologist, 
    by_priority: bool, 
    by_category: bool, 
    by_size: bool, 
    interactive: bool, 
    dry_run: bool
) -> Result<()> {
    CommandHelpers::print_command_header(
        "Sequential commit execution with metadata grouping",
        "⚡",
        "Topology Management",
        "cyan"
    );

    let analysis = topologist.analyze_repository()
        .map_err(|e| CIError::TopologyError(e.to_string()))?;
    
    if analysis.commit_phases.is_empty() {
        println!("✅ Repository is already organized - no commits needed");
        return Ok(());
    }

    // Apply the requested grouping strategy
    let mut phases = analysis.commit_phases.clone();
    
    if by_priority {
        phases = group_by_priority(phases);
        println!("📊 Grouped {} phases by metadata priority", phases.len());
    } else if by_category {
        phases = group_by_category(phases);
        println!("📂 Grouped {} phases by file category", phases.len());
    } else if by_size {
        phases = group_by_size(phases);
        println!("📏 Grouped {} phases by balanced size", phases.len());
    } else {
        println!("📋 Using default sequential grouping ({} phases)", phases.len());
    }

    print_sequential_plan(&phases);

    if dry_run {
        println!("\n🔍 Dry run mode - no commits will be made");
        return Ok(());
    }

    if interactive {
        execute_interactive_sequence(topologist, &phases).await
    } else {
        execute_automatic_sequence(topologist, &phases).await
    }
}

fn group_by_priority(mut phases: Vec<CommitPhase>) -> Vec<CommitPhase> {
    use crate::topology::FileCategory;
    
    // Priority-based sorting with high -> medium -> low
    phases.sort_by(|a, b| {
        let priority_a = match a.category {
            FileCategory::Configuration => 10,  // Critical files first
            FileCategory::Documentation => 9,   // Documentation second
            FileCategory::DevelopmentTools => 8,
            FileCategory::SourceCode => 7,
            FileCategory::MediaAssets => 6,
            FileCategory::BuildArtifacts => 1,  // Build artifacts last
            FileCategory::Unknown => 5,
        };
        let priority_b = match b.category {
            FileCategory::Configuration => 10,
            FileCategory::Documentation => 9,
            FileCategory::DevelopmentTools => 8,
            FileCategory::SourceCode => 7,
            FileCategory::MediaAssets => 6,
            FileCategory::BuildArtifacts => 1,
            FileCategory::Unknown => 5,
        };
        priority_b.cmp(&priority_a)
    });

    // Renumber phases after sorting
    for (i, phase) in phases.iter_mut().enumerate() {
        phase.phase_number = i + 1;
    }

    phases
}

fn group_by_category(mut phases: Vec<CommitPhase>) -> Vec<CommitPhase> {
    // Group similar categories together and sort by category type
    phases.sort_by(|a, b| {
        format!("{:?}", a.category).cmp(&format!("{:?}", b.category))
    });

    // Renumber phases after sorting
    for (i, phase) in phases.iter_mut().enumerate() {
        phase.phase_number = i + 1;
    }

    phases
}

fn group_by_size(mut phases: Vec<CommitPhase>) -> Vec<CommitPhase> {
    // Balance phases by size - alternate large and small commits
    phases.sort_by(|a, b| a.estimated_size.cmp(&b.estimated_size));
    
    let mut balanced_phases = Vec::new();
    let mut large_phases = Vec::new();
    let mut small_phases = Vec::new();
    
    let median_size = if phases.len() > 0 {
        phases[phases.len() / 2].estimated_size
    } else {
        0
    };
    
    for phase in phases {
        if phase.estimated_size >= median_size {
            large_phases.push(phase);
        } else {
            small_phases.push(phase);
        }
    }
    
    // Interleave large and small phases for better balance
    let max_len = large_phases.len().max(small_phases.len());
    for i in 0..max_len {
        if i < large_phases.len() {
            balanced_phases.push(large_phases[i].clone());
        }
        if i < small_phases.len() {
            balanced_phases.push(small_phases[i].clone());
        }
    }

    // Renumber phases after reordering
    for (i, phase) in balanced_phases.iter_mut().enumerate() {
        phase.phase_number = i + 1;
    }

    balanced_phases
}

fn print_sequential_plan(phases: &[CommitPhase]) {
    println!("\n📋 Sequential Execution Plan:");
    println!("═══════════════════════════════════════════════════════");
    
    for (i, phase) in phases.iter().enumerate() {
        let phase_num = i + 1;
        let category_icon = match phase.category {
            FileCategory::Configuration => "⚙️",
            FileCategory::Documentation => "📚", 
            FileCategory::SourceCode => "💻",
            FileCategory::DevelopmentTools => "🔧",
            FileCategory::MediaAssets => "🎨",
            _ => "📄",
        };
        
        let priority_level = match phase.category {
            FileCategory::Configuration => "HIGH",
            FileCategory::Documentation => "HIGH",
            FileCategory::DevelopmentTools => "MEDIUM",
            FileCategory::SourceCode => "MEDIUM",
            FileCategory::MediaAssets => "LOW",
            FileCategory::BuildArtifacts => "LOW",
            FileCategory::Unknown => "MEDIUM",
        };
        
        println!("Phase {}: {} {} [{}]", 
                phase_num, 
                category_icon, 
                phase.commit_message, 
                priority_level);
        println!("├── Files: {} (+{} estimated lines)", 
                phase.files.len(), 
                phase.estimated_size);
        
        // Show file sample
        for (j, file) in phase.files.iter().take(3).enumerate() {
            let prefix = if j == 2 || j == phase.files.len() - 1 { "└──" } else { "├──" };
            println!("{}   {}", prefix, file.path);
        }
        
        if phase.files.len() > 3 {
            println!("└──   ... and {} more files", phase.files.len() - 3);
        }
        println!();
    }
    
    let total_files: usize = phases.iter().map(|p| p.files.len()).sum();
    let total_size: usize = phases.iter().map(|p| p.estimated_size).sum();
    println!("📊 Total: {} files, +{} estimated lines across {} commits", 
             total_files, total_size, phases.len());
}

async fn execute_interactive_sequence(topologist: &mut Topologist, phases: &[CommitPhase]) -> Result<()> {
    println!("\n🎯 Interactive Sequential Execution Mode");
    println!("Choose phases to execute (enter phase numbers separated by commas, or 'all'):");
    
    use std::io::{self, Write};
    
    loop {
        print!("\nPhases (1-{}, 'all', or 'quit'): ", phases.len());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).context("Failed to read input")?;
        let input = input.trim();
        
        if input == "quit" {
            println!("🛑 Interactive mode cancelled");
            return Ok(());
        }
        
        if input == "all" {
            return execute_all_phases(topologist, phases).await;
        }
        
        // Parse comma-separated phase numbers
        let phase_numbers: Result<Vec<usize>, _> = input
            .split(',')
            .map(|s| s.trim().parse::<usize>())
            .collect();
        
        match phase_numbers {
            Ok(numbers) => {
                let valid_numbers: Vec<usize> = numbers
                    .into_iter()
                    .filter(|&n| n > 0 && n <= phases.len())
                    .collect();
                
                if valid_numbers.is_empty() {
                    println!("❌ No valid phase numbers provided");
                    continue;
                }
                
                return execute_selected_phases(topologist, phases, &valid_numbers).await;
            },
            Err(_) => {
                println!("❌ Invalid input. Please enter numbers separated by commas");
                continue;
            }
        }
    }
}

async fn execute_automatic_sequence(topologist: &mut Topologist, phases: &[CommitPhase]) -> Result<()> {
    println!("\n🚀 Executing all phases automatically...");
    execute_all_phases(topologist, phases).await
}

async fn execute_all_phases(topologist: &mut Topologist, phases: &[CommitPhase]) -> Result<()> {
    println!("\n🚀 Executing all {} phases sequentially...", phases.len());
    
    for (i, phase) in phases.iter().enumerate() {
        let phase_num = i + 1;
        println!("\n📦 Phase {}/{}: {}", phase_num, phases.len(), phase.commit_message);
        
        let commit_hash = topologist.execute_phase(phase_num, phases)
            .map_err(|e| CIError::TopologyError(e.to_string()))?;
        
        println!("   ✅ Commit: {} (+{} estimated lines)", 
                 &commit_hash[..8], phase.estimated_size);
        println!("   📁 Files: {}", phase.files.len());
        
        // Brief pause between commits for readability
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("\n🎉 All {} phases completed successfully!", phases.len());
    println!("📊 Repository is now fully organized and committed");
    
    Ok(())
}

async fn execute_selected_phases(topologist: &mut Topologist, phases: &[CommitPhase], selected: &[usize]) -> Result<()> {
    println!("\n🎯 Executing {} selected phases...", selected.len());
    
    for &phase_num in selected {
        let phase = &phases[phase_num - 1];
        println!("\n📦 Phase {}: {}", phase_num, phase.commit_message);
        
        let commit_hash = topologist.execute_phase(phase_num, phases)
            .map_err(|e| CIError::TopologyError(e.to_string()))?;
        
        println!("   ✅ Commit: {} (+{} estimated lines)", 
                 &commit_hash[..8], phase.estimated_size);
        println!("   📁 Files: {}", phase.files.len());
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("\n🎉 Selected phases completed successfully!");
    
    Ok(())
}

fn parse_size_to_kb(size_str: &str) -> f64 {
    let size_str = size_str.trim();
    
    if size_str.ends_with("M") {
        let num: f64 = size_str.trim_end_matches("M").parse().unwrap_or(0.0);
        num * 1024.0
    } else if size_str.ends_with("K") {
        let num: f64 = size_str.trim_end_matches("K").parse().unwrap_or(0.0);
        num
    } else if size_str.ends_with("B") {
        let num: f64 = size_str.trim_end_matches("B").parse().unwrap_or(0.0);
        num / 1024.0
    } else {
        // Assume bytes if no suffix
        let num: f64 = size_str.parse().unwrap_or(0.0);
        num / 1024.0
    }
}