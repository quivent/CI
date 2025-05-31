# CI Command Creation Guide

This guide provides optimized workflows and prompts for creating new commands in the CI tool.

## 🚀 NEW: Instant Command Creation (via CLAUDE.md)

**Just type:** `CI:[command-name]`

The AI assistant will:
1. Extract the command name from `CI:[name]` format
2. Ask: "What does '[name]' do?"
3. Auto-categorize based on keywords
4. Generate all necessary code
5. Update required files (commands/mod.rs, main.rs)
6. Build and install automatically

This approach:
- Reduces creation time to **5-10 seconds**
- Requires only 1 answer from you (description)
- AI handles all the complexity
- Zero cognitive overhead

Example:
```
CI:backup
> What does 'backup' do? Archive project files
> [AI generates code, updates files, builds, installs]
> Done! Use with: CI backup
```

**Note**: This uses a hybrid approach:
- AI handles the instant creation workflow via CLAUDE.md instructions
- Commands MUST use the optimized helper functions for consistency
- Templates ensure all commands follow established patterns

This combines the speed of AI-driven creation with the quality of pre-built helpers!

## Time Estimates

- **Instant Mode** (CI:[name]): **5-10 seconds** ✨
- **Manual Mode**: **30-60 seconds**
- **Traditional development**: **10-90 minutes**

## Quick Command Template

### Instant Command Creation (Fastest Method!)

Simply type:
```
CI:[command-name]
```

This launches an intelligent prompting system that:
1. Pre-fills the command name
2. Asks only essential questions
3. Infers everything else from context
4. Generates the command in seconds

#### Example Flow:
```
User: CI:backup
System: Creating command 'backup'...
> Purpose? Archive project files
> Any parameters? (Enter to skip) 
> Done! Command created with auto-detected FILE category.
```

### Manual Format (When You Need Control)

```
Format:
  Type: [C]reate | [E]dit  
  Command: name
  Purpose: description
  Category: (auto-detected if blank)
  Params: (optional)
  Steps: (optional) 
  Options: (optional)
  Changes: (for Edit only)
```

#### Minimal Required Fields:
- **Create**: Just Purpose (command name comes from CI:name)
- **Edit**: Just Changes (command name comes from CI:name)

#### Quick Selection Guide:
- **Type**: `C` = Create, `E` = Edit
- **Category** (optional): 
  - `G` = Git, `C` = Config, `F` = File, `S` = Status, `Y` = System
  - Leave blank = AI chooses based on command purpose
- **Common Options**: `p` = progress, `v` = verbose, `d` = debug, `n` = dry-run

#### Instant Create (Recommended):
```
CI:backup
> Purpose? Archive project files
> Done!
```

#### Instant Create with Parameters:
```
CI:sync
> Purpose? Sync with remote repository  
> Parameters? --force?, --branch?
> Done! (Auto-detected as GIT command)
```

#### Instant Edit:
```
CI:backup
> Edit existing command? Yes
> Changes? +param:--verbose +step:compress
> Done!
```

#### Manual Create (Full Control):
```
Format:
  Type: CREATE
  Command: backup
  Purpose: Archive project files
  Category: FILE
  Params: path?, output?
  Steps: validate→archive→display
  Options: +p
```

#### Manual Minimal:
```
Format:
  Type: C
  Command: status
  Purpose: Show project status
```

#### Complex Edit:
```
Format:
  Type: E
  Command: deploy
  Changes: +step:test -param:force
  Options: +d+n
```

#### Let AI Choose Category:
```
Format:
  Type: C
  Command: analyze
  Purpose: Analyze code quality and generate report
  Params: path?, --format?
  Steps: scan→analyze→report
```
(AI will likely choose FILE or STATUS based on the purpose)

#### Smart Prompting System:

**Instant Mode** (CI:command-name):
```
User: CI:analyze

System: Creating 'analyze' command...
> What does it do? Analyze code quality
> Any parameters? (Enter=none) --threshold?, --format?
> Analyzing purpose... Auto-detected: FILE category
> Generated with sensible defaults!
```

**Intelligent Context Detection**:
```
User: CI:commit

System: Creating 'commit' command...
> What does it do? Create git commits
> Detecting context... Found 'git' keyword
> Auto-configured as GIT command with:
  - RepositoryHelpers
  - Git validation
  - Green color scheme
> Any special parameters? (Enter=standard) 
> Done!
```

