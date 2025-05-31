use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process;
use std::time::Instant;
use tempfile::NamedTempFile;

use crate::errors::CIError;
use crate::helpers::path::get_ci_root;

pub fn create_command() -> Command {
    Command::new("verify")
        .about("Verify Collaborative Intelligence integration and functionality")
        .arg(
            Arg::new("component")
                .help("Specific component to verify")
                .value_parser(["claude", "agents", "memory", "config", "all"])
                .index(1)
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .value_name("SECONDS")
                .default_value("10")
                .help("Timeout for Claude commands (default: 10 seconds)")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Show detailed verification output")
        )
        .arg(
            Arg::new("quick")
                .short('q')
                .long("quick")
                .action(clap::ArgAction::SetTrue)
                .help("Run quick verification (skip intensive tests)")
        )
}

pub fn execute(matches: &ArgMatches) -> Result<()> {
    let component = matches.get_one::<String>("component");
    let timeout: u64 = matches.get_one::<String>("timeout")
        .unwrap()
        .parse()
        .with_context(|| "Invalid timeout value")?;
    let verbose = matches.get_flag("verbose");
    let quick = matches.get_flag("quick");
    
    println!("{}", "Verifying Collaborative Intelligence Integration".cyan().bold());
    println!("{}", "=".repeat(50).cyan());
    
    let ci_root = get_ci_root()?;
    println!("CI Repository: {}", ci_root.display().to_string().dimmed());
    println!();
    
    let start_time = Instant::now();
    let mut all_passed = true;
    
    match component.map(|s| s.as_str()) {
        Some("claude") => {
            all_passed &= verify_claude_cli(timeout, verbose)?;
        }
        Some("agents") => {
            all_passed &= verify_agent_system(verbose)?;
        }
        Some("memory") => {
            all_passed &= verify_memory_system(verbose)?;
        }
        Some("config") => {
            all_passed &= verify_configuration(verbose)?;
        }
        Some("all") | None => {
            all_passed &= verify_configuration(verbose)?;
            all_passed &= verify_claude_cli(timeout, verbose)?;
            all_passed &= verify_agent_system(verbose)?;
            all_passed &= verify_memory_system(verbose)?;
            
            if !quick {
                all_passed &= verify_integration_test(timeout, verbose)?;
            }
        }
        Some(unknown) => {
            return Err(CIError::InvalidInput(format!("Unknown component: {}", unknown)).into());
        }
    }
    
    let duration = start_time.elapsed();
    
    println!();
    println!("{}", "=".repeat(50).cyan());
    
    if all_passed {
        println!("{} All verifications passed!", "✓".green().bold());
        println!("Collaborative Intelligence system is working correctly.");
    } else {
        println!("{} Some verifications failed!", "✗".red().bold());
        println!("Run 'ci fix' to address the issues.");
    }
    
    println!("Verification completed in {:.2}s", duration.as_secs_f64());
    
    if !all_passed {
        std::process::exit(1);
    }
    
    Ok(())
}

fn verify_configuration(verbose: bool) -> Result<bool> {
    println!("{}", "Verifying Configuration...".bold());
    
    let ci_root = get_ci_root()?;
    let mut passed = true;
    
    // Check CLAUDE.md
    let claude_md = ci_root.join("CLAUDE.md");
    if claude_md.exists() {
        print_check_result("CLAUDE.md exists", true, verbose);
        
        // Validate CLAUDE.md content
        if let Ok(content) = fs::read_to_string(&claude_md) {
            let has_ci_config = content.contains("Collaborative Intelligence") || content.contains("CI");
            print_check_result("CLAUDE.md has CI configuration", has_ci_config, verbose);
            passed &= has_ci_config;
        }
    } else {
        print_check_result("CLAUDE.md exists", false, verbose);
        passed = false;
    }
    
    // Check AGENTS directory
    let agents_dir = ci_root.join("AGENTS");
    if agents_dir.exists() && agents_dir.is_dir() {
        print_check_result("AGENTS directory exists", true, verbose);
        
        // Count agents
        let agent_count = count_agents(&agents_dir).unwrap_or(0);
        let has_agents = agent_count > 0;
        print_check_result(&format!("Found {} agents", agent_count), has_agents, verbose);
        passed &= has_agents;
    } else {
        print_check_result("AGENTS directory exists", false, verbose);
        passed = false;
    }
    
    // Check AGENTS.md
    let agents_md = ci_root.join("AGENTS.md");
    if agents_md.exists() {
        print_check_result("AGENTS.md exists", true, verbose);
    } else {
        print_check_result("AGENTS.md exists", false, verbose);
        passed = false;
    }
    
    println!();
    Ok(passed)
}

