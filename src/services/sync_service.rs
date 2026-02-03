use std::collections::HashMap;

use crate::error::Result;
use crate::ide::IdeManager;
use crate::model::{ServerConfig, SyncAction, SyncPreviewItem};

/// Service for syncing MCP server configurations between IDEs
pub struct SyncService<'a> {
    ide_manager: &'a IdeManager,
}

impl<'a> SyncService<'a> {
    pub fn new(ide_manager: &'a IdeManager) -> Self {
        Self { ide_manager }
    }

    /// Generate a preview of what sync would do
    pub async fn preview(
        &self,
        source_ide: &str,
        target_ides: &[&str],
        server_names: Option<&[&str]>,
    ) -> Result<Vec<SyncPreviewItem>> {
        let source_adapter = self
            .ide_manager
            .get_adapter(source_ide)
            .ok_or_else(|| crate::error::McpmError::IdeNotFound(source_ide.to_string()))?;

        let source_servers = source_adapter.get_servers_map().await?;

        // Filter servers if specific names are provided
        let servers_to_sync: HashMap<String, ServerConfig> = if let Some(names) = server_names {
            source_servers
                .into_iter()
                .filter(|(name, _)| names.contains(&name.as_str()))
                .collect()
        } else {
            source_servers
        };

        let mut preview = Vec::new();

        for target_ide in target_ides {
            let target_adapter = self
                .ide_manager
                .get_adapter(target_ide)
                .ok_or_else(|| crate::error::McpmError::IdeNotFound(target_ide.to_string()))?;

            let target_servers = target_adapter.get_servers_map().await?;

            for name in servers_to_sync.keys() {
                let action = if target_servers.contains_key(name) {
                    SyncAction::Update
                } else {
                    SyncAction::Add
                };

                preview.push(SyncPreviewItem {
                    server_name: name.clone(),
                    action,
                    target_ide: target_ide.to_string(),
                });
            }
        }

        Ok(preview)
    }

    /// Execute sync operation
    pub async fn sync(
        &self,
        source_ide: &str,
        target_ides: &[&str],
        server_names: Option<&[&str]>,
    ) -> Result<Vec<SyncPreviewItem>> {
        let source_adapter = self
            .ide_manager
            .get_adapter(source_ide)
            .ok_or_else(|| crate::error::McpmError::IdeNotFound(source_ide.to_string()))?;

        let source_servers = source_adapter.get_servers_map().await?;

        // Filter servers if specific names are provided
        let servers_to_sync: HashMap<String, ServerConfig> = if let Some(names) = server_names {
            source_servers
                .into_iter()
                .filter(|(name, _)| names.contains(&name.as_str()))
                .collect()
        } else {
            source_servers
        };

        let mut results = Vec::new();

        for target_ide in target_ides {
            let target_adapter = self
                .ide_manager
                .get_adapter(target_ide)
                .ok_or_else(|| crate::error::McpmError::IdeNotFound(target_ide.to_string()))?;

            let target_servers = target_adapter.get_servers_map().await?;

            for (name, config) in &servers_to_sync {
                let action = if target_servers.contains_key(name) {
                    target_adapter.update_server(name, config.clone()).await?;
                    SyncAction::Update
                } else {
                    target_adapter.install_server(name, config.clone()).await?;
                    SyncAction::Add
                };

                results.push(SyncPreviewItem {
                    server_name: name.clone(),
                    action,
                    target_ide: target_ide.to_string(),
                });
            }
        }

        Ok(results)
    }

    /// Sync all servers from source to all detected target IDEs
    pub async fn sync_all_to_detected(&self, source_ide: &str) -> Result<Vec<SyncPreviewItem>> {
        let target_ides: Vec<&str> = self
            .ide_manager
            .detected_ides()
            .iter()
            .filter(|a| a.id() != source_ide)
            .map(|a| a.id())
            .collect();

        self.sync(source_ide, &target_ides, None).await
    }
}
