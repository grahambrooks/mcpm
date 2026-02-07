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
