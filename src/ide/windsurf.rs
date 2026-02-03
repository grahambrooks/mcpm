use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{McpmError, Result};
use crate::ide::traits::IdeAdapter;
use crate::model::{InstalledServer, ServerConfig, WindsurfConfig};

pub struct WindsurfAdapter {
    config_path: PathBuf,
}

impl WindsurfAdapter {
    pub fn new() -> Self {
        let config_path = Self::default_config_path();
        Self { config_path }
    }

    fn default_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".codeium/windsurf/mcp_config.json")
    }

    fn read_config(&self) -> Result<WindsurfConfig> {
        if !self.config_path.exists() {
            return Ok(WindsurfConfig::default());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let config: WindsurfConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn write_config(&self, config: &WindsurfConfig) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }
}

impl Default for WindsurfAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdeAdapter for WindsurfAdapter {
    fn id(&self) -> &str {
        "windsurf"
    }

    fn name(&self) -> &str {
        "Windsurf"
    }

    fn is_detected(&self) -> bool {
        // Windsurf is detected if config exists or .codeium directory exists
        self.config_path.exists()
            || dirs::home_dir()
                .map(|h| h.join(".codeium/windsurf").exists())
                .unwrap_or(false)
    }

    fn config_path(&self) -> Option<String> {
        Some(self.config_path.to_string_lossy().to_string())
    }

    async fn list_servers(&self) -> Result<Vec<InstalledServer>> {
        let config = self.read_config()?;
        Ok(config
            .mcp_servers
            .into_iter()
            .map(|(name, cfg)| InstalledServer {
                name,
                config: cfg,
                ide_id: self.id().to_string(),
            })
            .collect())
    }

    async fn get_server(&self, name: &str) -> Result<Option<InstalledServer>> {
        let config = self.read_config()?;
        Ok(config.mcp_servers.get(name).map(|cfg| InstalledServer {
            name: name.to_string(),
            config: cfg.clone(),
            ide_id: self.id().to_string(),
        }))
    }

    async fn install_server(&self, name: &str, server_config: ServerConfig) -> Result<()> {
        let mut config = self.read_config()?;
        config.mcp_servers.insert(name.to_string(), server_config);
        self.write_config(&config)?;
        Ok(())
    }

    async fn remove_server(&self, name: &str) -> Result<()> {
        let mut config = self.read_config()?;
        if config.mcp_servers.remove(name).is_none() {
            return Err(McpmError::ServerNotFound(name.to_string()));
        }
        self.write_config(&config)?;
        Ok(())
    }

    async fn update_server(&self, name: &str, server_config: ServerConfig) -> Result<()> {
        let mut config = self.read_config()?;
        if !config.mcp_servers.contains_key(name) {
            return Err(McpmError::ServerNotFound(name.to_string()));
        }
        config.mcp_servers.insert(name.to_string(), server_config);
        self.write_config(&config)?;
        Ok(())
    }

    async fn get_servers_map(&self) -> Result<HashMap<String, ServerConfig>> {
        let config = self.read_config()?;
        Ok(config.mcp_servers)
    }
}
