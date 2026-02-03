use crate::model::{IdeInfo, InstalledServer, RegistryServer, Screen};

/// Messages for the TEA (The Elm Architecture) pattern
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(Screen),
    GoBack,
    ToggleHelp,
    Quit,

    // Input mode
    EnterSearchMode,
    ExitSearchMode,
    EnterEnvInputMode,
    ExitEnvInputMode,
    SearchInput(char),
    SearchBackspace,
    ClearSearch,
    EnvInput(String, char),
    EnvBackspace(String),

    // IDE actions
    IdesLoaded(Vec<IdeInfo>),
    SelectIde(usize),
    NextIde,
    PreviousIde,
    IdeServersLoaded(String, Vec<InstalledServer>),

    // Registry actions
    FetchRegistry,
    RegistryLoaded(Vec<RegistryServer>),
    RegistryError(String),
    ToggleRegistrySource,
    SearchRegistry(String),
    SearchResultsLoaded(Vec<RegistryServer>),
    SelectRegistryServer(usize),
    NextServer,
    PreviousServer,
    ViewServerDetail(RegistryServer),

    // Install actions
    InstallServer(String, RegistryServer),
    ServerInstalled(String, String),
    InstallError(String),
    RemoveServer(String, String),
    ServerRemoved(String, String),
    RemoveError(String),

    // Sync actions
    SetSyncSource(usize),
    ToggleSyncTarget(usize),
    ToggleSyncServer(String),
    PreviewSync,
    SyncPreviewReady(Vec<crate::model::SyncPreviewItem>),
    ExecuteSync,
    SyncComplete,
    SyncError(String),

    // Status
    SetStatus(String),
    ClearStatus,

    // Async results
    Tick,
    Error(String),
}
