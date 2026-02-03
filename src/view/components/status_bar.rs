use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::model::Screen;

pub struct StatusBarWidget;

impl StatusBarWidget {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        current_screen: Screen,
        status_message: Option<&str>,
    ) {
        let nav_help = vec![
            Span::styled("1", Style::default().fg(Color::Yellow)),
            Span::raw(":Dashboard "),
            Span::styled("2", Style::default().fg(Color::Yellow)),
            Span::raw(":Registry "),
            Span::styled("3", Style::default().fg(Color::Yellow)),
            Span::raw(":Installed "),
            Span::styled("4", Style::default().fg(Color::Yellow)),
            Span::raw(":Detail "),
            Span::styled("5", Style::default().fg(Color::Yellow)),
            Span::raw(":Sync "),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(":Help "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(":Quit"),
        ];

        let status = if let Some(msg) = status_message {
            Line::from(vec![
                Span::styled(
                    format!("[{}] ", current_screen.title()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(msg, Style::default().fg(Color::Yellow)),
            ])
        } else {
            Line::from(nav_help)
        };

        let status_bar =
            Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(status_bar, area);
    }
}
