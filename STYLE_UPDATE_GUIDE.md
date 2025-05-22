# CI Style Update Guide

## Background
CI is a modernized Rust implementation of the CI CLI. While CI maintains the same command categories and color scheme as CI, it uses a simpler visual design. This specification outlines how to update CI's visual design to match CI's more decorative box-drawing output style while preserving CI's architectural improvements.

## Requirements

1. **Add CommandHelpers Struct** 
   - Create a `helpers` module in CI similar to CI
   - Implement a `CommandHelpers` struct with formatting methods
   - Port the box-drawing character styling from CI

2. **Update Help Display**
   - Modify `print_help_with_categories()` in CI's main.rs
   - Add box drawing characters for category sections
   - Match CI's indentation and layout pattern

3. **Update Command Output Functions**
   - Add consistent header formatting for all commands
   - Implement standard output functions (success, error, warning, info)
   - Use box drawing characters for section dividers

4. **Preserve Modern Architecture**
   - Maintain CI's category-based file organization
   - Keep async/await pattern and error handling
   - Preserve type safety improvements (PathBuf vs String)

## Implementation Details

1. The primary files to modify:
   - `/Users/joshkornreich/Documents/Projects/CI/src/main.rs`
   - Create `/Users/joshkornreich/Documents/Projects/CI/src/helpers/mod.rs`
   - Update all command files in `/Users/joshkornreich/Documents/Projects/CI/src/commands/`

2. Port the following functions from CI to CI:
   - `CommandHelpers::print_command_header`
   - `CommandHelpers::print_divider`
   - `CommandHelpers::print_success/error/warning/info`
   - `get_colored_command_help()` (adapt to CI's architecture)

3. Standardize output across all commands to use the new helpers

## Expected Outcomes
- CI will maintain its modular organization and technical advantages
- CI's visual output will closely match CI's box-drawing style
- Command categorization, colors, and emojis will remain the same

## Acceptance Criteria
- Help display shows box-drawing characters around category headers
- All commands display consistent box-drawn headers
- Modern Rust patterns and organization are preserved
- Design is visually consistent with CI's output style

## Reference Implementation
For reference, examine the implementation in the CI project:
- `/Users/joshkornreich/Documents/Projects/CI/src/main.rs` - Command help formatting
- `/Users/joshkornreich/Documents/Projects/CI/src/helpers/mod.rs` - Helper functions
- `/Users/joshkornreich/Documents/Projects/CI/src/commands/` - Command implementations

## Migration Steps
1. Create helpers module with CommandHelpers struct
2. Update main.rs to use box drawing characters in help display
3. Update one command module as a proof of concept
4. Roll out changes to remaining command modules
5. Test all commands for visual consistency