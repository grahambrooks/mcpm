use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::error::{McpmError, Result};
use crate::model::{
    EnvVarSpec, PackageInfo, RegistryServer, RemoteHeader, RemoteInfo, TransportType,
};
use crate::registry::traits::RegistryClient;

const MCP_REGISTRY_V0_URL: &str = "https://registry.modelcontextprotocol.io/v0/servers";
const MCP_REGISTRY_V01_URL: &str = "https://registry.modelcontextprotocol.io/v0.1/servers";

pub struct OfficialRegistryClient {
    client: reqwest::Client,
    base_url: &'static str,
    display_name: &'static str,
}

// --- Deserialization structs matching the MCP Registry API ---

#[derive(Debug, Deserialize)]
struct ApiResponse {
    servers: Vec<ApiServerEntry>,
    #[serde(default)]
    metadata: Option<ApiMetadata>,
}

#[derive(Debug, Deserialize)]
struct ApiMetadata {
    #[serde(default, rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiServerEntry {
    server: ApiServer,
    #[serde(default, rename = "_meta")]
    #[allow(dead_code)]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ApiServer {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    repository: Option<ApiRepository>,
    #[serde(default, rename = "websiteUrl")]
    website_url: Option<String>,
    #[serde(default)]
    packages: Option<Vec<ApiPackage>>,
    #[serde(default)]
    remotes: Option<Vec<ApiRemote>>,
    #[serde(default, rename = "_meta")]
    server_meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ApiRepository {
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiPackage {
    #[serde(default, rename = "registryType")]
    registry_type: Option<String>,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    transport: Option<ApiTransport>,
    #[serde(default, rename = "runtimeHint")]
    runtime_hint: Option<String>,
    #[serde(default, rename = "environmentVariables")]
    environment_variables: Option<Vec<ApiEnvVar>>,
}

#[derive(Debug, Deserialize)]
struct ApiTransport {
    #[serde(default, rename = "type")]
    transport_type: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiRemote {
    #[serde(default, rename = "type")]
    remote_type: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    headers: Option<Vec<ApiHeader>>,
}

#[derive(Debug, Deserialize)]
struct ApiHeader {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "isRequired")]
    is_required: Option<bool>,
    #[serde(default, rename = "isSecret")]
    is_secret: Option<bool>,
    #[serde(default)]
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiEnvVar {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "isRequired")]
    is_required: Option<bool>,
    #[serde(default, rename = "isSecret")]
    is_secret: Option<bool>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    default: Option<String>,
}

// --- Implementation ---

impl OfficialRegistryClient {
    pub fn new() -> Self {
        Self::with_url(MCP_REGISTRY_V01_URL, "MCP Registry")
    }

