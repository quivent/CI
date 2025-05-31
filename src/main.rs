//! CI - Command-line interface for Collaborative Intelligence
//!
//! Modern implementation of the Collaborative Intelligence CLI with enhanced features and categorized commands.

use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
// use tempfile;

mod version;

mod commands {
    pub mod intelligence;
    pub mod source_control;
    pub mod lifecycle;
    pub mod system;
    pub mod legacy;
    pub mod idea;
    pub mod config;
    pub mod detach;
    pub mod topology;
    pub mod agents;
    pub mod init;
    pub mod fix;
    pub mod verify;
    pub mod session;
    pub mod visualize;
    pub mod ls;
}

pub mod helpers;
pub mod tools;
pub mod errors;
pub mod topology;
pub mod shared;

use crate::helpers::agent_autoload::AgentAutoload;

// Legacy error module - kept for backward compatibility
// New code should use the errors module
#[deprecated(since = "1.1.0", note = "Use the errors module instead")]
mod error {
    use thiserror::Error;
    
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("Failed to load configuration: {0}")]
        ConfigError(String),
        
        #[error("IO error: {0}")]
        IoError(#[from] std::io::Error),
    }
    
    // Conversion from errors::CIError to legacy Error
    impl From<crate::errors::CIError> for Error {
        fn from(err: crate::errors::CIError) -> Self {
            match err {
                crate::errors::CIError::Config(s) => Error::ConfigError(s),
                crate::errors::CIError::IO(e) => Error::IoError(e),
                _ => Error::ConfigError(format!("Error: {}", err)),
            }
        }
    }
    
    // Conversion from legacy Error to errors::CIError
    impl From<Error> for crate::errors::CIError {
        fn from(err: Error) -> Self {
            match err {
                Error::ConfigError(s) => crate::errors::CIError::Config(s),
                Error::IoError(e) => crate::errors::CIError::IO(e),
            }
        }
    }
}

mod config;

