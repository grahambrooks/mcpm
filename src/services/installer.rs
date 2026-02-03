use std::collections::HashMap;

use crate::error::Result;
use crate::ide::IdeManager;
use crate::model::{RegistryServer, ServerConfig};

/// Service for installing and removing MCP servers
pub struct InstallerService<'a> {
    ide_manager: &'a IdeManager,
}

impl<'a> InstallerService<'a> {
    pub fn new(ide_manager: &'a IdeManager) -> Self {
        Self { ide_manager }
    }

    /// Install a server from the registry to a specific IDE
    pub async fn install(
        &self,
        ide_id: &str,
        server: &RegistryServer,
        env: HashMap<String, String>,
    ) -> Result<()> {
        let adapter = self
            .ide_manager
            .get_adapter(ide_id)
            .ok_or_else(|| crate::error::McpmError::IdeNotFound(ide_id.to_string()))?;

        let config = server.to_server_config(env);
        adapter.install_server(&server.name, config).await?;

        Ok(())
    }

    /// Install a server with a custom configuration
    pub async fn install_custom(
        &self,
        ide_id: &str,
        name: &str,
        config: ServerConfig,
    ) -> Result<()> {
        let adapter = self
            .ide_manager
            .get_adapter(ide_id)
            .ok_or_else(|| crate::error::McpmError::IdeNotFound(ide_id.to_string()))?;

        adapter.install_server(name, config).await?;

        Ok(())
    }

    /// Remove a server from a specific IDE
    pub async fn remove(&self, ide_id: &str, server_name: &str) -> Result<()> {
        let adapter = self
            .ide_manager
            .get_adapter(ide_id)
            .ok_or_else(|| crate::error::McpmError::IdeNotFound(ide_id.to_string()))?;

        adapter.remove_server(server_name).await?;

        Ok(())
    }

    /// Check if a server is installed in a specific IDE
    pub async fn is_installed(&self, ide_id: &str, server_name: &str) -> Result<bool> {
        let adapter = self
            .ide_manager
            .get_adapter(ide_id)
            .ok_or_else(|| crate::error::McpmError::IdeNotFound(ide_id.to_string()))?;

        let server = adapter.get_server(server_name).await?;
        Ok(server.is_some())
    }
}
