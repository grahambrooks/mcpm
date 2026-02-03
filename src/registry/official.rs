use async_trait::async_trait;
use serde::Deserialize;

use crate::error::{McpmError, Result};
use crate::model::RegistryServer;
use crate::registry::traits::RegistryClient;

const GITHUB_API_URL: &str = "https://api.github.com/repos/modelcontextprotocol/servers";
const GITHUB_REPO_URL: &str = "https://github.com/modelcontextprotocol/servers";

pub struct OfficialRegistryClient {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct GitHubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    name: Option<String>,
    description: Option<String>,
    license: Option<String>,
}

impl OfficialRegistryClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mcpm/0.1.0")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    async fn fetch_server_details(
        &self,
        server_path: &str,
        server_name: &str,
    ) -> Option<RegistryServer> {
        // Try to fetch package.json for the server
        let package_url = format!(
            "https://raw.githubusercontent.com/modelcontextprotocol/servers/main/{}/package.json",
            server_path
        );

        let (description, license, package_name) =
            if let Ok(response) = self.client.get(&package_url).send().await {
                if response.status().is_success() {
                    if let Ok(package) = response.json::<PackageJson>().await {
                        (package.description, package.license, package.name)
                    } else {
                        (None, None, None)
                    }
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

        let display_name = server_name.to_string();
        let npm_package =
            package_name.unwrap_or_else(|| format!("@modelcontextprotocol/server-{}", server_name));

        Some(RegistryServer {
            name: display_name,
            description: description.unwrap_or_else(|| format!("MCP server: {}", server_name)),
            repository: Some(format!("{}/tree/main/{}", GITHUB_REPO_URL, server_path)),
            vendor: Some("Model Context Protocol".to_string()),
            homepage: Some(GITHUB_REPO_URL.to_string()),
            license,
            install_command: Some("npx".to_string()),
            install_args: vec!["-y".to_string(), npm_package],
            env_vars: Vec::new(),
            registry_source: "GitHub MCP".to_string(),
        })
    }
}

impl Default for OfficialRegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegistryClient for OfficialRegistryClient {
    fn name(&self) -> &str {
        "GitHub MCP"
    }

    fn base_url(&self) -> &str {
        GITHUB_REPO_URL
    }

    async fn list_servers(&self) -> Result<Vec<RegistryServer>> {
        // Fetch the src directory contents from GitHub API
        let url = format!("{}/contents/src", GITHUB_API_URL);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| McpmError::Registry(format!("Failed to fetch from GitHub: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpmError::Registry(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let contents: Vec<GitHubContent> = response
            .json()
            .await
            .map_err(|e| McpmError::Registry(format!("Failed to parse GitHub response: {}", e)))?;

        // Filter for directories (each directory is a server)
        let server_dirs: Vec<_> = contents
            .into_iter()
            .filter(|c| c.content_type == "dir")
            .collect();

        let mut servers = Vec::new();

        // Fetch details for each server
        for dir in server_dirs {
            if let Some(server) = self.fetch_server_details(&dir.path, &dir.name).await {
                servers.push(server);
            }
        }

        // If we couldn't fetch individual details, create basic entries
        if servers.is_empty() {
            // Fallback: list known official servers
            servers = Self::known_servers();
        }

        Ok(servers)
    }

    async fn search(&self, query: &str) -> Result<Vec<RegistryServer>> {
        let all_servers = self.list_servers().await?;
        let query_lower = query.to_lowercase();

        Ok(all_servers
            .into_iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.description.to_lowercase().contains(&query_lower)
            })
            .collect())
    }

    async fn get_server(&self, name: &str) -> Result<Option<RegistryServer>> {
        let servers = self.list_servers().await?;
        Ok(servers.into_iter().find(|s| s.name == name))
    }
}

impl OfficialRegistryClient {
    /// Fallback list of known official MCP servers
    fn known_servers() -> Vec<RegistryServer> {
        let servers = vec![
            (
                "filesystem",
                "Secure file operations with configurable access controls",
                "@modelcontextprotocol/server-filesystem",
            ),
            (
                "github",
                "Repository management, file operations, and GitHub API integration",
                "@modelcontextprotocol/server-github",
            ),
            (
                "gitlab",
                "GitLab API integration for project management",
                "@modelcontextprotocol/server-gitlab",
            ),
            (
                "google-maps",
                "Location services, directions, and place details",
                "@modelcontextprotocol/server-google-maps",
            ),
            (
                "memory",
                "Knowledge graph-based persistent memory system",
                "@modelcontextprotocol/server-memory",
            ),
            (
                "postgres",
                "Read-only PostgreSQL database access with schema inspection",
                "@modelcontextprotocol/server-postgres",
            ),
            (
                "puppeteer",
                "Browser automation and web scraping capabilities",
                "@modelcontextprotocol/server-puppeteer",
            ),
            (
                "brave-search",
                "Web and local search using Brave's Search API",
                "@modelcontextprotocol/server-brave-search",
            ),
            (
                "fetch",
                "Web content fetching and conversion for efficient LLM usage",
                "@modelcontextprotocol/server-fetch",
            ),
            (
                "slack",
                "Channel management and messaging for Slack workspaces",
                "@modelcontextprotocol/server-slack",
            ),
            (
                "sequential-thinking",
                "Dynamic problem-solving through thought sequences",
                "@modelcontextprotocol/server-sequential-thinking",
            ),
            (
                "aws-kb-retrieval",
                "Retrieval from AWS Knowledge Base using Bedrock Agent Runtime",
                "@modelcontextprotocol/server-aws-kb-retrieval",
            ),
            (
                "everart",
                "AI image generation using various models",
                "@modelcontextprotocol/server-everart",
            ),
            (
                "everything",
                "Reference server with prompts, resources, and tools",
                "@modelcontextprotocol/server-everything",
            ),
            (
                "gdrive",
                "Google Drive integration for file access and search",
                "@modelcontextprotocol/server-gdrive",
            ),
            (
                "sentry",
                "Retrieving and analyzing issues from Sentry.io",
                "@modelcontextprotocol/server-sentry",
            ),
            (
                "time",
                "Time and timezone conversion capabilities",
                "@modelcontextprotocol/server-time",
            ),
        ];

        servers
            .into_iter()
            .map(|(name, description, package)| RegistryServer {
                name: name.to_string(),
                description: description.to_string(),
                repository: Some(format!("{}/tree/main/src/{}", GITHUB_REPO_URL, name)),
                vendor: Some("Model Context Protocol".to_string()),
                homepage: Some(GITHUB_REPO_URL.to_string()),
                license: Some("MIT".to_string()),
                install_command: Some("npx".to_string()),
                install_args: vec!["-y".to_string(), package.to_string()],
                env_vars: Vec::new(),
                registry_source: "GitHub MCP".to_string(),
            })
            .collect()
    }
}
