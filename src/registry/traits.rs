use async_trait::async_trait;

use crate::error::Result;
use crate::model::RegistryServer;

/// Trait for registry clients that fetch MCP server information
#[async_trait]
pub trait RegistryClient: Send + Sync {
    /// Get the name of this registry
    fn name(&self) -> &str;

    /// Get the base URL of this registry
    fn base_url(&self) -> &str;

    /// List all servers (or featured/popular servers)
    async fn list_servers(&self) -> Result<Vec<RegistryServer>>;

    /// Search for servers by query
    async fn search(&self, query: &str) -> Result<Vec<RegistryServer>>;

    /// Get details for a specific server
    async fn get_server(&self, name: &str) -> Result<Option<RegistryServer>>;
}
