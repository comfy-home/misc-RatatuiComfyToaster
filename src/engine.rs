//! A toast engine for displaying temporary messages in a terminal UI.
//! The `ToastEngine` manages the display of toasts, which are temporary messages that appear on the screen for a short duration. It supports different types of toasts (info, success, warning, error) and allows customization of their position and duration.
//!
//! The `ToastEngine` can be integrated into a terminal UI application using the `ratatui` crate. It provides a builder pattern for creating toasts and handles the timing for automatically hiding toasts after a specified duration.
//! # Tokio Integration
//! The `tokio` feature can be used to tightly integrate the toast engine with applications that use an event based pattern. In your
//! `Action` enum (or equivalent), add a variant that can be converted from `ToastMessage`. For example:
//! ```rust
//! use ratatui_comfy_toaster::ToastMessage;
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
#[cfg(not(feature = "tokio"))]
use std::marker::PhantomData;
use std::{
    borrow::Cow,
    collections::VecDeque,
    time::{Duration, Instant},
};

use ratatui::{
    layout::{Constraint, Rect, Size},
    style::Color,
    widgets::{Clear, Widget, WidgetRef},
};
use textwrap::wrap;

use crate::presets::ToastPreset;
use crate::title::{
    ToastTitle, ToastTitleAlign, ToastTitleSeparator, ToastTitleStyle, toast_content_rows,
    toast_copy_text, toast_horizontal_chrome, toast_vertical_padding_rows,
};
use crate::widget::Toast;

