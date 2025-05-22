/// {{description}}
pub async fn {{name}}(config: &Config) -> Result<()> {
    CommandHelpers::print_command_header(
        "{{description}}", 
        "{{emoji}}", 
        "{{category}}", 
        "{{color}}"
    );
    
    // Implement intelligence & discovery command logic
    // Example:
    // - Search for information in the CI repository
    // - Process and analyze agent data
    // - Explore project configurations
    
    CommandHelpers::print_success("Intelligence operation completed successfully");
    
    Ok(())
}