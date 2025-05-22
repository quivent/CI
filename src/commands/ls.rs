//! CI ls command - Enhanced file listing with grouping and visual formatting
//!
//! Provides intelligent file listing with automatic grouping by file type,
//! two-column split-screen layout, and color-coded categories.

use anyhow::{Context, Result};
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::helpers::CommandHelpers;

/// File type categories for grouping
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum FileType {
    Source,      // .rs, .js, .py, .java, .cpp, .c, .h, etc.
    Config,      // .json, .toml, .yaml, .yml, .ini, .conf, etc.
    Documentation, // .md, .txt, .rst, .adoc, etc.
    Build,       // Makefile, build.*, package.json, Cargo.toml, etc.
    Git,         // .git*, .gitignore, etc.
    Backup,      // .bak, .backup, .old, .tmp, etc.
    Binary,      // .exe, .bin, .so, .dll, etc.
    Media,       // .png, .jpg, .gif, .mp4, .mp3, etc.
    Archive,     // .zip, .tar, .gz, .7z, etc.
    Directory,   // Directories
    Other,       // Everything else
}

impl FileType {
    /// Get the display name for this file type
    fn display_name(&self) -> &'static str {
        match self {
            FileType::Source => "Source Code",
            FileType::Config => "Configuration",
            FileType::Documentation => "Documentation",
            FileType::Build => "Build Files",
            FileType::Git => "Git Files",
            FileType::Backup => "Backup Files",
            FileType::Binary => "Binaries",
            FileType::Media => "Media Files",
            FileType::Archive => "Archives",
            FileType::Directory => "Directories",
            FileType::Other => "Other Files",
        }
    }

    /// Get the color for this file type
    fn color(&self) -> Color {
        match self {
            FileType::Source => Color::Green,
            FileType::Config => Color::Blue,
            FileType::Documentation => Color::Cyan,
            FileType::Build => Color::Yellow,
            FileType::Git => Color::Magenta,
            FileType::Backup => Color::BrightBlack,
            FileType::Binary => Color::Red,
            FileType::Media => Color::BrightMagenta,
            FileType::Archive => Color::BrightYellow,
            FileType::Directory => Color::BrightBlue,
            FileType::Other => Color::White,
        }
    }

    /// Get the icon/emoji for this file type
    fn icon(&self) -> &'static str {
        match self {
            FileType::Source => "📝",
            FileType::Config => "⚙️",
            FileType::Documentation => "📚",
            FileType::Build => "🔧",
            FileType::Git => "🌿",
            FileType::Backup => "💾",
            FileType::Binary => "⚡",
            FileType::Media => "🎨",
            FileType::Archive => "📦",
            FileType::Directory => "📁",
            FileType::Other => "📄",
        }
    }
}

/// File entry with metadata
#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    path: PathBuf,
    file_type: FileType,
    is_dir: bool,
    size: Option<u64>,
}

