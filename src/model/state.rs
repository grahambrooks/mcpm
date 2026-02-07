use std::collections::HashMap;

use super::config::IdeInfo;
use super::server::{InstalledServer, RegistryServer};

/// Currently active screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Dashboard,
    Registry,
    Installed,
    ServerDetail,
    Sync,
}

impl Screen {
    pub fn from_key(key: char) -> Option<Screen> {
        match key {
            '1' => Some(Screen::Dashboard),
            '2' => Some(Screen::Registry),
            '3' => Some(Screen::Installed),
            '4' => Some(Screen::ServerDetail),
            '5' => Some(Screen::Sync),
            _ => None,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Registry => "Registry Browser",
            Screen::Installed => "Installed Servers",
            Screen::ServerDetail => "Server Details",
            Screen::Sync => "Sync Servers",
        }
    }
}

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
    EnvInput,
}

/// Application state
#[derive(Debug, Default)]
pub struct AppState {
    // Navigation
    pub screen: Screen,
    pub previous_screen: Option<Screen>,
    pub input_mode: InputMode,

    // IDE state
    pub ides: Vec<IdeInfo>,
    pub selected_ide_index: usize,
    pub ide_servers: HashMap<String, Vec<InstalledServer>>,

    // Registry state
    pub registry_servers: Vec<RegistryServer>,
    pub registry_loading: bool,
    pub registry_error: Option<String>,
    pub selected_registry_index: usize,
    pub registry_source: RegistrySource,

    // Search state
    pub search_query: String,
    pub search_results: Vec<RegistryServer>,

    // Server detail state
    pub selected_server: Option<RegistryServer>,
    pub env_inputs: HashMap<String, String>,
    pub env_input_index: usize,

    // Sync state
    pub sync_source_ide: Option<usize>,
    pub sync_target_ides: Vec<usize>,
    pub sync_servers: Vec<String>,
    pub sync_preview: Vec<SyncPreviewItem>,

    // General UI state
    pub status_message: Option<String>,
    pub show_help: bool,
    pub should_quit: bool,

    // Async loading state
    pub loading_tick: u64,

    // Version filtering
    pub show_all_versions: bool,
    pub registry_servers_latest: Vec<RegistryServer>,

