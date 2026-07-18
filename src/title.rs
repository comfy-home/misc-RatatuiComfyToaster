use std::borrow::Cow;

use ratatui::{style::Color, widgets::Padding};

/// How the title shares vertical space with the message body.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToastTitleLayout {
    /// Title on the first content row; message follows on subsequent rows.
    #[default]
    Compact,
    /// Title, separator row, then message rows.
    Gapped,
}

/// Separator style for [`ToastTitleLayout::Gapped`] toasts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToastTitleSeparator {
    /// Middle-dot fill across the row (same glyph as ComfyGit tiles).
    #[default]
    Dot,
    /// Horizontal line across the row.
    Line,
    /// Blank separator row (background only).
    Empty,
}

/// Horizontal placement of the title text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToastTitleAlign {
    #[default]
    Start,
    Center,
}

/// Title foreground/background treatment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToastTitleStyle {
    /// Type-colored bold text on the toast background.
    #[default]
    Plain,
    /// Type color as background with contrasting foreground.
    Highlight,
}

/// Title configuration carried through layout and rendering.
#[derive(Debug, Clone)]
pub struct ToastTitle {
    pub text: String,
    pub layout: ToastTitleLayout,
    pub separator: ToastTitleSeparator,
    pub align: ToastTitleAlign,
    pub style: ToastTitleStyle,
}

impl ToastTitle {
    pub fn compact(text: impl Into<Cow<'static, str>>) -> Self {
        let text = text.into();
        Self {
            text: text.into_owned(),
            layout: ToastTitleLayout::Compact,
            separator: ToastTitleSeparator::Dot,
            align: ToastTitleAlign::Start,
            style: ToastTitleStyle::Plain,
        }
    }

    pub fn gapped(text: impl Into<Cow<'static, str>>) -> Self {
        let mut title = Self::compact(text);
        title.layout = ToastTitleLayout::Gapped;
        title
    }

    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

/// Inner content rows (excluding border/padding/progress chrome).
pub fn toast_has_title(title: Option<&ToastTitle>) -> bool {
    title.is_some_and(|title| !title.is_empty())
}

/// Vertical padding rows counted in auto height (`top + bottom`).
pub fn toast_vertical_padding_rows(title: Option<&ToastTitle>) -> u16 {
    if toast_has_title(title) { 1 } else { 2 }
}

/// Per-side content padding inside the toast border.
pub fn toast_content_padding(title: Option<&ToastTitle>) -> Padding {
    match title.filter(|title| !title.is_empty()) {
        None => Padding::uniform(1),
        Some(title) => {
            let left = if title.style == ToastTitleStyle::Highlight
                && title.align == ToastTitleAlign::Start
            {
                0
            } else {
                1
            };
            Padding {
                left,
                right: 1,
                top: 0,
                bottom: 1,
            }
        }
    }
}

/// Total horizontal chrome (left border + right border + left padding + right padding).
/// This varies based on title style: `Highlight + Start` toasts use `left: 0` padding,
/// reducing the chrome by 1 column.
pub fn toast_horizontal_chrome(title: Option<&ToastTitle>) -> u16 {
    let padding = toast_content_padding(title);
    2 + padding.left + padding.right
}

pub fn toast_content_rows(title: Option<&ToastTitle>, message_lines: usize) -> u16 {
    let message_lines = message_lines.max(1) as u16;
    match title.filter(|title| !title.is_empty()) {
        None => message_lines,
        Some(title) => match title.layout {
            ToastTitleLayout::Compact => message_lines + 1,
            ToastTitleLayout::Gapped => message_lines + 2,
        },
    }
}

pub fn toast_copy_text(title: Option<&ToastTitle>, message: &str) -> String {
    let Some(title) = title.filter(|title| !title.is_empty()) else {
        return message.to_string();
    };
    format!("{}\n{message}", title.text)
}

pub fn dot_separator(width: u16) -> String {
    "·".repeat(width.max(1) as usize)
}

pub fn line_separator(width: u16) -> String {
    "─".repeat(width.max(1) as usize)
}

pub fn contrasting_fg(type_color: Color) -> Color {
    let (r, g, b) = match type_color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Red => (170, 0, 0),
        Color::Green => (0, 170, 0),
        Color::Yellow => (170, 170, 0),
        Color::Blue => (0, 0, 170),
        Color::Cyan => (0, 170, 170),
        Color::Magenta => (170, 0, 170),
        _ => return Color::White,
    };

    let lin = |c: u8| {
        let c = c as f32 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };

    let luminance = 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
    if luminance > 0.18 {
        Color::Black
    } else {
        Color::White
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_title_adds_one_content_row_over_message_only() {
        let without = toast_content_rows(None, 1);
        let with_compact = toast_content_rows(Some(&ToastTitle::compact("Title")), 1);
        assert_eq!(with_compact, without + 1);
    }

    #[test]
    fn gapped_title_adds_separator_row_over_compact() {
        let compact = toast_content_rows(Some(&ToastTitle::compact("Title")), 1);
        let gapped = toast_content_rows(Some(&ToastTitle::gapped("Title")), 1);
        assert_eq!(gapped, compact + 1);
    }

    #[test]
    fn contrasting_fg_yellow_returns_black() {
        assert_eq!(contrasting_fg(Color::Yellow), Color::Black);
    }

    #[test]
    fn contrasting_fg_green_returns_black() {
        assert_eq!(contrasting_fg(Color::Green), Color::Black);
    }

    #[test]
    fn contrasting_fg_blue_returns_white() {
        assert_eq!(contrasting_fg(Color::Blue), Color::White);
    }

    #[test]
    fn contrasting_fg_red_returns_white() {
        assert_eq!(contrasting_fg(Color::Red), Color::White);
    }

    #[test]
    fn contrasting_fg_white_rgb_returns_black() {
        assert_eq!(contrasting_fg(Color::Rgb(255, 255, 255)), Color::Black);
    }

    #[test]
    fn contrasting_fg_black_rgb_returns_white() {
        assert_eq!(contrasting_fg(Color::Rgb(0, 0, 0)), Color::White);
    }
}