**Edit Detection**:
```
User: CI:backup

System: Command 'backup' exists.
> Create new or [E]dit existing? E
> What changes? +param:--compress
> Applied changes to existing command!
```

### Quick Start Guide for Format:

1. **Basic Usage** - Just start typing:
```
Format:
  Type: CREATE
  Command: mycommand
  Purpose: Does something useful
  Category: FILE
```

2. **Copy-Paste Template**:
```
Format:
  Type: 
  Command: 
  Purpose: 
  Category: 
  Params: 
  Steps: 
  Options: 
```

3. **VSCode/Editor Snippet**:
Create a snippet with trigger `ciformat`:
```json
{
  "CI Format Command": {
    "prefix": "ciformat",
    "body": [
      "Format:",
      "  Type: ${1|C,E,CREATE,EDIT|}",
      "  Command: ${2:name}",
      "  Purpose: ${3:description}",
      "  Category: ${4|G,C,F,S,Y,GIT,CONFIG,FILE,STATUS,SYSTEM|}",
      "  Params: ${5:param1?, param2!}",
      "  Steps: ${6:step1→step2}",
      "  Options: ${7:+p}"
    ],
    "description": "Create CI command with Format (C=Create, E=Edit, G=Git, etc.)"
  }
}
```

4. **Quick Reference Card** (print this!):
```
Format Quick Keys:
┌─────────────┬────────┐
│ Type        │ Key    │
├─────────────┼────────┤
│ Create      │ C      │
│ Edit        │ E      │
└─────────────┴────────┘

┌─────────────┬────────┐
│ Category    │ Key    │
├─────────────┼────────┤
│ Git         │ G      │
│ Config      │ C      │
│ File        │ F      │
│ Status      │ S      │
│ System      │ Y      │
└─────────────┴────────┘

┌─────────────┬────────┐
│ Options     │ Key    │
├─────────────┼────────┤
│ +progress   │ p      │
│ +verbose    │ v      │
│ +debug      │ d      │
│ +dry-run    │ n      │
└─────────────┴────────┘
```

### Super-Compact Shortcut (One-liner)

```
CI:[NAME] | [PURPOSE] | [TYPE] | [PARAMS] | [STEPS] | [OPTIONS]
```

Example:
```
CI:backup | Archive project files | FILE | path?, output? | validate→archive→display | +p
```

### Standard Shortcut (Copy & Modify)

```
New CI command: [NAME]
Purpose: [ONE_LINE_DESCRIPTION]
Category: [Intelligence|Source Control|Project Lifecycle|System Management]
Parameters: [PARAM1:TYPE, PARAM2:TYPE, ...]
Logic:
1. [STEP_1]
2. [STEP_2]
3. [STEP_3]
Helpers needed: [path|git|config|progress|file]
```

### Example Using Shortcut

```
New CI command: backup
Purpose: Create timestamped project backups
Category: System Management
Parameters: path:Optional<String>, output:Optional<String>, exclude:Vec<String>
Logic:
1. Resolve project path
2. Create timestamped archive excluding common dirs
3. Display backup size and location
Helpers needed: path, progress, file
```

## Detailed Command Templates

### 1. Basic Command Structure

```
Create a new CI command called [COMMAND_NAME] that [DESCRIPTION]. 
Use the helper functions from src/helpers/ for all common operations:
- Use CommandHelpers::print_command_header() for the header
- Use CommandHelpers::resolve_project_path() for path handling
- Use CommandHelpers::print_success/error/warning() for status messages
- Use appropriate helpers from RepositoryHelpers and ConfigHelpers as needed

The command should:
1. [SPECIFIC REQUIREMENT 1]
2. [SPECIFIC REQUIREMENT 2]
3. [SPECIFIC REQUIREMENT 3]
```

### 2. Git-Related Command

```
Create a new CI command called [COMMAND_NAME] that performs git operations.
Use the existing helpers:
- RepositoryHelpers::is_inside_git_repo() to check if in a repo
- RepositoryHelpers::get_current_branch() for branch info
- RepositoryHelpers::get_repository_status() for status info
- CommandHelpers::run_command_with_output() for custom git commands

The command should:
1. Validate it's in a git repository
2. [SPECIFIC GIT OPERATION]
3. Display results using CommandHelpers status functions
```

### 3. Configuration Command

