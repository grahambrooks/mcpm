use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport type for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    #[default]
    Stdio,
    Http,
    Sse,
    #[serde(rename = "streamable-http")]
    StreamableHttp,
}

impl std::fmt::Display for TransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportType::Stdio => write!(f, "stdio"),
            TransportType::Http => write!(f, "http"),
            TransportType::Sse => write!(f, "sse"),
            TransportType::StreamableHttp => write!(f, "streamable-http"),
        }
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

/// Information about a distributable package for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageInfo {
    /// Package registry type (e.g., "npm", "oci", "pypi")
    #[serde(default)]
    pub registry_type: String,
    /// Package identifier (e.g., "@modelcontextprotocol/server-filesystem")
    #[serde(default)]
    pub identifier: String,
    /// Package version
    #[serde(default)]
    pub version: Option<String>,
    /// Transport type for this package
    #[serde(default)]
    pub transport_type: TransportType,
    /// Transport URL (for non-stdio transports)
    #[serde(default)]
    pub transport_url: Option<String>,
    /// Runtime hint (e.g., "docker")
    #[serde(default)]
    pub runtime_hint: Option<String>,
    /// Environment variables for this package
    #[serde(default)]
    pub env_vars: Vec<EnvVarSpec>,
}

/// A remote endpoint for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteInfo {
    /// Transport type (e.g., "streamable-http", "sse")
    #[serde(default)]
    pub transport_type: TransportType,
    /// URL of the remote endpoint
    #[serde(default)]
    pub url: String,
    /// Headers required for the remote endpoint
    #[serde(default)]
    pub headers: Vec<RemoteHeader>,
}

/// A header required by a remote endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteHeader {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub is_secret: bool,
    #[serde(default)]
    pub format: Option<String>,
}

/// A server from a registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryServer {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub install_command: Option<String>,
    #[serde(default)]
    pub install_args: Vec<String>,
    #[serde(default)]
    pub env_vars: Vec<EnvVarSpec>,
    #[serde(default)]
    pub packages: Vec<PackageInfo>,
    #[serde(default)]
    pub remotes: Vec<RemoteInfo>,
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
    #[serde(default)]
    pub is_secret: bool,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub default_value: Option<String>,
}

impl RegistryServer {
    /// Get the display name: title if available and non-empty, otherwise name
    pub fn display_name(&self) -> &str {
        self.title
            .as_deref()
            .filter(|t| !t.is_empty())
            .unwrap_or(&self.name)
    }

    /// Find the preferred stdio package (npm > pypi > oci)
    pub fn preferred_package(&self) -> Option<&PackageInfo> {
        // Prefer npm stdio packages
        let npm = self
            .packages
            .iter()
            .find(|p| p.registry_type == "npm" && p.transport_type == TransportType::Stdio);
        if npm.is_some() {
            return npm;
        }

        // Then pypi stdio
        let pypi = self
            .packages
            .iter()
            .find(|p| p.registry_type == "pypi" && p.transport_type == TransportType::Stdio);
        if pypi.is_some() {
            return pypi;
        }

        // Then any stdio package
        let stdio = self
            .packages
            .iter()
            .find(|p| p.transport_type == TransportType::Stdio);
        if stdio.is_some() {
            return stdio;
        }

        // Fall back to first package
        self.packages.first()
    }

    /// Collect all env vars from the server itself plus the preferred package
    pub fn all_env_vars(&self) -> Vec<&EnvVarSpec> {
        let mut vars: Vec<&EnvVarSpec> = self.env_vars.iter().collect();

        if let Some(pkg) = self.preferred_package() {
            for var in &pkg.env_vars {
                if !vars.iter().any(|v| v.name == var.name) {
                    vars.push(var);
                }
            }
        }

        vars
    }

    /// Determine the primary transport type for this server
    pub fn primary_transport(&self) -> TransportType {
        if let Some(pkg) = self.preferred_package() {
            return pkg.transport_type.clone();
        }
        if let Some(remote) = self.remotes.first() {
            return remote.transport_type.clone();
        }
        if self.install_command.is_some() {
            return TransportType::Stdio;
        }
        TransportType::Stdio
    }

    pub fn to_server_config(&self, env: HashMap<String, String>) -> ServerConfig {
        // If there's a preferred stdio package, use it
        if let Some(pkg) = self.preferred_package() {
            if pkg.transport_type == TransportType::Stdio {
                let (command, args) = Self::package_to_command(pkg);
                return ServerConfig::Stdio(StdioServerConfig {
                    command,
                    args,
                    env,
                });
            }

            // Non-stdio package with a transport URL
            if let Some(url) = &pkg.transport_url {
                return ServerConfig::Http(HttpServerConfig {
                    url: url.clone(),
                    headers: HashMap::new(),
                });
            }
        }

        // If there's a remote endpoint, use it
        if let Some(remote) = self.remotes.first() {
            let mut headers = HashMap::new();
            for header in &remote.headers {
                if let Some(value) = env.get(&header.name) {
                    headers.insert(header.name.clone(), value.clone());
                }
            }
            return ServerConfig::Http(HttpServerConfig {
                url: remote.url.clone(),
                headers,
            });
        }

        // Fallback: use legacy install_command/install_args
        ServerConfig::Stdio(StdioServerConfig {
            command: self
                .install_command
                .clone()
                .unwrap_or_else(|| "npx".to_string()),
            args: self.install_args.clone(),
            env,
        })
    }

    /// Convert a package to a command + args for stdio execution
    fn package_to_command(pkg: &PackageInfo) -> (String, Vec<String>) {
        match pkg.registry_type.as_str() {
            "npm" => (
                "npx".to_string(),
                vec!["-y".to_string(), pkg.identifier.clone()],
            ),
            "pypi" => (
                "uvx".to_string(),
                vec![pkg.identifier.clone()],
            ),
            _ => (
                "npx".to_string(),
                vec!["-y".to_string(), pkg.identifier.clone()],
            ),
        }
    }
}