const DEFAULT_MAX_TOAST_WIDTH: u16 = 50;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastBorderMode {
    #[default]
    SideRails,
    Full,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastProgressBarStyle {
    #[default]
    FullBlock,
    HalfBlock,
    Minimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToastPlacement {
    pub position: ToastPosition,
    pub offset: (i16, i16),
}

pub const DEFAULT_POSITION: ToastPlacement = ToastPlacement {
    position: ToastPosition::BottomRight,
    offset: (0, -1),
};

pub const DEFAULT_BG: Color = Color::DarkGray;

/// A toast engine for displaying temporary messages in a terminal UI.
/// The `ToastEngine` manages the display of toasts, which are temporary messages that appear on the screen for a short duration. It supports different types of toasts (info, success, warning, error) and allows customization of their position and duration.
/// You can call `show_toast` to display a toast, and `hide_toast` to hide the toast. To animate,
/// you can get the area of the toast using `toast_area` and implement your animation logic based on that area. #[derive(Debug)]
/// Caveat: If you're not using the `tokio` feature, create a `ToastEngine<()>`. There is a (hacky) impl to make it work without the `tokio` feature.
///
/// # Thread safety
/// `ToastEngine` is `Send` but **not** `Sync`. The type parameter `A` is bounded by `Send + 'static`,
/// but the engine itself holds a `VecDeque` that is not synchronized. To share a `ToastEngine` across
/// threads, wrap it in a `std::sync::Mutex` or `tokio::sync::Mutex`.
pub struct ToastEngine<A>
where
    A: From<ToastMessage> + Send + 'static,
{
    area: Rect,
    default_duration: Duration,
    default_border_mode: ToastBorderMode,
    default_progress_bar: bool,
    default_progress_bar_style: ToastProgressBarStyle,
    max_queue_depth: usize,
    dedup_enabled: bool,
    next_id: u64,
    #[cfg(feature = "tokio")]
    tx: Option<tokio::sync::mpsc::Sender<A>>,
    #[cfg(not(feature = "tokio"))]
    tx: Option<PhantomData<A>>,
    queue: VecDeque<ActiveToast>,
}

/// A builder for creating a `ToastEngine`. It allows you to set the default duration for toasts, and an optional channel sender for sending toast messages (if using the `tokio` feature).
pub struct ToastEngineBuilder<A>
where
    A: From<ToastMessage> + Send + 'static,
{
    area: Rect,
    default_duration: Duration,
    default_border_mode: ToastBorderMode,
    default_progress_bar: bool,
    default_progress_bar_style: ToastProgressBarStyle,
    max_queue_depth: usize,
    dedup_enabled: bool,
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
            default_border_mode: ToastBorderMode::SideRails,
            default_progress_bar: false,
            default_progress_bar_style: ToastProgressBarStyle::FullBlock,
            max_queue_depth: 4,
            dedup_enabled: true,
            tx: None,
        }
    }

    /// Sets the maximum number of toasts that can be queued at once.
    /// When the queue is full, incoming timed toasts are dropped. Sticky toasts
    /// displace the oldest timed toast if needed to ensure they are never lost.
    /// Defaults to 4.
    pub fn max_queue_depth(mut self, depth: usize) -> Self {
        self.max_queue_depth = depth.max(1);
        self
    }

    /// Enables or disables toast deduplication. When enabled (the default), incoming toasts with the
    /// same `message` + `toast_type` + `title` as an existing queued toast are not duplicated:
    /// - **Timed duplicates**: the existing toast's expiry timer is refreshed (update-in-place).
    /// - **Sticky duplicates**: the new toast is silently skipped.
    /// Pass `false` to disable deduplication and allow duplicate toasts.
    pub fn dedup(mut self, enabled: bool) -> Self {
        self.dedup_enabled = enabled;
        self
    }

    /// Sets the default duration for toasts. This duration will be used when showing a toast if no specific duration is provided.
    pub fn default_duration(mut self, duration: Duration) -> Self {
        self.default_duration = duration;
        self
    }

    /// Sets the default border mode for newly shown toasts.
    pub fn default_border_mode(mut self, border_mode: ToastBorderMode) -> Self {
        self.default_border_mode = border_mode;
        self
    }

    /// Enables or disables the bottom progress bar for timed toasts by default.
    pub fn default_progress_bar(mut self, show_progress_bar: bool) -> Self {
        self.default_progress_bar = show_progress_bar;
        self
    }

    /// Sets the default visual style for timed toast progress bars.
    pub fn default_progress_bar_style(mut self, progress_bar_style: ToastProgressBarStyle) -> Self {
        self.default_progress_bar_style = progress_bar_style;
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

/// The result of a user interaction with a toast.
///
/// # Security note
/// The `CopyRequested` variant contains an **untrusted** string derived from user-provided toast
/// messages. It is not sanitized or escaped. If you pipe this content into a shell, terminal, or
/// any interpreter, escape sequences embedded in the original message could be executed. Always
/// treat the payload as untrusted input when forwarding it to external systems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastInteraction {
    None,
    Dismissed,
    CopyRequested(String),
}

/// The messages that can be sent to the `ToastEngine` to control the display of toasts.
///
/// - `Show` carries the basic fields (`message`, `toast_type`, `position`) for backward compatibility.
/// - `ShowBuilder` carries a fully-configured `ToastBuilder` so that all builder fields (title, duration,
///   keep_on, progress bar, border mode, etc.) are preserved through the event loop.
/// - `Hide` indicates that a specific toast should be hidden by its unique ID.
///
///NOTE: You do have to handle the events yourself. Usually, its as simple as matching on the `ToastMessage` in your event loop and calling the appropriate methods on the `ToastEngine` to show or hide toasts based on the received messages.
#[derive(Debug, Clone)]
pub enum ToastMessage {
    Show {
        message: String,
        toast_type: ToastType,
        position: ToastPosition,
    },
    ShowBuilder(ToastBuilder),
    Hide {
        id: u64,
    },
}

/// A builder for creating a toast message. This struct allows you to specify the message content, type, position, and size constraints for a toast before showing it using the `ToastEngine`. The builder pattern provides a convenient way to configure the properties of a toast in a fluent manner.
#[derive(Debug, Default, Clone)]
pub struct ToastBuilder {
    title: Option<ToastTitle>,
    message: Cow<'static, str>,
    toast_type: ToastType,
    toast_bg: Color,
    position: ToastPosition,
    constraint: ToastConstraint,
    duration: Option<Duration>,
    keep_on: u8,
    offset: (i16, i16),
    border_mode: Option<ToastBorderMode>,
    show_progress_bar: Option<bool>,
    progress_bar_style: Option<ToastProgressBarStyle>,
}

#[derive(Debug, Clone)]
struct ActiveToast {
    id: u64,
    toast: Toast,
    title: Option<ToastTitle>,
    message: String,
    copy_text: String,
    position: ToastPosition,
    constraint: ToastConstraint,
    offset: (i16, i16),
    border_mode: ToastBorderMode,
    keep_on: bool,
    duration: Option<Duration>,
    show_progress_bar: bool,
    progress_bar_style: ToastProgressBarStyle,
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
            default_border_mode,
            default_progress_bar,
            default_progress_bar_style,
            max_queue_depth,
            dedup_enabled,
            next_id,
            tx,
            ..
        }: Self,
    ) -> Self {
        Self {
            area,
            default_duration,
            default_border_mode,
            default_progress_bar,
            default_progress_bar_style,
            max_queue_depth,
            dedup_enabled,
            next_id,
            tx,
            queue: VecDeque::new(),
        }
    }

    /// Creates a new `ToastEngine` from a `ToastEngineBuilder`. This method takes the configuration from the builder and initializes the `ToastEngine` accordingly. It sets up the area for displaying toasts, the default duration for toasts, and any channel sender if provided (when using the `tokio` feature).
    pub fn from_builder(
        ToastEngineBuilder {
            area,
            default_duration,
            default_border_mode,
            default_progress_bar,
            default_progress_bar_style,
            max_queue_depth,
            dedup_enabled,
            tx,
            ..
        }: ToastEngineBuilder<A>,
    ) -> Self {
        Self {
            area,
            default_duration,
            default_border_mode,
            default_progress_bar,
            default_progress_bar_style,
            max_queue_depth,
            dedup_enabled,
            next_id: 0,
            tx,
            queue: VecDeque::new(),
        }
    }

    /// Enqueues a toast. The front of the queue is always the currently displayed toast.
    ///
    /// Queueing rules:
    /// - If dedup is enabled and a toast with the same `message` + `toast_type` + `title` already
    ///   exists in the queue, the new toast is not enqueued. For timed duplicates, the existing
    ///   toast's expiry timer is refreshed instead.
    /// - If the queue is at `max_queue_depth` and the incoming toast is **sticky**, the oldest
    ///   timed toast is displaced to make room. If no timed toast exists, the oldest sticky toast
    ///   is displaced instead.
    /// - If the queue is at `max_queue_depth` and the incoming toast is **timed**, it is displayed
    ///   as a temporary +1 beyond `max_queue_depth` and auto-expires normally. Sticky toasts are
    ///   never displaced by timed toasts.
    pub fn show_toast(&mut self, toast: ToastBuilder) {
        let duration = toast.duration.unwrap_or(self.default_duration);
        let keep_on = toast.keep_on > 0;

        if self.dedup_enabled {
            let incoming_title = toast.title.as_ref().filter(|t| !t.is_empty()).map(|t| t.text.as_str());
            let incoming_type = toast.toast_type;
            let incoming_msg = toast.message.as_ref();

            for existing in self.queue.iter_mut() {
                let title_matches = existing.title.as_ref().map(|t| t.text.as_str()) == incoming_title;
                if title_matches
                    && existing.toast.type_ == incoming_type
                    && existing.message == incoming_msg
                {
                    if !existing.keep_on {
                        existing.expires_at = Some(Instant::now() + duration);
                    }
                    return;
                }
            }
        }

        let border_mode = toast.border_mode.unwrap_or(self.default_border_mode);
        let show_progress_bar =
            toast.show_progress_bar.unwrap_or(self.default_progress_bar) && !keep_on;
        let progress_bar_style = toast
            .progress_bar_style
            .unwrap_or(self.default_progress_bar_style);

        if self.queue.len() >= self.max_queue_depth && keep_on {
            if let Some(pos) = self.queue.iter().rposition(|t| !t.keep_on) {
                self.queue.remove(pos);
            } else {
                self.queue.remove(0);
            }
            // Timed toasts are allowed as +1 beyond max_queue_depth; no removal needed.
        }

        let area = calculate_toast_area(&toast, self.area, border_mode, show_progress_bar);
        let title = toast.title.clone().filter(|title| !title.is_empty());
        let message = toast.message.into_owned();
        let copy_text = toast_copy_text(title.as_ref(), &message);
        let id = self.next_id;
        self.next_id += 1;
        self.queue.push_back(ActiveToast {
            id,
            toast: Toast::new(
                &message,
                toast.toast_type,
                toast.toast_bg,
                border_mode,
                progress_bar_style,
            )
            .with_title(title.clone()),
            title,
            message,
            copy_text,
            position: toast.position,
            constraint: toast.constraint,
            offset: toast.offset,
            border_mode,
            keep_on,
            duration: if keep_on { None } else { Some(duration) },
            show_progress_bar,
            progress_bar_style,
            expires_at: if keep_on {
                None
            } else {
                Some(Instant::now() + duration)
            },
            area,
        });

        #[cfg(feature = "tokio")]
        if !keep_on {
            if let Some(tx) = &self.tx {
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(duration).await;
                    let _ = tx_clone.send(ToastMessage::Hide { id }.into()).await;
                });
            }
        }
    }

    /// Get the area where the toast will be rendered.
    pub fn toast_area(&self) -> Rect {
        self.queue
            .front()
            .map(|toast| toast.area)
            .unwrap_or_default()
    }

    /// Whether a toast is currently being displayed.
    pub fn has_toast(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Returns the number of toasts currently queued.
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Removes all expired timed toasts from the queue. Returns `true` if any
    /// were removed.
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        let before = self.queue.len();
        self.queue
            .retain(|toast| toast.expires_at.is_none_or(|exp| now < exp));
        self.queue.len() < before
    }

    /// Dismisses the front (currently displayed) toast and advances the queue.
    pub fn dismiss(&mut self) -> bool {
        self.dismiss_at(0)
    }

    /// Dismisses the toast at `index` in the queue. Returns `false` if the index is out of bounds.
    pub fn dismiss_at(&mut self, index: usize) -> bool {
        if index >= self.queue.len() {
            return false;
        }
        self.queue.remove(index);
        true
    }

    pub fn current_message(&self) -> Option<&str> {
        self.queue.front().map(|toast| toast.copy_text.as_str())
    }

    pub fn is_keep_on(&self) -> bool {
        self.queue.front().is_some_and(|toast| toast.keep_on)
    }

    pub fn contains(&self, column: u16, row: u16) -> bool {
        self.toast_index_at(column, row).is_some()
    }

    /// Returns the queue index of the topmost toast at the given coordinates.
    pub fn toast_index_at(&self, column: u16, row: u16) -> Option<usize> {
        self.queue
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, toast)| rect_contains(toast.area, column, row).then_some(index))
    }

    pub fn handle_click(
        &mut self,
        column: u16,
        row: u16,
        button: ToastMouseButton,
    ) -> ToastInteraction {
        let Some(index) = self.toast_index_at(column, row) else {
            return ToastInteraction::None;
        };

        match button {
            ToastMouseButton::Left => {
                if !self.queue[index].keep_on {
                    return ToastInteraction::None;
                }
                self.dismiss_at(index);
                ToastInteraction::Dismissed
            }
            ToastMouseButton::Right => {
                ToastInteraction::CopyRequested(self.queue[index].copy_text.clone())
            }
        }
    }

    pub fn handle_shortcut(&mut self, shortcut: ToastShortcut) -> ToastInteraction {
        match shortcut {
            ToastShortcut::Copy => self
                .current_message()
                .map(|message| ToastInteraction::CopyRequested(message.to_string()))
                .unwrap_or(ToastInteraction::None),
            ToastShortcut::Dismiss => {
                if !self.is_keep_on() {
                    return ToastInteraction::None;
                }
                self.dismiss();
                ToastInteraction::Dismissed
            }
        }
    }

    /// Hides the currently displayed toast, if any. This method sets the current toast to `None`, which will cause it to no longer be rendered on the screen.
    pub fn hide_toast(&mut self) {
        self.dismiss();
    }

    /// Hides the toast with the given ID, if it is still in the queue. This is used by the tokio
    /// integration to ensure that a `ToastMessage::Hide` message targets the specific toast that
    /// timed out, rather than blindly dismissing the front of the queue (which may be a different
    /// toast by the time the message is processed).
    pub fn hide_toast_by_id(&mut self, id: u64) -> bool {
        if let Some(pos) = self.queue.iter().position(|toast| toast.id == id) {
            self.queue.remove(pos);
            true
        } else {
            false
        }
    }

    /// Enables or disables toast deduplication at runtime. Dedup is enabled by default. When enabled,
    /// incoming toasts with the same `message` + `toast_type` + `title` as an existing queued toast
    /// are not duplicated: timed duplicates refresh the existing toast's expiry timer, sticky
    /// duplicates are skipped. Pass `false` to allow duplicates.
    pub fn set_dedup(&mut self, enabled: bool) {
        self.dedup_enabled = enabled;
    }

    /// Sets the area for the toast engine. This method allows you to update the area where toasts will be displayed, which can be useful if the layout of your terminal UI changes and you need to adjust the toast display area accordingly.
    pub fn set_area(&mut self, area: Rect) {
        self.set_area_avoiding(area, &[]);
    }

    /// Sets the area for the toast engine while avoiding overlap with already-occupied regions.
    pub fn set_area_avoiding(&mut self, area: Rect, occupied: &[Rect]) {
        self.area = area;
        let mut stacked: Vec<Rect> = occupied.to_vec();
        for toast in self.queue.iter_mut() {
            let desired_area = calculate_toast_area_with_layout(ToastLayoutParams {
                title: toast.title.as_ref(),
                message: &toast.message,
                position: toast.position,
                constraint: &toast.constraint,
                offset: toast.offset,
                area: self.area,
                border_mode: toast.border_mode,
                show_progress_bar: toast.show_progress_bar,
            });
            toast.area = avoid_occupied_areas(desired_area, self.area, &stacked, toast.position);
            stacked.push(toast.area);
        }
    }
}

