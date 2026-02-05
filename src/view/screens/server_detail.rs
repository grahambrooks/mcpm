use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::model::AppState;
use crate::view::components::IdeTabsWidget;

pub struct ServerDetailScreen;

impl ServerDetailScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // IDE selector
                Constraint::Min(10),   // Server details
                Constraint::Length(6), // Env vars input
                Constraint::Length(4), // Actions
            ])
            .split(area);

        // IDE selector for installation target
        IdeTabsWidget::render(frame, chunks[0], &state.ides, state.selected_ide_index);

        // Server details
        if let Some(server) = &state.selected_server {
            let details_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(chunks[1]);

            // Left side - basic info
            let mut info_lines = vec![
                Line::from(vec![Span::styled(
                    server.display_name(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
            ];

            // Show namespaced name if title differs
            if server.title.is_some() && server.display_name() != server.name {
                info_lines.push(Line::from(vec![Span::styled(
                    format!("({})", server.name),
                    Style::default().fg(Color::DarkGray),
                )]));
            }

            info_lines.push(Line::from(""));

            info_lines.push(Line::from(vec![Span::styled(
                "Description: ",
                Style::default().fg(Color::Gray),
            )]));
            info_lines.push(Line::from(vec![Span::styled(
                &server.description,
                Style::default().fg(Color::White),
            )]));
            info_lines.push(Line::from(""));

            // Version
            if let Some(version) = &server.version {
                info_lines.push(Line::from(vec![
                    Span::styled("Version: ", Style::default().fg(Color::Gray)),
                    Span::styled(version, Style::default().fg(Color::Green)),
                ]));
            }

            // Transport type
            let transport = server.primary_transport();
            info_lines.push(Line::from(vec![
                Span::styled("Transport: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    transport.to_string(),
                    Style::default().fg(Color::Magenta),
                ),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("Source: ", Style::default().fg(Color::Gray)),
                Span::styled(&server.registry_source, Style::default().fg(Color::Yellow)),
            ]));
            info_lines.push(Line::from(vec![
                Span::styled("Vendor: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    server.vendor.as_deref().unwrap_or("N/A"),
                    Style::default().fg(Color::White),
                ),
            ]));
            info_lines.push(Line::from(vec![
                Span::styled("License: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    server.license.as_deref().unwrap_or("N/A"),
                    Style::default().fg(Color::White),
                ),
            ]));

            let info_widget = Paragraph::new(info_lines).block(
                Block::default()
                    .title("Server Information")
                    .borders(Borders::ALL),
            );
            frame.render_widget(info_widget, details_chunks[0]);

            // Right side - install command / remote URL
            let install_lines = if !server.remotes.is_empty() && server.install_command.is_none() {
                // Remote-only server: show URL
                let remote = &server.remotes[0];
                vec![
                    Line::from(vec![Span::styled(
                        "Remote URL: ",
                        Style::default().fg(Color::Gray),
                    )]),
                    Line::from(vec![Span::styled(
                        &remote.url,
                        Style::default().fg(Color::Green),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        format!("Transport: {}", remote.transport_type),
                        Style::default().fg(Color::Magenta),
                    )]),
                ]
            } else {
                let command = server.install_command.as_deref().unwrap_or("npx");
                let args = server.install_args.join(" ");

                vec![
                    Line::from(vec![Span::styled(
                        "Command: ",
                        Style::default().fg(Color::Gray),
                    )]),
                    Line::from(vec![Span::styled(
                        command,
                        Style::default().fg(Color::Green),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Arguments: ",
                        Style::default().fg(Color::Gray),
                    )]),
                    Line::from(vec![Span::styled(args, Style::default().fg(Color::Blue))]),
                ]
            };

            let install_widget = Paragraph::new(install_lines)
                .block(Block::default().title("Installation").borders(Borders::ALL));
            frame.render_widget(install_widget, details_chunks[1]);

            // Environment variables input
            Self::render_env_vars(frame, chunks[2], state);
        } else {
            let no_server = Paragraph::new(
                "No server selected.\nGo to Registry (2) and select a server to view details.",
            )
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .title("Server Details")
                    .borders(Borders::ALL),
            );
            frame.render_widget(no_server, chunks[1]);

            // Empty env vars section
            let empty_env = Paragraph::new("").block(
                Block::default()
                    .title("Environment Variables")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(empty_env, chunks[2]);
        }

        // Actions
        let actions = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("i", Style::default().fg(Color::Yellow)),
                Span::raw(" Install  "),
                Span::styled("I", Style::default().fg(Color::Yellow)),
                Span::raw(" Install to All IDEs  "),
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" Switch IDE"),
            ]),
            Line::from(vec![
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw(" Edit env vars  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" Back to Registry"),
            ]),
        ])
        .block(
            Block::default()
                .title("Actions")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(actions, chunks[3]);
    }

    fn render_env_vars(frame: &mut Frame, area: Rect, state: &AppState) {
        let server = match &state.selected_server {
            Some(s) => s,
            None => return,
        };

        let all_vars = server.all_env_vars();

        if all_vars.is_empty() {
            let no_env = Paragraph::new("No environment variables required.")
                .style(Style::default().fg(Color::Gray))
                .block(
                    Block::default()
                        .title("Environment Variables")
                        .borders(Borders::ALL),
                );
            frame.render_widget(no_env, area);
            return;
        }

        let items: Vec<ListItem> = all_vars
            .iter()
            .enumerate()
            .map(|(i, var)| {
                let value = state
                    .env_inputs
                    .get(&var.name)
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let mut spans = vec![
                    Span::styled(
                        &var.name,
                        if i == state.env_input_index {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        },
                    ),
                ];

                if var.required {
                    spans.push(Span::styled("*", Style::default().fg(Color::Red)));
                }

                if var.is_secret {
                    spans.push(Span::styled(" [secret]", Style::default().fg(Color::Red)));
                }

                spans.push(Span::raw(": "));

                if value.is_empty() {
                    if let Some(default) = &var.default_value {
                        spans.push(Span::styled(
                            format!("(default: {})", default),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                } else {
                    spans.push(Span::styled(value, Style::default().fg(Color::Green)));
                }

                if i == state.env_input_index {
                    spans.push(Span::styled("│", Style::default().fg(Color::Yellow)));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title("Environment Variables (e to edit)")
                .borders(Borders::ALL),
        );
        frame.render_widget(list, area);
    }
}
