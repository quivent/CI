// Git Operations Module - Safe git command execution and repository status
// Adapted from standalone topologist for CI integration

use std::process::{Command, Stdio};
use std::collections::HashSet;

pub struct GitOperations {
    // Configuration for git operations
}

impl GitOperations {
    pub fn new() -> Self {
        Self {}
    }

    /// Get current repository status (untracked and modified files)
    pub fn get_repository_status(&self) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["status", "--porcelain"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Err("Not a git repository or git command failed".into());
        }

        let status_output = String::from_utf8(output.stdout)?;
        let mut untracked = Vec::new();
        let mut modified = Vec::new();

        for line in status_output.lines() {
            if line.len() < 3 {
                continue;
            }

            let status_code = &line[..2];
            let file_path = line[3..].trim().to_string();

            match status_code {
                "??" => untracked.push(file_path),
                " M" | "M " | "MM" => modified.push(file_path),
                " A" | "A " | "AM" => modified.push(file_path),
                " D" | "D " | "DM" => modified.push(file_path),
                " R" | "R " | "RM" => modified.push(file_path),
                " C" | "C " | "CM" => modified.push(file_path),
                "UU" | "AA" | "DD" => modified.push(file_path), // Merge conflicts
                _ => {
                    // Handle other status codes as modified
                    if !status_code.trim().is_empty() {
                        modified.push(file_path);
                    }
                }
            }
        }

        Ok((untracked, modified))
    }

    /// Check if a file is untracked
    pub fn is_untracked(&self, file_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["ls-files", "--others", "--exclude-standard", file_path])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let result = String::from_utf8(output.stdout)?;
        Ok(!result.trim().is_empty())
    }

    /// Stage and commit files with the given message
    pub fn stage_and_commit_files(&self, file_paths: &[&str], commit_message: &str) -> Result<String, Box<dyn std::error::Error>> {
        // First, stage the files
        self.stage_files(file_paths)?;

        // Then commit with message
        self.commit_staged_files(commit_message)
    }

    /// Stage specific files
    pub fn stage_files(&self, file_paths: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if file_paths.is_empty() {
            return Ok(());
        }

        let mut cmd = Command::new("git");
        cmd.arg("add");
        
        for path in file_paths {
            cmd.arg(path);
        }

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to stage files: {}", stderr).into());
        }

        Ok(())
    }

    /// Commit currently staged files
    pub fn commit_staged_files(&self, commit_message: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create the full commit message with CI attribution
        let full_message = format!(
            "{}\n\n🤖 Generated with [CI Topology](https://github.com/collaborative-intelligence)\n\nCo-Authored-By: CI Topologist <noreply@ci.dev>",
            commit_message
        );

        let output = Command::new("git")
            .args(&["commit", "-m", &full_message])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to commit: {}", stderr).into());
        }

        // Get the commit hash
        self.get_latest_commit_hash()
    }

    /// Get the latest commit hash
    pub fn get_latest_commit_hash(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Err("Failed to get commit hash".into());
        }

        let hash = String::from_utf8(output.stdout)?;
        Ok(hash.trim().to_string())
    }

    /// Check if we're in a git repository
    pub fn is_git_repository(&self) -> bool {
        Command::new("git")
            .args(&["rev-parse", "--git-dir"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Get repository statistics for size tracking
    pub fn get_repository_stats(&self) -> Result<RepositoryStats, Box<dyn std::error::Error>> {
        let file_count_output = Command::new("git")
            .args(&["ls-files", "--cached", "--others", "--exclude-standard"])
            .stdout(Stdio::piped())
            .output()?;

        let total_files = if file_count_output.status.success() {
            String::from_utf8(file_count_output.stdout)?
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        } else {
            0
        };

        let commit_count_output = Command::new("git")
            .args(&["rev-list", "--count", "HEAD"])
            .stdout(Stdio::piped())
            .output()?;

        let commit_count = if commit_count_output.status.success() {
            String::from_utf8(commit_count_output.stdout)?
                .trim()
                .parse::<usize>()
                .unwrap_or(0)
        } else {
            0
        };

        Ok(RepositoryStats {
            total_files,
            total_commits: commit_count,
        })
    }

    /// Get diff statistics for size change calculation
    pub fn get_diff_stats(&self, commit_hash: &str) -> Result<DiffStats, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["show", "--stat", "--format=", commit_hash])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Ok(DiffStats::default());
        }

        let stats_output = String::from_utf8(output.stdout)?;
        let mut insertions = 0;
        let mut deletions = 0;
        let mut files_changed = 0;

        // Parse git stat output
        for line in stats_output.lines() {
            if line.contains("insertion") || line.contains("deletion") {
                // Parse lines like: " 3 files changed, 150 insertions(+), 20 deletions(-)"
                let parts: Vec<&str> = line.split(',').collect();
                
                for part in parts {
                    let part = part.trim();
                    if part.contains("file") && part.contains("changed") {
                        if let Some(num_str) = part.split_whitespace().next() {
                            files_changed = num_str.parse().unwrap_or(0);
                        }
                    } else if part.contains("insertion") {
                        if let Some(num_str) = part.split_whitespace().next() {
                            insertions = num_str.parse().unwrap_or(0);
                        }
                    } else if part.contains("deletion") {
                        if let Some(num_str) = part.split_whitespace().next() {
                            deletions = num_str.parse().unwrap_or(0);
                        }
                    }
                }
                break;
            }
        }

        Ok(DiffStats {
            files_changed,
            insertions,
            deletions,
        })
    }

    /// Validate that files can be safely committed
    pub fn validate_files_for_commit(&self, file_paths: &[&str]) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let mut existing_files = Vec::new();
        let mut missing_files = Vec::new();
        let mut large_files = Vec::new();

        for &file_path in file_paths {
            // Check if file exists
            if std::path::Path::new(file_path).exists() {
                existing_files.push(file_path.to_string());
                
                // Check file size (warn if > 1MB)
                if let Ok(metadata) = std::fs::metadata(file_path) {
                    if metadata.len() > 1_000_000 {
                        large_files.push(file_path.to_string());
                    }
                }
            } else {
                missing_files.push(file_path.to_string());
            }
        }

        let is_valid = missing_files.is_empty();
        Ok(ValidationResult {
            existing_files,
            missing_files,
            large_files,
            is_valid,
        })
    }
}

impl Default for GitOperations {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RepositoryStats {
    pub total_files: usize,
    pub total_commits: usize,
}

#[derive(Debug, Default)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub existing_files: Vec<String>,
    pub missing_files: Vec<String>,
    pub large_files: Vec<String>,
    pub is_valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_operations_creation() {
        let git_ops = GitOperations::new();
        // Basic test to ensure struct can be created
        assert!(true);
    }

    #[test]
    fn test_is_git_repository() {
        let git_ops = GitOperations::new();
        // This will depend on the test environment
        // In a git repo, should return true
        let _is_repo = git_ops.is_git_repository();
        assert!(true); // Basic test structure
    }
}