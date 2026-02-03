use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::model::{AppState, SyncAction};

pub struct SyncScreen;

impl SyncScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Source IDE selection
                Constraint::Length(7), // Target IDE selection
                Constraint::Min(8),    // Preview
                Constraint::Length(4), // Actions
            ])
            .split(area);

        // Source IDE selection
        Self::render_source_selection(frame, chunks[0], state);

        // Target IDE selection
        Self::render_target_selection(frame, chunks[1], state);

        // Sync preview
        Self::render_preview(frame, chunks[2], state);

        // Actions
        let actions = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("s", Style::default().fg(Color::Yellow)),
                Span::raw(" Set source  "),
                Span::styled("t", Style::default().fg(Color::Yellow)),
                Span::raw(" Toggle target  "),
                Span::styled("p", Style::default().fg(Color::Yellow)),
                Span::raw(" Preview sync"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" Execute sync  "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" Navigate  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" Cancel"),
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

    fn render_source_selection(frame: &mut Frame, area: Rect, state: &AppState) {
        let items: Vec<ListItem> = state
            .ides
            .iter()
            .enumerate()
            .filter(|(_, ide)| ide.detected)
            .map(|(i, ide)| {
                let is_selected = state.sync_source_ide == Some(i);
                let marker = if is_selected { "◉" } else { "○" };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(marker, style),
                    Span::raw(" "),
                    Span::styled(&ide.name, style),
                    Span::styled(
                        format!(" ({} servers)", ide.server_count),
                        Style::default().fg(Color::Gray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title("Source IDE (press 's' on IDE to select)")
                .borders(Borders::ALL),
        );
        frame.render_widget(list, area);
    }

    fn render_target_selection(frame: &mut Frame, area: Rect, state: &AppState) {
        let items: Vec<ListItem> = state
            .ides
            .iter()
            .enumerate()
            .filter(|(i, ide)| ide.detected && state.sync_source_ide != Some(*i))
            .map(|(i, ide)| {
                let is_selected = state.sync_target_ides.contains(&i);
                let marker = if is_selected { "☑" } else { "☐" };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(marker, style),
                    Span::raw(" "),
                    Span::styled(&ide.name, style),
                    Span::styled(
                        format!(" ({} servers)", ide.server_count),
                        Style::default().fg(Color::Gray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title("Target IDEs (press 't' to toggle)")
                .borders(Borders::ALL),
        );
        frame.render_widget(list, area);
    }

    fn render_preview(frame: &mut Frame, area: Rect, state: &AppState) {
        if state.sync_preview.is_empty() {
            let hint = if state.sync_source_ide.is_none() {
                "Select a source IDE to begin."
            } else if state.sync_target_ides.is_empty() {
                "Select at least one target IDE."
            } else {
                "Press 'p' to preview sync changes."
            };

            let empty = Paragraph::new(hint)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().title("Sync Preview").borders(Borders::ALL));
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = state
            .sync_preview
            .iter()
            .map(|item| {
                let (action_str, color) = match item.action {
                    SyncAction::Add => ("ADD", Color::Green),
                    SyncAction::Update => ("UPDATE", Color::Yellow),
                    SyncAction::Skip => ("SKIP", Color::Gray),
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("[{}]", action_str), Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(&item.server_name, Style::default().fg(Color::Cyan)),
                    Span::raw(" → "),
                    Span::styled(&item.target_ide, Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(format!(
                    "Sync Preview ({} changes)",
                    state.sync_preview.len()
                ))
                .borders(Borders::ALL),
        );
        frame.render_widget(list, area);
    }
}