fn verify_claude_cli(timeout: u64, verbose: bool) -> Result<bool> {
    println!("{}", "Verifying Claude CLI...".bold());
    
    let mut passed = true;
    
    // Check if Claude CLI is installed
    let claude_available = process::Command::new("claude")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    
    print_check_result("Claude CLI installed", claude_available, verbose);
    
    if !claude_available {
        println!();
        return Ok(false);
    }
    
    // Test Claude CLI responsiveness with timeout
    println!("  Testing Claude CLI responsiveness...");
    
    let test_result = test_claude_responsiveness(timeout, verbose)?;
    print_check_result(&format!("Claude CLI responds within {}s", timeout), test_result, verbose);
    passed &= test_result;
    
    println!();
    Ok(passed)
}

fn verify_agent_system(verbose: bool) -> Result<bool> {
    println!("{}", "Verifying Agent System...".bold());
    
    let ci_root = get_ci_root()?;
    let agents_dir = ci_root.join("AGENTS");
    let mut passed = true;
    
    if !agents_dir.exists() {
        print_check_result("AGENTS directory exists", false, verbose);
        println!();
        return Ok(false);
    }
    
    // Check core agents
    let core_agents = ["Athena", "Architect", "Developer", "Debugger"];
    let mut found_core_agents = 0;
    
    for agent in &core_agents {
        let agent_dir = agents_dir.join(agent);
        let agent_exists = agent_dir.exists() && agent_dir.is_dir();
        
        if agent_exists {
            found_core_agents += 1;
            
            // Check agent structure
            let readme_exists = agent_dir.join("README.md").exists();
            let memory_exists = agent_dir.join("MEMORY.md").exists();
            let learning_exists = agent_dir.join("ContinuousLearning.md").exists();
            let sessions_exists = agent_dir.join("Sessions").exists();
            
            if verbose {
                print_check_result(&format!("{} agent exists", agent), true, true);
                print_check_result(&format!("  {} README.md", agent), readme_exists, true);
                print_check_result(&format!("  {} MEMORY.md", agent), memory_exists, true);
                print_check_result(&format!("  {} ContinuousLearning.md", agent), learning_exists, true);
                print_check_result(&format!("  {} Sessions dir", agent), sessions_exists, true);
            }
            
            passed &= readme_exists && memory_exists && learning_exists && sessions_exists;
        } else if verbose {
            print_check_result(&format!("{} agent exists", agent), false, true);
        }
    }
    
    let core_agent_coverage = found_core_agents as f32 / core_agents.len() as f32;
    let has_sufficient_core_agents = core_agent_coverage >= 0.5; // At least 50% of core agents
    
    print_check_result(
        &format!("Core agents available ({}/{})", found_core_agents, core_agents.len()),
        has_sufficient_core_agents,
        verbose
    );
    passed &= has_sufficient_core_agents;
    
    // Check total agent count
    let total_agents = count_agents(&agents_dir).unwrap_or(0);
    let has_agents = total_agents > 0;
    print_check_result(&format!("Total agents: {}", total_agents), has_agents, verbose);
    passed &= has_agents;
    
    println!();
    Ok(passed)
}

fn verify_memory_system(verbose: bool) -> Result<bool> {
    println!("{}", "Verifying Memory System...".bold());
    
    let ci_root = get_ci_root()?;
    let agents_dir = ci_root.join("AGENTS");
    let mut passed = true;
    
    if !agents_dir.exists() {
        print_check_result("AGENTS directory exists", false, verbose);
        println!();
        return Ok(false);
    }
    
    let mut agents_with_memory = 0;
    let mut agents_with_learning = 0;
    let mut total_agents = 0;
    
    if let Ok(entries) = fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                total_agents += 1;
                let agent_path = entry.path();
                
                let memory_file = agent_path.join("MEMORY.md");
                let learning_file = agent_path.join("ContinuousLearning.md");
                
                if memory_file.exists() {
                    agents_with_memory += 1;
                    
                    // Check if memory file has content
                    if verbose {
                        if let Ok(content) = fs::read_to_string(&memory_file) {
                            let has_content = content.len() > 100; // Basic content check
                            print_check_result(
                                &format!("  {} memory has content", entry.file_name().to_string_lossy()),
                                has_content,
                                true
                            );
                        }
                    }
                }
                
                if learning_file.exists() {
                    agents_with_learning += 1;
                }
            }
        }
    }
    
    if total_agents > 0 {
        let memory_coverage = agents_with_memory as f32 / total_agents as f32;
        let learning_coverage = agents_with_learning as f32 / total_agents as f32;
        
        let good_memory_coverage = memory_coverage >= 0.8;
        let good_learning_coverage = learning_coverage >= 0.8;
        
        print_check_result(
            &format!("Memory files present ({}/{})", agents_with_memory, total_agents),
            good_memory_coverage,
            verbose
        );
        print_check_result(
            &format!("Learning files present ({}/{})", agents_with_learning, total_agents),
            good_learning_coverage,
            verbose
        );
        
        passed &= good_memory_coverage;
        passed &= good_learning_coverage;
    } else {
        print_check_result("No agents found", false, verbose);
        passed = false;
    }
    
    println!();
    Ok(passed)
}

