// Shared Configuration Utilities for CI CLI
// Common configuration handling used across multiple modules

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SharedConfig {
    pub topology: Option<TopologyConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopologyConfig {
    pub enabled: bool,
    pub auto_analyze: bool,
    pub size_tracking: bool,
    pub cleanup_after_integration: bool,
    pub max_phase_size: usize,
    pub preferred_strategy: String,
}

impl Default for TopologyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_analyze: true,
            size_tracking: true,
            cleanup_after_integration: false,
            max_phase_size: 2000,
            preferred_strategy: "size_optimized".to_string(),
        }
    }
}

impl Default for SharedConfig {
    fn default() -> Self {
        Self {
            topology: Some(TopologyConfig::default()),
        }
    }
}