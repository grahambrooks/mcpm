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
            // Compute nav help as plain string to measure its width
            let nav_text: String = nav_help.iter().map(|s| s.content.as_ref()).collect();
            let status_prefix = format!("[{}] ", current_screen.title());
            let status_text = format!("{}{}", status_prefix, msg);

            // If there's room, show status on left and nav on right with padding
            let width = area.width as usize;
            let nav_width = nav_text.chars().count();
            let status_width = status_text.chars().count();
            let gap = width.saturating_sub(status_width + nav_width);

            let mut spans = vec![
                Span::styled(
                    status_prefix,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(msg, Style::default().fg(Color::Yellow)),
            ];

            if gap >= 2 {
                spans.push(Span::raw(" ".repeat(gap)));
                spans.extend(nav_help);
            }

            Line::from(spans)
        } else {
            Line::from(nav_help)
        };

        let status_bar =
            Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(status_bar, area);
    }
}
