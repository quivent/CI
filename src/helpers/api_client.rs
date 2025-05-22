use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct CIApiClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
pub struct AgentsResponse {
    pub content: String,
    pub format: String,
    pub agent_count: usize,
    pub active_count: usize,
    pub inactive_count: usize,
    pub agents: Vec<AgentInfo>,
}

#[derive(Deserialize, Serialize)]
pub struct AgentInfo {
    pub name: String,
    pub path: String,
    pub status: String,
    pub has_readme: bool,
    pub has_metadata: bool,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct DocumentResponse {
    pub content: String,
    pub format: String,
    pub metadata: DocumentMetadata,
}

#[derive(Deserialize)]
pub struct DocumentMetadata {
    pub path: String,
    pub relative_path: String,
    pub last_modified: String,
    pub size: u64,
}

#[derive(Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
}

impl CIApiClient {
    pub fn new() -> Self {
        let port = std::env::var("CI_API_PORT").unwrap_or_else(|_| "8080".to_string());
        let base_url = format!("http://127.0.0.1:{}", port);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        CIApiClient { base_url, client }
    }

    pub async fn health_check(&self) -> Result<HealthResponse> {
        let url = format!("{}/api/health", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to check API health")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API health check failed with status: {}",
                response.status()
            ));
        }

        response
            .json::<HealthResponse>()
            .await
            .context("Failed to parse health response")
    }

    pub async fn get_agents(&self) -> Result<AgentsResponse> {
        let url = format!("{}/api/agents", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch agents")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch agents with status: {}",
                response.status()
            ));
        }

        response
            .json::<AgentsResponse>()
            .await
            .context("Failed to parse agents response")
    }

    pub async fn get_agent_info(&self, agent_name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/agents/{}", self.base_url, agent_name);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch agent info")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch agent info with status: {}",
                response.status()
            ));
        }

        response
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse agent info response")
    }

    pub async fn get_document(&self, document_type: &str) -> Result<DocumentResponse> {
        let url = format!("{}/api/documents/{}", self.base_url, document_type);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch document")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch document with status: {}",
                response.status()
            ));
        }

        response
            .json::<DocumentResponse>()
            .await
            .context("Failed to parse document response")
    }

    pub async fn is_api_available(&self) -> bool {
        self.health_check().await.is_ok()
    }
}