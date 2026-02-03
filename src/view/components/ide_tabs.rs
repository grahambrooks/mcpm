use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::model::IdeInfo;

pub struct IdeTabsWidget;

impl IdeTabsWidget {
    pub fn render(frame: &mut Frame, area: Rect, ides: &[IdeInfo], selected_index: usize) {
        let titles: Vec<Line> = ides
            .iter()
            .map(|ide| {
                let style = if ide.detected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let indicator = if ide.detected { "●" } else { "○" };
                let server_count = if ide.server_count > 0 {
                    format!(" ({})", ide.server_count)
                } else {
                    String::new()
                };

                Line::from(vec![
                    Span::styled(indicator, style),
                    Span::raw(" "),
                    Span::styled(&ide.name, style),
                    Span::styled(server_count, style),
                ])
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .title("IDEs")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .select(selected_index)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, area);
    }
}