```
Create a new CI command called [COMMAND_NAME] that manages configuration.
Use the existing helpers:
- ConfigHelpers::create_or_update_env_file() for .env management
- ConfigHelpers::create_claude_config() for CLAUDE.md
- CommandHelpers::read_file_content() and write_file_content()
- CommandHelpers::update_markdown_section() for markdown files

The command should:
1. [CONFIGURATION TASK]
2. Validate paths using CommandHelpers::resolve_project_path()
3. Show progress using CommandHelpers::with_progress()
```

## Implementation Checklist

When implementing a new command:

1. **Create command file**: `src/commands/[command_name].rs`
2. **Export in mod.rs**: Add to `src/commands/mod.rs`
3. **Add to CLI enum**: Update `Commands` enum in `src/main.rs`
4. **Add match arm**: Update the match statement in `src/main.rs`
5. **Add tests**: Create tests in `tests/[command_name]_tests.rs`
6. **Update help**: Add to the help system if needed

## Code Template

```rust
use colored::*;
use std::path::PathBuf;
use crate::helpers::{CommandHelpers, RepositoryHelpers, ConfigHelpers};
use crate::errors::Result;

pub fn [command_name](/* parameters */) {
    CommandHelpers::print_command_header(
        "[Command Title]",
        "[emoji]",
        "[Category]",
        "[color]"
    );
    
    // Path resolution
    let project_dir = match CommandHelpers::resolve_project_path(&path) {
        Ok(path) => path,
        Err(e) => {
            CommandHelpers::print_error(&format!("Invalid path: {}", e));
            return;
        }
    };
    
    // Implementation using helpers
    
    CommandHelpers::print_success("Operation completed successfully");
}
```

## Helper Function Quick Reference

### CommandHelpers
- `print_command_header()` - Display formatted header
- `print_success/error/warning/info()` - Status messages
- `resolve_project_path()` - Path resolution
- `is_git_repository()` - Git repo check
- `run_command_with_output()` - Execute commands
- `create_file_with_content()` - Write files
- `read_file_content()` - Read files
- `with_progress()` - Progress indicators
- `prompt_confirmation()` - User prompts

### RepositoryHelpers
- `init_git_repository()` - Initialize git
- `create_default_gitignore()` - Create .gitignore
- `get_repository_status()` - Get repo info
- `update_gitignore()` - Update .gitignore

### ConfigHelpers
- `create_or_update_env_file()` - Manage .env
- `create_claude_config()` - Create CLAUDE.md
- `create_claude_local_config()` - Create CLAUDE.local.md

## Speed Optimization Tips

1. **Use the shortcut format** for initial planning
2. **Copy a similar existing command** as a starting point
3. **Use helper functions** for all common operations
4. **Test incrementally** after each major addition
5. **Use batch operations** for creating multiple files

## Example: Creating a "Backup" Command

### Using the Shortcut Format

```
New CI command: backup
Purpose: Create timestamped project backups
Category: System Management
Parameters: path:Optional<String>, output:Optional<String>
Logic:
1. Resolve and validate project path
2. Create archive with timestamp using tar/zip
3. Exclude node_modules, target, .git
4. Display backup location and size
Helpers needed: path, progress, command execution
```

### Resulting Implementation

This shortcut expands to a full implementation that:
- Uses `CommandHelpers::print_command_header()` for UI
- Uses `CommandHelpers::resolve_project_path()` for paths
- Uses `CommandHelpers::with_progress()` for backup operation
- Uses `CommandHelpers::run_command_with_output()` for tar/zip
- Uses `CommandHelpers::print_success()` for completion

## Category Color Mapping

- **Intelligence & Discovery**: Blue
- **Source Control**: Green  
- **Project Lifecycle**: Yellow
- **System Management**: Cyan

## Common Emoji Mappings

- 🧠 Intelligence & Discovery
- 📊 Source Control
- 🚀 Project Lifecycle
- ⚙️ System Management
- 💾 Backup/Save operations
- 🔍 Search operations
- 📝 Edit operations
- 🗑️ Delete operations
- ✅ Verification operations
- 🔄 Sync operations

Using these templates and shortcuts, you can rapidly create new CI commands while maintaining consistency and leveraging the existing helper infrastructure.

## Quick Reference Card

### Super-Compact Command Format
```
CI:[NAME] | [PURPOSE] | [TYPE] | [PARAMS] | [STEPS] | [OPTIONS]
```

### What is TYPE/CATEGORY? (Optional)
The Category is now OPTIONAL. When provided, it determines:

