# $CI_REPO_PATH Environment Variable Handling

## Core Functions for Environment Variable Processing

### Path Resolution (`src/common/paths.rs`)
- The central mechanism for resolving the CI repository path is in `PathResolver::get_ci_path()`:
  - Loads environment variables from `.env` file using `dotenv()`
  - Attempts to use `CI_REPO_PATH` from environment if available
  - Falls back to searching common installation paths if not found

### Creation/Management of .env Files

1. **`fix.rs` - Fix Command**
   - Checks for `.env` file and creates it if missing
   - Looks for `.env.example` and copies its content when available
   - Otherwise creates a basic `.env` file with `CI_REPO_PATH` entry
   - Actively attempts to find a valid CI repo path to update `.env` with
   - Updates existing `.env` files with correct `CI_REPO_PATH` values

2. **`init.rs` - Init Command**
   - Initializes new projects
   - Relies on `PathResolver::get_ci_path()` to resolve CI repo paths
   - Sets up `.gitignore` to exclude `.env` files from version control

3. **`integrate.rs` - Integrate Command**
   - Integrates CI into existing projects
   - Uses `PathResolver::get_ci_path()` to resolve CI repo paths
   - Updates `.gitignore` to include `.env` files

4. **`directory_utils.rs` - Legacy Path Resolution**
   - Contains deprecated version of `get_ci_path()`
   - Still loads `.env` with `dotenv()` and checks for `CI_REPO_PATH`
   - Provides same fallback mechanism as `PathResolver::get_ci_path()`

5. **`local.rs` - Local Command**
   - Creates local configuration files (CLAUDE.local.md)
   - Uses `PathResolver::get_ci_path()` to find the CI repository

## .env File Handling Overview

1. **Creation of .env Files**
   - Primary creation happens in `fix.rs` when no `.env` file exists
   - Creates basic template with `CI_REPO_PATH` entry
   - May copy from `.env.example` if available

2. **Reading from .env Files**
   - Uses the `dotenv` crate to load variables from `.env` files
   - `dotenv()` is called in `PathResolver::get_ci_path()` and in legacy `directory_utils.rs`

3. **Updating .env Files**
   - `fix.rs` is the main module that updates existing `.env` files
   - Handles finding and replacing `CI_REPO_PATH` entries
   - Attempts to discover valid repo paths if environment variable isn't set

4. **Error Handling**
   - Provides clear error messages when `CI_REPO_PATH` is not set or invalid
   - Offers guidance on how to fix environment variable issues

## Build and Deployment

- `.env` files are properly excluded from git via `.gitignore` entries
- Multiple commands ensure `.gitignore` is updated to exclude `.env` files