/// Determine file type based on extension and name patterns
fn classify_file(path: &Path) -> FileType {
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check if it's a directory
    if path.is_dir() {
        return FileType::Directory;
    }

    // Git files
    if file_name.starts_with(".git") || file_name == "gitignore" || extension == "gitignore" {
        return FileType::Git;
    }

    // Backup files
    if matches!(extension.as_str(), "bak" | "backup" | "old" | "tmp" | "orig" | "swp") 
        || file_name.ends_with(".bak") || file_name.ends_with(".backup") {
        return FileType::Backup;
    }

    // Source code files
    if matches!(extension.as_str(), 
        "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "java" | "cpp" | "c" | "h" | "hpp" | 
        "cs" | "php" | "rb" | "go" | "swift" | "kt" | "scala" | "clj" | "hs" | "ml" | 
        "elm" | "dart" | "vue" | "svelte" | "sol" | "zig" | "nim"
    ) {
        return FileType::Source;
    }

    // Configuration files
    if matches!(extension.as_str(), 
        "json" | "toml" | "yaml" | "yml" | "ini" | "conf" | "config" | "xml" | "env" | "properties"
    ) || matches!(file_name.as_str(), 
        "dockerfile" | "makefile" | ".env" | ".envrc" | "config" | "settings"
    ) {
        return FileType::Config;
    }

    // Build files
    if matches!(file_name.as_str(), 
        "cargo.toml" | "package.json" | "pom.xml" | "build.gradle" | "makefile" | "cmakelists.txt" |
        "build.sh" | "build.py" | "gulpfile.js" | "webpack.config.js" | "rollup.config.js"
    ) || file_name.contains("build") || file_name.contains("make") {
        return FileType::Build;
    }

    // Documentation files
    if matches!(extension.as_str(), 
        "md" | "txt" | "rst" | "adoc" | "org" | "tex" | "pdf" | "doc" | "docx"
    ) || matches!(file_name.as_str(), 
        "readme" | "changelog" | "license" | "authors" | "contributors"
    ) {
        return FileType::Documentation;
    }

    // Binary files
    if matches!(extension.as_str(), 
        "exe" | "bin" | "so" | "dll" | "dylib" | "a" | "lib" | "deb" | "rpm" | "msi" | "pkg"
    ) {
        return FileType::Binary;
    }

    // Media files
    if matches!(extension.as_str(), 
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "bmp" | "ico" | "webp" | 
        "mp4" | "avi" | "mov" | "mkv" | "webm" | 
        "mp3" | "wav" | "flac" | "ogg" | "m4a"
    ) {
        return FileType::Media;
    }

    // Archive files
    if matches!(extension.as_str(), 
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "dmg" | "iso"
    ) {
        return FileType::Archive;
    }

    FileType::Other
}

/// Get the terminal width for layout calculations
fn get_terminal_width() -> usize {
    term_size::dimensions().map(|(w, _)| w).unwrap_or(80)
}

/// Collect and classify files from the given directory
fn collect_files(dir: &Path) -> Result<Vec<FileEntry>> {
    let mut files = Vec::new();
    
    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let metadata = entry.metadata().context("Failed to read file metadata")?;
        
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        
        // Skip hidden files unless explicitly included
        if file_name.starts_with('.') && !matches!(file_name.as_str(), ".git" | ".gitignore" | ".env") {
            continue;
        }
        
        let file_type = classify_file(&path);
        let is_dir = metadata.is_dir();
        let size = if is_dir { None } else { Some(metadata.len()) };
        
        files.push(FileEntry {
            name: file_name,
            path: path.clone(),
            file_type,
            is_dir,
            size,
        });
    }
    
    Ok(files)
}

/// Group files by type and sort within groups
fn group_and_sort_files(files: Vec<FileEntry>) -> HashMap<FileType, Vec<FileEntry>> {
    let mut groups: HashMap<FileType, Vec<FileEntry>> = HashMap::new();
    
    for file in files {
        groups.entry(file.file_type.clone()).or_insert_with(Vec::new).push(file);
    }
    
    // Sort files within each group alphabetically
    for files in groups.values_mut() {
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }
    
    groups
}

/// Format file size for display (only show for files > 1KB)
fn format_size(size: Option<u64>) -> String {
    match size {
        Some(bytes) => {
            if bytes < 1024 {
                "      ".to_string() // Don't show size for small files
            } else if bytes < 1024 * 1024 {
                format!("{:>5.1}K", bytes as f64 / 1024.0)
            } else if bytes < 1024 * 1024 * 1024 {
                format!("{:>5.1}M", bytes as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:>5.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        }
        None => "      ".to_string(), // Directories get size calculated separately
    }
}

/// Get directory size (number of items) formatted to match file size width
fn get_directory_size(path: &Path) -> String {
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let count = entries.count();
            if count == 0 {
                format!("{:>6}", "empty")
            } else if count < 10 {
                format!("{:>6}", format!("{}item{}", count, if count == 1 { "" } else { "s" }))
            } else if count < 100 {
                format!("{:>6}", format!("{}+", count))
            } else {
                format!("{:>6}", "many")
            }
        }
        Err(_) => format!("{:>6}", "?"),
    }
}