    // CLI target: jump to this server once the registry loads
    pub target_server: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RegistrySource {
    #[default]
    Official,
    Legacy,
}

impl RegistrySource {
    pub fn name(&self) -> &'static str {
        match self {
            RegistrySource::Official => "MCP Registry",
            RegistrySource::Legacy => "MCP Registry (v0)",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            RegistrySource::Official => RegistrySource::Legacy,
            RegistrySource::Legacy => RegistrySource::Official,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncPreviewItem {
    pub server_name: String,
    pub action: SyncAction,
    pub target_ide: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    Add,
    Update,
    Skip,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn selected_ide(&self) -> Option<&IdeInfo> {
        self.ides.get(self.selected_ide_index)
    }

    pub fn current_registry_server(&self) -> Option<&RegistryServer> {
        self.displayed_servers().get(self.selected_registry_index)
    }

    pub fn displayed_servers(&self) -> &[RegistryServer] {
        if !self.search_query.is_empty() {
            &self.search_results
        } else if self.show_all_versions {
            &self.registry_servers
        } else {
            &self.registry_servers_latest
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_server(name: &str) -> RegistryServer {
        RegistryServer {
            name: name.to_string(),
            description: String::new(),
            title: None,
            version: None,
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

    // --- Screen::from_key tests ---

    #[test]
    fn screen_from_key_valid() {
        assert_eq!(Screen::from_key('1'), Some(Screen::Dashboard));
        assert_eq!(Screen::from_key('2'), Some(Screen::Registry));
        assert_eq!(Screen::from_key('3'), Some(Screen::Installed));
        assert_eq!(Screen::from_key('4'), Some(Screen::ServerDetail));
        assert_eq!(Screen::from_key('5'), Some(Screen::Sync));
    }

    #[test]
    fn screen_from_key_invalid() {
        assert_eq!(Screen::from_key('0'), None);
        assert_eq!(Screen::from_key('6'), None);
        assert_eq!(Screen::from_key('a'), None);
    }

    // --- Screen::title tests ---

    #[test]
    fn screen_title() {
        assert_eq!(Screen::Dashboard.title(), "Dashboard");
        assert_eq!(Screen::Registry.title(), "Registry Browser");
        assert_eq!(Screen::Installed.title(), "Installed Servers");
        assert_eq!(Screen::ServerDetail.title(), "Server Details");
        assert_eq!(Screen::Sync.title(), "Sync Servers");
    }

    // --- RegistrySource tests ---

    #[test]
    fn registry_source_toggle_round_trip() {
        let source = RegistrySource::Official;
        assert_eq!(source.toggle(), RegistrySource::Legacy);
        assert_eq!(source.toggle().toggle(), RegistrySource::Official);
    }

    #[test]
    fn registry_source_name() {
        assert_eq!(RegistrySource::Official.name(), "MCP Registry");
        assert_eq!(RegistrySource::Legacy.name(), "MCP Registry (v0)");
    }

    // --- AppState::displayed_servers tests ---

    #[test]
    fn displayed_servers_returns_latest_by_default() {
        let mut state = AppState::new();
        state.registry_servers_latest = vec![make_test_server("latest")];
        state.registry_servers = vec![make_test_server("all1"), make_test_server("all2")];
        let displayed = state.displayed_servers();
        assert_eq!(displayed.len(), 1);
        assert_eq!(displayed[0].name, "latest");
    }

    #[test]
    fn displayed_servers_returns_all_when_show_all_versions() {
        let mut state = AppState::new();
        state.show_all_versions = true;
        state.registry_servers = vec![make_test_server("all1"), make_test_server("all2")];
        state.registry_servers_latest = vec![make_test_server("latest")];
        let displayed = state.displayed_servers();
        assert_eq!(displayed.len(), 2);
    }

    #[test]
    fn displayed_servers_returns_search_results_when_query_non_empty() {
        let mut state = AppState::new();
        state.search_query = "test".to_string();
        state.search_results = vec![make_test_server("result")];
        state.registry_servers_latest = vec![make_test_server("latest")];
        let displayed = state.displayed_servers();
        assert_eq!(displayed.len(), 1);
        assert_eq!(displayed[0].name, "result");
    }

    // --- AppState::current_registry_server tests ---

    #[test]
    fn current_registry_server_correct_index() {
        let mut state = AppState::new();
        state.registry_servers_latest = vec![
            make_test_server("first"),
            make_test_server("second"),
            make_test_server("third"),
        ];
        state.selected_registry_index = 1;
        let server = state.current_registry_server().unwrap();
        assert_eq!(server.name, "second");
    }

    #[test]
    fn current_registry_server_out_of_bounds() {
        let mut state = AppState::new();
        state.selected_registry_index = 5;
        assert!(state.current_registry_server().is_none());
    }

    // --- AppState::selected_ide tests ---

    #[test]
    fn selected_ide_returns_none_when_empty() {
        let state = AppState::new();
        assert!(state.selected_ide().is_none());
    }

    #[test]
    fn selected_ide_returns_correct_ide() {
        let mut state = AppState::new();
        state.ides = vec![
            IdeInfo {
                id: "claude".to_string(),
                name: "Claude Desktop".to_string(),
                detected: true,
                config_path: None,
                server_count: 0,
            },
            IdeInfo {
                id: "cursor".to_string(),
                name: "Cursor".to_string(),
                detected: false,
                config_path: None,
                server_count: 0,
            },
        ];
        state.selected_ide_index = 1;
        let ide = state.selected_ide().unwrap();
        assert_eq!(ide.id, "cursor");
    }
}