impl ActiveToast {
    fn progress_ratio(&self) -> Option<f64> {
        if !self.show_progress_bar {
            return None;
        }

        let duration = self.duration?;
        if duration.is_zero() {
            return Some(0.0);
        }

        let expires_at = self.expires_at?;
        let remaining = expires_at
            .checked_duration_since(Instant::now())
            .unwrap_or_default();
        Some((remaining.as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0))
    }
}

fn avoid_occupied_areas(
    mut area: Rect,
    bounds: Rect,
    occupied: &[Rect],
    position: ToastPosition,
) -> Rect {
    let mut blockers = occupied
        .iter()
        .copied()
        .filter(|blocker| {
            blocker.width > 0 && blocker.height > 0 && horizontal_overlap(area, *blocker)
        })
        .collect::<Vec<_>>();

    match position {
        ToastPosition::BottomLeft | ToastPosition::BottomRight => {
            blockers.sort_by_key(|blocker| std::cmp::Reverse(blocker.y))
        }
        ToastPosition::TopLeft | ToastPosition::TopRight => {
            blockers.sort_by_key(|blocker| blocker.y)
        }
        ToastPosition::Center => return area,
    }

    for blocker in blockers {
        if !rects_overlap(area, blocker) {
            continue;
        }

        area.y = match position {
            ToastPosition::BottomLeft | ToastPosition::BottomRight => {
                blocker.y.saturating_sub(area.height.saturating_add(1))
            }
            ToastPosition::TopLeft | ToastPosition::TopRight => {
                blocker.y.saturating_add(blocker.height).saturating_add(1)
            }
            ToastPosition::Center => area.y,
        };
        area = apply_offset(area, bounds, (0, 0));
    }

    area
}

