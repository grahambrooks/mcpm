use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::Result;
use crate::model::{IdeInfo, InstalledServer, ServerConfig};

/// Trait for IDE adapters that manage MCP server configurations
#[async_trait]
pub trait IdeAdapter: Send + Sync {
    /// Unique identifier for this IDE
    fn id(&self) -> &str;

    /// Display name for this IDE
    fn name(&self) -> &str;

    /// Check if this IDE is detected/installed on the system
    fn is_detected(&self) -> bool;

    /// Get the path to the MCP configuration file
    fn config_path(&self) -> Option<String>;

    /// Get IDE info
    fn info(&self) -> IdeInfo {
        IdeInfo {
            id: self.id().to_string(),
            name: self.name().to_string(),
            detected: self.is_detected(),
            config_path: self.config_path(),
            server_count: 0, // Will be populated later
        }
    }

    /// List all installed MCP servers
    async fn list_servers(&self) -> Result<Vec<InstalledServer>>;

    /// Get a specific server's configuration
    async fn get_server(&self, name: &str) -> Result<Option<InstalledServer>>;

    /// Install/add an MCP server
    async fn install_server(&self, name: &str, config: ServerConfig) -> Result<()>;

    /// Remove an MCP server
    async fn remove_server(&self, name: &str) -> Result<()>;

    /// Update a server's configuration
    async fn update_server(&self, name: &str, config: ServerConfig) -> Result<()>;

    /// Get all servers as a map (for sync operations)
    async fn get_servers_map(&self) -> Result<HashMap<String, ServerConfig>> {
        let servers = self.list_servers().await?;
        Ok(servers.into_iter().map(|s| (s.name, s.config)).collect())
    }
}

/// Collection of all IDE adapters
pub struct IdeManager {
    adapters: Vec<Box<dyn IdeAdapter>>,
}

impl IdeManager {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    pub fn register(&mut self, adapter: Box<dyn IdeAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn list_ides(&self) -> Vec<IdeInfo> {
        self.adapters.iter().map(|a| a.info()).collect()
    }

    pub fn detected_ides(&self) -> Vec<&dyn IdeAdapter> {
        self.adapters
            .iter()
            .filter(|a| a.is_detected())
            .map(|a| a.as_ref())
            .collect()
    }

    pub fn get_adapter(&self, id: &str) -> Option<&dyn IdeAdapter> {
        self.adapters
            .iter()
            .find(|a| a.id() == id)
            .map(|a| a.as_ref())
    }

    pub fn adapters(&self) -> &[Box<dyn IdeAdapter>] {
        &self.adapters
    }
}

impl Default for IdeManager {
    fn default() -> Self {
        Self::new()
    }
}
