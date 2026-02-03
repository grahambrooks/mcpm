pub mod components;
pub mod screens;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::model::{AppState, Screen};
use crate::view::components::StatusBarWidget;
use crate::view::screens::*;

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // Render current screen
    match state.screen {
        Screen::Dashboard => DashboardScreen::render(frame, chunks[0], state),
        Screen::Registry => RegistryScreen::render(frame, chunks[0], state),
        Screen::Installed => InstalledScreen::render(frame, chunks[0], state),
        Screen::ServerDetail => ServerDetailScreen::render(frame, chunks[0], state),
        Screen::Sync => SyncScreen::render(frame, chunks[0], state),
    }

    // Status bar
    StatusBarWidget::render(
        frame,
        chunks[1],
        state.screen,
        state.status_message.as_deref(),
    );

    // Help overlay
    if state.show_help {
        render_help_overlay(frame);
    }
}

fn render_help_overlay(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());

    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "MCPM - MCP Server Manager",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from("  1-5     Switch screens"),
        Line::from("  Tab     Switch IDE"),
        Line::from("  ↑↓      Navigate lists"),
        Line::from("  Enter   Select/Confirm"),
        Line::from("  Esc     Back/Cancel"),
        Line::from("  q       Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Registry",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from("  /       Search"),
        Line::from("  s       Switch registry source"),
        Line::from("  r       Refresh"),
        Line::from("  i       Install to selected IDE"),
        Line::from("  I       Install to all IDEs"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Installed",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from("  d       Delete server"),
        Line::from("  e       Edit server"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Sync",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from("  s       Set source IDE"),
        Line::from("  t       Toggle target IDE"),
        Line::from("  p       Preview sync"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press ? or Esc to close",
            Style::default().fg(Color::Gray),
        )]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
