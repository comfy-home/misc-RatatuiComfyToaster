//! A toast engine for displaying temporary messages in a terminal UI.
//! The `ToastEngine` manages the display of toasts, which are temporary messages that appear on the screen for a short duration. It supports different types of toasts (info, success, warning, error) and allows customization of their position and duration.
//!
//! The `ToastEngine` can be integrated into a terminal UI application using the `ratatui` crate. It provides a builder pattern for creating toasts and handles the timing for automatically hiding toasts after a specified duration.
//! # Tokio Integration
//! The `tokio` feature can be used to tightly integrate the toast engine with applications that use an event based pattern. In your
//! `Action` enum (or equivalent), add a variant that can be converted from `ToastMessage`. For example:
//! ```rust
//! enum Action {
//!     ShowToast(ToastMessage),
//!     // other variants...
//! }
//! ```
//! Then, when you want to show a toast, you can send a `ToastMessage::Show` action through your application's event system, although you do need
//! to handle the `Show` event yourself. When the toast times out, the `ToastEngine` will automatically send a `ToastMessage::Hide` action, which you should also handle to hide the toast.
//! Disable the `tokio` feature if you want to manage the timing of hiding toasts yourself, or if your application does not use an event based pattern.
//!
//! # Animating Toasts
//! The current implementation does not include animations for showing or hiding toasts. However, you can
//! use libraries like [tachyonfx](https://github.com/ratatui/tachyonfx) to add animations to your toasts. You would need to implement the animation logic in your event handling code, triggering animations when showing or hiding toasts based on the `ToastMessage` actions.
use std::{
    borrow::Cow,
    time::{Duration, Instant},
};
#[cfg(not(feature = "tokio"))]
use std::marker::PhantomData;

use ratatui::{
    layout::{Constraint, Rect, Size},
    widgets::{Clear, Widget, WidgetRef},
};
use textwrap::wrap;

use crate::widget::Toast;

const DEFAULT_MAX_TOAST_WIDTH: u16 = 50;
const TOAST_HORIZONTAL_CHROME: u16 = 4;
const TOAST_VERTICAL_CHROME: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToastPlacement {
    pub position: ToastPosition,
    pub offset: (i16, i16),
}

pub const DEFAULT_POSITION: ToastPlacement = ToastPlacement {
    position: ToastPosition::BottomRight,
    offset: (0, -1),
};

/// A toast engine for displaying temporary messages in a terminal UI.
/// The `ToastEngine` manages the display of toasts, which are temporary messages that appear on the screen for a short duration. It supports different types of toasts (info, success, warning, error) and allows customization of their position and duration.
/// You can call `show_toast` to display a toast, and `hide_toast` to hide the toast. To animate,
/// you can get the area of the toast using `toast_area` and implement your animation logic based on that area. #[derive(Debug)]
/// Caveat: If you're not using the `tokio` feature, create a `ToastEngine<()>`. There is a (hacky) impl to make it work without the `tokio` feature.
pub struct ToastEngine<A>
where
    A: From<ToastMessage> + Send + 'static,
{
    area: Rect,
    default_duration: Duration,
    #[cfg(feature = "tokio")]
    tx: Option<tokio::sync::mpsc::Sender<A>>,
    #[cfg(not(feature = "tokio"))]
    tx: Option<PhantomData<A>>,
    current_toast: Option<ActiveToast>,
}

/// A builder for creating a `ToastEngine`. It allows you to set the default duration for toasts, and an optional channel sender for sending toast messages (if using the `tokio` feature).
pub struct ToastEngineBuilder<A>
where
    A: From<ToastMessage> + Send + 'static,
{
    area: Rect,
    default_duration: Duration,
    #[cfg(feature = "tokio")]
    tx: Option<tokio::sync::mpsc::Sender<A>>,
    #[cfg(not(feature = "tokio"))]
    tx: Option<PhantomData<A>>,
}

impl<A> ToastEngineBuilder<A>
where
    A: From<ToastMessage> + Send + 'static,
{
    /// Creates a new `ToastEngineBuilder` with the specified area for displaying toasts. The default duration for toasts is set to 3 seconds, and no channel sender is configured by default.
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            default_duration: Duration::from_secs(3),
            tx: None,
        }
    }

    /// Sets the default duration for toasts. This duration will be used when showing a toast if no specific duration is provided.
    pub fn default_duration(mut self, duration: Duration) -> Self {
        self.default_duration = duration;
        self
    }

    /// Configures a channel sender for sending toast messages. This is used when the `tokio` feature is enabled to allow the `ToastEngine` to send messages to hide toasts after the duration expires.
    #[cfg(feature = "tokio")]
    pub fn action_tx(mut self, tx: tokio::sync::mpsc::Sender<A>) -> Self {
        self.tx = Some(tx);
        self
    }

    /// Builds the `ToastEngine` using the configured settings. This method consumes the builder and returns a new instance of `ToastEngine`.
    pub fn build(self) -> ToastEngine<A> {
        ToastEngine::from_builder(self)
    }
}