1. **Helper modules to use**:
   - `GIT` → Automatically imports and uses `RepositoryHelpers`
   - `CONFIG` → Automatically imports and uses `ConfigHelpers`
   - `FILE` → Uses file operation helpers
   - `STATUS` → Minimal helpers (mainly display functions)
   - `SYSTEM` → Uses `CommandHelpers` for system operations
   - **Blank/Auto** → AI analyzes purpose and chooses appropriate helpers

2. **Visual appearance in CLI**:
   - `GIT` → Green color, 📊 emoji
   - `CONFIG` → Yellow color, 🔧 emoji
   - `FILE` → Cyan color, 📁 emoji
   - `STATUS` → Blue color, 📊 emoji
   - `SYSTEM` → Cyan color, ⚙️ emoji

3. **Default validations**:
   - `GIT` → Automatically checks if you're in a git repository
   - `CONFIG` → Validates configuration files exist
   - `FILE` → Checks file permissions and paths
   - `STATUS` → Minimal validation (read-only operations)
   - `SYSTEM` → Validates system permissions

4. **Error handling**:
   - `GIT` → Git-specific errors (branch conflicts, remote issues)
   - `CONFIG` → Configuration errors (invalid format, missing keys)
   - `FILE` → File system errors (permissions, disk space)
   - `STATUS` → Display errors only
   - `SYSTEM` → System/permission errors

5. **Command placement in help menu**:
   - Commands are grouped by category in the help output
   - Each category has its own section with appropriate header
   
Example help menu structure:
```
CI 1.1.0
Commands:

┌─────────────────────────────────────────┐
│  🧠 Intelligence & Discovery            │  ← Blue
└─────────────────────────────────────────┘
  agents     List available AI agents
  load       Load an agent

┌─────────────────────────────────────────┐
│  📊 Source Control                      │  ← Green (GIT category)
└─────────────────────────────────────────┘
  commit     Commit changes
  sync       Sync with remote    ← Your GIT command appears here
  
┌─────────────────────────────────────────┐
│  🚀 Project Lifecycle                   │  ← Yellow (CONFIG category)
└─────────────────────────────────────────┘
  init       Initialize project
  setup      Setup configuration ← Your CONFIG command appears here
```

### Types
- `GIT` - Git operations → Uses RepositoryHelpers, green color
- `CONFIG` - Configuration management → Uses ConfigHelpers, yellow color
- `FILE` - File operations → Uses file helpers, cyan color
- `STATUS` - Information display → Minimal helpers, blue color
- `SYSTEM` - System operations → Uses CommandHelpers, cyan color

### Common Parameters
- `path?` - Optional path
- `path!` - Required path
- `--flag` - Boolean flag
- `name` - Required string
- `names[]` - Array/list

### Step Shortcuts
- `→` - Then/next step
- `validate` - Check prerequisites
- `execute` - Run main operation
- `display` - Show results
- `cleanup` - Clean up resources

### Options Flags
Append these to control generation:
- `!h` - Don't use helper functions
- `!e` - Don't handle errors
- `!c` - Don't add colors
- `!t` - Don't add tests
- `!p` - Don't show progress
- `+d` - Add debug mode
- `+v` - Add verbose output
- `+n` - Add dry-run mode

### Examples
```
# Standard commands with defaults
CI:status | Show project info | STATUS | path? | check→gather→display
CI:sync | Sync with remote | GIT | --force | fetch→merge→push

# Commands with options
CI:export | Export config | CONFIG | format | read→transform→write | +v+d
CI:analyze | Code analysis | FILE | path!, --verbose | scan→process→report | !p
CI:debug | Debug mode check | SYSTEM | --enable | validate→apply | !c!h

# Minimal command without helpers or colors
CI:raw | Direct execution | SYSTEM | cmd | execute | !h!c!e
```

### TYPE Selection Guide
- Use `GIT` when: Working with repositories, branches, commits
- Use `CONFIG` when: Managing .env, CLAUDE.md, or settings files
- Use `FILE` when: Reading, writing, or transforming files
- Use `STATUS` when: Displaying information without modifications
- Use `SYSTEM` when: Running system commands or managing CI itself

### Create Command in Under 1 Minute
1. Choose TYPE based on primary operation
2. Write one-liner: `CI:name | purpose | type | params | steps | options`
3. Run: `ci create-command "your-one-liner"`
4. Test: `ci name --help`

This format reduces command specification time from 5-10 minutes to under 60 seconds!

## Format Comparison

