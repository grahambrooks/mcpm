use async_trait::async_trait;
use serde::Deserialize;

use crate::error::{McpmError, Result};
use crate::model::RegistryServer;
use crate::registry::traits::RegistryClient;

const SMITHERY_REGISTRY_URL: &str = "https://registry.smithery.ai";

pub struct SmitheryRegistryClient {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct SmitheryListResponse {
    servers: Vec<SmitheryServer>,
}

#[derive(Debug, Deserialize)]
struct SmitheryServer {
    #[serde(rename = "qualifiedName")]
    qualified_name: String,
    #[serde(default, rename = "displayName")]
    display_name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default, rename = "useCount")]
    #[allow(dead_code)]
    use_count: Option<u64>,
    #[serde(default)]
    connections: Option<Vec<SmitheryConnection>>,
}

#[derive(Debug, Deserialize)]
struct SmitheryConnection {
    #[serde(default, rename = "type")]
    #[allow(dead_code)]
    connection_type: Option<String>,
    #[serde(default)]
    config: Option<SmitheryConfig>,
}

#[derive(Debug, Deserialize)]
struct SmitheryConfig {
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    args: Option<Vec<String>>,
}

impl SmitheryRegistryClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn convert_server(server: SmitheryServer) -> RegistryServer {
        let name = server
            .display_name
            .unwrap_or_else(|| server.qualified_name.clone());

        // Extract install info from connections if available
        let (install_command, install_args) = server
            .connections
            .as_ref()
            .and_then(|conns| conns.first())
            .and_then(|conn| conn.config.as_ref())
            .map(|config| {
                (
                    config.command.clone().unwrap_or_else(|| "npx".to_string()),
                    config.args.clone().unwrap_or_default(),
                )
            })
            .unwrap_or_else(|| Self::generate_install_info(&server.qualified_name));

        RegistryServer {
            name,
            description: server.description.unwrap_or_default(),
            repository: None,
            vendor: Some(server.qualified_name.clone()),
            homepage: server.homepage,
            license: None,
            install_command: Some(install_command),
            install_args,
            env_vars: Vec::new(),
            registry_source: "Smithery.ai".to_string(),
        }
    }

    fn generate_install_info(qualified_name: &str) -> (String, Vec<String>) {
        // Smithery uses npx @smithery/cli run <qualified-name>
        (
            "npx".to_string(),
            vec![
                "-y".to_string(),
                "@smithery/cli".to_string(),
                "run".to_string(),
                qualified_name.to_string(),
            ],
        )
    }
}

impl Default for SmitheryRegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegistryClient for SmitheryRegistryClient {
    fn name(&self) -> &str {
        "Smithery.ai"
    }

    fn base_url(&self) -> &str {
        SMITHERY_REGISTRY_URL
    }

    async fn list_servers(&self) -> Result<Vec<RegistryServer>> {
        let url = format!("{}/servers", SMITHERY_REGISTRY_URL);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| McpmError::Registry(format!("Failed to fetch servers: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpmError::Registry(format!(
                "Registry returned status: {}",
                response.status()
            )));
        }

        let data: SmitheryListResponse = response
            .json()
            .await
            .map_err(|e| McpmError::Registry(format!("Failed to parse response: {}", e)))?;

        Ok(data.servers.into_iter().map(Self::convert_server).collect())
    }

    async fn search(&self, query: &str) -> Result<Vec<RegistryServer>> {
        let url = format!(
            "{}/servers?q={}",
            SMITHERY_REGISTRY_URL,
            urlencoding::encode(query)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| McpmError::Registry(format!("Failed to search servers: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpmError::Registry(format!(
                "Registry returned status: {}",
                response.status()
            )));
        }

        let data: SmitheryListResponse = response
            .json()
            .await
            .map_err(|e| McpmError::Registry(format!("Failed to parse response: {}", e)))?;

        Ok(data.servers.into_iter().map(Self::convert_server).collect())
    }

    async fn get_server(&self, name: &str) -> Result<Option<RegistryServer>> {
        // Search for the specific server
        let servers = self.search(name).await?;
        Ok(servers
            .into_iter()
            .find(|s| s.name == name || s.vendor.as_deref() == Some(name)))
    }
}
