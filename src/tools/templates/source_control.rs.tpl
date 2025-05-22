/// {{description}}
pub async fn {{name}}(config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "{{description}}", 
        "{{emoji}}", 
        "{{category}}", 
        "{{color}}"
    );
    
    // Implement source control command logic
    // Example:
    // - Get current directory or use provided path
    // - Check that directory is a git repository
    // - Perform git operations using RepositoryHelpers
    // - Format and display results
    
    CommandHelpers::print_success("Source control operation completed successfully");
    
    Ok(())
}