### When to Use Each Format

#### Interactive Prompt Format (Format)
✅ **Best for:**
- First-time command creation
- Complex commands with many parameters
- When you need clear structure
- Team collaboration (very readable)
- When you prefer filling out a form

❌ **Not ideal for:**
- Quick one-off commands
- Experienced users who know the structure

#### Super-Compact Format (CI:)
✅ **Best for:**
- Rapid prototyping
- Experienced users
- Simple commands
- Quick iterations

❌ **Not ideal for:**
- Complex parameter lists
- First-time users
- When readability matters

### Overhead Comparison
```
CI:[name] instant:               1 entry + 1-2 prompts (5-10 seconds)
Manual Format:                   5-10 lines (30-60 seconds)
Old CI: one-liner:              1 line, but requires all info (15-30 seconds)

Key insight: CI:[name] shifts overhead from human to AI
- Human types less, thinks less
- AI infers more, prompts intelligently
- Total time: 80% reduction
```

### Examples Side-by-Side

**Creating a sync command:**

Interactive (Full words):
```
Format:
  Type: CREATE
  Command: sync
  Purpose: Sync with remote
  Category: GIT
  Params: --force?, --branch?
  Steps: fetch→merge→push
  Options: +p
```

Interactive (Quick keys):
```
Format:
  Type: C
  Command: sync
  Purpose: Sync with remote
  Category: G
  Params: --force?, --branch?
  Steps: fetch→merge→push
  Options: +p
```

Compact:
```
CI:sync | Sync with remote | GIT | --force?, --branch? | fetch→merge→push | +p
```

All three generate the exact same command!

### How Category Works (When Specified or Auto-detected)

#### Auto-Category Detection
When no category is specified, the AI analyzes your purpose to determine:
- Commands with "git", "commit", "branch", "merge" → GIT category
- Commands with "config", "setup", "settings" → CONFIG category  
- Commands with "read", "write", "file", "archive" → FILE category
- Commands with "show", "list", "display", "info" → STATUS category
- Commands with "install", "build", "system" → SYSTEM category

#### Manual Category Selection
When you specify a Category, it automatically generates different boilerplate:

**Example 1: GIT Category**
```
Format:
  Type: C
  Command: sync
  Category: G
```
Generates:
```rust
use crate::helpers::{CommandHelpers, RepositoryHelpers};  // ← Git helpers
pub fn sync() {
    CommandHelpers::print_command_header(
        "Sync with remote",
        "📊",                     // ← Git emoji
        "Source Control",         // ← Git category name
        "green"                   // ← Git color
    );
    
    // Auto-validates git repository
    if !RepositoryHelpers::is_inside_git_repo(&path) {
        CommandHelpers::print_error("Not in a git repository");
        return;
    }
    // ... rest of implementation
}
```

**Example 2: CONFIG Category**
```
Format:
  Type: C
  Command: setup
  Category: C
```
Generates:
```rust
use crate::helpers::{CommandHelpers, ConfigHelpers};  // ← Config helpers
pub fn setup() {
    CommandHelpers::print_command_header(
        "Setup configuration",
        "🔧",                    // ← Config emoji
        "Configuration",         // ← Config category name
        "yellow"                 // ← Config color
    );
    
    // Auto-validates config files
    let config_path = path.join("CLAUDE.md");
    if !config_path.exists() {
        ConfigHelpers::create_claude_config(&path)?;
    }
    // ... rest of implementation
}
```

## Edit Command Quick Reference

### Symbols
- `+` Add something new
- `-` Remove something
- `~` Change/modify something
- `→` Step flow direction

### Common Edits One-liners
```
EDIT:cmd | +param:--dry-run |              # Add dry-run flag
EDIT:cmd | -param:verbose |                # Remove parameter
EDIT:cmd | ~type:GIT |                     # Change to git command
EDIT:cmd | +step:validate |                # Add validation step
EDIT:cmd | ~helpers:add | +p               # Add helpers and progress
EDIT:cmd | | !c!e                          # Remove colors and errors
EDIT:cmd | ~step:old→new |                 # Replace step

# Quick helper toggle
EDIT:cmd | ~helpers:remove |               # Remove all helpers
EDIT:cmd | ~helpers:add |                  # Add standard helpers
```

### Instant Command Evolution
From simple to advanced in 3 edits:
```
1. EDIT:backup | ~helpers:add |            # Add helpers
2. EDIT:backup | +param:--verbose | +p     # Add verbosity
3. EDIT:backup | +step:compress | +v       # Add compression
```

