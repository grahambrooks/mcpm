use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::model::AppState;
use crate::view::components::IdeTabsWidget;

pub struct DashboardScreen;

impl DashboardScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // IDE tabs
                Constraint::Min(10),   // Main content
                Constraint::Length(6), // Quick actions
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "MCPM",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - MCP Server Manager"),
        ]))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // IDE tabs
        IdeTabsWidget::render(frame, chunks[1], &state.ides, state.selected_ide_index);

        // Main content - IDE details and server summary
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        // Selected IDE info
        Self::render_ide_info(frame, content_chunks[0], state);

        // Server summary
        Self::render_server_summary(frame, content_chunks[1], state);

        // Quick actions
        Self::render_quick_actions(frame, chunks[3]);
    }

    fn render_ide_info(frame: &mut Frame, area: Rect, state: &AppState) {
        let content = if let Some(ide) = state.selected_ide() {
            let status = if ide.detected {
                Span::styled("● Detected", Style::default().fg(Color::Green))
            } else {
                Span::styled("○ Not Found", Style::default().fg(Color::Red))
            };

            let config_path = ide
                .config_path.as_deref()
                .unwrap_or("N/A");

            vec![
                Line::from(vec![
                    Span::styled("IDE: ", Style::default().fg(Color::Gray)),
                    Span::styled(&ide.name, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Gray)),
                    status,
                ]),
                Line::from(vec![
                    Span::styled("Config: ", Style::default().fg(Color::Gray)),
                    Span::styled(config_path, Style::default().fg(Color::Blue)),
                ]),
                Line::from(vec![
                    Span::styled("Servers: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        ide.server_count.to_string(),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
            ]
        } else {
            vec![Line::from("No IDE selected")]
        };

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .title("Selected IDE")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );

        frame.render_widget(paragraph, area);
    }

    fn render_server_summary(frame: &mut Frame, area: Rect, state: &AppState) {
        let total_servers: usize = state.ides.iter().map(|ide| ide.server_count).sum();
        let detected_ides = state.ides.iter().filter(|ide| ide.detected).count();

        let content = vec![
            Line::from(vec![
                Span::styled("Total Servers: ", Style::default().fg(Color::Gray)),
                Span::styled(total_servers.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Detected IDEs: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}/{}", detected_ides, state.ides.len()),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Registry: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    state.registry_source.name(),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("Cached Servers: ", Style::default().fg(Color::Gray)),
                if state.registry_loading {
                    Span::styled("Loading...", Style::default().fg(Color::Yellow))
                } else {
                    Span::styled(
                        state.registry_servers_latest.len().to_string(),
                        Style::default().fg(Color::White),
                    )
                },
            ]),
        ];

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .title("Summary")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );

        frame.render_widget(paragraph, area);
    }

    fn render_quick_actions(frame: &mut Frame, area: Rect) {
        let actions = vec![
            ListItem::new(Line::from(vec![
                Span::styled("2", Style::default().fg(Color::Yellow)),
                Span::raw(" Browse Registry - Search and install MCP servers"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("3", Style::default().fg(Color::Yellow)),
                Span::raw(" View Installed - Manage installed servers"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("5", Style::default().fg(Color::Yellow)),
                Span::raw(" Sync Servers - Copy configs between IDEs"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" Switch IDE - Navigate between IDEs"),
            ])),
        ];

        let list = List::new(actions).block(
            Block::default()
                .title("Quick Actions")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );

        frame.render_widget(list, area);
    }
}
