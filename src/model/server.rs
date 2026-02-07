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
    pub(crate) fn package_to_command(pkg: &PackageInfo) -> (String, Vec<String>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_server(name: &str, version: Option<&str>) -> RegistryServer {
        RegistryServer {
            name: name.to_string(),
            description: String::new(),
            title: None,
            version: version.map(|v| v.to_string()),
            repository: None,
            vendor: None,
            homepage: None,
            license: None,
            icon_url: None,
            keywords: vec![],
            install_command: None,
            install_args: vec![],
            env_vars: vec![],
            packages: vec![],
            remotes: vec![],
            registry_source: String::new(),
        }
    }

    fn make_package(registry_type: &str, identifier: &str, transport: TransportType) -> PackageInfo {
        PackageInfo {
            registry_type: registry_type.to_string(),
            identifier: identifier.to_string(),
            version: None,
            transport_type: transport,
            transport_url: None,
            runtime_hint: None,
            env_vars: vec![],
        }
    }

    // --- TransportType Display tests ---

    #[test]
    fn transport_type_display() {
        assert_eq!(TransportType::Stdio.to_string(), "stdio");
        assert_eq!(TransportType::Http.to_string(), "http");
        assert_eq!(TransportType::Sse.to_string(), "sse");
        assert_eq!(TransportType::StreamableHttp.to_string(), "streamable-http");
    }

    // --- ServerConfig::transport_type tests ---

    #[test]
    fn server_config_transport_type_stdio() {
        let config = ServerConfig::Stdio(StdioServerConfig {
            command: "npx".to_string(),
            args: vec![],
            env: HashMap::new(),
        });
        assert!(matches!(config.transport_type(), TransportType::Stdio));
    }

    #[test]
    fn server_config_transport_type_http() {
        let config = ServerConfig::Http(HttpServerConfig {
            url: "https://example.com".to_string(),
            headers: HashMap::new(),
        });
        assert!(matches!(config.transport_type(), TransportType::Http));
    }

    // --- ServerConfig::display_command tests ---

    #[test]
    fn display_command_without_args() {
        let config = ServerConfig::Stdio(StdioServerConfig {
            command: "npx".to_string(),
            args: vec![],
            env: HashMap::new(),
        });
        assert_eq!(config.display_command(), "npx");
    }

    #[test]
    fn display_command_with_args() {
        let config = ServerConfig::Stdio(StdioServerConfig {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "server".to_string()],
            env: HashMap::new(),
        });
        assert_eq!(config.display_command(), "npx -y server");
    }

    #[test]
    fn display_command_http() {
        let config = ServerConfig::Http(HttpServerConfig {
            url: "https://example.com/mcp".to_string(),
            headers: HashMap::new(),
        });
        assert_eq!(config.display_command(), "https://example.com/mcp");
    }

    // --- RegistryServer::display_name tests ---

    #[test]
    fn display_name_with_title() {
        let mut server = make_test_server("my-server", None);
        server.title = Some("My Server".to_string());
        assert_eq!(server.display_name(), "My Server");
    }

    #[test]
    fn display_name_without_title() {
        let server = make_test_server("my-server", None);
        assert_eq!(server.display_name(), "my-server");
    }

    #[test]
    fn display_name_with_empty_title() {
        let mut server = make_test_server("my-server", None);
        server.title = Some(String::new());
        assert_eq!(server.display_name(), "my-server");
    }

    // --- RegistryServer::preferred_package tests ---

    #[test]
    fn preferred_package_none_when_empty() {
        let server = make_test_server("test", None);
        assert!(server.preferred_package().is_none());
    }

    #[test]
    fn preferred_package_prefers_npm_stdio() {
        let mut server = make_test_server("test", None);
        server.packages = vec![
            make_package("pypi", "pypi-pkg", TransportType::Stdio),
            make_package("npm", "npm-pkg", TransportType::Stdio),
        ];
        let pkg = server.preferred_package().unwrap();
        assert_eq!(pkg.registry_type, "npm");
        assert_eq!(pkg.identifier, "npm-pkg");
    }

    #[test]
    fn preferred_package_prefers_pypi_stdio_over_other() {
        let mut server = make_test_server("test", None);
        server.packages = vec![
            make_package("oci", "oci-pkg", TransportType::Stdio),
            make_package("pypi", "pypi-pkg", TransportType::Stdio),
        ];
        let pkg = server.preferred_package().unwrap();
        assert_eq!(pkg.registry_type, "pypi");
    }

    #[test]
    fn preferred_package_any_stdio_over_non_stdio() {
        let mut server = make_test_server("test", None);
        server.packages = vec![
            make_package("npm", "npm-http", TransportType::Http),
            make_package("oci", "oci-stdio", TransportType::Stdio),
        ];
        let pkg = server.preferred_package().unwrap();
        assert_eq!(pkg.identifier, "oci-stdio");
    }

    #[test]
    fn preferred_package_falls_back_to_first() {
        let mut server = make_test_server("test", None);
        server.packages = vec![
            make_package("npm", "npm-http", TransportType::Http),
            make_package("pypi", "pypi-sse", TransportType::Sse),
        ];
        let pkg = server.preferred_package().unwrap();
        assert_eq!(pkg.identifier, "npm-http");
    }

    // --- RegistryServer::all_env_vars tests ---

    #[test]
    fn all_env_vars_combines_and_deduplicates() {
        let mut server = make_test_server("test", None);
        server.env_vars = vec![
            EnvVarSpec {
                name: "API_KEY".to_string(),
                description: None,
                required: true,
                is_secret: false,
                format: None,
                default_value: None,
            },
            EnvVarSpec {
                name: "TOKEN".to_string(),
                description: None,
                required: false,
                is_secret: false,
                format: None,
                default_value: None,
            },
        ];
        server.packages = vec![{
            let mut pkg = make_package("npm", "test-pkg", TransportType::Stdio);
            pkg.env_vars = vec![
                EnvVarSpec {
                    name: "API_KEY".to_string(), // duplicate
                    description: Some("from package".to_string()),
                    required: false,
                    is_secret: false,
                    format: None,
                    default_value: None,
                },
                EnvVarSpec {
                    name: "SECRET".to_string(),
                    description: None,
                    required: false,
                    is_secret: true,
                    format: None,
                    default_value: None,
                },
            ];
            pkg
        }];

        let vars = server.all_env_vars();
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names, vec!["API_KEY", "TOKEN", "SECRET"]);
    }

    // --- RegistryServer::primary_transport tests ---

    #[test]
    fn primary_transport_from_package() {
        let mut server = make_test_server("test", None);
        server.packages = vec![make_package("npm", "pkg", TransportType::Stdio)];
        assert!(matches!(server.primary_transport(), TransportType::Stdio));
    }

    #[test]
    fn primary_transport_from_remote() {
        let mut server = make_test_server("test", None);
        server.remotes = vec![RemoteInfo {
            transport_type: TransportType::StreamableHttp,
            url: "https://example.com".to_string(),
            headers: vec![],
        }];
        assert!(matches!(
            server.primary_transport(),
            TransportType::StreamableHttp
        ));
    }

    #[test]
    fn primary_transport_fallback_stdio() {
        let server = make_test_server("test", None);
        assert!(matches!(server.primary_transport(), TransportType::Stdio));
    }

    // --- RegistryServer::to_server_config tests ---

    #[test]
    fn to_server_config_stdio_package() {
        let mut server = make_test_server("test", None);
        server.packages = vec![make_package("npm", "@mcp/server-fs", TransportType::Stdio)];
        let config = server.to_server_config(HashMap::new());
        match &config {
            ServerConfig::Stdio(cfg) => {
                assert_eq!(cfg.command, "npx");
                assert_eq!(cfg.args, vec!["-y", "@mcp/server-fs"]);
            }
            _ => panic!("expected Stdio config"),
        }
    }

    #[test]
    fn to_server_config_remote() {
        let mut server = make_test_server("test", None);
        server.remotes = vec![RemoteInfo {
            transport_type: TransportType::StreamableHttp,
            url: "https://api.example.com/mcp".to_string(),
            headers: vec![],
        }];
        let config = server.to_server_config(HashMap::new());
        match &config {
            ServerConfig::Http(cfg) => {
                assert_eq!(cfg.url, "https://api.example.com/mcp");
            }
            _ => panic!("expected Http config"),
        }
    }

    #[test]
    fn to_server_config_fallback_uses_install_command() {
        let mut server = make_test_server("test", None);
        server.install_command = Some("my-command".to_string());
        server.install_args = vec!["--flag".to_string()];
        let config = server.to_server_config(HashMap::new());
        match &config {
            ServerConfig::Stdio(cfg) => {
                assert_eq!(cfg.command, "my-command");
                assert_eq!(cfg.args, vec!["--flag"]);
            }
            _ => panic!("expected Stdio config"),
        }
    }

    #[test]
    fn to_server_config_fallback_defaults_to_npx() {
        let server = make_test_server("test", None);
        let config = server.to_server_config(HashMap::new());
        match &config {
            ServerConfig::Stdio(cfg) => {
                assert_eq!(cfg.command, "npx");
            }
            _ => panic!("expected Stdio config"),
        }
    }

    // --- RegistryServer::package_to_command tests ---

    #[test]
    fn package_to_command_npm() {
        let pkg = make_package("npm", "@mcp/server", TransportType::Stdio);
        let (cmd, args) = RegistryServer::package_to_command(&pkg);
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["-y", "@mcp/server"]);
    }

    #[test]
    fn package_to_command_pypi() {
        let pkg = make_package("pypi", "mcp-server", TransportType::Stdio);
        let (cmd, args) = RegistryServer::package_to_command(&pkg);
        assert_eq!(cmd, "uvx");
        assert_eq!(args, vec!["mcp-server"]);
    }

    #[test]
    fn package_to_command_unknown_falls_back_to_npx() {
        let pkg = make_package("oci", "some-image", TransportType::Stdio);
        let (cmd, args) = RegistryServer::package_to_command(&pkg);
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["-y", "some-image"]);
    }
}