/// Display a group of files with enhanced formatting and organization
fn display_group(group_type: &FileType, files: &[FileEntry], column_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    
    // Enhanced group header with better visual hierarchy
    let header = format!(
        "{} {} {}",
        group_type.icon(),
        group_type.display_name().to_uppercase(),
        format!("({})", files.len()).bright_black()
    );
    lines.push(header.color(group_type.color()).bold().to_string());
    
    // Visual separator with consistent styling
    let separator_char = match group_type {
        FileType::Directory => "━",
        FileType::Source => "─",
        FileType::Config => "┄",
        FileType::Documentation => "┈",
        _ => "─",
    };
    let separator = separator_char.repeat(column_width.saturating_sub(4));
    lines.push(format!("  {}", separator.color(group_type.color()).dimmed()));
    
    // Organize files with sub-grouping for better readability
    let organized_files = organize_files_in_group(files);
    
    for (subgroup_name, subgroup_files) in organized_files {
        // Add subgroup header if there are multiple subgroups
        if !subgroup_name.is_empty() && subgroup_files.len() < files.len() {
            let subgroup_header = format!("  {} {}", "▸".bright_black(), subgroup_name.bright_black());
            lines.push(subgroup_header);
        }
        
        // File entries with enhanced formatting
        for file in &subgroup_files {
            let size_str = if file.is_dir {
                get_directory_size(&file.path)
            } else {
                format_size(file.size)
            };
            
            let (name_color, prefix) = if file.is_dir {
                (Color::BrightBlue, "📁 ")
            } else {
                (group_type.color(), get_file_prefix(&file.name))
            };
            
            // Enhanced format with better spacing and alignment
            let available_name_width = column_width.saturating_sub(12); // More space for prefix and size
            let truncated_name = if file.name.len() > available_name_width {
                format!("{}…", &file.name[..available_name_width.saturating_sub(1)])
            } else {
                file.name.clone()
            };
            
            let file_line = format!(
                "  {} {}{}", 
                size_str.bright_black(),
                prefix.bright_black(),
                truncated_name.color(name_color)
            );
            lines.push(file_line);
        }
        
        // Add spacing between subgroups
        if !subgroup_name.is_empty() && subgroup_files.len() < files.len() {
            lines.push("".to_string());
        }
    }
    
    lines
}

/// Organize files within a group for better visual hierarchy with horizontal splits
fn organize_files_in_group(files: &[FileEntry]) -> Vec<(String, Vec<FileEntry>)> {
    let mut directories = Vec::new();
    let mut regular_files = Vec::new();
    
    // Separate directories and files
    for file in files {
        if file.is_dir {
            directories.push(file.clone());
        } else {
            regular_files.push(file.clone());
        }
    }
    
    let mut result = Vec::new();
    let has_directories = !directories.is_empty();
    
    // Add directories first if present, with horizontal splits for large groups
    if !directories.is_empty() {
        if directories.len() > 8 {
            // Split large directory groups horizontally
            let mid = directories.len() / 2;
            result.push(("Directories (Part 1)".to_string(), directories[..mid].to_vec()));
            result.push(("Directories (Part 2)".to_string(), directories[mid..].to_vec()));
        } else {
            result.push(("Directories".to_string(), directories));
        }
    }
    
    // Add regular files with intelligent horizontal grouping
    if !regular_files.is_empty() {
        let subgroups = create_horizontal_file_splits(&regular_files);
        
        for (subgroup_name, subgroup_files) in subgroups {
            let display_name = if !has_directories && subgroup_name.is_empty() {
                "".to_string() // No header if only one group of files
            } else if subgroup_name.is_empty() {
                "Files".to_string()
            } else {
                format!("Files - {}", subgroup_name)
            };
            result.push((display_name, subgroup_files));
        }
    }
    
    result
}

