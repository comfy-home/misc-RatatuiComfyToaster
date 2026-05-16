use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    style::{Modifier, Style},
    symbols::{self},
    widgets::{Block, Borders, Widget, WidgetRef},
};
use textwrap::wrap;

use crate::engine::{ToastBorderMode, ToastProgressBarStyle, ToastType};
use crate::title::{
    contrasting_fg, dot_separator, line_separator, toast_content_padding, toast_content_rows,
    ToastTitle, ToastTitleAlign, ToastTitleLayout, ToastTitleSeparator, ToastTitleStyle,
};

/// A simple widget that represents a toast message. It displays a message with a border colored according to the toast type.
#[derive(Debug, Clone)]
pub struct Toast {
    title: Option<ToastTitle>,
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
            title: None,
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

    pub fn with_title(mut self, title: Option<ToastTitle>) -> Self {
        self.title = title.filter(|title| !title.is_empty());
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
        let fg = if offset < filled {
            color
        } else {
            Color::DarkGray
        };
        buf[(area.x + offset, area.y)]
            .set_symbol(symbol)
            .set_fg(fg)
            .set_bg(background);
    }
}

fn fill_row(buf: &mut Buffer, area: Rect, style: Style) {
    for x in area.x..area.x.saturating_add(area.width) {
        buf[(x, area.y)].set_symbol(" ").set_style(style);
    }
}

fn title_text_x_in_row(title_len: usize, align: ToastTitleAlign, row_width: u16) -> u16 {
    match align {
        ToastTitleAlign::Start => 0,
        ToastTitleAlign::Center => row_width.saturating_sub(title_len as u16) / 2,
    }
}

fn title_row_layout(
    _outer: Rect,
    content_row: Rect,
    title: &ToastTitle,
    border_mode: ToastBorderMode,
) -> (Rect, u16) {
    let text_x = content_row.x + title_text_x_in_row(title.text.chars().count(), title.align, content_row.width);

    if title.style == ToastTitleStyle::Highlight && title.align == ToastTitleAlign::Start {
        let extend_left = match border_mode {
            ToastBorderMode::SideRails => 1,
            ToastBorderMode::Full => 1,
        };
        let paint_area = Rect {
            x: content_row.x.saturating_sub(extend_left),
            y: content_row.y,
            width: content_row.width.saturating_add(extend_left),
            height: 1,
        };
        return (paint_area, text_x);
    }

    (content_row, text_x)
}

fn render_title_row(
    buf: &mut Buffer,
    paint_area: Rect,
    text_x: u16,
    title: &ToastTitle,
    type_color: Color,
    toast_bg: Color,
) {
    if paint_area.width == 0 {
        return;
    }

    let base_style = Style::default().bg(toast_bg);
    fill_row(buf, paint_area, base_style);

    let title_len = title.text.chars().count() as u16;

    if title.style == ToastTitleStyle::Highlight {
        let highlight_style = Style::default()
            .fg(contrasting_fg(type_color))
            .bg(type_color);
        let (highlight_start, highlight_end) = match title.align {
            ToastTitleAlign::Start => (paint_area.x, (text_x + title_len + 1).min(paint_area.x + paint_area.width)),
            ToastTitleAlign::Center => {
                let band = (title_len + 4).min(paint_area.width);
                let start = paint_area.x + paint_area.width.saturating_sub(band) / 2;
                (start, (start + band).min(paint_area.x + paint_area.width))
            }
        };
        for x in highlight_start..highlight_end {
            buf[(x, paint_area.y)].set_symbol(" ").set_style(highlight_style);
        }
    }

    let text_style = match title.style {
        ToastTitleStyle::Plain => Style::default()
            .fg(type_color)
            .bg(toast_bg)
            .add_modifier(Modifier::BOLD),
        ToastTitleStyle::Highlight => Style::default()
            .fg(contrasting_fg(type_color))
            .bg(type_color)
            .add_modifier(Modifier::BOLD),
    };

    for (offset, ch) in title.text.chars().enumerate() {
        let x = text_x + offset as u16;
        if x >= paint_area.x + paint_area.width {
            break;
        }
        let mut encoded = [0u8; 4];
        let symbol = ch.encode_utf8(&mut encoded);
        buf[(x, paint_area.y)].set_symbol(symbol).set_style(text_style);
    }
}