fn rect_contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn horizontal_overlap(left: Rect, right: Rect) -> bool {
    left.x < right.x.saturating_add(right.width) && right.x < left.x.saturating_add(left.width)
}

fn rects_overlap(left: Rect, right: Rect) -> bool {
    horizontal_overlap(left, right)
        && left.y < right.y.saturating_add(right.height)
        && right.y < left.y.saturating_add(left.height)
}

impl ToastBuilder {
    /// Create a new instance of a `ToastBuilder`
    pub fn new(message: Cow<'static, str>) -> Self {
        Self {
            title: None,
            message,
            toast_type: ToastType::Info,
            toast_bg: DEFAULT_BG,
            position: DEFAULT_POSITION.position,
            constraint: ToastConstraint::Auto,
            duration: None,
            keep_on: 0,
            offset: DEFAULT_POSITION.offset,
            border_mode: None,
            show_progress_bar: None,
            progress_bar_style: None,
        }
    }

    /// Compact title on the first content row (no extra separator row).
    pub fn title(mut self, title: impl Into<Cow<'static, str>>) -> Self {
        let title = ToastTitle::compact(title);
        self.title = (!title.is_empty()).then_some(title);
        self
    }

    /// Title with a separator row before the message.
    pub fn title_gapped(mut self, title: impl Into<Cow<'static, str>>) -> Self {
        let title = ToastTitle::gapped(title);
        self.title = (!title.is_empty()).then_some(title);
        self
    }

