use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{McpmError, Result};
use crate::ide::traits::IdeAdapter;
use crate::model::{InstalledServer, ServerConfig, VSCodeConfig};

pub struct VSCodeAdapter {
    config_path: PathBuf,
}

impl VSCodeAdapter {
    pub fn new() -> Self {
        let config_path = Self::default_config_path();
        Self { config_path }
    }

    fn default_config_path() -> PathBuf {
        // VS Code uses workspace-level config, but we'll use user-level for global servers
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir()
                .unwrap_or_default()
                .join("Library/Application Support/Code/User/globalStorage/mcp.json")
        }
        #[cfg(target_os = "windows")]
        {
            dirs::config_dir()
                .unwrap_or_default()
                .join("Code/User/globalStorage/mcp.json")
        }
        #[cfg(target_os = "linux")]
        {
            dirs::config_dir()
                .unwrap_or_default()
                .join("Code/User/globalStorage/mcp.json")
        }
    }

    fn read_config(&self) -> Result<VSCodeConfig> {
        if !self.config_path.exists() {
            return Ok(VSCodeConfig::default());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let config: VSCodeConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn write_config(&self, config: &VSCodeConfig) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }
}

impl Default for VSCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdeAdapter for VSCodeAdapter {
    fn id(&self) -> &str {
        "vscode"
    }

    fn name(&self) -> &str {
        "VS Code"
    }

    fn is_detected(&self) -> bool {
        // Check if VS Code is installed
        #[cfg(target_os = "macos")]
        {
            PathBuf::from("/Applications/Visual Studio Code.app").exists()
                || std::process::Command::new("code")
                    .arg("--version")
                    .output()
                    .is_ok()
        }
        #[cfg(not(target_os = "macos"))]
        {
            std::process::Command::new("code")
                .arg("--version")
                .output()
                .is_ok()
        }
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