## Complete Command Lifecycle

### 1. Create (Choose Your Format)

Interactive Format:
```
Format:
  Type: CREATE
  Command: newcmd
  Purpose: Purpose here
  Category: TYPE
  Params: params
  Steps: steps
  Options: options
```

OR Compact Format:
```
CI:newcmd | Purpose here | TYPE | params | steps | options
```

### 2. Implement
Use helper functions based on TYPE:
- GIT → RepositoryHelpers
- CONFIG → ConfigHelpers  
- FILE → File operations
- STATUS → Minimal helpers
- SYSTEM → CommandHelpers

### 3. Test
```bash
cargo build
ci newcmd --help
```

### 4. Edit/Evolve
```
EDIT:newcmd | changes | options
```

### 5. Documentation
Commands are self-documenting through:
- Help text in command definition
- Parameter descriptions
- Clear function names

## Quick Decision Tree

1. **What type of operation?**
   - Git work → TYPE: GIT
   - Config files → TYPE: CONFIG
   - File ops → TYPE: FILE
   - Info display → TYPE: STATUS
   - System/CI → TYPE: SYSTEM

2. **What helpers needed?**
   - Paths → CommandHelpers::resolve_project_path()
   - Git → RepositoryHelpers functions
   - Config → ConfigHelpers functions
   - UI → CommandHelpers::print_* functions

3. **What options?**
   - Need progress? → +p
   - Need verbose? → +v
   - Skip helpers? → !h
   - No colors? → !c

## Example: Full Command Creation

1. **Specify**: 
   ```
   CI:analyze | Analyze code quality | FILE | path!, --format? | scan→process→report | +p+v
   ```

2. **Create**: Command generates with:
   - CommandHelpers::print_command_header()
   - File scanning logic
   - Progress indicators
   - Verbose output option

3. **Test & Refine**:
   ```
   EDIT:analyze | +param:--threshold | 
   EDIT:analyze | +step:validate-threshold |
   ```

The entire process takes under 5 minutes!

## Editing Existing Commands

### Quick Edit Format
```
EDIT:[NAME] | [CHANGES] | [OPTIONS]
```

### Change Types
- `+param:type` - Add new parameter
- `-param` - Remove parameter  
- `~param:newtype` - Change parameter type
- `+step` - Add new step
- `-step` - Remove step
- `~step→newstep` - Replace step
- `~type:NEWTYPE` - Change command type
- `~color:newcolor` - Change color scheme
- `~helpers:add|remove` - Modify helper usage

### Edit Examples
```
# Add a verbose flag to existing command
EDIT:backup | +param:--verbose | 

# Change command type and add progress
EDIT:analyze | ~type:SYSTEM | +p

# Add new step and parameter
EDIT:sync | +param:--branch +step:verify |

# Remove helper functions and add debug
EDIT:status | ~helpers:remove | +d

# Complete refactor with new type and steps
EDIT:export | ~type:FILE ~step:read→validate→transform→write | +v
```

### Bulk Edit Operations
```
# Apply changes to multiple commands
BULK-EDIT:[cmd1,cmd2,cmd3] | [CHANGES] | [OPTIONS]

# Example: Add verbose flag to all git commands
BULK-EDIT:[sync,commit,push] | +param:--verbose |

# Example: Remove colors from all status commands  
BULK-EDIT:[status,info,list] | | !c
```

### Edit Workflow
1. **Identify change**: What needs to be modified?
2. **Write edit command**: `EDIT:name | changes | options`
3. **Apply**: `ci edit-command "EDIT:name | changes | options"`
4. **Test**: `ci name --help` to verify changes

### Common Edit Patterns
```
# Convert simple command to use helpers
EDIT:mycommand | ~helpers:add | 

# Add error handling to existing command
EDIT:mycommand | | -!e

# Make command more verbose with progress
EDIT:mycommand | +param:--verbose | +p+v

# Refactor to different command type
EDIT:mycommand | ~type:GIT ~color:green |
```

### Advanced Edits
```
# Change entire command flow
EDIT:deploy | ~step:validate→build→test→deploy +param:--skip-tests | +n

# Convert from one type to another with full refactor
EDIT:backup | ~type:GIT ~step:check-repo→create-branch→archive→commit | 

# Add conditional logic
EDIT:sync | +step:check-conflicts→resolve +param:--auto-resolve |
```