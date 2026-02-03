use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::model::{InstalledServer, RegistryServer};

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

                let description = if server.description.len() > 60 {
                    format!("{}...", &server.description[..57])
                } else {
                    server.description.clone()
                };

                let content = Line::from(vec![
                    Span::styled(&server.name, style.fg(Color::Cyan)),
                    Span::styled(" - ", style),
                    Span::styled(description, style.fg(Color::Gray)),
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
                let command_display = if command.len() > 50 {
                    format!("{}...", &command[..47])
                } else {
                    command
                };

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
