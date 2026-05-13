use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    style::Style,
    symbols::{self},
    widgets::{Block, Borders, Padding, Paragraph, Widget, WidgetRef, Wrap},
};

use crate::engine::{ToastBorderMode, ToastProgressBarStyle, ToastType};

/// A simple widget that represents a toast message. It displays a message with a border colored according to the toast type.
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub type_: ToastType,
    pub bg: Color,
    pub border_mode: ToastBorderMode,
    pub progress_bar_style: ToastProgressBarStyle,
    progress_ratio: Option<f64>,
}

impl Toast {
    /// Creates a new `Toast` widget with the given message and type.
    pub fn new(
        message: &str,
        type_: ToastType,
        bg: Color,
        border_mode: ToastBorderMode,
        progress_bar_style: ToastProgressBarStyle,
    ) -> Self {
        Self {
            message: message.to_string(),
            type_,
            bg,
            border_mode,
            progress_bar_style,
            progress_ratio: None,
        }
    }

    pub fn with_progress_ratio(mut self, progress_ratio: Option<f64>) -> Self {
        self.progress_ratio = progress_ratio;
        self
    }

    pub fn with_progress_bar_style(mut self, progress_bar_style: ToastProgressBarStyle) -> Self {
        self.progress_bar_style = progress_bar_style;
        self
    }
}

fn render_progress_bar(
    buf: &mut Buffer,
    area: Rect,
    progress_ratio: f64,
    color: Color,
    background: Color,
    style: ToastProgressBarStyle,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let progress_ratio = progress_ratio.clamp(0.0, 1.0);
    let filled = ((area.width as f64) * progress_ratio).round() as u16;
    let (filled_symbol, empty_symbol) = match style {
        ToastProgressBarStyle::FullBlock => ("█", "░"),
        ToastProgressBarStyle::HalfBlock => ("▄", " "),
        ToastProgressBarStyle::Minimal => ("_", " "),
    };

    for offset in 0..area.width {
        let symbol = if offset < filled {
            filled_symbol
        } else {
            empty_symbol
        };
        let fg = if offset < filled { color } else { Color::DarkGray };
        buf[(area.x + offset, area.y)]
            .set_symbol(symbol)
            .set_fg(fg)
            .set_bg(background);
    }
}

impl WidgetRef for Toast {
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        const PADDING: u16 = 1;
        let borders = match self.border_mode {
            ToastBorderMode::SideRails => Borders::LEFT | Borders::RIGHT,
            ToastBorderMode::Full => Borders::ALL,
        };
        let block = Block::default()
            .borders(borders)
            .border_set(symbols::border::QUADRANT_OUTSIDE)
            .padding(Padding::uniform(PADDING))
            .style(Style::default().bg(self.bg))
            .border_style(Style::default().fg(self.type_.into()).bg(self.bg));
        let inner = block.inner(area);
        block.render(area, buf);

        let (message_area, progress_area) = if self.progress_ratio.is_some() && inner.height > 0 {
            (
                Rect {
                    x: inner.x,
                    y: inner.y,
                    width: inner.width,
                    height: inner.height.saturating_sub(1),
                },
                Some(Rect {
                    x: inner.x,
                    y: inner.y + inner.height.saturating_sub(1),
                    width: inner.width,
                    height: 1,
                }),
            )
        } else {
            (inner, None)
        };

        if message_area.width > 0 && message_area.height > 0 {
            Paragraph::new(self.message.as_str())
                .style(Style::default().fg(Color::White).bg(self.bg))
                .wrap(Wrap { trim: false })
                .render(message_area, buf);
        }

        if let (Some(progress_ratio), Some(progress_area)) = (self.progress_ratio, progress_area) {
            render_progress_bar(
                buf,
                progress_area,
                progress_ratio,
                self.type_.into(),
                self.bg,
                self.progress_bar_style,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_full_block_uses_full_block_glyph() {
        let area = Rect::new(0, 0, 8, 4);
        let mut buf = Buffer::empty(area);
        Toast::new(
            "hello",
            ToastType::Info,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::FullBlock,
        )
        .with_progress_ratio(Some(0.5))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(2, 2)].symbol(), "█");
    }

    #[test]
    fn progress_bar_half_block_uses_half_block_glyph() {
        let area = Rect::new(0, 0, 8, 4);
        let mut buf = Buffer::empty(area);
        Toast::new(
            "hello",
            ToastType::Info,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::HalfBlock,
        )
        .with_progress_ratio(Some(0.5))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(2, 2)].symbol(), "▄");
    }

    #[test]
    fn progress_bar_minimal_uses_underscore_glyph() {
        let area = Rect::new(0, 0, 8, 4);
        let mut buf = Buffer::empty(area);
        Toast::new(
            "hello",
            ToastType::Info,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::Minimal,
        )
        .with_progress_ratio(Some(0.5))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(2, 2)].symbol(), "_");
    }
}