    pub fn with_url(base_url: &'static str, display_name: &'static str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mcpm/0.1.0")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url,
            display_name,
        }
    }

    pub fn v0() -> Self {
        Self::with_url(MCP_REGISTRY_V0_URL, "MCP Registry (v0)")
    }

    pub fn v01() -> Self {
        Self::with_url(MCP_REGISTRY_V01_URL, "MCP Registry")
    }

    /// Fetch a single page from the API
    async fn fetch_page(&self, cursor: Option<&str>, query: Option<&str>) -> Result<ApiResponse> {
        let mut url = self.base_url.to_string();
        let mut params = Vec::new();

        if let Some(q) = query {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", urlencoding::encode(c)));
        }
        // Request larger pages
        params.push("limit=100".to_string());

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| McpmError::Registry(format!("Failed to fetch from MCP Registry: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpmError::Registry(format!(
                "MCP Registry returned status: {}",
                response.status()
            )));
        }

        let data: ApiResponse = response
            .json()
            .await
            .map_err(|e| McpmError::Registry(format!("Failed to parse MCP Registry response: {}", e)))?;

        Ok(data)
    }

    /// Convert an API server entry to our internal RegistryServer model
    fn convert_entry(entry: ApiServerEntry, source_name: &str) -> RegistryServer {
        let server = entry.server;

        // Extract packages
        let packages: Vec<PackageInfo> = server
            .packages
            .unwrap_or_default()
            .into_iter()
            .map(Self::convert_package)
            .collect();

        // Extract remotes
        let remotes: Vec<RemoteInfo> = server
            .remotes
            .unwrap_or_default()
            .into_iter()
            .map(Self::convert_remote)
            .collect();

        // Derive install_command and install_args from the preferred package
        let (install_command, install_args) = Self::derive_install_info(&packages);

        // Collect top-level env vars from the preferred package
        let env_vars: Vec<EnvVarSpec> = packages
            .first()
            .map(|p| p.env_vars.clone())
            .unwrap_or_default();

        // Extract metadata from _meta
        let (license, vendor, keywords) = Self::extract_meta(&server.server_meta);

        let repository = server.repository.and_then(|r| r.url);

        RegistryServer {
            name: server.name,
            description: server.description.unwrap_or_default(),
            title: server.title,
            version: server.version,
            repository,
            vendor,
            homepage: server.website_url,
            license,
            icon_url: None,
            keywords,
            install_command,
            install_args,
            env_vars,
            packages,
            remotes,
            registry_source: source_name.to_string(),
        }
    }

    fn convert_package(pkg: ApiPackage) -> PackageInfo {
        let transport_type = pkg
            .transport
            .as_ref()
            .and_then(|t| t.transport_type.as_deref())
            .map(Self::parse_transport_type)
            .unwrap_or_default();

        let transport_url = pkg.transport.as_ref().and_then(|t| t.url.clone());

        let env_vars: Vec<EnvVarSpec> = pkg
            .environment_variables
            .unwrap_or_default()
            .into_iter()
            .map(Self::convert_env_var)
            .collect();

        PackageInfo {
            registry_type: pkg.registry_type.unwrap_or_default(),
            identifier: pkg.identifier.unwrap_or_default(),
            version: pkg.version,
            transport_type,
            transport_url,
            runtime_hint: pkg.runtime_hint,
            env_vars,
        }
    }

    fn convert_remote(remote: ApiRemote) -> RemoteInfo {
        let transport_type = remote
            .remote_type
            .as_deref()
            .map(Self::parse_transport_type)
            .unwrap_or(TransportType::StreamableHttp);

        let headers: Vec<RemoteHeader> = remote
            .headers
            .unwrap_or_default()
            .into_iter()
            .map(|h| RemoteHeader {
                name: h.name,
                description: h.description,
                is_required: h.is_required.unwrap_or(false),
                is_secret: h.is_secret.unwrap_or(false),
                format: h.format,
            })
            .collect();

        RemoteInfo {
            transport_type,
            url: remote.url.unwrap_or_default(),
            headers,
        }
    }

    fn convert_env_var(var: ApiEnvVar) -> EnvVarSpec {
        EnvVarSpec {
            name: var.name,
            description: var.description,
            required: var.is_required.unwrap_or(false),
            is_secret: var.is_secret.unwrap_or(false),
            format: var.format,
            default_value: var.default,
        }
    }

    fn parse_transport_type(s: &str) -> TransportType {
        match s {
            "stdio" => TransportType::Stdio,
            "sse" => TransportType::Sse,
            "streamable-http" => TransportType::StreamableHttp,
            "http" => TransportType::Http,
            _ => TransportType::Stdio,
        }
    }

    /// Derive install command/args from the preferred package in the list
    fn derive_install_info(packages: &[PackageInfo]) -> (Option<String>, Vec<String>) {
        // Prefer npm stdio package
        let preferred = packages
            .iter()
            .find(|p| p.registry_type == "npm" && p.transport_type == TransportType::Stdio)
            .or_else(|| {
                packages
                    .iter()
                    .find(|p| p.registry_type == "pypi" && p.transport_type == TransportType::Stdio)
            })
            .or_else(|| {
                packages
                    .iter()
                    .find(|p| p.transport_type == TransportType::Stdio)
            })
            .or(packages.first());

        match preferred {
            Some(pkg) if pkg.transport_type == TransportType::Stdio => {
                match pkg.registry_type.as_str() {
                    "npm" => (
                        Some("npx".to_string()),
                        vec!["-y".to_string(), pkg.identifier.clone()],
                    ),
                    "pypi" => (Some("uvx".to_string()), vec![pkg.identifier.clone()]),
                    _ => (
                        Some("npx".to_string()),
                        vec!["-y".to_string(), pkg.identifier.clone()],
                    ),
                }
            }
            _ => (None, Vec::new()),
        }
    }

    /// Extract license, publisher/vendor, and keywords from the server-level _meta
    fn extract_meta(meta: &Option<Value>) -> (Option<String>, Option<String>, Vec<String>) {
        let meta = match meta {
            Some(v) => v,
            None => return (None, None, Vec::new()),
        };

        // Look through all meta keys for publisher-provided metadata
        let obj = match meta.as_object() {
            Some(o) => o,
            None => return (None, None, Vec::new()),
        };

        let mut license = None;
        let mut publisher = None;
        let mut keywords = Vec::new();

        for (_key, value) in obj {
            if let Some(inner) = value.as_object() {
                if license.is_none() {
                    license = inner
                        .get("license")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
                if publisher.is_none() {
                    publisher = inner
                        .get("publisher")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
                if keywords.is_empty() {
                    if let Some(kw) = inner.get("keywords").and_then(|v| v.as_array()) {
                        keywords = kw
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }
            }
        }

        (license, publisher, keywords)
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
        self.display_name
    }

    fn base_url(&self) -> &str {
        self.base_url
    }

    async fn list_servers(&self) -> Result<Vec<RegistryServer>> {
        let mut all_servers = Vec::new();
        let mut cursor: Option<String> = None;
        let max_pages = 5;
        let source_name = self.display_name.to_string();

        for _ in 0..max_pages {
            let page = self.fetch_page(cursor.as_deref(), None).await?;

            all_servers.extend(
                page.servers
                    .into_iter()
                    .map(|e| Self::convert_entry(e, &source_name)),
            );

            match page.metadata.and_then(|m| m.next_cursor) {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }

        Ok(all_servers)
    }

    async fn search(&self, query: &str) -> Result<Vec<RegistryServer>> {
        let source_name = self.display_name.to_string();
        let page = self.fetch_page(None, Some(query)).await?;
        Ok(page
            .servers
            .into_iter()
            .map(|e| Self::convert_entry(e, &source_name))
            .collect())
    }

    async fn get_server(&self, name: &str) -> Result<Option<RegistryServer>> {
        let source_name = self.display_name.to_string();
        let page = self.fetch_page(None, Some(name)).await?;
        let servers: Vec<RegistryServer> = page
            .servers
            .into_iter()
            .map(|e| Self::convert_entry(e, &source_name))
            .collect();
        Ok(servers.into_iter().find(|s| s.name == name))
    }
}