fn render_separator_row(
    buf: &mut Buffer,
    area: Rect,
    separator: ToastTitleSeparator,
    type_color: Color,
    toast_bg: Color,
) {
    let style = Style::default().fg(type_color).bg(toast_bg);
    fill_row(buf, area, Style::default().bg(toast_bg));

    let Some(text) = (match separator {
        ToastTitleSeparator::Dot => Some(dot_separator(area.width)),
        ToastTitleSeparator::Line => Some(line_separator(area.width)),
        ToastTitleSeparator::Empty => None,
    }) else {
        return;
    };

    for (offset, ch) in text.chars().enumerate() {
        if offset as u16 >= area.width {
            break;
        }
        let mut encoded = [0u8; 4];
        let symbol = ch.encode_utf8(&mut encoded);
        buf[(area.x + offset as u16, area.y)]
            .set_symbol(symbol)
            .set_style(style);
    }
}

fn render_message_row(buf: &mut Buffer, area: Rect, line: &str, toast_bg: Color) {
    fill_row(buf, area, Style::default().bg(toast_bg));
    let style = Style::default().fg(Color::White).bg(toast_bg);
    for (offset, ch) in line.chars().enumerate() {
        if offset as u16 >= area.width {
            break;
        }
        let mut encoded = [0u8; 4];
        let symbol = ch.encode_utf8(&mut encoded);
        buf[(area.x + offset as u16, area.y)]
            .set_symbol(symbol)
            .set_style(style);
    }
}

fn row_rect(area: Rect, row: u16) -> Rect {
    Rect {
        x: area.x,
        y: area.y.saturating_add(row),
        width: area.width,
        height: 1,
    }
}

fn render_toast_body(buf: &mut Buffer, outer: Rect, area: Rect, toast: &Toast) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let message_lines: Vec<String> = if toast.message.is_empty() {
        vec![String::new()]
    } else {
        wrap(&toast.message, area.width as usize)
            .into_iter()
            .map(|line| line.to_string())
            .collect()
    };
    let message_line_count = message_lines.len().max(1);
    let content_rows = toast_content_rows(toast.title.as_ref(), message_line_count) as usize;
    let rows = std::cmp::min(content_rows, area.height as usize);
    let type_color = toast.type_.into();

    let (title_row, separator_row, message_start) =
        match toast.title.as_ref().filter(|title| !title.is_empty()) {
            None => (None, None, 0usize),
            Some(title) if title.layout == ToastTitleLayout::Gapped => {
                (Some(0usize), Some(1usize), 2usize)
            }
            Some(_) => (Some(0usize), None, 1usize),
        };

    if let (Some(title), Some(row)) = (toast.title.as_ref(), title_row) {
        if row < rows {
            let content_row = row_rect(area, row as u16);
            let (paint_area, text_x) =
                title_row_layout(outer, content_row, title, toast.border_mode);
            render_title_row(buf, paint_area, text_x, title, type_color, toast.bg);
        }
    }

    if let (Some(title), Some(row)) = (toast.title.as_ref(), separator_row) {
        if row < rows {
            render_separator_row(
                buf,
                row_rect(area, row as u16),
                title.separator,
                type_color,
                toast.bg,
            );
        }
    }

    for (index, line) in message_lines.iter().enumerate() {
        let row = message_start + index;
        if row >= rows {
            break;
        }
        render_message_row(buf, row_rect(area, row as u16), line.as_str(), toast.bg);
    }
}

