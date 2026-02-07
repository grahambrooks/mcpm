use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::model::AppState;
use crate::view::components::{SearchBarWidget, ServerListWidget};

pub struct RegistryScreen;

impl RegistryScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Length(3), // Registry source toggle
                Constraint::Min(10),   // Server list
                Constraint::Length(4), // Help
            ])
            .split(area);

        // Search bar
        SearchBarWidget::render(
            frame,
            chunks[0],
            &state.search_query,
            state.input_mode,
            "Type to search servers...",
        );

        // Registry source toggle
        let source_line = Line::from(vec![
            Span::styled("Source: ", Style::default().fg(Color::Gray)),
            Span::styled(
                state.registry_source.name(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " (Press 's' to switch)",
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let source_widget = Paragraph::new(source_line).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
        frame.render_widget(source_widget, chunks[1]);

        // Server list
        if state.registry_loading {
            let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let spinner = spinner_chars[(state.loading_tick as usize / 2) % spinner_chars.len()];
            let loading = Paragraph::new(format!("{} Loading servers...", spinner))
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .title("Registry Servers")
                        .borders(Borders::ALL),
                );
            frame.render_widget(loading, chunks[2]);
        } else if let Some(error) = &state.registry_error {
            let error_widget = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .title("Error")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red)),
                );
            frame.render_widget(error_widget, chunks[2]);
        } else {
            let servers = state.displayed_servers();
            let title = if state.show_all_versions {
                format!(
                    "Registry Servers ({} versions, {} unique)",
                    servers.len(),
                    state.registry_servers_latest.len()
                )
            } else {
                format!("Registry Servers ({} servers)", servers.len())
            };
            ServerListWidget::render_registry_servers(
                frame,
                chunks[2],
                servers,
                state.selected_registry_index,
                &title,
            );
        }

        // Help text
        let help = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(" Search  "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" Navigate  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" Details  "),
                Span::styled("i", Style::default().fg(Color::Yellow)),
                Span::raw(" Install  "),
                Span::styled("I", Style::default().fg(Color::Yellow)),
                Span::raw(" Install All"),
            ]),
            Line::from(vec![
                Span::styled("s", Style::default().fg(Color::Yellow)),
                Span::raw(" Switch Registry  "),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw(" Refresh  "),
                Span::styled("v", Style::default().fg(Color::Yellow)),
                Span::raw(" Toggle Versions  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" Clear Search"),
            ]),
        ])
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(help, chunks[3]);
    }
}
