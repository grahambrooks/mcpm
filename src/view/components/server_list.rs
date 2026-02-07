use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::model::{InstalledServer, RegistryServer};

/// Truncate a string to at most `max_chars` characters, appending "..." if truncated.
/// Also replaces newlines with spaces for single-line display.
fn truncate_str(s: &str, max_chars: usize) -> String {
    let clean: String = s.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
    if clean.chars().count() > max_chars {
        let truncated: String = clean.chars().take(max_chars).collect();
        format!("{}...", truncated)
    } else {
        clean
    }
}

pub struct ServerListWidget;

impl ServerListWidget {
    pub fn render_registry_servers(
        frame: &mut Frame,
        area: Rect,
        servers: &[RegistryServer],
        selected_index: usize,
        title: &str,
    ) {
        let items: Vec<ListItem> = servers
            .iter()
            .enumerate()
            .map(|(i, server)| {
                let style = if i == selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let display_name = server.display_name();

                // Compute used width: border (1) + padding (1) + name + version + " - " + border (1) + padding (1)
                let mut used: usize = 4 + display_name.chars().count() + 3; // borders/padding + name + " - "
                if let Some(version) = &server.version {
                    used += 2 + version.chars().count(); // " v" + version
                }
                let available = (area.width as usize).saturating_sub(used);
                let description = truncate_str(&server.description, available);

                let mut spans = vec![
                    Span::styled(display_name, style.fg(Color::Cyan)),
                ];

                if let Some(version) = &server.version {
                    spans.push(Span::styled(
                        format!(" v{}", version),
                        style.fg(Color::DarkGray),
                    ));
                }

                spans.push(Span::styled(" - ", style));
                spans.push(Span::styled(description, style.fg(Color::Gray)));

                let content = Line::from(spans);

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        let mut state = ListState::default();
        state.select(Some(selected_index));

        frame.render_stateful_widget(list, area, &mut state);
    }

    pub fn render_installed_servers(
        frame: &mut Frame,
        area: Rect,
        servers: &[InstalledServer],
        selected_index: usize,
        title: &str,
    ) {
        let items: Vec<ListItem> = servers
            .iter()
            .enumerate()
            .map(|(i, server)| {
                let style = if i == selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let command = server.config.display_command();
                // Compute available width: area - borders/padding (4) - name - " → " (3)
                let used: usize = 4 + server.name.chars().count() + 3;
                let available = (area.width as usize).saturating_sub(used);
                let command_display = truncate_str(&command, available);

                let content = Line::from(vec![
                    Span::styled(&server.name, style.fg(Color::Green)),
                    Span::styled(" → ", style),
                    Span::styled(command_display, style.fg(Color::Gray)),
                ]);

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        let mut state = ListState::default();
        state.select(Some(selected_index));

        frame.render_stateful_widget(list, area, &mut state);
    }
}
