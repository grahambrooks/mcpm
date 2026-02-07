use std::collections::HashMap;

use crossterm::event::{Event, KeyCode, KeyEvent};
use tokio::sync::oneshot;

use crate::error::Result;
use crate::ide::{create_ide_manager, IdeManager};
use crate::model::{AppState, IdeInfo, InputMode, RegistryServer, RegistrySource, Screen};
use crate::registry::{OfficialRegistryClient, RegistryClient};
use crate::services::{InstallerService, SyncService};

pub struct App {
    pub state: AppState,
    ide_manager: IdeManager,
    registry_rx: Option<oneshot::Receiver<std::result::Result<Vec<RegistryServer>, String>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
            ide_manager: create_ide_manager(),
            registry_rx: None,
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        // Load IDE info (fast, local)
        self.load_ides().await;
        Ok(())
    }

    /// Spawn a background task to fetch the registry. Non-blocking.
    pub fn start_registry_fetch(&mut self) {
        self.state.registry_loading = true;
        self.state.registry_error = None;
        self.state.loading_tick = 0;

        let source = self.state.registry_source;
        let (tx, rx) = oneshot::channel();
        self.registry_rx = Some(rx);

        tokio::spawn(async move {
            let client = match source {
                RegistrySource::Official => OfficialRegistryClient::v01(),
                RegistrySource::Legacy => OfficialRegistryClient::v0(),
            };
            let result = client.list_servers().await.map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Poll for completed async tasks. Call each tick in the event loop.
    pub fn poll_tasks(&mut self) {
        if self.state.registry_loading {
            self.state.loading_tick = self.state.loading_tick.wrapping_add(1);
        }

        if let Some(rx) = &mut self.registry_rx {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(servers) => {
                            let latest = deduplicate_latest(&servers);
                            self.state.registry_servers = servers;
                            self.state.registry_servers_latest = latest;
                            self.state.selected_registry_index = 0;
                        }
                        Err(e) => {
                            self.state.registry_error = Some(e);
                        }
                    }
                    self.state.registry_loading = false;
                    self.registry_rx = None;
                }
                Err(oneshot::error::TryRecvError::Empty) => {
                    // Still loading, nothing to do
                }
                Err(oneshot::error::TryRecvError::Closed) => {
                    // Sender dropped without sending — treat as error
                    self.state.registry_error =
                        Some("Registry fetch task failed unexpectedly".to_string());
                    self.state.registry_loading = false;
                    self.registry_rx = None;
                }
            }
        }
    }

    async fn load_ides(&mut self) {
        let mut ides: Vec<IdeInfo> = self.ide_manager.list_ides();

        // Load server counts for each IDE
        for ide in &mut ides {
            if let Some(adapter) = self.ide_manager.get_adapter(&ide.id) {
                if let Ok(servers) = adapter.list_servers().await {
                    ide.server_count = servers.len();
                    self.state.ide_servers.insert(ide.id.clone(), servers);
                }
            }
        }

        self.state.ides = ides;
    }

    fn search_registry(&mut self, query: &str) {
        if query.is_empty() {
            self.state.search_results.clear();
            return;
        }

        let query_lower = query.to_lowercase();
        let source = if self.state.show_all_versions {
            &self.state.registry_servers
        } else {
            &self.state.registry_servers_latest
        };
        self.state.search_results = source
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.display_name().to_lowercase().contains(&query_lower)
                    || s.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();
        self.state.selected_registry_index = 0;
    }

    pub async fn handle_event(&mut self, event: Event) -> Result<()> {
        if let Event::Key(key) = event {
            self.handle_key(key).await?;
        }
        Ok(())
    }

    async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle input modes first
        match self.state.input_mode {
            InputMode::Search => {
                return self.handle_search_input(key).await;
            }
            InputMode::EnvInput => {
                return self.handle_env_input(key).await;
            }
            InputMode::Normal => {}
        }

        // Global keybindings
        match key.code {
            KeyCode::Char('q') => {
                self.state.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('?') => {
                self.state.show_help = !self.state.show_help;
                return Ok(());
            }
            KeyCode::Esc => {
                if self.state.show_help {
                    self.state.show_help = false;
                } else {
                    self.state.status_message = None;
                }
                return Ok(());
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(screen) = Screen::from_key(c) {
                    self.state.previous_screen = Some(self.state.screen);
                    self.state.screen = screen;
                }
                return Ok(());
            }
            KeyCode::Tab => {
                // Cycle through IDEs
                if !self.state.ides.is_empty() {
                    self.state.selected_ide_index =
                        (self.state.selected_ide_index + 1) % self.state.ides.len();
                }
                return Ok(());
            }
            KeyCode::BackTab => {
                // Cycle through IDEs backwards
                if !self.state.ides.is_empty() {
                    if self.state.selected_ide_index == 0 {
                        self.state.selected_ide_index = self.state.ides.len() - 1;
                    } else {
                        self.state.selected_ide_index -= 1;
                    }
                }
                return Ok(());
            }
            _ => {}
        }

        // Screen-specific keybindings
        match self.state.screen {
            Screen::Registry => self.handle_registry_key(key).await?,
            Screen::Installed => self.handle_installed_key(key).await?,
            Screen::ServerDetail => self.handle_detail_key(key).await?,
            Screen::Sync => self.handle_sync_key(key).await?,
            Screen::Dashboard => self.handle_dashboard_key(key).await?,
        }

        Ok(())
    }

    async fn handle_search_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::Normal;
                self.state.search_query.clear();
                self.state.search_results.clear();
                self.state.selected_registry_index = 0;
            }
            KeyCode::Enter => {
                self.state.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.state.search_query.pop();
                let query = self.state.search_query.clone();
                self.search_registry(&query);
            }
            KeyCode::Char(c) => {
                self.state.search_query.push(c);
                let query = self.state.search_query.clone();
                self.search_registry(&query);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_env_input(&mut self, key: KeyEvent) -> Result<()> {
        let server = match &self.state.selected_server {
            Some(s) => s,
            None => return Ok(()),
        };

        let all_vars = server.all_env_vars();
        let current_var = match all_vars.get(self.state.env_input_index) {
            Some(v) => v.name.clone(),
            None => return Ok(()),
        };

        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::Normal;
            }
            KeyCode::Enter | KeyCode::Tab => {
                // Move to next env var
                if self.state.env_input_index < all_vars.len() - 1 {
                    self.state.env_input_index += 1;
                } else {
                    self.state.input_mode = InputMode::Normal;
                }
            }
            KeyCode::BackTab => {
                // Move to previous env var
                if self.state.env_input_index > 0 {
                    self.state.env_input_index -= 1;
                }
            }
            KeyCode::Backspace => {
                if let Some(value) = self.state.env_inputs.get_mut(&current_var) {
                    value.pop();
                }
            }
            KeyCode::Char(c) => {
                self.state
                    .env_inputs
                    .entry(current_var)
                    .or_default()
                    .push(c);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_dashboard_key(&mut self, key: KeyEvent) -> Result<()> {
        if let KeyCode::Char('r') = key.code {
            self.load_ides().await;
            self.state.status_message = Some("Refreshed IDE information".to_string());
        }
        Ok(())
    }

    async fn handle_registry_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('/') => {
                self.state.input_mode = InputMode::Search;
            }
            KeyCode::Char('s') => {
                self.state.registry_source = self.state.registry_source.toggle();
                self.start_registry_fetch();
            }
            KeyCode::Char('r') => {
                self.start_registry_fetch();
            }
            KeyCode::Char('v') => {
                self.state.show_all_versions = !self.state.show_all_versions;
                self.state.selected_registry_index = 0;
                // Re-run search if active
                if !self.state.search_query.is_empty() {
                    let query = self.state.search_query.clone();
                    self.search_registry(&query);
                }
            }
            KeyCode::Up => {
                if self.state.selected_registry_index > 0 {
                    self.state.selected_registry_index -= 1;
                }
            }
            KeyCode::Down => {
                let max = self.state.displayed_servers().len().saturating_sub(1);
                if self.state.selected_registry_index < max {
                    self.state.selected_registry_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(server) = self.state.current_registry_server().cloned() {
                    self.state.selected_server = Some(server);
                    self.state.screen = Screen::ServerDetail;
                }
            }
            KeyCode::Char('i') => {
                // Quick install to selected IDE
                if let Some(server) = self.state.current_registry_server().cloned() {
                    self.install_server(&server).await?;
                }
            }
            KeyCode::Char('I') => {
                // Install to all detected IDEs
                if let Some(server) = self.state.current_registry_server().cloned() {
                    self.install_server_to_all_ides(&server).await?;
                }
            }
            KeyCode::Esc => {
                if !self.state.search_query.is_empty() {
                    self.state.search_query.clear();
                    self.state.search_results.clear();
                    self.state.selected_registry_index = 0;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_installed_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('d') => {
                // Delete selected server
                self.state.status_message =
                    Some("Delete functionality not yet implemented".to_string());
            }
            KeyCode::Char('e') => {
                // Edit selected server
                self.state.status_message =
                    Some("Edit functionality not yet implemented".to_string());
            }
            KeyCode::Char('r') => {
                self.load_ides().await;
                self.state.status_message = Some("Refreshed installed servers".to_string());
            }
            KeyCode::Up | KeyCode::Down => {
                // TODO: Navigate installed servers list
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_detail_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('i') => {
                if let Some(server) = self.state.selected_server.clone() {
                    self.install_server(&server).await?;
                }
            }
            KeyCode::Char('I') => {
                if let Some(server) = self.state.selected_server.clone() {
                    self.install_server_to_all_ides(&server).await?;
                }
            }
            KeyCode::Char('e') => {
                if self.state.selected_server.is_some() {
                    self.state.input_mode = InputMode::EnvInput;
                    self.state.env_input_index = 0;
                }
            }
            KeyCode::Esc => {
                self.state.screen = Screen::Registry;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_sync_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('s') => {
                // Set source IDE (using selected_ide_index)
                let detected_ides: Vec<usize> = self
                    .state
                    .ides
                    .iter()
                    .enumerate()
                    .filter(|(_, ide)| ide.detected)
                    .map(|(i, _)| i)
                    .collect();

                if detected_ides.contains(&self.state.selected_ide_index) {
                    self.state.sync_source_ide = Some(self.state.selected_ide_index);
                    // Remove from targets if it was there
                    self.state
                        .sync_target_ides
                        .retain(|&i| i != self.state.selected_ide_index);
                    self.state.sync_preview.clear();
                }
            }
            KeyCode::Char('t') => {
                // Toggle target IDE
                let idx = self.state.selected_ide_index;
                if self.state.sync_source_ide != Some(idx) {
                    if let Some(ide) = self.state.ides.get(idx) {
                        if ide.detected {
                            if self.state.sync_target_ides.contains(&idx) {
                                self.state.sync_target_ides.retain(|&i| i != idx);
                            } else {
                                self.state.sync_target_ides.push(idx);
                            }
                            self.state.sync_preview.clear();
                        }
                    }
                }
            }
            KeyCode::Char('p') => {
                // Preview sync
                self.preview_sync().await?;
            }
            KeyCode::Enter => {
                // Execute sync
                self.execute_sync().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn install_server(&mut self, server: &crate::model::RegistryServer) -> Result<()> {
        let ide = match self.state.selected_ide() {
            Some(ide) if ide.detected => ide,
            _ => {
                self.state.status_message = Some("Select a detected IDE first".to_string());
                return Ok(());
            }
        };

        let installer = InstallerService::new(&self.ide_manager);
        let env = self.state.env_inputs.clone();

        match installer.install(&ide.id, server, env).await {
            Ok(()) => {
                self.state.status_message =
                    Some(format!("Installed {} to {}", server.name, ide.name));
                // Refresh IDE servers
                self.load_ides().await;
            }
            Err(e) => {
                self.state.status_message = Some(format!("Install failed: {}", e));
            }
        }

        Ok(())
    }

    async fn install_server_to_all_ides(
        &mut self,
        server: &crate::model::RegistryServer,
    ) -> Result<()> {
        let detected_ides: Vec<String> = self
            .state
            .ides
            .iter()
            .filter(|ide| ide.detected)
            .map(|ide| ide.id.clone())
            .collect();

        if detected_ides.is_empty() {
            self.state.status_message = Some("No IDEs detected".to_string());
            return Ok(());
        }

        let installer = InstallerService::new(&self.ide_manager);
        let env = self.state.env_inputs.clone();

        let mut success_count = 0;
        let mut failed_ides = Vec::new();

        for ide_id in &detected_ides {
            match installer.install(ide_id, server, env.clone()).await {
                Ok(()) => success_count += 1,
                Err(_) => failed_ides.push(ide_id.clone()),
            }
        }

        // Refresh IDE servers
        self.load_ides().await;

        if failed_ides.is_empty() {
            self.state.status_message = Some(format!(
                "Installed {} to {} IDEs",
                server.name, success_count
            ));
        } else {
            self.state.status_message = Some(format!(
                "Installed {} to {} IDEs, failed: {}",
                server.name,
                success_count,
                failed_ides.join(", ")
            ));
        }

        Ok(())
    }

    async fn preview_sync(&mut self) -> Result<()> {
        let source_idx = match self.state.sync_source_ide {
            Some(idx) => idx,
            None => {
                self.state.status_message = Some("Select a source IDE first".to_string());
                return Ok(());
            }
        };

        if self.state.sync_target_ides.is_empty() {
            self.state.status_message = Some("Select at least one target IDE".to_string());
            return Ok(());
        }

        let source_id = &self.state.ides[source_idx].id;
        let target_ids: Vec<&str> = self
            .state
            .sync_target_ides
            .iter()
            .filter_map(|&i| self.state.ides.get(i))
            .map(|ide| ide.id.as_str())
            .collect();

        let sync_service = SyncService::new(&self.ide_manager);
        match sync_service.preview(source_id, &target_ids, None).await {
            Ok(preview) => {
                self.state.sync_preview = preview;
            }
            Err(e) => {
                self.state.status_message = Some(format!("Preview failed: {}", e));
            }
        }

        Ok(())
    }

    async fn execute_sync(&mut self) -> Result<()> {
        if self.state.sync_preview.is_empty() {
            self.state.status_message = Some("Generate a preview first".to_string());
            return Ok(());
        }

        let source_idx = self.state.sync_source_ide.unwrap();
        let source_id = &self.state.ides[source_idx].id;
        let target_ids: Vec<&str> = self
            .state
            .sync_target_ides
            .iter()
            .filter_map(|&i| self.state.ides.get(i))
            .map(|ide| ide.id.as_str())
            .collect();

        let sync_service = SyncService::new(&self.ide_manager);
        match sync_service.sync(source_id, &target_ids, None).await {
            Ok(results) => {
                self.state.status_message = Some(format!("Synced {} servers", results.len()));
                self.state.sync_preview.clear();
                // Refresh IDE servers
                self.load_ides().await;
            }
            Err(e) => {
                self.state.status_message = Some(format!("Sync failed: {}", e));
            }
        }

        Ok(())
    }

    pub fn should_quit(&self) -> bool {
        self.state.should_quit
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare two version strings using semver-like logic.
/// Returns Ordering::Greater if a > b.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_segments = |s: &str| -> Vec<u64> {
        s.split('.')
            .map(|seg| seg.parse::<u64>().unwrap_or(0))
            .collect()
    };

    let a_segs = parse_segments(a);
    let b_segs = parse_segments(b);

    let max_len = a_segs.len().max(b_segs.len());
    for i in 0..max_len {
        let av = a_segs.get(i).copied().unwrap_or(0);
        let bv = b_segs.get(i).copied().unwrap_or(0);
        match av.cmp(&bv) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    // If numeric comparison is equal, fall back to string comparison
    a.cmp(b)
}

/// Deduplicate servers by name, keeping only the latest version of each.
fn deduplicate_latest(servers: &[RegistryServer]) -> Vec<RegistryServer> {
    let mut best: HashMap<String, &RegistryServer> = HashMap::new();

    for server in servers {
        let entry = best.entry(server.name.clone());
        entry
            .and_modify(|existing| {
                let existing_ver = existing.version.as_deref().unwrap_or("0.0.0");
                let new_ver = server.version.as_deref().unwrap_or("0.0.0");
                if compare_versions(new_ver, existing_ver) == std::cmp::Ordering::Greater {
                    *existing = server;
                }
            })
            .or_insert(server);
    }

    // Preserve original ordering (order of first appearance)
    let mut seen = std::collections::HashSet::new();
    servers
        .iter()
        .filter_map(|s| {
            if seen.insert(s.name.clone()) {
                best.get(&s.name).map(|&s| s.clone())
            } else {
                None
            }
        })
        .collect()
}
