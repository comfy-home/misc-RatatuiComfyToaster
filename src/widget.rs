use ratatui::{
    style::Color,
    style::Style,
    symbols::{self},
    widgets::{Block, Borders, Padding, Paragraph, Widget, WidgetRef, Wrap},
};

use crate::engine::ToastType;

/// A simple widget that represents a toast message. It displays a message with a border colored according to the toast type.
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub type_: ToastType,
    pub bg: Color,
}

impl Toast {
    /// Creates a new `Toast` widget with the given message and type.
    pub fn new(message: &str, type_: ToastType, bg: Color) -> Self {
        Self {
            message: message.to_string(),
            type_,
            bg,
        }
    }
}

impl WidgetRef for Toast {
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        const PADDING: u16 = 1;
        let paragraph = Paragraph::new(self.message.as_str())
            .style(Style::default().fg(Color::White).bg(self.bg))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .border_set(symbols::border::QUADRANT_OUTSIDE)
                    .padding(Padding::uniform(PADDING))
                    .style(Style::default().bg(self.bg))
                    .border_style(Style::default().fg(self.type_.into()).bg(self.bg)),
            );
        paragraph.render(area, buf);
    }
}
