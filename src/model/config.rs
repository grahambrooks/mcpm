use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::server::ServerConfig;

/// Generic MCP servers configuration structure
/// Most IDEs use this format with slight variations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServersConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, ServerConfig>,
}

/// Claude Desktop specific config (has additional fields)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeDesktopConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, ServerConfig>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Claude Code config structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeCodeConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, ServerConfig>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Cursor config structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CursorConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, ServerConfig>,
}

/// VS Code config structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VSCodeConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, ServerConfig>,
}

/// Windsurf config structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindsurfConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, ServerConfig>,
}

/// Represents an IDE with its detection status
#[derive(Debug, Clone)]
pub struct IdeInfo {
    pub id: String,
    pub name: String,
    pub detected: bool,
    pub config_path: Option<String>,
    pub server_count: usize,
}
