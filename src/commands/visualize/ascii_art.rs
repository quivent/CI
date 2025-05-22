use colored::*;
use crate::VisualizationTheme;

pub struct AsciiArtGenerator {
    theme: VisualizationTheme,
}

impl AsciiArtGenerator {
    pub fn new(theme: VisualizationTheme) -> Self {
        Self { theme }
    }
    
    pub fn create_header(&self, title: &str) -> String {
        let width = 80;
        let title_len = title.len();
        let padding = (width - title_len - 4) / 2;
        
        let top_border = format!("╔{}╗", "═".repeat(width - 2));
        let title_line = format!("║{}{title}{}║", 
                                " ".repeat(padding),
                                " ".repeat(width - title_len - padding - 2));
        let bottom_border = format!("╚{}╝", "═".repeat(width - 2));
        
        match self.theme {
            VisualizationTheme::Dark => format!("{}\n{}\n{}", 
                                              top_border.blue(),
                                              title_line.blue().bold(),
                                              bottom_border.blue()),
            VisualizationTheme::Light => format!("{}\n{}\n{}", 
                                               top_border.green(),
                                               title_line.green().bold(),
                                               bottom_border.green()),
            VisualizationTheme::Contrast => format!("{}\n{}\n{}", 
                                                 top_border.magenta(),
                                                 title_line.magenta().bold(),
                                                 bottom_border.magenta()),
            VisualizationTheme::Terminal => format!("{}\n{}\n{}", 
                                                     top_border.white(),
                                                     title_line.white().bold(),
                                                     bottom_border.white()),
        }
    }
    
    pub fn create_section_header(&self, title: &str) -> String {
        let separator = "─".repeat(60);
        match self.theme {
            VisualizationTheme::Dark => format!("{} {} {}", 
                                              separator.blue(),
                                              title.bold().cyan(),
                                              separator.blue()),
            VisualizationTheme::Light => format!("{} {} {}", 
                                               separator.green(),
                                               title.bold().yellow(),
                                               separator.green()),
            VisualizationTheme::Contrast => format!("{} {} {}", 
                                                 separator.magenta(),
                                                 title.bold().white(),
                                                 separator.magenta()),
            VisualizationTheme::Terminal => format!("{} {} {}", 
                                                     separator.white(),
                                                     title.bold().white(),
                                                     separator.white()),
        }
    }
    
    pub fn create_box(&self, content: &str, width: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let max_content_width = width - 4; // Account for borders and padding
        
        let mut result = String::new();
        
        // Top border
        result.push_str(&format!("┌{}┐\n", "─".repeat(width - 2)));
        
        // Content lines
        for line in lines {
            let padded_line = if line.len() > max_content_width {
                format!("│ {:<width$} │\n", &line[..max_content_width], width = max_content_width)
            } else {
                format!("│ {:<width$} │\n", line, width = max_content_width)
            };
            result.push_str(&padded_line);
        }
        
        // Bottom border
        result.push_str(&format!("└{}┘", "─".repeat(width - 2)));
        
        match self.theme {
            VisualizationTheme::Dark => result.blue().to_string(),
            VisualizationTheme::Light => result.green().to_string(),
            VisualizationTheme::Contrast => result.magenta().to_string(),
            VisualizationTheme::Terminal => result.white().to_string(),
        }
    }
    
    pub fn create_tree_node(&self, level: usize, is_last: bool, content: &str) -> String {
        let prefix = if level == 0 {
            String::new()
        } else {
            let mut p = String::new();
            for i in 0..level {
                if i == level - 1 {
                    p.push_str(if is_last { "└── " } else { "├── " });
                } else {
                    p.push_str("│   ");
                }
            }
            p
        };
        
        let line = format!("{}{}", prefix, content);
        match self.theme {
            VisualizationTheme::Dark => {
                if level == 0 {
                    line.bold().cyan().to_string()
                } else {
                    format!("{}{}", prefix.blue(), content.white())
                }
            },
            VisualizationTheme::Light => {
                if level == 0 {
                    line.bold().green().to_string()
                } else {
                    format!("{}{}", prefix.green(), content.white())
                }
            },
            VisualizationTheme::Contrast => {
                if level == 0 {
                    line.bold().magenta().to_string()
                } else {
                    format!("{}{}", prefix.magenta(), content.white())
                }
            },
            VisualizationTheme::Terminal => {
                if level == 0 {
                    line.bold().white().to_string()
                } else {
                    line.white().to_string()
                }
            },
        }
    }
    
    pub fn create_progress_bar(&self, percentage: f32, width: usize) -> String {
        let filled_width = (percentage * width as f32) as usize;
        let empty_width = width - filled_width;
        
        let filled = "█".repeat(filled_width);
        let empty = "░".repeat(empty_width);
        let bar = format!("{}{}", filled, empty);
        
        match self.theme {
            VisualizationTheme::Dark => format!("{}",
                if percentage > 0.8 { bar.green() }
                else if percentage > 0.5 { bar.yellow() }
                else { bar.red() }
            ),
            VisualizationTheme::Light => bar.green().to_string(),
            VisualizationTheme::Contrast => bar.magenta().to_string(),
            VisualizationTheme::Terminal => bar.white().to_string(),
        }
    }
    
    pub fn create_connection_diagram(&self, nodes: &[(String, Vec<String>)]) -> String {
        let mut result = String::new();
        
        for (i, (node, connections)) in nodes.iter().enumerate() {
            // Main node
            result.push_str(&format!("┌─[{}]─┐\n", node));
            
            // Connections
            for (j, connection) in connections.iter().enumerate() {
                let is_last = j == connections.len() - 1;
                let connector = if is_last { "└─→" } else { "├─→" };
                result.push_str(&format!("{}  {}\n", connector, connection));
            }
            
            if i < nodes.len() - 1 {
                result.push_str("│\n");
            }
        }
        
        match self.theme {
            VisualizationTheme::Dark => result.blue().to_string(),
            VisualizationTheme::Light => result.green().to_string(),
            VisualizationTheme::Contrast => result.magenta().to_string(),
            VisualizationTheme::Terminal => result.white().to_string(),
        }
    }
    
    pub fn create_grid(&self, items: &[String], cols: usize) -> String {
        let mut result = String::new();
        let col_width = 20;
        
        for chunk in items.chunks(cols) {
            let mut line = String::new();
            for (i, item) in chunk.iter().enumerate() {
                let formatted_item = format!("{:width$}", 
                                           if item.len() > col_width { 
                                               item[..col_width-3].to_string() + "..." 
                                           } else { 
                                               item.clone() 
                                           }, 
                                           width = col_width);
                line.push_str(&formatted_item);
                if i < chunk.len() - 1 {
                    line.push_str(" │ ");
                }
            }
            result.push_str(&format!("{}\n", line));
        }
        
        match self.theme {
            VisualizationTheme::Dark => result.cyan().to_string(),
            VisualizationTheme::Light => result.green().to_string(),
            VisualizationTheme::Contrast => result.magenta().to_string(),
            VisualizationTheme::Terminal => result.white().to_string(),
        }
    }
}