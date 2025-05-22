use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::helpers::path::get_ci_root;

/// Documentation routing middleware to handle file migrations
/// This system provides dynamic routing for documentation that has been
/// centralized from various locations to the standard docs/ structure
#[derive(Debug, Clone)]
pub struct DocumentationRouter {
    /// Mapping of logical document names to current file paths
    route_map: HashMap<String, DocumentRoute>,
    /// CI root directory for resolving relative paths
    ci_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRoute {
    /// Current location of the document
    pub current_path: String,
    /// Legacy paths where this document used to exist
    pub legacy_paths: Vec<String>,
    /// Type of document (agents, protocols, architecture, etc.)
    pub document_type: String,
    /// Last known modification time for caching
    pub last_modified: Option<String>,
    /// Whether the document is in transition
    pub migration_status: MigrationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStatus {
    /// Document is in its final centralized location
    Centralized,
    /// Document is being migrated, both old and new paths may exist
    InTransition,
    /// Document is in legacy location and needs migration
    Legacy,
    /// Document has been moved but references haven't been updated
    ReferencesOutdated,
}

impl DocumentationRouter {
    /// Create a new documentation router
    pub fn new() -> Result<Self> {
        let ci_root = get_ci_root()?;
        let mut router = Self {
            route_map: HashMap::new(),
            ci_root,
        };
        
        // Initialize with known document routes
        router.initialize_routes()?;
        
        Ok(router)
    }
    
    /// Initialize the routing map with known document locations
    fn initialize_routes(&mut self) -> Result<()> {
        // Agents documentation
        self.add_route(
            "agents",
            DocumentRoute {
                current_path: "docs/agent-system/agents-overview.md".to_string(),
                legacy_paths: vec![
                    "AGENTS.md".to_string(),
                    "AGENT_INDEX.md".to_string(),
                ],
                document_type: "agents".to_string(),
                last_modified: None,
                migration_status: MigrationStatus::Centralized,
            }
        );
        
        // Agent index/listing
        self.add_route(
            "agent_index",
            DocumentRoute {
                current_path: "docs/agent-system/agent-index.md".to_string(),
                legacy_paths: vec![
                    "AGENT_INDEX.md".to_string(),
                ],
                document_type: "agents".to_string(),
                last_modified: None,
                migration_status: MigrationStatus::Centralized,
            }
        );
        
        // Protocol documents
        self.add_route(
            "response_format_protocol",
            DocumentRoute {
                current_path: "docs/protocols/response-format-protocol.md".to_string(),
                legacy_paths: vec![
                    "NOTIFICATION_TO_ALL_AGENTS_RESPONSE_FORMAT.md".to_string(),
                    "NOTIFICATION_RESPONSE_FORMAT.md".to_string(),
                ],
                document_type: "protocols".to_string(),
                last_modified: None,
                migration_status: MigrationStatus::Centralized,
            }
        );
        
        self.add_route(
            "task_management_protocol",
            DocumentRoute {
                current_path: "docs/protocols/task-management-protocol.md".to_string(),
                legacy_paths: vec![
                    "NOTIFICATION_TRUST_BASED_TASK_MANAGEMENT.md".to_string(),
                ],
                document_type: "protocols".to_string(),
                last_modified: None,
                migration_status: MigrationStatus::Centralized,
            }
        );
        
        // Architecture documents
        self.add_route(
            "repository_topology",
            DocumentRoute {
                current_path: "docs/system-design/repository-topology.md".to_string(),
                legacy_paths: vec![
                    "repository_topology.md".to_string(),
                ],
                document_type: "architecture".to_string(),
                last_modified: None,
                migration_status: MigrationStatus::Centralized,
            }
        );
        
        self.add_route(
            "knowledge_distribution",
            DocumentRoute {
                current_path: "docs/system-design/knowledge-distribution.md".to_string(),
                legacy_paths: vec![
                    "Architectures/KnowledgeDistribution/KNOWLEDGE_DISTRIBUTION_ARCHITECTURE.md".to_string(),
                    "Architectures/KNOWLEDGE_DISTRIBUTION_ARCHITECTURE.md".to_string(),
                ],
                document_type: "architecture".to_string(),
                last_modified: None,
                migration_status: MigrationStatus::InTransition,
            }
        );
        
        // Learning system documents
        self.add_route(
            "learning_update_protocol",
            DocumentRoute {
                current_path: "docs/protocols/learning-update-protocol.md".to_string(),
                legacy_paths: vec![
                    "Documentation/LearningUpdateProtocol.md".to_string(),
                    "Documentation/LearningUpdateCommand.md".to_string(),
                ],
                document_type: "protocols".to_string(),
                last_modified: None,
                migration_status: MigrationStatus::InTransition,
            }
        );
        
        Ok(())
    }
    
    /// Add a route to the router
    fn add_route(&mut self, key: &str, route: DocumentRoute) {
        self.route_map.insert(key.to_string(), route);
    }
    
    /// Get the current path for a document by logical name
    pub fn get_document_path(&self, document_key: &str) -> Option<PathBuf> {
        self.route_map.get(document_key)
            .map(|route| self.ci_root.join(&route.current_path))
    }
    
    /// Try to find a document by checking both current and legacy paths
    pub fn find_document(&self, document_key: &str) -> Result<Option<PathBuf>> {
        if let Some(route) = self.route_map.get(document_key) {
            // Try current path first
            let current_path = self.ci_root.join(&route.current_path);
            if current_path.exists() {
                return Ok(Some(current_path));
            }
            
            // Try legacy paths if current doesn't exist
            for legacy_path in &route.legacy_paths {
                let legacy_full_path = self.ci_root.join(legacy_path);
                if legacy_full_path.exists() {
                    return Ok(Some(legacy_full_path));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Try to find any document by path (checking both current and legacy locations)
    pub fn find_document_by_path(&self, path: &str) -> Result<Option<PathBuf>> {
        // First try the path as-is relative to CI root
        let direct_path = self.ci_root.join(path);
        if direct_path.exists() {
            return Ok(Some(direct_path));
        }
        
        // Search through all routes to see if this path matches any current or legacy paths
        for route in self.route_map.values() {
            // Check current path
            if route.current_path == path {
                let current_path = self.ci_root.join(&route.current_path);
                if current_path.exists() {
                    return Ok(Some(current_path));
                }
            }
            
            // Check legacy paths
            for legacy_path in &route.legacy_paths {
                if legacy_path == path {
                    let legacy_full_path = self.ci_root.join(legacy_path);
                    if legacy_full_path.exists() {
                        return Ok(Some(legacy_full_path));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get document content by logical name
    pub fn get_document_content(&self, document_key: &str) -> Result<Option<String>> {
        if let Some(path) = self.find_document(document_key)? {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read document: {}", path.display()))?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }
    
    /// List all available documents
    pub fn list_documents(&self) -> Vec<String> {
        self.route_map.keys().cloned().collect()
    }
    
    /// Get migration status for a document
    pub fn get_migration_status(&self, document_key: &str) -> Option<&MigrationStatus> {
        self.route_map.get(document_key)
            .map(|route| &route.migration_status)
    }
    
    /// Check if a document exists in any location
    pub fn document_exists(&self, document_key: &str) -> bool {
        self.find_document(document_key).ok().flatten().is_some()
    }
    
    /// Get all legacy paths for a document (for reference stub creation)
    pub fn get_legacy_paths(&self, document_key: &str) -> Vec<PathBuf> {
        self.route_map.get(document_key)
            .map(|route| route.legacy_paths.iter()
                .map(|path| self.ci_root.join(path))
                .collect())
            .unwrap_or_default()
    }
    
    /// Create a reference stub at a legacy location
    pub fn create_reference_stub(&self, document_key: &str, legacy_path: &str) -> Result<()> {
        if let Some(route) = self.route_map.get(document_key) {
            let stub_path = self.ci_root.join(format!("{}.reference", legacy_path));
            let stub_content = format!(
                r#"# Document Relocated
                
This document has been moved as part of the Core Document Centralization Protocol.

**New Location**: `{}`

**Original Location**: `{}`

**Migration Date**: {}

**Document Type**: {}

**Migration Status**: {:?}

Please update any references to point to the new location.

For more information about the document centralization effort, see:
- `docs/development/repository-restructuring.md`
- `docs/system-design/documentation-linting-report.md`

---

*This reference stub was automatically generated by the CI Documentation Router*
"#,
                route.current_path,
                legacy_path,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                route.document_type,
                route.migration_status
            );
            
            fs::write(&stub_path, stub_content)
                .with_context(|| format!("Failed to create reference stub: {}", stub_path.display()))?;
        }
        
        Ok(())
    }
    
    /// Auto-detect and register new routes by scanning the docs directory
    pub fn scan_and_update_routes(&mut self) -> Result<()> {
        let docs_dir = self.ci_root.join("docs");
        if !docs_dir.exists() {
            return Ok(());
        }
        
        // Scan for new documents in the standardized structure
        self.scan_directory(&docs_dir, "")
            .with_context(|| "Failed to scan docs directory for route updates")?;
        
        Ok(())
    }
    
    /// Recursively scan a directory for markdown files to register as routes
    fn scan_directory(&mut self, dir: &Path, prefix: &str) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                let new_prefix = if prefix.is_empty() {
                    dir_name.to_string()
                } else {
                    format!("{}/{}", prefix, dir_name)
                };
                
                self.scan_directory(&path, &new_prefix)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let file_name = path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                let relative_path = path.strip_prefix(&self.ci_root)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.display().to_string());
                
                let document_key = if prefix.is_empty() {
                    file_name.to_string()
                } else {
                    format!("{}_{}", prefix.replace('/', "_"), file_name)
                };
                
                // Only add if not already in the route map
                if !self.route_map.contains_key(&document_key) {
                    let document_type = prefix.split('/').next().unwrap_or("unknown").to_string();
                    
                    self.add_route(&document_key, DocumentRoute {
                        current_path: relative_path,
                        legacy_paths: Vec::new(),
                        document_type,
                        last_modified: None,
                        migration_status: MigrationStatus::Centralized,
                    });
                }
            }
        }
        
        Ok(())
    }
}

/// Get the global documentation router instance
pub fn get_documentation_router() -> Result<DocumentationRouter> {
    DocumentationRouter::new()
}

/// High-level function to get document content with automatic routing
pub fn get_document_content(document_key: &str) -> Result<Option<String>> {
    let router = get_documentation_router()?;
    router.get_document_content(document_key)
}

/// High-level function to find a document path with automatic routing
pub fn find_document_path(document_key: &str) -> Result<Option<PathBuf>> {
    let router = get_documentation_router()?;
    router.find_document(document_key)
}

/// Helper function specifically for agents documentation
pub fn get_agents_documentation() -> Result<Option<String>> {
    get_document_content("agents")
}

/// Helper function to get agent index
pub fn get_agent_index() -> Result<Option<String>> {
    get_document_content("agent_index")
}