#[derive(Parser)]
#[command(
    name = "CI",
    about = "Collaborative Intelligence CLI",
    long_about = "A modern command-line interface for the Collaborative Intelligence system",
    version,
    disable_version_flag(true)
)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,
    
    /// Display version information
    #[arg(short = 'V', long = "version")]
    show_version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    //
    // Intelligence & Discovery Commands
    //
    
    /// Display the intent and purpose of the CI tool
    Intent,
    
    /// List all available Collaborative Intelligence agents
    Agents,
    
    /// Enhanced agent management with full lifecycle operations
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    
    /// Start a Claude Code session with a specified agent loaded
    Load {
        /// Agent name
        agent: String,
        
        /// Context for memory loading
        #[arg(short, long)]
        context: Option<String>,
        
        /// Memory path to load (defaults to standard agent memory)
        #[arg(short = 'f', long)]
        path: Option<PathBuf>,
        
        /// Prompt before launching Claude Code (overrides default auto-launch)
        #[arg(short, long)]
        prompt: bool,
    },
    
    /// Start a Claude Code session with adaptive memory from CLAUDE.adaptation.md
    Adapt {
        /// Path to target directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// List projects integrated with Collaborative Intelligence
    Projects,
    
    /// Visualize CI architecture, commands, agents, and workflows
    /// 
    /// Quick shortcuts:
    ///   --web --dark     # Interactive web view (auto-cleanup)
    ///   --svg --light    # SVG diagram (auto-cleanup)  
    ///   --web --save     # Save files permanently
    Visualize {
        #[command(subcommand)]
        view: VisualizationCommands,
    },
    
    /// Manage ideas, concepts, and inspirations
    Idea {
        /// Subcommand (list, add, view, update, delete, categories, tags)
        subcommand: String,
        
        /// Idea title (for add, update)
        #[arg(short, long)]
        title: Option<String>,
        
        /// Idea description (for add, update)
        #[arg(short, long)]
        description: Option<String>,
        
        /// Idea category (for add, update, list)
        #[arg(short, long)]
        category: Option<String>,
        
        /// Comma-separated tags (for add, update)
        #[arg(short, long)]
        tags: Option<String>,
        
        /// Idea ID (for view, update, delete)
        #[arg(short, long)]
        id: Option<String>,
        
        /// Idea status (for update, list)
        #[arg(short, long)]
        status: Option<String>,
        
        /// Idea priority (for update)
        #[arg(short, long)]
        priority: Option<String>,
        
        /// Filter for list operation
        #[arg(short, long)]
        filter: Option<String>,
    },
    
    //
    // Source Control Commands
    //
    
    /// Display detailed status of the git repository and working tree
    Status,
    
    /// Display extremely detailed status with CI integration diagnostics
    StatusDetailed {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Include system information
        #[arg(short, long)]
        system: bool,
        
        /// Include agent details
        #[arg(short, long)]
        agents: bool,
    },
    
    /// Manage GitHub repositories using gh CLI
    Repo {
        #[command(subcommand)]
        command: Option<RepoCommands>,
    },
    
    /// Clean build artifacts from the project
    Clean,
    
    /// Update .gitignore with appropriate patterns for CI
    Ignore,
    
    /// Run ignore and then stage all untracked and unstaged files
    Stage,
    
    /// Configure git remotes for personal and organizational repositories
    Remotes,
    
    /// Run ignore, stage files, analyze changes, and commit with a detailed message
    Commit {
        /// Commit message (optional, will prompt if not provided)
        #[arg(short, long)]
        message: Option<String>,
    },
    
    /// Run ignore, stage, commit, and push in one operation
    Deploy,
    
    //
    // Project Lifecycle Commands
    //
    
    /// Initialize a project with CI
    Init {
        /// Project name
        project_name: String,

        /// Comma-separated list of agents
        #[arg(long, default_value = "Athena,ProjectArchitect")]
        agents: String,

        /// Disable fast activation
        #[arg(long)]
        no_fast: bool,

        // Hidden parameters kept for backward compatibility
        #[arg(long, hide = true)]
        integration: Option<String>,
        
        #[arg(long, hide = true)]
        ci_path: Option<PathBuf>,
    },
    
    /// Integrate CI into an existing project
    Integrate {
        /// Path to target directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Comma-separated list of agents
        #[arg(long, default_value = "Athena,ProjectArchitect")]
        agents: String,

        /// Disable fast activation
        #[arg(long)]
        no_fast: bool,

        /// Integration type (standalone, override)
        #[arg(long, default_value = "standalone")]
        integration: Option<String>,
        
        #[arg(long, hide = true)]
        ci_path: Option<PathBuf>,
    },
    
    /// Detach CI integration but keep configuration
    Detach {
        /// Path to project
        #[arg(default_value = ".")]
        path: Option<String>,
        
        /// Keep override file (rename to .bak)
        #[arg(long)]
        keep_override: bool,
    },
    
    /// Repair CI integration issues
    Fix {
        /// Path to target directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Fix type: verify
        fix_type: Option<String>,
    },
    
    /// Verify CI integration is working properly
    Verify {
        /// Path to target directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// Create or update CLAUDE.local.md file for local CI configuration
    Local {
        /// Path to target directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// Detect and migrate from legacy CI to CI standalone mode
    Migrate {
        /// Path to target directory
        #[arg(default_value = ".")]
        path: PathBuf,
        
        /// Create backups of existing files
        #[arg(long, default_value = "true")]
        backup: bool,
        
        /// Only detect CI integration, don't migrate
        #[arg(long)]
        detect_only: bool,
        
        /// Display verbose output
        #[arg(long)]
        verbose: bool,
    },
    
    //
    // System Management Commands
    //
    
    /// Evolve the CI tool with Claude Code assistance
    Evolve,
    
    /// Manage API keys for external services
    Key {
        #[command(subcommand)]
        command: Option<KeyCommands>,
    },
    
    /// Rebuild the CI binary after code changes
    Rebuild,
    
    /// Install CI tool to system path
    Install,
    
    /// Create symlinks to the CI binary in system paths
    Link,
    
    /// Remove symlinks to the CI binary from system paths
    Unlink,
    
    /// Create legacy command symlinks for CI compatibility
    Legacy {
        /// Create symlinks for legacy commands
        #[arg(short, long)]
        create: bool,
        
        /// Remove symlinks for legacy commands
        #[arg(short, long)]
        remove: bool,
        
        /// List available legacy commands
        #[arg(short, long)]
        list: bool,
    },
    
    /// Generate comprehensive documentation
    Docs,
    
    /// Fix common compiler warnings
    FixWarnings,
    
    /// Add a new command to CI
    AddCommand {
        /// Name of the command to create
        #[arg(value_name = "NAME")]
        name: Option<String>,
        
        /// Description of the command
        #[arg(value_name = "DESCRIPTION")]
        description: Option<String>,
        
        /// Command category (Intelligence, SourceControl, Lifecycle, System)
        #[arg(value_name = "CATEGORY")]
        category: Option<String>,
    },
    
    /// Manage CI commands (create, edit, list)
    Command {
        /// Subcommand to execute (create, edit, list)
        subcommand: String,
    },
    
    /// Print version information
    Version,
    
    /// Test window title functionality (debug command)
    #[command(hide = true)]
    TestWindowTitle,
    
    //
    // Topology Management Commands
    //
    
    /// Repository topology management and intelligent commit organization
    #[command(subcommand)]
    Topologist(crate::commands::topology::TopologyCommands),
    
    /// Manage Collaborative Intelligence sessions
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
    
    /// Enhanced file listing with intelligent grouping
    /// 
    /// Lists files in the current or specified directory, automatically grouped
    /// by file type with a two-column layout and color coding. Files are sorted
    /// alphabetically within each group, with larger groups prioritized at the top.
    /// 
    /// Examples:
    ///   ci ls                   # List files in current directory
    ///   ci ls /path/to/dir      # List files in specified directory
    Ls {
        /// Directory to list (defaults to current directory)
        #[arg(value_name = "DIR")]
        directory: Option<String>,
    },

    /// Manage CI configuration files and settings
    /// 
    /// The config command provides a complete system for managing project
    /// configuration, including creating, reading, updating, and displaying
    /// configuration values. CI uses a simple JSON-based configuration format
    /// that is designed to be human-readable and easily editable.
    /// 
    /// Examples:
    ///   ci config init                    # Initialize config with defaults
    ///   ci config init --agents=A,B,C     # Initialize with specific agents
    ///   ci config get --key=active_agents # Get a specific config value
    ///   ci config set --key=fast_activation --value=true  # Update a value
    ///   ci config show                    # Display all configuration values
    ///   ci config show --format=json      # Show config in JSON format
    Config {
        /// Subcommand to execute (init, get, set, show)
        ///
        /// - init: Create a new configuration file
        /// - get: Retrieve a specific configuration value
        /// - set: Update a configuration value
        /// - show: Display all configuration values
        subcommand: String,
        
        /// Path to target directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        
        /// Project name for new configuration (for init)
        /// 
        /// If not specified, the directory name will be used
        #[arg(long)]
        project_name: Option<String>,
        
        /// Comma-separated list of agents to activate (for init)
        /// 
        /// Example: --agents=Athena,ProjectArchitect,Developer
        /// Defaults to Athena,ProjectArchitect if not specified
        #[arg(long)]
        agents: Option<String>,
        
        /// Enable fast activation mode (for init)
        /// 
        /// Fast activation speeds up agent loading by caching agent memory
        /// Default is true if not specified
        #[arg(long)]
        fast: Option<bool>,
        
        /// Configuration key to get or set
        /// 
        /// Common keys: project_name, active_agents, fast_activation
        /// Custom metadata can be accessed with any key name
        #[arg(long)]
        key: Option<String>,
        
        /// Configuration value to set
        /// 
        /// For boolean values, use "true" or "false"
        /// For arrays, use comma-separated values (e.g., "value1,value2,value3")
        /// JSON values can be used for complex settings
        #[arg(long)]
        value: Option<String>,
        
        /// Output format for the show command (text or json)
        /// 
        /// - text: Human-readable formatted output (default)
        /// - json: Machine-readable JSON format
        #[arg(long, default_value = "text")]
        format: Option<String>,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    /// List repositories
    List,
    
    /// Create a new repository
    Create {
        /// Repository name
        name: String,
        
        /// Repository description
        #[arg(short, long)]
        description: Option<String>,
        
        /// Make repository private
        #[arg(long)]
        private: bool,
    },
    
    /// Clone a repository
    Clone {
        /// Repository URL or shorthand (owner/repo)
        repo: String,
        
        /// Directory to clone into (defaults to repo name)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    
    /// View repository details
    View {
        /// Repository name or URL
        repo: String,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    /// List all stored API keys
    List,
    
    /// Add a new API key
    Add {
        /// Service name (e.g., openai, anthropic)
        service: String,
        
        /// Key name (e.g., api_key, access_token)
        key_name: String,
        
        /// Key value
        key_value: String,
        
        /// Environment (optional, e.g., dev, prod)
        #[arg(short, long)]
        env: Option<String>,
        
        /// Store key in project-specific config
        #[arg(short, long)]
        project: bool,
    },
    
    /// Remove an API key
    Remove {
        /// Service name
        service: String,
        
        /// Key name
        key_name: String,
        
        /// Environment (optional)
        #[arg(short, long)]
        env: Option<String>,
    },
    
    /// Export API keys for shell environment
    Export,
}

#[derive(Subcommand)]
enum AgentCommands {
    /// List available agents
    List {
        /// Show only enabled agents
        #[arg(long)]
        enabled_only: bool,
        
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show detailed information about an agent
    Info {
        /// Name of the agent
        agent_name: String,
    },
    
    /// Create a new agent
    Create {
        /// Name of the new agent
        agent_name: String,
        
        /// Create from template
        #[arg(short, long)]
        template: Option<String>,
        
        /// Enable the agent after creation
        #[arg(long)]
        enable: bool,
    },
    
    /// Enable an agent in the current project
    Enable {
        /// Name of the agent to enable
        agent_name: String,
    },
    
    /// Disable an agent in the current project
    Disable {
        /// Name of the agent to disable
        agent_name: String,
    },
    
    /// Activate an agent for the current session
    Activate {
        /// Name of the agent to activate
        agent_name: String,
        
        /// Context for agent activation
        context: Option<String>,
    },
    
    /// Load agent memory and capabilities
    Load {
        /// Name of the agent to load
        agent_name: String,
    },
    
    /// Create an agent from a template
    Template {
        /// Name of the template
        template_name: String,
        
        /// Name for the new agent
        agent_name: Option<String>,
    },
    
    /// Deploy CI tool globally with latest changes
    Deploy {
        /// Force deployment even if no changes detected
        #[arg(long)]
        force: bool,
        
        /// Create backup of existing global binary
        #[arg(long)]
        backup: bool,
    },
}

#[derive(Subcommand)]
enum VisualizationCommands {
    /// Display CI ecosystem architecture overview
    /// 
    /// Examples:
    ///   ci visualize overview --web --dark      # Interactive web view (temp file)
    ///   ci visualize overview --svg --light     # SVG diagram (temp file)
    ///   ci visualize overview --web --save      # Save web file permanently
    ///   ci visualize overview                   # Terminal view
    Overview {
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<VisualizationFormat>,
        
        /// Visual theme
        #[arg(short, long, value_enum)]
        theme: Option<VisualizationTheme>,
        
        /// Enable interactive mode
        #[arg(short, long)]
        interactive: bool,
        
        /// Export to specific file
        #[arg(short, long)]
        export: Option<String>,
        
        /// Save temp files instead of auto-deleting
        #[arg(long)]
        save: bool,
        
        /// Use web format (shortcut for --format web)
        #[arg(long)]
        web: bool,
        
        /// Use SVG format (shortcut for --format svg)
        #[arg(long)]
        svg: bool,
        
        /// Use dark theme (shortcut for --theme dark)
        #[arg(long)]
        dark: bool,
        
        /// Use light theme (shortcut for --theme light)
        #[arg(long)]
        light: bool,
    },
    
    /// Show command hierarchy and relationships
    /// 
    /// Examples:
    ///   ci visualize commands --web --tree     # Interactive command tree
    ///   ci visualize commands --svg --light    # SVG command diagram
    ///   ci visualize commands --tree           # Terminal tree view
    Commands {
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<VisualizationFormat>,
        
        /// Filter by command group
        #[arg(short, long)]
        group: Option<String>,
        
        /// Show command tree structure
        #[arg(short, long)]
        tree: bool,
        
        /// Enable interactive navigation
        #[arg(short, long)]
        interactive: bool,
        
        /// Save temp files instead of auto-deleting
        #[arg(long)]
        save: bool,
        
        /// Use web format (shortcut for --format web)
        #[arg(long)]
        web: bool,
        
        /// Use SVG format (shortcut for --format svg)
        #[arg(long)]
        svg: bool,
        
        /// Use dark theme (shortcut for --theme dark)
        #[arg(long)]
        dark: bool,
        
        /// Use light theme (shortcut for --theme light)
        #[arg(long)]
        light: bool,
    },
    
    /// Visualize agent network and relationships
    /// 
    /// Examples:
    ///   ci visualize agents --web --network    # Interactive agent network
    ///   ci visualize agents --svg --dark       # SVG agent diagram
    ///   ci visualize agents --network          # Terminal network view
    Agents {
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<VisualizationFormat>,
        
        /// Filter by agent category
        #[arg(short, long)]
        category: Option<String>,
        
        /// Show as network graph
        #[arg(short, long)]
        network: bool,
        
        /// Enable interactive exploration
        #[arg(short, long)]
        interactive: bool,
        
        /// Save temp files instead of auto-deleting
        #[arg(long)]
        save: bool,
        
        /// Use web format (shortcut for --format web)
        #[arg(long)]
        web: bool,
        
        /// Use SVG format (shortcut for --format svg)
        #[arg(long)]
        svg: bool,
        
        /// Use dark theme (shortcut for --theme dark)
        #[arg(long)]
        dark: bool,
        
        /// Use light theme (shortcut for --theme light)
        #[arg(long)]
        light: bool,
    },
    
    /// Display common workflow patterns
    /// 
    /// Examples:
    ///   ci visualize workflows --web --beginner  # Interactive beginner guide
    ///   ci visualize workflows --svg --light     # SVG workflow diagram
    ///   ci visualize workflows --beginner        # Terminal beginner view
    Workflows {
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<VisualizationFormat>,
        
        /// Filter workflows for beginners
        #[arg(short, long)]
        beginner: bool,
        
        /// Show specific workflow category
        #[arg(short, long)]
        category: Option<String>,
        
        /// Save temp files instead of auto-deleting
        #[arg(long)]
        save: bool,
        
        /// Use web format (shortcut for --format web)
        #[arg(long)]
        web: bool,
        
        /// Use SVG format (shortcut for --format svg)
        #[arg(long)]
        svg: bool,
        
        /// Use dark theme (shortcut for --theme dark)
        #[arg(long)]
        dark: bool,
        
        /// Use light theme (shortcut for --theme light)
        #[arg(long)]
        light: bool,
    },
    
    /// Show project-specific CI integration
    /// 
    /// Examples:
    ///   ci visualize project --web --detailed   # Interactive project analysis
    ///   ci visualize project --svg --dark       # SVG project diagram
    ///   ci visualize project --detailed         # Terminal detailed view
    Project {
        /// Project name (defaults to current project)
        name: Option<String>,
        
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<VisualizationFormat>,
        
        /// Show detailed integration status
        #[arg(short, long)]
        detailed: bool,
        
        /// Save temp files instead of auto-deleting
        #[arg(long)]
        save: bool,
        
        /// Use web format (shortcut for --format web)
        #[arg(long)]
        web: bool,
        
        /// Use SVG format (shortcut for --format svg)
        #[arg(long)]
        svg: bool,
        
        /// Use dark theme (shortcut for --theme dark)
        #[arg(long)]
        dark: bool,
        
        /// Use light theme (shortcut for --theme light)
        #[arg(long)]
        light: bool,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum VisualizationFormat {
    /// Terminal-native ASCII art with colors
    Terminal,
    /// Interactive web-based visualization
    Web,
    /// Static SVG diagram
    Svg,
    /// Mermaid diagram format
    Mermaid,
    /// Auto-detect best available format
    Auto,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum VisualizationTheme {
    /// Dark theme optimized for dark terminals
    Dark,
    /// Light theme for light backgrounds
    Light,
    /// High contrast for accessibility
    Contrast,
    /// Terminal-native colors
    Terminal,
}

#[derive(Subcommand)]
enum SessionCommands {
    /// List sessions
    List {
        /// Filter sessions by agent
        #[arg(short, long)]
        agent: Option<String>,
        
        /// Filter sessions by status
        #[arg(short, long)]
        status: Option<String>,
        
        /// Show only recent sessions
        #[arg(short, long, default_value = "10")]
        recent: String,
    },
    
    /// Create a new session
    Create {
        /// Name of the agent for this session
        agent_name: String,
        
        /// Name of the session
        session_name: String,
        
        /// Session description
        #[arg(short, long)]
        description: Option<String>,
        
        /// Comma-separated tags
        #[arg(short, long)]
        tags: Option<String>,
    },
    
    /// Show session information
    Info {
        /// Name of the agent
        agent_name: String,
        
        /// Name of the session
        session_name: String,
    },
    
    /// Archive a session
    Archive {
        /// Name of the agent
        agent_name: String,
        
        /// Name of the session
        session_name: String,
    },
    
    /// Clean up old sessions
    Cleanup {
        /// Archive sessions older than N days
        #[arg(short, long, default_value = "30")]
        days: String,
        
        /// Show what would be cleaned up without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

/// Returns colored category headers and commands for help output
fn get_colored_command_help() -> String {
    let mut help_text = String::new();
    
    // Header
    help_text.push_str("Commands:\n");
    help_text.push_str("\n");
    
    // System Management category (Blue)
    help_text.push_str(&format!("  {}", "┌─────────────────────────────────────────────────────────────────┐".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}  {}", "│".blue(), "🧠 System Management".blue().bold()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}", "└─────────────────────────────────────────────────────────────────┘".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "evolve".blue(), "Evolve the CI tool with Claude Code assistance".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "key".blue(), "Manage API keys for external services".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "rebuild".blue(), "Rebuild the CI binary after code changes".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "install".blue(), "Install CI tool to system path".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "link".blue(), "Create symlinks to the CI binary in system paths".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "unlink".blue(), "Remove symlinks to the CI binary from system paths".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "legacy".blue(), "Create legacy command symlinks for CI compatibility".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "docs".blue(), "Generate comprehensive documentation".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "command".blue(), "Manage CI commands (create, edit, list)".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "version".blue(), "Print version information".blue()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "config".blue(), "Manage CI configuration".blue()));
    help_text.push_str("\n\n");
    
    // Source Control category (Green)
    help_text.push_str(&format!("  {}", "┌─────────────────────────────────────────────────────────────────┐".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}  {}", "│".green(), "📊 Source Control".green().bold()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}", "└─────────────────────────────────────────────────────────────────┘".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "status".green(), "Display detailed status of the git repository and working tree".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "status-detailed".green(), "Display comprehensive status with CI integration diagnostics".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "repo".green(), "Manage GitHub repositories using gh CLI".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "clean".green(), "Clean build artifacts from the project".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "ignore".green(), "Update .gitignore with appropriate patterns for CI".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "stage".green(), "Run ignore and then stage all untracked and unstaged files".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "remotes".green(), "Configure git remotes for personal and organizational repositories".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "commit".green(), "Run ignore, stage files, analyze changes, and commit with a detailed message".green()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "deploy".green(), "Run ignore, stage, commit, and push in one operation".green()));
    help_text.push_str("\n\n");
    
    // Project Lifecycle category (Yellow)
    help_text.push_str(&format!("  {}", "┌─────────────────────────────────────────────────────────────────┐".yellow()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}  {}", "│".yellow(), "🚀 Project Lifecycle".yellow().bold()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}", "└─────────────────────────────────────────────────────────────────┘".yellow()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "init".yellow(), "Initialize a project with Collaborative Intelligence".yellow()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "integrate".yellow(), "Integrate CI into an existing project".yellow()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "detach".yellow(), "Detach CI integration but keep configuration".yellow()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "fix".yellow(), "Repair CI integration issues".yellow()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "verify".yellow(), "Verify CI integration is working properly".yellow()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "local".yellow(), "Create or update CLAUDE.local.md file for local CI configuration".yellow()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "migrate".yellow(), "Detect and migrate from CI to CI standalone mode".yellow()));
    help_text.push_str("\n\n");
    
    // Intelligence & Discovery category (Cyan)
    help_text.push_str(&format!("  {}", "┌─────────────────────────────────────────────────────────────────┐".cyan()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}  {}", "│".cyan(), "⚙️ Intelligence & Discovery".cyan().bold()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}", "└─────────────────────────────────────────────────────────────────┘".cyan()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "intent".cyan(), "Display the intent and purpose of the CI tool".cyan()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "agents".cyan(), "List all available Collaborative Intelligence agents".cyan()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "agent".cyan(), "Enhanced agent management with full lifecycle operations".cyan()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "load".cyan(), "Start a Claude Code session with a specified agent loaded".cyan()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "projects".cyan(), "List projects integrated with Collaborative Intelligence".cyan()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "idea".cyan(), "Manage ideas, concepts, and inspirations".cyan()));
    help_text.push_str("\n\n");
    
    // Topology Management category (Magenta)
    help_text.push_str(&format!("  {}", "┌─────────────────────────────────────────────────────────────────┐".magenta()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}  {}", "│".magenta(), "🔄 Topology Management".magenta().bold()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {}", "└─────────────────────────────────────────────────────────────────┘".magenta()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "topologist".magenta(), "Repository topology management and intelligent commit organization".magenta()));
    help_text.push_str("\n");
    help_text.push_str(&format!("             {}", "↳ git-analysis: Detailed git repository topology analysis".dimmed()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "session".magenta(), "Manage Collaborative Intelligence sessions".magenta()));
    help_text.push_str("\n");
    help_text.push_str(&format!("  {:<12} {}", "ls".magenta(), "Enhanced file listing with intelligent grouping".magenta()));
    
    help_text
}

/// Print help with colorized categories
fn print_help_with_categories() {
    println!("{} {}", "CI".green().bold(), "1.0.0".green());
    println!("{}", "A modern command-line interface for the Collaborative Intelligence system".white());
    println!();
    println!("{} ci <COMMAND>", "Usage:".bold());
    println!();
    
    // Print custom formatted commands with colors
    println!("{}", get_colored_command_help());
    println!();

    println!("Note: For detailed documentation, run '{}'", "CI intent".bold());
}

// Helper function for temporary files (currently unused)
// fn temp_file() -> anyhow::Result<tempfile::NamedTempFile> {
//     tempfile::NamedTempFile::new().context("Failed to create temporary file")
// }

/// Handle agent commands
async fn handle_agent_command(command: &AgentCommands) -> anyhow::Result<()> {
    match command {
        AgentCommands::List { enabled_only: _, verbose: _ } => {
            let args = clap::ArgMatches::default();
            // Convert to clap ArgMatches - this is a simplified version
            // In practice, you'd want to create proper ArgMatches
            commands::agents::execute(&args)
        },
        AgentCommands::Info { agent_name: _ } => {
            // Create ArgMatches for info subcommand
            let args = clap::ArgMatches::default();
            commands::agents::execute(&args)
        },
        AgentCommands::Create { agent_name: _, template: _, enable: _ } => {
            let args = clap::ArgMatches::default();
            commands::agents::execute(&args)
        },
        AgentCommands::Enable { agent_name: _ } => {
            let args = clap::ArgMatches::default();
            commands::agents::execute(&args)
        },
        AgentCommands::Disable { agent_name: _ } => {
            let args = clap::ArgMatches::default();
            commands::agents::execute(&args)
        },
        AgentCommands::Activate { agent_name: _, context: _ } => {
            let args = clap::ArgMatches::default();
            commands::agents::execute(&args)
        },
        AgentCommands::Load { agent_name } => {
            // Create ArgMatches with the agent_name for the load subcommand
            let mut cmd = commands::agents::create_command();
            let args = cmd.try_get_matches_from(vec!["agent", "load", agent_name])
                .unwrap_or_else(|_| clap::ArgMatches::default());
            commands::agents::execute(&args)
        },
        AgentCommands::Template { template_name: _, agent_name: _ } => {
            let args = clap::ArgMatches::default();
            commands::agents::execute(&args)
        },
        AgentCommands::Deploy { force: _, backup: _ } => {
            let args = clap::ArgMatches::default();
            commands::agents::execute(&args)
        },
    }
}

/// Handle session commands
async fn handle_session_command(command: &SessionCommands) -> anyhow::Result<()> {
    match command {
        SessionCommands::List { agent: _, status: _, recent: _ } => {
            let args = clap::ArgMatches::default();
            commands::session::execute(&args)
        },
        SessionCommands::Create { agent_name: _, session_name: _, description: _, tags: _ } => {
            let args = clap::ArgMatches::default();
            commands::session::execute(&args)
        },
        SessionCommands::Info { agent_name: _, session_name: _ } => {
            let args = clap::ArgMatches::default();
            commands::session::execute(&args)
        },
        SessionCommands::Archive { agent_name: _, session_name: _ } => {
            let args = clap::ArgMatches::default();
            commands::session::execute(&args)
        },
        SessionCommands::Cleanup { days: _, dry_run: _ } => {
            let args = clap::ArgMatches::default();
            commands::session::execute(&args)
        },
    }
}

/// Handle the legacy command for managing legacy command compatibility
async fn legacy_command(create: bool, remove: bool, list: bool, _config: &config::Config) -> anyhow::Result<()> {
    use crate::commands::legacy;
    use crate::helpers::CommandHelpers;
    
    CommandHelpers::print_command_header(
        "Manage legacy command compatibility", 
        "⚙️", 
        "System Management", 
        "blue"
    );
    
    // If no flags specified, default to listing commands
    let list = list || (!create && !remove);
    
    if list {
        // Display legacy command help
        println!("{}", legacy::get_legacy_commands_help());
    }
    
    if create {
        // Create symlinks for legacy commands
        let bin_dir = dirs::home_dir()
            .map(|p| p.join(".local/bin"))
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
            
        // Create the directory if it doesn't exist
        if !bin_dir.exists() {
            std::fs::create_dir_all(&bin_dir)
                .context("Failed to create bin directory")?;
        }
        
        let count = legacy::create_legacy_command_symlinks(&bin_dir)?;
        CommandHelpers::print_success(&format!("Created {} legacy command symlinks in {}", count, bin_dir.display()));
        
        if count > 0 {
            println!("\nMake sure {} is in your PATH to use legacy commands", bin_dir.display());
            println!("You may need to add the following to your shell profile:");
            println!("  export PATH=\"$HOME/.local/bin:$PATH\"");
        }
    }
    
    if remove {
        // Remove symlinks for legacy commands
        let bin_dir = dirs::home_dir()
            .map(|p| p.join(".local/bin"))
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
            
        if bin_dir.exists() {
            let count = legacy::remove_legacy_command_symlinks(&bin_dir)?;
            CommandHelpers::print_success(&format!("Removed {} legacy command symlinks from {}", count, bin_dir.display()));
        }
    }
    
    Ok(())
}

/// Run standardization check for Standardist agent protocols
fn run_standardization_check() -> anyhow::Result<()> {
    // Set window title for current agent session
    use crate::helpers::agent_autoload::AgentAutoload;
    AgentAutoload::set_agent_session_window_title("Standardist", "Analyzing");
    
    // Only run if we're in a CI project directory
    let current_dir = std::env::current_dir()?;
    if current_dir.join("CLAUDE.md").exists() || current_dir.join("Cargo.toml").exists() {
        if let Err(_) = crate::tools::quick_standardization_check() {
            // Silently continue if standardization check fails to avoid blocking normal operations
        }
    }
    
    // Update title to show completion
    AgentAutoload::update_agent_session_title("Standardist", "Analysis", "Complete");
    
    Ok(())
}

/// Check if the current directory requires agent activation
fn check_agent_activation_requirements() -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    
    // Check if this is a CI project that requires agents
    if AgentAutoload::is_agent_required(&current_dir)? {
        if let Some(config) = AgentAutoload::parse_agent_config(&current_dir)? {
            // Display notification if agent activation is configured
            if config.auto_activate {
                println!("{}", "📋 Agent activation configured for this project".cyan().dimmed());
                println!("{}", "   Agents should be automatically loaded from CLAUDE.md".dimmed());
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger
    env_logger::init();
    
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    // Check for legacy command invocation
    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()));
        
    if let Some(exe) = exe_name {
        // If the executable name is not 'ci' or 'CI', check if it's a legacy command
        if exe != "ci" && exe != "CI" && commands::legacy::is_legacy_command(&exe) {
            // This is a legacy command invocation
            let legacy_args = if args.len() > 1 { args[1..].to_vec() } else { vec![] };
            
            // Load configuration
            let config = config::Config::load()
                .context("Failed to load configuration")?;
                
            // Process the legacy command
            return commands::legacy::process_legacy_command(&exe, &legacy_args, &config).await;
        }
    }
    
    // Check for instant command pattern first (CI:command)
    if args.len() > 1 && args[1].starts_with("CI:") {
        return match tools::process_instant_command(&args[1]) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                Err(e)
            }
        };
    }
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Set verbose and debug flags in environment
    if cli.verbose {
        std::env::set_var("CI_VERBOSE", "true");
    }
    if cli.debug {
        std::env::set_var("CI_DEBUG", "true");
    }
    
    // Handle version flag
    if cli.show_version {
        version::version();
        return Ok(());
    }
    
    // Custom help display for no arguments or no command
    if std::env::args().len() <= 1 || cli.command.is_none() {
        print_help_with_categories();
        return Ok(());
    }
    
    // Load configuration
    let config = config::Config::load()
        .context("Failed to load configuration")?;
    
    // Check for agent activation requirements in current directory
    check_agent_activation_requirements()?;
    
    // Run standardization check for Standardist agent (skip for agent commands to avoid interference)
    let skip_standardization = match &cli.command {
        Some(Commands::Agent { .. }) => true,
        Some(Commands::Agents) => true,
        _ => false,
    };
    
    if !skip_standardization {
        run_standardization_check()?;
    }
    
    // Get the command name for window title
    let command_name = match &cli.command {
        Some(cmd) => {
            match cmd {
                Commands::Intent => "intent",
                Commands::Agents => "agents", 
                Commands::Agent { .. } => "agent",
                Commands::Load { .. } => "load",
                Commands::Adapt { .. } => "adapt",
                Commands::Projects => "projects",
                Commands::Idea { .. } => "idea",
                Commands::Status => "status",
                Commands::StatusDetailed { .. } => "status-detailed",
                Commands::Repo { .. } => "repo",
                Commands::Clean => "clean",
                Commands::Ignore => "ignore",
                Commands::Stage => "stage",
                Commands::Remotes => "remotes",
                Commands::Commit { .. } => "commit",
                Commands::Deploy => "deploy",
                Commands::Init { .. } => "init",
                Commands::Integrate { .. } => "integrate",
                Commands::Fix { .. } => "fix",
                Commands::Verify { .. } => "verify",
                Commands::Local { .. } => "local",
                Commands::Detach { .. } => "detach",
                Commands::Migrate { .. } => "migrate",
                Commands::Evolve => "evolve",
                Commands::Key { .. } => "key",
                Commands::Rebuild => "rebuild",
                Commands::Install => "install",
                Commands::Link => "link",
                Commands::Unlink => "unlink",
                Commands::Legacy { .. } => "legacy",
                Commands::Docs => "docs",
                Commands::FixWarnings => "fix-warnings",
                Commands::AddCommand { .. } => "add-command",
                Commands::Command { .. } => "command",
                Commands::Version => "version",
                Commands::TestWindowTitle => "test-window-title",
                Commands::Topologist { .. } => "topologist",
                Commands::Session { .. } => "session",
                Commands::Visualize { .. } => "visualize",
                Commands::Ls { .. } => "ls",
                Commands::Config { .. } => "config",
            }
        },
        None => "help"
    };
    
    // Set window title for the command and store command name for progress updates
    use crate::helpers::CommandHelpers;
    std::env::set_var("CI_CURRENT_COMMAND", command_name);
    CommandHelpers::set_window_title(command_name);
    
    // Process commands and ensure title is restored
    let result = match cli.command.unwrap() {
        // Intelligence & Discovery Commands
        Commands::Intent => {
            commands::intelligence::intent(&config).await
        },
        Commands::Agents => {
            // Use the stylized agents list instead of the full documentation
            let app = commands::agents::create_command();
            let args = vec!["agent".to_string(), "list".to_string()];
            
            match app.try_get_matches_from(&args) {
                Ok(matches) => {
                    if let Some(("list", sub_matches)) = matches.subcommand() {
                        commands::agents::list_agents(sub_matches)
                    } else {
                        commands::agents::execute(&matches)
                    }
                },
                Err(e) => {
                    // Debug the error and fallback to intelligence::agents if there's an issue
                    eprintln!("Agent command parsing failed: {}", e);
                    commands::intelligence::agents(&config).await
                }
            }
        },
        Commands::Agent { command } => {
            handle_agent_command(&command).await
        },
        Commands::Load { agent, context, path, prompt } => {
            // Invert the prompt flag - default is auto-launch (true), --prompt makes it false
            let auto_yes = !prompt;
            commands::intelligence::load_agent(&agent, context.as_deref(), path.as_deref(), auto_yes, &config).await
        },
        Commands::Adapt { path } => {
            commands::intelligence::adapt_session(&path, &config).await
        },
        Commands::Projects => {
            commands::intelligence::projects(&config).await
        },
        Commands::Idea { subcommand, title, description, category, tags, id, status, priority, filter } => {
            commands::idea::idea(
                &subcommand,
                title.as_deref(),
                description.as_deref(),
                category.as_deref(),
                tags.as_deref(),
                id.as_deref(),
                status.as_deref(),
                priority.as_deref(),
                filter.as_deref(),
                &config
            ).await
        },
        
        // Source Control Commands
        Commands::Status => {
            commands::source_control::status(&config).await
        },
        Commands::StatusDetailed { format, system, agents } => {
            commands::source_control::status_detailed(&format, system, agents, &config).await
        },
        Commands::Repo { command } => {
            commands::source_control::repo(&command, &config).await
        },
        Commands::Clean => {
            commands::source_control::clean(&config).await
        },
        Commands::Ignore => {
            commands::source_control::ignore(&config).await
        },
        Commands::Stage => {
            commands::source_control::stage(&config).await
        },
        Commands::Remotes => {
            commands::source_control::remotes(&config).await
        },
        Commands::Commit { message } => {
            let message_str = message.as_deref();
            commands::source_control::commit(message_str, &config).await
        },
        Commands::Deploy => {
            commands::source_control::deploy(&config).await
        },
        
        // Project Lifecycle Commands
        Commands::Init { project_name, agents, no_fast, integration, ci_path } => {
            // Use enhanced init command if available, fallback to legacy
            let app = commands::init::create_command();
            let args: Vec<String> = vec![
                "init".to_string(),
                project_name.clone(),
                "--agents".to_string(),
                agents.clone(),
            ];
            
            match app.try_get_matches_from(&args) {
                Ok(matches) => commands::init::execute(&matches),
                Err(_) => {
                    // Fallback to legacy implementation
                    let integration_str = integration.as_deref().unwrap_or("standalone");
                    commands::lifecycle::init(&project_name, &agents, integration_str, no_fast, ci_path.as_deref(), &config).await
                }
            }
        },
        Commands::Integrate { path, agents, no_fast, integration, ci_path } => {
            // Use default value "standalone" if integration is None
            let integration_str = integration.as_deref().unwrap_or("standalone");
            commands::lifecycle::integrate(&path, &agents, integration_str, no_fast, ci_path.as_deref(), &config).await
        },
        Commands::Fix { path, fix_type } => {
            // Use enhanced fix command if available, fallback to legacy
            let app = commands::fix::create_command();
            let mut args = vec!["fix".to_string()];
            if let Some(ref ft) = fix_type {
                args.push(ft.clone());
            }
            
            match app.try_get_matches_from(&args) {
                Ok(matches) => commands::fix::execute(&matches),
                Err(_) => {
                    // Fallback to legacy implementation
                    commands::lifecycle::fix(&path, fix_type.as_deref(), &config).await
                }
            }
        },
        Commands::Verify { path } => {
            // Use enhanced verify command if available, fallback to legacy
            let app = commands::verify::create_command();
            let args = vec!["verify".to_string()];
            
            match app.try_get_matches_from(&args) {
                Ok(matches) => commands::verify::execute(&matches),
                Err(_) => {
                    // Fallback to legacy implementation
                    commands::lifecycle::verify(&path, &config).await
                }
            }
        },
        Commands::Local { path } => {
            commands::lifecycle::local(&path, &config).await
        },
        Commands::Detach { path, keep_override } => {
            commands::detach::detach(&path, keep_override, &config).await
        },
        Commands::Migrate { path, backup, detect_only, verbose } => {
            commands::lifecycle::migrate(&path, backup, detect_only, verbose, &config).await
        },
        
        // System Management Commands
        Commands::Evolve => {
            commands::system::evolve(&config).await
        },
        Commands::Key { command } => {
            commands::system::key(&command, &config).await
        },
        Commands::Rebuild => {
            commands::system::rebuild(&config).await
        },
        Commands::Install => {
            commands::system::install(&config).await
        },
        Commands::Link => {
            commands::system::link(&config).await
        },
        Commands::Unlink => {
            commands::system::unlink(&config).await
        },
        Commands::Legacy { create, remove, list } => {
            legacy_command(create, remove, list, &config).await
        },
        Commands::Docs => {
            commands::system::docs(&config).await
        },
        Commands::FixWarnings => {
            commands::system::fix_warnings(&config).await
        },
        Commands::AddCommand { name, description, category } => {
            commands::system::add_command(
                name.as_deref(),
                description.as_deref(),
                category.as_deref(),
                &config
            ).await
        },
        Commands::Command { subcommand } => {
            commands::system::command(&subcommand, &config).await
        },
        Commands::Version => {
            commands::system::version(&config).await
        },
        Commands::TestWindowTitle => {
            CommandHelpers::test_window_title_progress()
        },
        Commands::Topologist(topology_command) => {
            commands::topology::topology(&topology_command, &config).await
        },
        Commands::Session { command } => {
            handle_session_command(&command).await
        },
        Commands::Visualize { view } => {
            commands::visualize::handle_visualization_command(&view, &config).await
        },
        Commands::Ls { directory } => {
            commands::ls::execute(directory.as_deref(), &config).await
        },
        Commands::Config { subcommand, path, project_name, agents, fast, key, value, format } => {
            commands::config::config(
                &subcommand,
                &path,
                project_name.as_deref(),
                agents.as_deref(),
                fast,
                key.as_deref(),
                value.as_deref(),
                format.as_deref()
            ).await
        },
    };
    
    // Restore window title and return result
    CommandHelpers::restore_window_title();
    result
}