fn verify_integration_test(timeout: u64, verbose: bool) -> Result<bool> {
    println!("{}", "Running Integration Test...".bold());
    
    let test_passed = run_claude_integration_test(timeout, verbose)?;
    print_check_result("Claude Code integration test", test_passed, verbose);
    
    println!();
    Ok(test_passed)
}

fn test_claude_responsiveness(timeout: u64, verbose: bool) -> Result<bool> {
    // Create a simple test file
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "Hello Claude, please respond with 'OK' if you can see this.")?;
    
    let start_time = Instant::now();
    
    // Run Claude with timeout
    let output = process::Command::new("timeout")
        .arg(format!("{}s", timeout))
        .arg("claude")
        .stdin(fs::File::open(temp_file.path())?)
        .output();
    
    let elapsed = start_time.elapsed();
    
    if verbose {
        println!("    Claude response time: {:.2}s", elapsed.as_secs_f64());
    }
    
    match output {
        Ok(output) if output.status.success() => {
            let response = String::from_utf8_lossy(&output.stdout);
            let responded = !response.trim().is_empty();
            
            if verbose && responded {
                println!("    Claude responded: {}", response.trim().chars().take(50).collect::<String>());
                if response.len() > 50 {
                    println!("    ... (truncated)");
                }
            }
            
            Ok(responded)
        }
        Ok(output) => {
            if verbose {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.trim().is_empty() {
                    println!("    Claude error: {}", stderr.trim());
                }
            }
            Ok(false)
        }
        Err(e) => {
            if verbose {
                println!("    Failed to run Claude: {}", e);
            }
            Ok(false)
        }
    }
}

fn run_claude_integration_test(timeout: u64, verbose: bool) -> Result<bool> {
    // Create integration test file
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "Hello Athena, confirm that the Collaborative Intelligence system is integrated.")?;
    writeln!(temp_file, "Please respond with 'INTEGRATION_VERIFIED' if successful.")?;
    
    if verbose {
        println!("    Running Claude integration test...");
    }
    
    let start_time = Instant::now();
    
    // Run Claude with the test input
    let output = process::Command::new("timeout")
        .arg(format!("{}s", timeout))
        .arg("claude")
        .stdin(fs::File::open(temp_file.path())?)
        .output();
    
    let elapsed = start_time.elapsed();
    
    if verbose {
        println!("    Integration test time: {:.2}s", elapsed.as_secs_f64());
    }
    
    match output {
        Ok(output) if output.status.success() => {
            let response = String::from_utf8_lossy(&output.stdout);
            
            // Check for integration verification
            let has_verification = response.contains("INTEGRATION_VERIFIED");
            let has_athena_response = response.contains("[Athena]") || response.contains("Athena");
            
            if verbose {
                if has_verification {
                    println!("    ✓ Found integration verification marker");
                }
                if has_athena_response {
                    println!("    ✓ Athena agent responded");
                }
                
                if !response.trim().is_empty() {
                    println!("    Response preview: {}", 
                        response.trim().lines().next().unwrap_or("").chars().take(60).collect::<String>());
                }
            }
            
            Ok(has_verification || has_athena_response)
        }
        Ok(output) => {
            if verbose {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("    Integration test failed with exit code: {}", output.status);
                if !stderr.trim().is_empty() {
                    println!("    Error: {}", stderr.trim());
                }
            }
            Ok(false)
        }
        Err(e) => {
            if verbose {
                println!("    Failed to run integration test: {}", e);
            }
            Ok(false)
        }
    }
}

fn count_agents(agents_dir: &Path) -> Result<usize> {
    let mut count = 0;
    
    if let Ok(entries) = fs::read_dir(agents_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    if !name_str.starts_with('.') {
                        count += 1;
                    }
                }
            }
        }
    }
    
    Ok(count)
}

fn print_check_result(description: &str, passed: bool, verbose: bool) {
    if verbose || !passed {
        let symbol = if passed { "✓".green() } else { "✗".red() };
        let status = if passed { "PASS".green() } else { "FAIL".red() };
        println!("  {} {} [{}]", symbol, description, status);
    } else {
        let symbol = "✓".green();
        println!("  {} {}", symbol, description);
    }
}