    pub fn title_separator(mut self, separator: ToastTitleSeparator) -> Self {
        if let Some(title) = &mut self.title {
            title.separator = separator;
        }
        self
    }

    pub fn title_align(mut self, align: ToastTitleAlign) -> Self {
        if let Some(title) = &mut self.title {
            title.align = align;
        }
        self
    }

    pub fn title_highlight(mut self) -> Self {
        if let Some(title) = &mut self.title {
            title.style = ToastTitleStyle::Highlight;
        }
        self
    }

    /// Apply a named title layout preset (see [`ToastPreset`]).
    pub fn preset(mut self, preset: ToastPreset, title: impl Into<Cow<'static, str>>) -> Self {
        self.title = if preset.uses_title() {
            let title = preset.title(title);
            (!title.is_empty()).then_some(title)
        } else {
            None
        };
        self
    }

    pub fn toast_type(mut self, toast_type: ToastType) -> Self {
        self.toast_type = toast_type;
        self
    }

    pub fn toast_bg(mut self, toast_bg: Color) -> Self {
        self.toast_bg = toast_bg;
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

    pub fn border_mode(mut self, border_mode: ToastBorderMode) -> Self {
        self.border_mode = Some(border_mode);
        self
    }

    pub fn show_progress_bar(mut self, show_progress_bar: bool) -> Self {
        self.show_progress_bar = Some(show_progress_bar);
        self
    }

    pub fn progress_bar_style(mut self, progress_bar_style: ToastProgressBarStyle) -> Self {
        self.progress_bar_style = Some(progress_bar_style);
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
        title,
        message,
        position,
        constraint,
        offset,
        ..
    }: &ToastBuilder,
    area: Rect,
    border_mode: ToastBorderMode,
    show_progress_bar: bool,
) -> Rect {
    calculate_toast_area_with_layout(ToastLayoutParams {
        title: title.as_ref(),
        message,
        position: *position,
        constraint,
        offset: *offset,
        area,
        border_mode,
        show_progress_bar,
    })
}

struct ToastLayoutParams<'a> {
    title: Option<&'a ToastTitle>,
    message: &'a str,
    position: ToastPosition,
    constraint: &'a ToastConstraint,
    offset: (i16, i16),
    area: Rect,
    border_mode: ToastBorderMode,
    show_progress_bar: bool,
}