/// Create horizontal splits for files based on type and size
fn create_horizontal_file_splits(files: &[FileEntry]) -> Vec<(String, Vec<FileEntry>)> {
    if files.len() <= 6 {
        // Small groups don't need splitting
        return vec![("".to_string(), files.to_vec())];
    }
    
    // Group files by extension for better organization
    let mut extension_groups: std::collections::HashMap<String, Vec<FileEntry>> = 
        std::collections::HashMap::new();
    
    for file in files {
        let extension = file.name.split('.').last().unwrap_or("").to_lowercase();
        let group_key = match extension.as_str() {
            "rs" => "Rust".to_string(),
            "js" | "ts" | "jsx" | "tsx" => "JavaScript/TypeScript".to_string(),
            "py" => "Python".to_string(),
            "json" | "toml" | "yaml" | "yml" | "ini" => "Config".to_string(),
            "md" | "txt" | "rst" => "Documentation".to_string(),
            "png" | "jpg" | "jpeg" | "gif" | "svg" => "Images".to_string(),
            "zip" | "tar" | "gz" | "7z" => "Archives".to_string(),
            "" => "No Extension".to_string(),
            _ => "Other".to_string(),
        };
        extension_groups.entry(group_key).or_insert_with(Vec::new).push(file.clone());
    }
    
    let mut result = Vec::new();
    
    // Sort groups by size and importance
    let mut sorted_groups: Vec<(String, Vec<FileEntry>)> = extension_groups.into_iter().collect();
    sorted_groups.sort_by(|a, b| {
        // Prioritize important file types, then by count
        let a_priority = get_extension_priority(&a.0);
        let b_priority = get_extension_priority(&b.0);
        a_priority.cmp(&b_priority).then_with(|| b.1.len().cmp(&a.1.len()))
    });
    
    // Add groups, splitting large ones horizontally if needed
    for (group_name, group_files) in sorted_groups {
        if group_files.len() > 10 {
            // Split very large groups into chunks
            let chunk_size = (group_files.len() + 1) / 2; // Split into roughly 2 parts
            for (i, chunk) in group_files.chunks(chunk_size).enumerate() {
                let chunk_name = if i == 0 {
                    group_name.clone()
                } else {
                    format!("{} (continued)", group_name)
                };
                result.push((chunk_name, chunk.to_vec()));
            }
        } else {
            result.push((group_name, group_files));
        }
    }
    
    // If we only have one group, don't show a subgroup name
    if result.len() == 1 {
        result[0].0 = "".to_string();
    }
    
    result
}

/// Get priority for extension groups (lower = higher priority)
fn get_extension_priority(group_name: &str) -> u8 {
    match group_name {
        "Rust" => 1,
        "JavaScript/TypeScript" => 2,
        "Python" => 3,
        "Config" => 4,
        "Documentation" => 5,
        "Images" => 6,
        "Archives" => 7,
        "No Extension" => 8,
        "Other" => 9,
        _ => 10,
    }
}

/// Get appropriate file prefix icon based on file extension
fn get_file_prefix(filename: &str) -> &'static str {
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    match extension.as_str() {
        "rs" => "🦀 ",
        "js" | "ts" => "📜 ",
        "py" => "🐍 ",
        "json" | "toml" | "yaml" | "yml" => "⚙️ ",
        "md" | "txt" => "📄 ",
        "git" | "gitignore" => "🌿 ",
        "png" | "jpg" | "jpeg" | "gif" => "🖼️ ",
        "zip" | "tar" | "gz" => "📦 ",
        _ => "📋 ",
    }
}