/// The type of toast to display. This enum defines the different types of toasts that can be shown, such as informational messages, success messages, warnings, and errors. Each variant can be styled differently when rendered.
#[derive(Debug, Default, Clone, Copy)]
pub enum ToastType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// The position on the screen where the toast should be displayed. This enum defines various positions for toasts, including top-left, top-right, bottom-left, bottom-right, and center. The `ToastEngine` uses this information to calculate the appropriate area for rendering the toast based on the specified position.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastPosition {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

/// The constraint for the toast's size. This enum defines how the size of the toast should be determined. The `Auto` variant allows the toast to automatically size itself based on the message content, while the `Uniform` and `Manual` variants allow for more specific control over the width and height of the toast.
#[derive(Debug, Default, Clone)]
pub enum ToastConstraint {
    #[default]
    Auto,
    Uniform(Constraint),
    Manual {
        width: Constraint,
        height: Constraint,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastMouseButton {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastShortcut {
    Copy,
    Dismiss,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastInteraction {
    None,
    Dismissed,
    CopyRequested(String),
}

/// The messages that can be sent to the `ToastEngine` to control the display of toasts. The `Show` variant contains the message to display, the type of toast, and its position, while the `Hide` variant indicates that any currently displayed toast should be hidden.
///
///NOTE: You do have to handle the events yourself. Usually, its as simple as matching on the `ToastMessage` in your event loop and calling the appropriate methods on the `ToastEngine` to show or hide toasts based on the received messages.
#[derive(Debug, Clone)]
pub enum ToastMessage {
    Show {
        message: String,
        toast_type: ToastType,
        position: ToastPosition,
    },
    Hide,
}

/// A builder for creating a toast message. This struct allows you to specify the message content, type, position, and size constraints for a toast before showing it using the `ToastEngine`. The builder pattern provides a convenient way to configure the properties of a toast in a fluent manner.
#[derive(Debug, Default)]
pub struct ToastBuilder {
    message: Cow<'static, str>,
    toast_type: ToastType,
    position: ToastPosition,
    constraint: ToastConstraint,
    duration: Option<Duration>,
    keep_on: u8,
    offset: (i16, i16),
}

#[derive(Debug, Clone)]
struct ActiveToast {
    toast: Toast,
    message: String,
    position: ToastPosition,
    constraint: ToastConstraint,
    offset: (i16, i16),
    keep_on: bool,
    expires_at: Option<Instant>,
    area: Rect,
}

impl<A> ToastEngine<A>
where
    A: From<ToastMessage> + Send + 'static,
{
    /// Creates a new `ToastEngine`. Consider using the `ToastEngineBuilder` instead.
    pub fn new(
        ToastEngine {
            area,
            default_duration,
            tx,
            ..
        }: Self,
    ) -> Self {
        Self {
            area,
            default_duration,
            tx,
            current_toast: None,
        }
    }

    /// Creates a new `ToastEngine` from a `ToastEngineBuilder`. This method takes the configuration from the builder and initializes the `ToastEngine` accordingly. It sets up the area for displaying toasts, the default duration for toasts, and any channel sender if provided (when using the `tokio` feature).
    pub fn from_builder(
        ToastEngineBuilder {
            area,
            default_duration,
            tx,
            ..
        }: ToastEngineBuilder<A>,
    ) -> Self {
        Self {
            area,
            default_duration,
            tx,
            current_toast: None,
        }
    }

    /// Shows a toast message using the provided `ToastBuilder`. This method calculates the area for the toast based on the message content and the specified position, creates a new `Toast` instance, and sets it as the current toast to be rendered. If the `tokio` feature is enabled and a channel sender is configured, it also spawns a task to automatically hide the toast after the default duration.
    pub fn show_toast(&mut self, toast: ToastBuilder) {
        let duration = toast.duration.unwrap_or(self.default_duration);
        let keep_on = toast.keep_on > 0;
        let area = calculate_toast_area(&toast, self.area);
        let message = toast.message.into_owned();
        self.current_toast = Some(ActiveToast {
            toast: Toast::new(&message, toast.toast_type),
            message,
            position: toast.position,
            constraint: toast.constraint,
            offset: toast.offset,
            keep_on,
            expires_at: if keep_on { None } else { Some(Instant::now() + duration) },
            area,
        });

        #[cfg(feature = "tokio")]
        if !keep_on {
            if let Some(tx) = &self.tx {
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(duration).await;
                    let _ = tx_clone.send(ToastMessage::Hide.into()).await;
                });
            }
        }
    }

    /// Get the area where the toast will be rendered.
    pub fn toast_area(&self) -> Rect {
        self.current_toast.as_ref().map(|toast| toast.area).unwrap_or_default()
    }

    /// Whether a toast is currently being displayed.
    pub fn has_toast(&self) -> bool {
        self.current_toast.is_some()
    }

    pub fn tick(&mut self) -> bool {
        let expired = self
            .current_toast
            .as_ref()
            .and_then(|toast| toast.expires_at)
            .is_some_and(|expires_at| Instant::now() >= expires_at);

        if expired {
            self.current_toast = None;
            return true;
        }

        false
    }

    pub fn dismiss(&mut self) -> bool {
        let had_toast = self.current_toast.is_some();
        self.current_toast = None;
        had_toast
    }

    pub fn current_message(&self) -> Option<&str> {
        self.current_toast.as_ref().map(|toast| toast.message.as_str())
    }

    pub fn is_keep_on(&self) -> bool {
        self.current_toast.as_ref().is_some_and(|toast| toast.keep_on)
    }

    pub fn contains(&self, column: u16, row: u16) -> bool {
        self.current_toast.as_ref().is_some_and(|toast| {
            column >= toast.area.x
                && column < toast.area.x + toast.area.width
                && row >= toast.area.y
                && row < toast.area.y + toast.area.height
        })
    }

    pub fn handle_click(
        &mut self,
        column: u16,
        row: u16,
        button: ToastMouseButton,
    ) -> ToastInteraction {
        if !self.contains(column, row) || !self.is_keep_on() {
            return ToastInteraction::None;
        }

        match button {
            ToastMouseButton::Left => {
                self.dismiss();
                ToastInteraction::Dismissed
            }
            ToastMouseButton::Right => self
                .current_message()
                .map(|message| ToastInteraction::CopyRequested(message.to_string()))
                .unwrap_or(ToastInteraction::None),
        }
    }

    pub fn handle_shortcut(&mut self, shortcut: ToastShortcut) -> ToastInteraction {
        if !self.is_keep_on() {
            return ToastInteraction::None;
        }

        match shortcut {
            ToastShortcut::Copy => self
                .current_message()
                .map(|message| ToastInteraction::CopyRequested(message.to_string()))
                .unwrap_or(ToastInteraction::None),
            ToastShortcut::Dismiss => {
                self.dismiss();
                ToastInteraction::Dismissed
            }
        }
    }

    /// Hides the currently displayed toast, if any. This method sets the current toast to `None`, which will cause it to no longer be rendered on the screen.
    pub fn hide_toast(&mut self) {
        self.dismiss();
    }

    /// Sets the area for the toast engine. This method allows you to update the area where toasts will be displayed, which can be useful if the layout of your terminal UI changes and you need to adjust the toast display area accordingly.
    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
        if let Some(toast) = &mut self.current_toast {
            toast.area = calculate_toast_area_with_layout(
                &toast.message,
                toast.position,
                &toast.constraint,
                toast.offset,
                self.area,
            );
        }
    }
}

impl ToastBuilder {
    /// Create a new instance of a `ToastBuilder`
    pub fn new(message: Cow<'static, str>) -> Self {
        Self {
            message,
            toast_type: ToastType::Info,
            position: DEFAULT_POSITION.position,
            constraint: ToastConstraint::Auto,
            duration: None,
            keep_on: 0,
            offset: DEFAULT_POSITION.offset,
        }
    }