fn calculate_toast_area_with_layout(
    ToastLayoutParams {
        title,
        message,
        position,
        constraint,
        offset,
        area,
        border_mode,
        show_progress_bar,
    }: ToastLayoutParams<'_>,
) -> Rect {
    use ToastConstraint::*;
    use ToastPosition::*;
    let toast_vertical_chrome = toast_vertical_chrome(title, border_mode, show_progress_bar);
    let h_chrome = toast_horizontal_chrome(title);
    let max_text_width = DEFAULT_MAX_TOAST_WIDTH.saturating_sub(h_chrome).max(1);

    let text_width = match constraint {
        Auto => {
            let line_width = title
                .iter()
                .map(|title| title.text.as_str())
                .chain(std::iter::once(message))
                .flat_map(str::lines)
                .map(|line| unicode_width::UnicodeWidthStr::width(line) as u16)
                .max()
                .unwrap_or(1);
            std::cmp::min(max_text_width, line_width.max(1))
        }
        Uniform(c) => area
            .centered_horizontally(*c)
            .width
            .saturating_sub(h_chrome)
            .max(1),
        Manual { width, .. } => area
            .centered_horizontally(*width)
            .width
            .saturating_sub(h_chrome)
            .max(1),
    };
    let width = text_width + h_chrome;
    let message_lines = wrap(message, text_width as usize).len().max(1);
    let content_rows = toast_content_rows(title.filter(|title| !title.is_empty()), message_lines);
    let height = match constraint {
        Auto => content_rows + toast_vertical_chrome,
        Uniform(c) => area
            .centered_vertically(*c)
            .height
            .max(toast_vertical_chrome + 1),
        Manual { height, .. } => area
            .centered_vertically(*height)
            .height
            .max(toast_vertical_chrome + 1),
    };

    let rect = if let Center = position {
        area.centered(width.into(), height.into())
    } else {
        position.calculate_position(area, Size { width, height })
    };

    apply_offset(rect, area, offset)
}

