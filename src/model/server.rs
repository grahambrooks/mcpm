use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport type for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Http,
    Sse,
}

impl Default for TransportType {
    fn default() -> Self {
        Self::Stdio
    }
}

/// Configuration for an MCP server (as stored in IDE configs)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerConfig {
    Stdio(StdioServerConfig),
    Http(HttpServerConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioServerConfig {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    pub url: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

impl ServerConfig {
    pub fn transport_type(&self) -> TransportType {
        match self {
            ServerConfig::Stdio(_) => TransportType::Stdio,
            ServerConfig::Http(_) => TransportType::Http,
        }
    }

    pub fn display_command(&self) -> String {
        match self {
            ServerConfig::Stdio(cfg) => {
                if cfg.args.is_empty() {
                    cfg.command.clone()
                } else {
                    format!("{} {}", cfg.command, cfg.args.join(" "))
                }
            }
            ServerConfig::Http(cfg) => cfg.url.clone(),
        }
    }
}

/// An installed server in an IDE
#[derive(Debug, Clone)]
pub struct InstalledServer {
    pub name: String,
    pub config: ServerConfig,
    pub ide_id: String,
}

/// A server from a registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryServer {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub install_command: Option<String>,
    #[serde(default)]
    pub install_args: Vec<String>,
    #[serde(default)]
    pub env_vars: Vec<EnvVarSpec>,
    #[serde(default)]
    pub registry_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarSpec {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

impl RegistryServer {
    pub fn to_server_config(&self, env: HashMap<String, String>) -> ServerConfig {
        ServerConfig::Stdio(StdioServerConfig {
            command: self
                .install_command
                .clone()
                .unwrap_or_else(|| "npx".to_string()),
            args: self.install_args.clone(),
            env,
        })
    }
}
