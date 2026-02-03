use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{McpmError, Result};
use crate::ide::traits::IdeAdapter;
use crate::model::{InstalledServer, ServerConfig, StdioServerConfig};

/// Continue.dev uses a different config format (JSON with experimental MCP support)
/// We'll handle the conversion between formats
pub struct ContinueDevAdapter {
    config_path: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct ContinueConfig {
    #[serde(default)]
    experimental: ContinueExperimental,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct ContinueExperimental {
    #[serde(rename = "modelContextProtocolServers", default)]
    mcp_servers: Vec<ContinueMcpServer>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ContinueMcpServer {
    name: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
}

impl ContinueDevAdapter {
    pub fn new() -> Self {
        let config_path = Self::default_config_path();
        Self { config_path }
    }

    fn default_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".continue/config.json")
    }

    fn read_config(&self) -> Result<ContinueConfig> {
        if !self.config_path.exists() {
            return Ok(ContinueConfig::default());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let config: ContinueConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn write_config(&self, config: &ContinueConfig) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn convert_to_server_config(server: &ContinueMcpServer) -> ServerConfig {
        ServerConfig::Stdio(StdioServerConfig {
            command: server.command.clone(),
            args: server.args.clone(),
            env: server.env.clone(),
        })
    }

    fn convert_from_server_config(name: &str, config: &ServerConfig) -> Option<ContinueMcpServer> {
        match config {
            ServerConfig::Stdio(stdio) => Some(ContinueMcpServer {
                name: name.to_string(),
                command: stdio.command.clone(),
                args: stdio.args.clone(),
                env: stdio.env.clone(),
            }),
            ServerConfig::Http(_) => None, // Continue doesn't support HTTP transport
        }
    }
}

impl Default for ContinueDevAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdeAdapter for ContinueDevAdapter {
    fn id(&self) -> &str {
        "continue"
    }

    fn name(&self) -> &str {
        "Continue.dev"
    }

    fn is_detected(&self) -> bool {
        // Continue is detected if .continue directory exists
        dirs::home_dir()
            .map(|h| h.join(".continue").exists())
            .unwrap_or(false)
    }

    fn config_path(&self) -> Option<String> {
        Some(self.config_path.to_string_lossy().to_string())
    }

    async fn list_servers(&self) -> Result<Vec<InstalledServer>> {
        let config = self.read_config()?;
        Ok(config
            .experimental
            .mcp_servers
            .iter()
            .map(|server| InstalledServer {
                name: server.name.clone(),
                config: Self::convert_to_server_config(server),
                ide_id: self.id().to_string(),
            })
            .collect())
    }

    async fn get_server(&self, name: &str) -> Result<Option<InstalledServer>> {
        let config = self.read_config()?;
        Ok(config
            .experimental
            .mcp_servers
            .iter()
            .find(|s| s.name == name)
            .map(|server| InstalledServer {
                name: server.name.clone(),
                config: Self::convert_to_server_config(server),
                ide_id: self.id().to_string(),
            }))
    }

    async fn install_server(&self, name: &str, server_config: ServerConfig) -> Result<()> {
        let mut config = self.read_config()?;

        let mcp_server =
            Self::convert_from_server_config(name, &server_config).ok_or_else(|| {
                McpmError::InstallFailed("Continue.dev only supports stdio transport".to_string())
            })?;

        // Remove existing server with same name if present
        config.experimental.mcp_servers.retain(|s| s.name != name);
        config.experimental.mcp_servers.push(mcp_server);

        self.write_config(&config)?;
        Ok(())
    }

    async fn remove_server(&self, name: &str) -> Result<()> {
        let mut config = self.read_config()?;
        let original_len = config.experimental.mcp_servers.len();
        config.experimental.mcp_servers.retain(|s| s.name != name);

        if config.experimental.mcp_servers.len() == original_len {
            return Err(McpmError::ServerNotFound(name.to_string()));
        }

        self.write_config(&config)?;
        Ok(())
    }

    async fn update_server(&self, name: &str, server_config: ServerConfig) -> Result<()> {
        let mut config = self.read_config()?;

        let exists = config
            .experimental
            .mcp_servers
            .iter()
            .any(|s| s.name == name);
        if !exists {
            return Err(McpmError::ServerNotFound(name.to_string()));
        }

        let mcp_server =
            Self::convert_from_server_config(name, &server_config).ok_or_else(|| {
                McpmError::InstallFailed("Continue.dev only supports stdio transport".to_string())
            })?;

        config.experimental.mcp_servers.retain(|s| s.name != name);
        config.experimental.mcp_servers.push(mcp_server);

        self.write_config(&config)?;
        Ok(())
    }

    async fn get_servers_map(&self) -> Result<HashMap<String, ServerConfig>> {
        let config = self.read_config()?;
        Ok(config
            .experimental
            .mcp_servers
            .iter()
            .map(|s| (s.name.clone(), Self::convert_to_server_config(s)))
            .collect())
    }
}
