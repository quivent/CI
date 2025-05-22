//! Progress indicator for CLI applications
//!
//! This module provides a spinning progress indicator for
//! command-line applications.

use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use colored::Colorize;

/// A spinner that shows an animation while a task is in progress
pub struct Spinner {
    message: String,
    active: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// Create a new spinner with a message
    pub fn new(message: &str) -> Self {
        Spinner {
            message: message.to_string(),
            active: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Start the spinner animation
    pub fn start(&mut self) {
        // Only start if not already running
        if self.active.load(Ordering::SeqCst) {
            return;
        }

        // Set the active flag
        self.active.store(true, Ordering::SeqCst);
        
        // Print the message
        print!("{} ", self.message.blue());
        io::stdout().flush().unwrap();

        // Clone the active flag for the thread
        let active = self.active.clone();

        // Start the animation thread
        self.handle = Some(thread::spawn(move || {
            let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']; // Braille pattern spinner
            let mut i = 0;
            
            while active.load(Ordering::SeqCst) {
                // Print the spinner character
                print!("{}", spinner_chars[i].to_string().cyan());
                io::stdout().flush().unwrap();
                
                // Sleep briefly
                thread::sleep(Duration::from_millis(100));
                
                // Clear the spinner character
                print!("\x08");
                io::stdout().flush().unwrap();
                
                // Move to the next character
                i = (i + 1) % spinner_chars.len();
            }
        }));
    }

    /// Stop the spinner animation and show a result
    pub fn stop(&mut self, result: SpinnerResult) {
        // Only stop if running
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        // Set the active flag to false
        self.active.store(false, Ordering::SeqCst);

        // Wait for the animation thread to finish
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        // Print the result symbol
        match result {
            SpinnerResult::Success => println!("{}", "✓".green()),
            SpinnerResult::Failure => println!("{}", "✗".red()),
            SpinnerResult::Warning => println!("{}", "!".yellow()),
            SpinnerResult::Info => println!("{}", "•".blue()),
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        // Ensure the spinner is stopped when dropped
        if self.active.load(Ordering::SeqCst) {
            self.stop(SpinnerResult::Info);
        }
    }
}

/// The result of a task with a spinner
pub enum SpinnerResult {
    /// Task succeeded (green checkmark)
    Success,
    /// Task failed (red cross)
    Failure,
    /// Task completed with warnings (yellow exclamation)
    Warning,
    /// Informational result (blue dot)
    Info,
}

/// Run a function with a spinner showing progress
pub fn with_spinner<F, R>(message: &str, f: F) -> anyhow::Result<R>
where
    F: FnOnce() -> anyhow::Result<R>,
{
    let mut spinner = Spinner::new(message);
    spinner.start();
    
    match f() {
        Ok(result) => {
            spinner.stop(SpinnerResult::Success);
            Ok(result)
        }
        Err(err) => {
            spinner.stop(SpinnerResult::Failure);
            Err(err)
        }
    }
}

/// Run a function and collect progress updates
pub fn with_progress_updates<F, R>(
    message: &str,
    f: F,
    updates: &[&str],
) -> anyhow::Result<R>
where
    F: FnOnce(&dyn Fn(usize)) -> anyhow::Result<R>,
{
    println!("{}", message.blue());
    
    // Create a progress update function
    let update_progress = |step: usize| {
        if step < updates.len() {
            print!("  {} {}", "→".cyan(), updates[step]);
            io::stdout().flush().unwrap();
            println!(" {}", "✓".green());
        }
    };
    
    // Run the function with progress updates
    let result = f(&update_progress);
    
    match &result {
        Ok(_) => println!("{} {}", "✓".green(), "Complete".green()),
        Err(_) => println!("{} {}", "✗".red(), "Failed".red()),
    }
    
    result
}