impl WidgetRef for Toast {
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let borders = match self.border_mode {
            ToastBorderMode::SideRails => Borders::LEFT | Borders::RIGHT,
            ToastBorderMode::Full => Borders::ALL,
        };
        let content_padding = toast_content_padding(self.title.as_ref());
        let block = Block::default()
            .borders(borders)
            .border_set(symbols::border::QUADRANT_OUTSIDE)
            .padding(content_padding)
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
            render_toast_body(buf, area, message_area, self);
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
    use crate::title::ToastTitleAlign;

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

    #[test]
    fn compact_title_renders_on_first_body_row() {
        let area = Rect::new(0, 0, 20, 6);
        let mut buf = Buffer::empty(area);
        Toast::new(
            "details",
            ToastType::Error,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::FullBlock,
        )
        .with_title(Some(ToastTitle::compact("Build Failed")))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(2, 0)].symbol(), "B");
        assert_eq!(buf[(2, 0)].fg, Color::Red);
        assert_eq!(buf[(2, 0)].bg, Color::DarkGray);
        assert_eq!(buf[(2, 1)].symbol(), "d");
        assert_eq!(buf[(2, 1)].fg, Color::White);
    }

    #[test]
    fn titled_toast_has_no_empty_row_above_title() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);
        Toast::new(
            "details",
            ToastType::Error,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::FullBlock,
        )
        .with_title(Some(ToastTitle::compact("Title")))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(2, 0)].symbol(), "T");
    }

    #[test]
    fn untitled_toast_keeps_top_padding_row_above_message() {
        let area = Rect::new(0, 0, 20, 4);
        let mut buf = Buffer::empty(area);
        Toast::new(
            "details",
            ToastType::Error,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::FullBlock,
        )
        .render_ref(area, &mut buf);

        assert_eq!(buf[(2, 0)].symbol(), " ");
        assert_eq!(buf[(2, 0)].bg, Color::DarkGray);
        assert_eq!(buf[(2, 1)].symbol(), "d");
    }

    #[test]
    fn highlight_title_uses_contrasting_text_on_type_background() {
        let area = Rect::new(0, 0, 20, 6);
        let mut buf = Buffer::empty(area);
        let mut title = ToastTitle::compact("Error");
        title.style = ToastTitleStyle::Highlight;
        Toast::new(
            "details",
            ToastType::Error,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::FullBlock,
        )
        .with_title(Some(title))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(1, 0)].fg, Color::White);
        assert_eq!(buf[(1, 0)].bg, Color::Red);
    }

    #[test]
    fn highlight_start_extends_through_left_border_without_gap() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);
        let mut title = ToastTitle::compact("Err");
        title.style = ToastTitleStyle::Highlight;
        Toast::new(
            "details",
            ToastType::Error,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::FullBlock,
        )
        .with_title(Some(title))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(0, 0)].bg, Color::Red);
        assert_eq!(buf[(1, 0)].bg, Color::Red);
    }

    #[test]
    fn gapped_title_renders_separator_between_title_and_message() {
        let area = Rect::new(0, 0, 20, 7);
        let mut buf = Buffer::empty(area);
        Toast::new(
            "details",
            ToastType::Info,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::FullBlock,
        )
        .with_title(Some(ToastTitle::gapped("Scope")))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(2, 0)].symbol(), "S");
        assert_eq!(buf[(2, 1)].symbol(), "·");
        assert_eq!(buf[(2, 1)].fg, Color::Blue);
        assert_eq!(buf[(2, 2)].symbol(), "d");
    }

    #[test]
    fn centered_title_is_horizontally_centered() {
        let area = Rect::new(0, 0, 20, 6);
        let mut buf = Buffer::empty(area);
        let mut title = ToastTitle::compact("Go");
        title.align = ToastTitleAlign::Center;
        Toast::new(
            "ok",
            ToastType::Info,
            Color::DarkGray,
            ToastBorderMode::SideRails,
            ToastProgressBarStyle::FullBlock,
        )
        .with_title(Some(title))
        .render_ref(area, &mut buf);

        assert_eq!(buf[(9, 0)].symbol(), "G");
    }
}
