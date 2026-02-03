pub mod claude_code;
pub mod claude_desktop;
pub mod codex;
pub mod continue_dev;
pub mod cursor;
pub mod github_copilot;
pub mod traits;
pub mod vscode;
pub mod windsurf;

pub use claude_code::ClaudeCodeAdapter;
pub use claude_desktop::ClaudeDesktopAdapter;
pub use codex::CodexAdapter;
pub use continue_dev::ContinueDevAdapter;
pub use cursor::CursorAdapter;
pub use github_copilot::GithubCopilotAdapter;
pub use traits::{IdeAdapter, IdeManager};
pub use vscode::VSCodeAdapter;
pub use windsurf::WindsurfAdapter;

/// Create an IdeManager with all supported IDE adapters registered
pub fn create_ide_manager() -> IdeManager {
    let mut manager = IdeManager::new();
    manager.register(Box::new(ClaudeDesktopAdapter::new()));
    manager.register(Box::new(ClaudeCodeAdapter::new()));
    manager.register(Box::new(CodexAdapter::new()));
    manager.register(Box::new(CursorAdapter::new()));
    manager.register(Box::new(VSCodeAdapter::new()));
    manager.register(Box::new(WindsurfAdapter::new()));
    manager.register(Box::new(ContinueDevAdapter::new()));
    manager.register(Box::new(GithubCopilotAdapter::new()));
    manager
}
