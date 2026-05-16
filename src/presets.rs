use std::borrow::Cow;

use crate::title::{ToastTitle, ToastTitleAlign, ToastTitleSeparator, ToastTitleStyle};

/// Named title-layout presets for quick toast styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPreset {
    /// Message body only (no title row).
    #[default]
    MessageOnly,
    /// Compact title, plain style, start-aligned.
    CompactPlainStart,
    /// Compact title with type-colored highlight band, start-aligned.
    CompactHighlightStart,
    /// Compact title, plain style, centered.
    CompactPlainCenter,
    /// Compact title with highlight band, centered.
    CompactHighlightCenter,
    /// Gapped title with dot separator, plain style, start-aligned.
    GappedDotStart,
    /// Gapped title with line separator, start-aligned.
    GappedLineStart,
    /// Gapped title with blank separator row, start-aligned.
    GappedEmptyStart,
    /// Gapped title with dot separator, highlight band, centered.
    GappedDotHighlightCenter,
}

impl ToastPreset {
    /// Build a [`ToastTitle`] for the given preset and title text.
    pub fn title(self, text: impl Into<Cow<'static, str>>) -> ToastTitle {
        let text = text.into().into_owned();
        match self {
            Self::MessageOnly => ToastTitle::compact(text),
            Self::CompactPlainStart => ToastTitle::compact(text),
            Self::CompactHighlightStart => {
                let mut title = ToastTitle::compact(text);
                title.style = ToastTitleStyle::Highlight;
                title
            }
            Self::CompactPlainCenter => {
                let mut title = ToastTitle::compact(text);
                title.align = ToastTitleAlign::Center;
                title
            }
            Self::CompactHighlightCenter => {
                let mut title = ToastTitle::compact(text);
                title.style = ToastTitleStyle::Highlight;
                title.align = ToastTitleAlign::Center;
                title
            }
            Self::GappedDotStart => ToastTitle::gapped(text),
            Self::GappedLineStart => {
                let mut title = ToastTitle::gapped(text);
                title.separator = ToastTitleSeparator::Line;
                title
            }
            Self::GappedEmptyStart => {
                let mut title = ToastTitle::gapped(text);
                title.separator = ToastTitleSeparator::Empty;
                title
            }
            Self::GappedDotHighlightCenter => {
                let mut title = ToastTitle::gapped(text);
                title.style = ToastTitleStyle::Highlight;
                title.align = ToastTitleAlign::Center;
                title
            }
        }
    }

    pub fn uses_title(self) -> bool {
        !matches!(self, Self::MessageOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToastTitleLayout;

    #[test]
    fn gapped_dot_highlight_center_configures_title_fields() {
        let title = ToastPreset::GappedDotHighlightCenter.title("Bump");
        assert_eq!(title.layout, ToastTitleLayout::Gapped);
        assert_eq!(title.separator, ToastTitleSeparator::Dot);
        assert_eq!(title.style, ToastTitleStyle::Highlight);
        assert_eq!(title.align, ToastTitleAlign::Center);
    }
}
