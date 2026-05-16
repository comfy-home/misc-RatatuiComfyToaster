use std::borrow::Cow;

use ratatui::style::Color;

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

pub fn contrasting_fg(_type_color: Color) -> Color {
    Color::White
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
}