fn toast_vertical_chrome(
    title: Option<&ToastTitle>,
    border_mode: ToastBorderMode,
    show_progress_bar: bool,
) -> u16 {
    let border_rows = match border_mode {
        ToastBorderMode::SideRails => 0,
        ToastBorderMode::Full => 2,
    };
    let progress_rows = u16::from(show_progress_bar);
    toast_vertical_padding_rows(title) + border_rows + progress_rows
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
        for toast in self.queue.iter() {
            Clear.render(toast.area, buf);
            toast
                .toast
                .clone()
                .with_progress_ratio(toast.progress_ratio())
                .with_progress_bar_style(toast.progress_bar_style)
                .render_ref(toast.area, buf);
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
        assert_eq!(
            interaction,
            ToastInteraction::CopyRequested("copy me".to_string())
        );

        let interaction = engine.handle_shortcut(ToastShortcut::Dismiss);
        assert_eq!(interaction, ToastInteraction::Dismissed);
        assert!(!engine.has_toast());
    }

    #[test]
    fn default_background_is_dark_gray() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(ToastBuilder::new("bg".into()));

        assert_eq!(
            engine.queue.front().map(|toast| toast.toast.bg),
            Some(DEFAULT_BG)
        );
    }

    #[test]
    fn builder_background_overrides_default() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(ToastBuilder::new("bg".into()).toast_bg(Color::Blue));

        assert_eq!(
            engine.queue.front().map(|toast| toast.toast.bg),
            Some(Color::Blue)
        );
    }

    #[test]
    fn avoiding_bottom_right_overlap_moves_toast_up() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(ToastBuilder::new("sticky".into()).position(ToastPosition::BottomRight));

        let blocker = Rect::new(60, 20, 20, 4);
        engine.set_area_avoiding(Rect::new(0, 0, 80, 25), &[blocker]);

        let area = engine.toast_area();
        assert!(area.y + area.height < blocker.y + blocker.height);
        assert!(area.y < blocker.y);
    }

    #[test]
    fn avoiding_top_left_overlap_moves_toast_down() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(ToastBuilder::new("sticky".into()).position(ToastPosition::TopLeft));

        let blocker = Rect::new(0, 0, 20, 4);
        engine.set_area_avoiding(Rect::new(0, 0, 80, 25), &[blocker]);

        let area = engine.toast_area();
        assert!(area.y > blocker.y);
        assert!(area.y >= blocker.y + blocker.height);
    }

    #[test]
    fn timed_toasts_queue_and_drain_in_order() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .max_queue_depth(3)
            .build();
        engine.show_toast(ToastBuilder::new("first".into()));
        engine.show_toast(ToastBuilder::new("second".into()));
        engine.show_toast(ToastBuilder::new("third".into()));

        assert_eq!(engine.queue_len(), 3);
        assert_eq!(engine.current_message(), Some("first"));

        engine.dismiss();
        assert_eq!(engine.current_message(), Some("second"));

        engine.dismiss();
        assert_eq!(engine.current_message(), Some("third"));

        engine.dismiss();
        assert!(!engine.has_toast());
    }

    #[test]
    fn timed_toast_displayed_as_overflow_when_queue_full() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .max_queue_depth(2)
            .build();
        engine.show_toast(ToastBuilder::new("a".into()));
        engine.show_toast(ToastBuilder::new("b".into()));
        engine.show_toast(ToastBuilder::new("overflow".into()));

        assert_eq!(engine.queue_len(), 3);
        assert_eq!(engine.current_message(), Some("a"));
    }

    #[test]
    fn sticky_displaces_oldest_timed_when_queue_full() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .max_queue_depth(2)
            .build();
        engine.show_toast(ToastBuilder::new("timed-a".into()));
        engine.show_toast(ToastBuilder::new("timed-b".into()));
        engine.show_toast(ToastBuilder::new("important".into()).keep_on(1));

        assert_eq!(engine.queue_len(), 2);
        let messages: Vec<_> = engine.queue.iter().map(|t| t.message.as_str()).collect();
        assert!(messages.contains(&"important"), "sticky must be in queue");
    }

    #[test]
    fn sticky_displaces_oldest_sticky_when_queue_full() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .max_queue_depth(2)
            .build();
        engine.show_toast(ToastBuilder::new("sticky-1".into()).keep_on(1));
        engine.show_toast(ToastBuilder::new("sticky-2".into()).keep_on(1));
        engine.show_toast(ToastBuilder::new("sticky-3".into()).keep_on(1));

        assert_eq!(engine.queue_len(), 2);
        assert_eq!(engine.current_message(), Some("sticky-2"));
        let messages: Vec<_> = engine.queue.iter().map(|t| t.message.as_str()).collect();
        assert!(
            messages.contains(&"sticky-3"),
            "new sticky must be in queue"
        );
        assert!(
            !messages.contains(&"sticky-1"),
            "oldest sticky must be displaced"
        );
    }

    #[test]
    fn sticky_blocks_queue_advancement() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(ToastBuilder::new("sticky-front".into()).keep_on(1));
        engine.show_toast(ToastBuilder::new("next".into()));

        assert!(!engine.tick(), "sticky should not be ticked away");
        assert_eq!(engine.current_message(), Some("sticky-front"));

        engine.dismiss();
        assert_eq!(engine.current_message(), Some("next"));
    }

    #[test]
    fn progress_bar_increases_auto_height_for_timed_toasts() {
        let layout = |show_progress_bar| ToastLayoutParams {
            title: None,
            message: "hello",
            position: ToastPosition::BottomRight,
            constraint: &ToastConstraint::Auto,
            offset: (0, 0),
            area: Rect::new(0, 0, 80, 25),
            border_mode: ToastBorderMode::SideRails,
            show_progress_bar,
        };
        let normal = calculate_toast_area_with_layout(layout(false));
        let with_progress = calculate_toast_area_with_layout(layout(true));

        assert_eq!(with_progress.height, normal.height + 1);
    }

    #[test]
    fn full_border_increases_auto_height() {
        let layout = |border_mode| ToastLayoutParams {
            title: None,
            message: "hello",
            position: ToastPosition::BottomRight,
            constraint: &ToastConstraint::Auto,
            offset: (0, 0),
            area: Rect::new(0, 0, 80, 25),
            border_mode,
            show_progress_bar: false,
        };
        let side_rails = calculate_toast_area_with_layout(layout(ToastBorderMode::SideRails));
        let full = calculate_toast_area_with_layout(layout(ToastBorderMode::Full));

        assert_eq!(full.height, side_rails.height + 2);
    }

    #[test]
    fn sticky_toasts_ignore_progress_bar_requests() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .default_progress_bar(true)
            .build();
        engine.show_toast(ToastBuilder::new("sticky".into()).keep_on(1));

        let toast = engine.queue.front().expect("toast queued");
        assert!(!toast.show_progress_bar);
        assert_eq!(toast.progress_ratio(), None);
    }

    #[test]
    fn title_is_included_in_copy_text() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(
            ToastBuilder::new("target path cannot be empty".into())
                .title("New Scope:")
                .keep_on(1),
        );

        assert_eq!(
            engine.current_message(),
            Some("New Scope:\ntarget path cannot be empty")
        );
        assert_eq!(
            engine.handle_shortcut(ToastShortcut::Copy),
            ToastInteraction::CopyRequested("New Scope:\ntarget path cannot be empty".to_string())
        );
    }

    #[test]
    fn click_dismisses_specific_sticky_toast_not_only_front() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .max_queue_depth(3)
            .build();
        engine.show_toast(
            ToastBuilder::new("sticky-1".into())
                .keep_on(1)
                .position(ToastPosition::BottomRight),
        );
        engine.show_toast(
            ToastBuilder::new("sticky-2".into())
                .keep_on(1)
                .position(ToastPosition::BottomRight),
        );
        engine.set_area(Rect::new(0, 0, 80, 25));

        let second = engine.queue.get(1).expect("second sticky queued");
        let click_x = second.area.x + second.area.width / 2;
        let click_y = second.area.y + second.area.height / 2;

        let interaction = engine.handle_click(click_x, click_y, ToastMouseButton::Left);
        assert_eq!(interaction, ToastInteraction::Dismissed);
        assert_eq!(engine.queue_len(), 1);
        assert_eq!(engine.current_message(), Some("sticky-1"));
    }

    #[test]
    fn click_copy_uses_target_toast_message() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .max_queue_depth(3)
            .build();
        engine.show_toast(
            ToastBuilder::new("first".into())
                .keep_on(1)
                .position(ToastPosition::BottomRight),
        );
        engine.show_toast(
            ToastBuilder::new("second".into())
                .keep_on(1)
                .position(ToastPosition::BottomRight),
        );
        engine.set_area(Rect::new(0, 0, 80, 25));

        let second = engine.queue.get(1).expect("second sticky queued");
        let click_x = second.area.x + second.area.width / 2;
        let click_y = second.area.y + second.area.height / 2;

        let interaction = engine.handle_click(click_x, click_y, ToastMouseButton::Right);
        assert_eq!(
            interaction,
            ToastInteraction::CopyRequested("second".to_string())
        );
        assert_eq!(engine.queue_len(), 2);
    }

    #[test]
    fn compact_title_replaces_top_padding_for_single_line_message() {
        let area = Rect::new(0, 0, 80, 25);
        let constraint = ToastConstraint::Auto;
        let titled = ToastTitle::compact("Build Failed");
        let without_title = calculate_toast_area_with_layout(ToastLayoutParams {
            title: None,
            message: "details",
            position: ToastPosition::BottomRight,
            constraint: &constraint,
            offset: (0, 0),
            area,
            border_mode: ToastBorderMode::SideRails,
            show_progress_bar: false,
        });
        let with_title = calculate_toast_area_with_layout(ToastLayoutParams {
            title: Some(&titled),
            message: "details",
            position: ToastPosition::BottomRight,
            constraint: &constraint,
            offset: (0, 0),
            area,
            border_mode: ToastBorderMode::SideRails,
            show_progress_bar: false,
        });

        assert_eq!(with_title.height, without_title.height);
    }

    #[test]
    fn dedup_timed_duplicate_refreshes_expiry() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .dedup(true)
            .build();
        engine.show_toast(
            ToastBuilder::new("saving".into())
                .duration(Duration::from_secs(5))
                .toast_type(ToastType::Info),
        );
        let first_expiry = engine.queue.front().unwrap().expires_at.unwrap();

        std::thread::sleep(Duration::from_millis(50));

        engine.show_toast(
            ToastBuilder::new("saving".into())
                .duration(Duration::from_secs(5))
                .toast_type(ToastType::Info),
        );

        assert_eq!(engine.queue_len(), 1, "duplicate should not enqueue");
        let refreshed_expiry = engine.queue.front().unwrap().expires_at.unwrap();
        assert!(
            refreshed_expiry > first_expiry,
            "expiry should be refreshed"
        );
    }

    #[test]
    fn dedup_sticky_duplicate_is_skipped() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .dedup(true)
            .build();
        engine.show_toast(
            ToastBuilder::new("error".into())
                .toast_type(ToastType::Error)
                .keep_on(1),
        );
        engine.show_toast(
            ToastBuilder::new("error".into())
                .toast_type(ToastType::Error)
                .keep_on(1),
        );

        assert_eq!(engine.queue_len(), 1, "sticky duplicate should be skipped");
    }

    #[test]
    fn dedup_different_type_is_not_duplicate() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .dedup(true)
            .build();
        engine.show_toast(ToastBuilder::new("msg".into()).toast_type(ToastType::Info));
        engine.show_toast(ToastBuilder::new("msg".into()).toast_type(ToastType::Warning));

        assert_eq!(engine.queue_len(), 2, "different type should not dedup");
    }

    #[test]
    fn dedup_different_title_is_not_duplicate() {
        let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 80, 25))
            .dedup(true)
            .build();
        engine.show_toast(ToastBuilder::new("msg".into()).title("A"));
        engine.show_toast(ToastBuilder::new("msg".into()).title("B"));

        assert_eq!(engine.queue_len(), 2, "different title should not dedup");
    }

    #[test]
    fn dedup_enabled_by_default() {
        let mut engine: ToastEngine<()> =
            ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.show_toast(ToastBuilder::new("msg".into()));
        engine.show_toast(ToastBuilder::new("msg".into()));

        assert_eq!(engine.queue_len(), 1, "dedup should be on by default");
    }

    #[test]
    fn dedup_runtime_toggle() {
        let mut engine: ToastEngine<()> =
            ToastEngineBuilder::new(Rect::new(0, 0, 80, 25)).build();
        engine.set_dedup(false);
        engine.show_toast(ToastBuilder::new("msg".into()));
        engine.set_dedup(true);
        engine.show_toast(ToastBuilder::new("msg".into()));
        assert_eq!(engine.queue_len(), 1, "dedup should work after set_dedup(true)");
        engine.set_dedup(false);
        engine.show_toast(ToastBuilder::new("msg".into()));
        assert_eq!(engine.queue_len(), 2, "dedup should be off after set_dedup(false)");
    }
}