/// Display files in a precise split-screen layout with intelligent grouping
fn display_two_column_layout(groups: HashMap<FileType, Vec<FileEntry>>) {
    let terminal_width = get_terminal_width();
    let separator_width = 3; // " │ "
    let usable_width = terminal_width.saturating_sub(separator_width);
    let column_width = usable_width / 2;
    
    // Sort groups by priority and size
    let mut sorted_groups: Vec<(FileType, Vec<FileEntry>)> = groups.into_iter().collect();
    sorted_groups.sort_by(|a, b| {
        // Prioritize important file types first, then by size
        group_priority(&a.0).cmp(&group_priority(&b.0))
            .then_with(|| b.1.len().cmp(&a.1.len()))
    });
    
    // Generate all content lines with group separators
    let mut all_content_lines = Vec::new();
    for (group_type, files) in &sorted_groups {
        if !files.is_empty() {
            let group_lines = display_group(group_type, files, column_width);
            all_content_lines.extend(group_lines);
            all_content_lines.push("".to_string()); // Separator between groups
        }
    }
    
    // Remove trailing empty line
    if !all_content_lines.is_empty() && all_content_lines.last() == Some(&"".to_string()) {
        all_content_lines.pop();
    }
    
    // Calculate optimal split point for balanced columns
    let total_lines = all_content_lines.len();
    if total_lines == 0 {
        return;
    }
    
    // Split content into two balanced columns using line-by-line distribution
    let (left_lines, right_lines) = split_content_evenly(&all_content_lines);
    
    // Ensure both columns have equal height for clean display
    let max_height = left_lines.len().max(right_lines.len());
    let mut left_padded = left_lines;
    let mut right_padded = right_lines;
    
    while left_padded.len() < max_height {
        left_padded.push("".to_string());
    }
    while right_padded.len() < max_height {
        right_padded.push("".to_string());
    }
    
    // Display the split-screen layout with precise formatting
    display_split_screen(&left_padded, &right_padded, column_width);
}

/// Get priority order for file type groups (lower number = higher priority)
fn group_priority(file_type: &FileType) -> u8 {
    match file_type {
        FileType::Directory => 0,   // Directories first
        FileType::Source => 1,      // Source code
        FileType::Config => 2,      // Configuration files
        FileType::Build => 3,       // Build files
        FileType::Documentation => 4, // Documentation
        FileType::Git => 5,         // Git files
        FileType::Binary => 6,      // Binaries
        FileType::Archive => 7,     // Archives
        FileType::Media => 8,       // Media files
        FileType::Backup => 9,      // Backup files
        FileType::Other => 10,      // Everything else
    }
}

/// Split content evenly between two columns using a line-by-line balancing approach
fn split_content_evenly(lines: &[String]) -> (Vec<String>, Vec<String>) {
    if lines.is_empty() {
        return (Vec::new(), Vec::new());
    }
    
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut left_to_right = true; // Start with left column
    
    // We'll alternate between columns, but with some intelligence about group boundaries
    let mut i = 0;
    while i < lines.len() {
        let current_line = &lines[i];
        
        // Check if this is a group header (has icons and tends to be bold/colored)
        let is_group_header = is_likely_group_header(current_line);
        
        if is_group_header {
            // For group headers, try to keep the whole group together
            let group_end = find_group_end(lines, i);
            let group_size = group_end - i;
            
            // Decide which column to put this group in based on current balance
            let target_column = if left.len() <= right.len() { 
                &mut left 
            } else { 
                &mut right 
            };
            
            // Add the entire group to the target column
            for j in i..group_end {
                target_column.push(lines[j].clone());
            }
            
            i = group_end;
        } else {
            // For non-group content, distribute line by line
            if left_to_right {
                left.push(current_line.clone());
            } else {
                right.push(current_line.clone());
            }
            left_to_right = !left_to_right;
            i += 1;
        }
    }
    
    (left, right)
}

/// Check if a line is likely a group header based on common patterns
fn is_likely_group_header(line: &str) -> bool {
    let clean_line = strip_ansi_codes(line);
    
    // Group headers typically have icons and category names in uppercase
    clean_line.contains("📁") || clean_line.contains("🦀") || clean_line.contains("⚙️") ||
    clean_line.contains("📄") || clean_line.contains("🌿") || clean_line.contains("📋") ||
    (clean_line.contains("(") && clean_line.contains(")") && clean_line.chars().any(|c| c.is_uppercase()))
}

/// Find the end of a group starting at the given index
fn find_group_end(lines: &[String], start: usize) -> usize {
    let mut end = start + 1;
    
    // Look for the next group header or end of content
    while end < lines.len() {
        if is_likely_group_header(&lines[end]) {
            break;
        }
        
        // Stop at empty lines that might be group separators, but include one for spacing
        if lines[end].trim().is_empty() {
            end += 1; // Include the empty line
            break;
        }
        
        end += 1;
    }
    
    end.min(lines.len())
}

