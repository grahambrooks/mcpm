use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::model::InputMode;

pub struct SearchBarWidget;

impl SearchBarWidget {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        query: &str,
        input_mode: InputMode,
        placeholder: &str,
    ) {
        let (border_color, text) = match input_mode {
            InputMode::Search => (Color::Yellow, format!("{}│", query)),
            _ => {
                if query.is_empty() {
                    (Color::Gray, placeholder.to_string())
                } else {
                    (Color::White, query.to_string())
                }
            }
        };

        let search = Paragraph::new(text).block(
            Block::default()
                .title("Search (/ to focus)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );

        frame.render_widget(search, area);
    }
}
