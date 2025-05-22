// Shared Git Utilities for CI CLI
// Common git operations used across multiple modules

use std::process::{Command, Stdio};

pub fn is_git_repository() -> bool {
    Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn get_current_branch() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err("Failed to get current branch".into());
    }

    let branch = String::from_utf8(output.stdout)?;
    Ok(branch.trim().to_string())
}

pub fn get_repository_root() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err("Failed to get repository root".into());
    }

    let root = String::from_utf8(output.stdout)?;
    Ok(root.trim().to_string())
}