/// Display the final split-screen layout with precise terminal formatting
fn display_split_screen(left_lines: &[String], right_lines: &[String], column_width: usize) {
    for (left, right) in left_lines.iter().zip(right_lines.iter()) {
        let left_formatted = format_line_fixed_width(left, column_width);
        let right_formatted = format_line_fixed_width(right, column_width);
        
        // Use a more visible separator with proper spacing
        println!("{} │ {}", left_formatted, right_formatted);
    }
}

/// Format a line to fit exactly within the column width with proper padding
fn format_line_fixed_width(line: &str, width: usize) -> String {
    // Strip ANSI color codes for accurate length calculation
    let clean_line = strip_ansi_codes(line);
    let clean_len = clean_line.len();
    
    if clean_len <= width {
        // Line fits - pad to exact width
        let padding_needed = width - clean_len;
        format!("{}{}", line, " ".repeat(padding_needed))
    } else {
        // Line is too long - truncate with ellipsis, preserving color codes
        let available = width.saturating_sub(1); // Reserve space for ellipsis
        let mut result = String::new();
        let mut char_count = 0;
        let mut in_escape = false;
        let mut last_color_reset = String::new();
        
        for ch in line.chars() {
            if ch == '\x1b' {
                in_escape = true;
                result.push(ch);
                last_color_reset.clear();
                last_color_reset.push(ch);
            } else if in_escape {
                result.push(ch);
                last_color_reset.push(ch);
                if ch == 'm' {
                    in_escape = false;
                }
            } else if char_count < available {
                result.push(ch);
                char_count += 1;
            } else {
                break;
            }
        }
        
        // Add ellipsis and color reset if needed
        result.push('…');
        if !last_color_reset.is_empty() && last_color_reset != "\x1b[0m" {
            result.push_str("\x1b[0m"); // Reset color after truncation
        }
        
        // Pad to exact width - account for the ellipsis
        let final_clean_len = strip_ansi_codes(&result).len();
        if final_clean_len < width {
            let padding_needed = width - final_clean_len;
            result.push_str(&" ".repeat(padding_needed));
        }
        
        result
    }
}

/// Strip ANSI escape sequences for accurate length calculation
fn strip_ansi_codes(input: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;
    
    for ch in input.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Execute the ls command
pub async fn execute(directory: Option<&str>, _config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "Enhanced file listing with intelligent grouping", 
        "📋", 
        "File System", 
        "cyan"
    );
    
    let target_dir = directory.unwrap_or(".");
    let path = Path::new(target_dir);
    
    if !path.exists() {
        return Err(anyhow::anyhow!("Directory does not exist: {}", target_dir));
    }
    
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", target_dir));
    }
    
    // Collect and classify files
    let files = collect_files(path)?;
    
    if files.is_empty() {
        println!("{}", "Directory is empty".bright_black());
        return Ok(());
    }
    
    // Group and display
    let groups = group_and_sort_files(files);
    
    // Print directory header
    println!("{} {}", "Directory:".bold(), path.display().to_string().cyan());
    println!();
    
    display_two_column_layout(groups);
    
    Ok(())
}

/// Create the clap command for the ls subcommand
pub fn create_command() -> clap::Command {
    clap::Command::new("ls")
        .about("Enhanced file listing with intelligent grouping")
        .long_about("Lists files in the current or specified directory, automatically grouped by file type with a two-column layout and color coding.")
        .arg(
            clap::Arg::new("directory")
                .help("Directory to list (defaults to current directory)")
                .value_name("DIR")
                .index(1)
        )
}

/// Execute the ls command with clap matches
pub fn execute_with_matches(matches: &clap::ArgMatches, config: &Config) -> Result<()> {
    let directory = matches.get_one::<String>("directory").map(|s| s.as_str());
    
    tokio::runtime::Runtime::new()?.block_on(execute(directory, config))
}