    pub fn toast_type(mut self, toast_type: ToastType) -> Self {
        self.toast_type = toast_type;
        self
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn constraint(mut self, constraint: ToastConstraint) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn keep_on(mut self, keep_on: u8) -> Self {
        self.keep_on = keep_on;
        self
    }

    pub fn offset(mut self, x: i16, y: i16) -> Self {
        self.offset = (x, y);
        self
    }

    pub fn placement(mut self, placement: ToastPlacement) -> Self {
        self.position = placement.position;
        self.offset = placement.offset;
        self
    }
}

fn calculate_toast_area(
    ToastBuilder {
        message,
        position,
        constraint,
        offset,
        ..
    }: &ToastBuilder,
    area: Rect,
) -> Rect {
    calculate_toast_area_with_layout(message, *position, constraint, *offset, area)
}

fn calculate_toast_area_with_layout(
    message: &str,
    position: ToastPosition,
    constraint: &ToastConstraint,
    offset: (i16, i16),
    area: Rect,
) -> Rect {
    use ToastConstraint::*;
    use ToastPosition::*;
    let max_text_width = DEFAULT_MAX_TOAST_WIDTH.saturating_sub(TOAST_HORIZONTAL_CHROME).max(1);

    let text_width = match constraint {
        Auto => {
            let line_width = message.lines().map(|line| line.chars().count() as u16).max().unwrap_or(1);
            std::cmp::min(max_text_width, line_width.max(1))
        }
        Uniform(c) => area.centered_horizontally(*c).width.saturating_sub(TOAST_HORIZONTAL_CHROME).max(1),
        Manual { width, .. } => area.centered_horizontally(*width).width.saturating_sub(TOAST_HORIZONTAL_CHROME).max(1),
    };
    let width = text_width + TOAST_HORIZONTAL_CHROME;
    let wrapped_text = wrap(message, text_width as usize);
    let height = match constraint {
        Auto => wrapped_text.len() as u16 + TOAST_VERTICAL_CHROME,
        Uniform(c) => area.centered_vertically(*c).height.max(TOAST_VERTICAL_CHROME + 1),
        Manual { height, .. } => area.centered_vertically(*height).height.max(TOAST_VERTICAL_CHROME + 1),
    };

    let rect = if let Center = position {
        area.centered(width.into(), height.into())
    } else {
        position.calculate_position(area, Size { width, height })
    };

    apply_offset(rect, area, offset)
}

fn apply_offset(rect: Rect, bounds: Rect, (x_offset, y_offset): (i16, i16)) -> Rect {
    let min_x = bounds.x as i32;
    let max_x = (bounds.x + bounds.width.saturating_sub(rect.width)) as i32;
    let min_y = bounds.y as i32;
    let max_y = (bounds.y + bounds.height.saturating_sub(rect.height)) as i32;

    let x = (rect.x as i32 + x_offset as i32).clamp(min_x, max_x.max(min_x)) as u16;
    let y = (rect.y as i32 + y_offset as i32).clamp(min_y, max_y.max(min_y)) as u16;

    Rect { x, y, ..rect }
}

impl ToastPosition {
    fn calculate_position(&self, area: Rect, Size { width, height }: Size) -> Rect {
        use ToastPosition::*;
        match self {
            TopLeft => Rect {
                x: area.x,
                y: area.y,
                width,
                height,
            },
            TopRight => Rect {
                x: area.x + area.width.saturating_sub(width),
                y: area.y,
                width,
                height,
            },
            BottomLeft => Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(height),
                width,
                height,
            },
            BottomRight => Rect {
                x: area.x + area.width.saturating_sub(width),
                y: area.y + area.height.saturating_sub(height),
                width,
                height,
            },
            Center => Rect {
                x: area.x + (area.width.saturating_sub(width)) / 2,
                y: area.y + (area.height.saturating_sub(height)) / 2,
                width,
                height,
            },
        }
    }
}

impl From<ToastType> for ratatui::style::Color {
    fn from(value: ToastType) -> Self {
        use ToastType::*;
        match value {
            Info => Self::Blue,
            Success => Self::Green,
            Warning => Self::Yellow,
            Error => Self::Red,
        }
    }
}

impl<A> WidgetRef for ToastEngine<A>
where
    A: From<ToastMessage> + Send + 'static,
{
    fn render_ref(&self, _area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if let Some(toast) = &self.current_toast {
            Clear.render(toast.area, buf);
            toast.toast.render_ref(toast.area, buf);
        }
    }
}

impl<A> Widget for &ToastEngine<A>
where
    A: From<ToastMessage> + Send + 'static,
{
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        self.render_ref(area, buf);
    }
}

impl From<ToastMessage> for () {
    fn from(_value: ToastMessage) -> Self {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_on_toast_does_not_expire() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(ToastBuilder::new("sticky".into()).keep_on(1));

        assert!(engine.has_toast());
        assert!(engine.is_keep_on());
        assert!(!engine.tick());
        assert!(engine.has_toast());
    }

    #[test]
    fn sticky_shortcuts_are_exposed() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(ToastBuilder::new("copy me".into()).keep_on(1));

        let interaction = engine.handle_shortcut(ToastShortcut::Copy);
        assert_eq!(interaction, ToastInteraction::CopyRequested("copy me".to_string()));

        let interaction = engine.handle_shortcut(ToastShortcut::Dismiss);
        assert_eq!(interaction, ToastInteraction::Dismissed);
        assert!(!engine.has_toast());
    }
}
