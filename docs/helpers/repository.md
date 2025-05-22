# Repository Helpers

The `RepositoryHelpers` module provides functions for working with git repositories, including status checks, commits, and repository management.

## Repository Status Functions

### `is_inside_git_repo`

Checks if a path is inside a git repository.

```rust
if RepositoryHelpers::is_inside_git_repo(path) {
    // Code that requires a git repository
}
```

### `get_git_root`

Gets the root directory of the git repository.

```rust
let root = RepositoryHelpers::get_git_root(path)?;
```

### `get_current_branch`

Gets the current branch name.

```rust
let branch = RepositoryHelpers::get_current_branch(path)?;
```

### `get_repository_status`

Gets detailed repository status information.

```rust
let status = RepositoryHelpers::get_repository_status(path)?;
println!("Current branch: {:?}", status.current_branch);
println!("Has changes: {}", status.has_uncommitted_changes);
println!("Commit count: {}", status.commit_count);
```

### `display_status`

Displays formatted repository status information.

```rust
let status = RepositoryHelpers::get_repository_status(path)?;
RepositoryHelpers::display_status(&status);
```

Output:
```
Repository Status:
  Branch: main
  Commits: 42
  Changes: clean
  Remote: https://github.com/user/repo.git
```

### `show_diff_statistics`

Shows statistics about the current changes in the repository.

```rust
RepositoryHelpers::show_diff_statistics(path)?;
```

Output:
```
Staged changes:
 src/main.rs | 15 ++++-----------
 1 file changed, 4 insertions(+), 11 deletions(-)

Unstaged changes:
 README.md | 3 ++-
 1 file changed, 2 insertions(+), 1 deletion(-)
```

### `show_recent_commits`

Shows the most recent commits in the repository.

```rust
RepositoryHelpers::show_recent_commits(path, 5)?;
```

Output:
```
Recent commits:
* a1b2c3d Fix bug in parser
* e4f5g6h Add new feature
* i7j8k9l Update documentation
* m0n1o2p Initial commit
```

### `has_unstaged_changes`

Checks if there are unstaged changes in the repository.

```rust
if RepositoryHelpers::has_unstaged_changes(path) {
    CommandHelpers::print_warning("You have unstaged changes");
}
```

## Repository Modification Functions

### `init_git_repository`

Initializes a new git repository.

```rust
RepositoryHelpers::init_git_repository(path)?;
```

### `create_default_gitignore`

Creates a default .gitignore file if it doesn't exist.

```rust
RepositoryHelpers::create_default_gitignore(path)?;
```

### `update_gitignore`

Adds specific patterns to .gitignore.

```rust
let patterns = ["*.log", "dist/", "node_modules/"];
let updated = RepositoryHelpers::update_gitignore(path, &patterns)?;
if updated {
    CommandHelpers::print_success("Updated .gitignore with new patterns");
}
```

### `stage_files`

Stages files matching a pattern.

```rust
RepositoryHelpers::stage_files(path, "src/*.rs")?;
```

### `get_staged_files`

Gets a list of staged files.

```rust
let files = RepositoryHelpers::get_staged_files(path)?;
println!("Staged files: {}", files.join(", "));
```

### `create_commit`

Creates a commit with the given message.

```rust
RepositoryHelpers::create_commit(path, "Fix bug in parser")?;
```

### `push_to_remote`

Pushes to a remote repository.

```rust
RepositoryHelpers::push_to_remote(path, "origin", "main", false)?;
```

## Commit Message Generation

### `generate_commit_message`

Analyzes changes and generates a commit message.

```rust
let (message, details) = RepositoryHelpers::generate_commit_message(path).await?;
println!("Suggested commit message: {}", message);
println!("Details:\n{}", details);
```

## Types

### `RepositoryStatus`

Structure to hold repository status information.

```rust
pub struct RepositoryStatus {
    pub is_git_repo: bool,
    pub current_branch: Option<String>,
    pub has_uncommitted_changes: bool,
    pub commit_count: usize,
    pub has_remote: bool,
    pub remote_url: Option<String>,
    pub commits_ahead: usize,
    pub commits_behind: usize,
}
```