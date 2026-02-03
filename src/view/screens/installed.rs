use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::model::AppState;
use crate::view::components::{IdeTabsWidget, ServerListWidget};

pub struct InstalledScreen;

impl InstalledScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // IDE tabs
                Constraint::Min(10),   // Server list
                Constraint::Length(4), // Help
            ])
            .split(area);

        // IDE tabs
        IdeTabsWidget::render(frame, chunks[0], &state.ides, state.selected_ide_index);

        // Server list for selected IDE
        if let Some(ide) = state.selected_ide() {
            if !ide.detected {
                let not_found = Paragraph::new(format!(
                    "{} not detected on this system.\nConfig path: {}",
                    ide.name,
                    ide.config_path.as_deref().unwrap_or("N/A")
                ))
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .title("IDE Not Detected")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                );
                frame.render_widget(not_found, chunks[1]);
            } else if let Some(servers) = state.ide_servers.get(&ide.id) {
                if servers.is_empty() {
                    let empty = Paragraph::new("No MCP servers installed for this IDE.\nPress 'a' to add a server or go to Registry (2) to browse.")
                        .style(Style::default().fg(Color::Gray))
                        .block(
                            Block::default()
                                .title(format!("{} - Installed Servers", ide.name))
                                .borders(Borders::ALL),
                        );
                    frame.render_widget(empty, chunks[1]);
                } else {
                    let title = format!("{} - {} servers", ide.name, servers.len());
                    ServerListWidget::render_installed_servers(
                        frame, chunks[1], servers,
                        0, // TODO: track selected server index per IDE
                        &title,
                    );
                }
            } else {
                let loading = Paragraph::new("Loading servers...")
                    .style(Style::default().fg(Color::Yellow))
                    .block(
                        Block::default()
                            .title(format!("{} - Installed Servers", ide.name))
                            .borders(Borders::ALL),
                    );
                frame.render_widget(loading, chunks[1]);
            }
        }

        // Help text
        let help = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" Switch IDE  "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" Navigate  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" View Details  "),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(" Delete  "),
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw(" Edit  "),
                Span::styled("a", Style::default().fg(Color::Yellow)),
                Span::raw(" Add Server"),
            ]),
        ])
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(help, chunks[2]);
    }
}
