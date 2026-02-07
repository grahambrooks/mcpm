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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_servers_config_deserialize_stdio_and_http() {
        let json = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem"],
                    "env": {"HOME": "/home/user"}
                },
                "remote": {
                    "url": "https://example.com/mcp"
                }
            }
        }"#;
        let config: McpServersConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mcp_servers.len(), 2);

        let fs = config.mcp_servers.get("filesystem").unwrap();
        match fs {
            ServerConfig::Stdio(cfg) => {
                assert_eq!(cfg.command, "npx");
                assert_eq!(cfg.args, vec!["-y", "@modelcontextprotocol/server-filesystem"]);
                assert_eq!(cfg.env.get("HOME").unwrap(), "/home/user");
            }
            _ => panic!("expected Stdio config for filesystem"),
        }

        let remote = config.mcp_servers.get("remote").unwrap();
        match remote {
            ServerConfig::Http(cfg) => {
                assert_eq!(cfg.url, "https://example.com/mcp");
            }
            _ => panic!("expected Http config for remote"),
        }
    }

    #[test]
    fn claude_desktop_config_preserves_extra_fields() {
        let json = r#"{
            "mcpServers": {
                "test": {
                    "command": "test-cmd"
                }
            },
            "theme": "dark",
            "windowSize": [800, 600]
        }"#;
        let config: ClaudeDesktopConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        assert!(config.extra.contains_key("theme"));
        assert!(config.extra.contains_key("windowSize"));

        // Round-trip: serialize and deserialize again
        let serialized = serde_json::to_string(&config).unwrap();
        let roundtrip: ClaudeDesktopConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(roundtrip.mcp_servers.len(), 1);
        assert_eq!(
            roundtrip.extra.get("theme").unwrap(),
            &serde_json::Value::String("dark".to_string())
        );
    }

    #[test]
    fn server_config_untagged_enum_distinguishes_stdio_from_http() {
        // Stdio: has "command"
        let stdio_json = r#"{"command": "npx", "args": ["-y", "pkg"]}"#;
        let stdio: ServerConfig = serde_json::from_str(stdio_json).unwrap();
        assert!(matches!(stdio, ServerConfig::Stdio(_)));

        // Http: has "url"
        let http_json = r#"{"url": "https://example.com"}"#;
        let http: ServerConfig = serde_json::from_str(http_json).unwrap();
        assert!(matches!(http, ServerConfig::Http(_)));
    }

    #[test]
    fn mcp_servers_config_serialize_round_trip() {
        let mut servers = HashMap::new();
        servers.insert(
            "test-stdio".to_string(),
            ServerConfig::Stdio(super::super::server::StdioServerConfig {
                command: "node".to_string(),
                args: vec!["server.js".to_string()],
                env: HashMap::new(),
            }),
        );
        servers.insert(
            "test-http".to_string(),
            ServerConfig::Http(super::super::server::HttpServerConfig {
                url: "https://example.com".to_string(),
                headers: HashMap::new(),
            }),
        );

        let config = McpServersConfig {
            mcp_servers: servers,
        };

        let json = serde_json::to_string(&config).unwrap();
        let roundtrip: McpServersConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.mcp_servers.len(), 2);
        assert!(roundtrip.mcp_servers.contains_key("test-stdio"));
        assert!(roundtrip.mcp_servers.contains_key("test-http"));
    }
}
