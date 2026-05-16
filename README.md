# ratatui-comfy-toaster

An advanced toast notification engine for [Ratatui](https://ratatui.rs/) terminal UI applications.

## Origin & Attribution

**ratatui-comfy-toaster** is an enhanced fork of [ratatui-toaster](https://crates.io/crates/ratatui-toaster) by JayanAXHF. While the original provided a solid foundation for toast notifications in terminal UIs, this fork adds significant functionality for production-grade applications requiring interactive and persistent notifications.

> **Attribution**: Originally based on `ratatui-toaster v0.1.2` by JayanAXHF <sunil.chdry@gmail.com>.

---

<details><summary>👀 What's new in v0.3.2 ...</summary>

### 💥 💥 💥 This Release's Top Picks ...  💥 💥 💥

#### **1. &nbsp;&nbsp;&nbsp;Expiration Progress Bar**
- Now your timed toasts can display an optional expiry bar
- Available are 3 styles:
    - FullBlock: ████
    - HalfBlock: ▄▄▄▄ 
    - Minimal: ____
- See documentation for more info...

#### **2. &nbsp;&nbsp;&nbsp;Toasts now support two border modes:**
- `ToastBorderMode::SideRails` keeps the original left/right look
- `ToastBorderMode::Full` renders a full box border for stronger separation
    - It's useful mainly with `Center` positioned toasts


<sub>...  🎉 Enjoy!</sub>

<br><br>


---
<sup>... ✨ auto-injected by [ComfyGit](https://github.com/comfy-home/ComfyGit)       |       For detailed changelog [CLICK HERE](https://github.com/comfy-home/misc-RatatuiComfyToaster/releases/tag/v0.3.2)</sup>

---

</details>

<sup>_The_ 👆 _"What's new" section_ ☝️ _is created automatically using our other project - [ComfyGit](https://github.com/comfy-home/ComfyGit). It can do this, and a LOT more..._</sup>

**Enjoying the Toaster project?** Dropping a ⭐ on our [GitHub](https://github.com/comfy-home/misc-RatatuiComfyToaster) repo would absolutely make our day...

**Any issues, or suggestions?** Click [HERE](https://github.com/comfy-home/misc-RatatuiComfyToaster/issues) and let us know.

---

## Features:

### 🆕 Sticky (Persistent) Toasts

The most significant addition is the `keep_on()` mechanism:

- **`keep_on(0)`** (default) — Timed toasts that auto-dismiss after the duration
- **`keep_on(1)`** — Sticky toasts that remain visible until explicitly dismissed

Sticky toasts are perfect for errors, warnings, or important messages that users must acknowledge.

### 🖱️ Mouse Interaction

Sticky toasts support full mouse interaction:

- **Left-click** — Dismiss the toast
- **Right-click** — Request copy of toast message to clipboard

### ⌨️ Keyboard Shortcuts

Programmatic interaction via `ToastShortcut`:

- `ToastShortcut::Dismiss` — Dismiss sticky toast
- `ToastShortcut::Copy` — Request copy action

Returns `ToastInteraction` indicating what action occurred:
- `ToastInteraction::Dismissed` — Toast was dismissed
- `ToastInteraction::CopyRequested(String)` — User requested to copy message
- `ToastInteraction::None` — No action

### 🎨 Customizable Background

Per-toast background color support:

```rust
ToastBuilder::new("Deployment queued".into())
    .toast_bg(Color::Blue)
```

Or use the default dark gray (`DEFAULT_BG`) for consistent styling.

### 🏷️ Optional Title Line

Toasts support optional titles with compact or gapped layout, alignment, and highlight styling.

**Compact title** (default) uses the first content row for the title and following rows for the message, without stacking the title as extra wrapped lines. A one-line message with a compact title uses two content rows (title, message).

```rust
use ratatui_comfy_toaster::{ToastBuilder, ToastType};

ToastBuilder::new("Target path cannot be empty".into())
    .title("New Scope:")
    .toast_type(ToastType::Error)
    .keep_on(1);
```

**Gapped title** inserts a separator row between title and message (dot fill by default, same middle-dot glyph as ComfyGit tiles):

```rust
use ratatui_comfy_toaster::{
    ToastBuilder, ToastTitleSeparator, ToastTitleAlign, ToastType,
};

ToastBuilder::new("Details".into())
    .title_gapped("Build Failed")
    .title_separator(ToastTitleSeparator::Line)
    .title_align(ToastTitleAlign::Center)
    .title_highlight()
    .toast_type(ToastType::Error);
```

| Option | Values | Default |
|--------|--------|---------|
| Layout | `.title()` compact, `.title_gapped()` | compact |
| Separator | `Dot`, `Line`, `Empty` | `Dot` (gapped only) |
| Align | `Start`, `Center` | `Start` |
| Style | plain, `.title_highlight()` | plain |

With `title_highlight()`, the title background uses the toast type color and the text uses a contrasting foreground (white on red/yellow/green/blue). For start alignment, the highlight extends through the left border column (no gray gap before the title band) and one column past the title text; centered highlights add two columns on each side of the title.

Gapped separator rows use the toast type color for dot/line glyphs. Toasts without a title keep top padding (an empty row above the message); titled toasts start the title on the first inner row.

Copy actions return `title + "\n" + message` (separator rows are not copied).

### 🎛️ Title presets

Named layout presets live in `presets.rs` and apply title layout, separator, alignment, and highlight in one call:

```rust
use ratatui_comfy_toaster::{ToastBuilder, ToastPreset, ToastType};

ToastBuilder::new("Target path cannot be empty".into())
    .preset(ToastPreset::GappedDotHighlightCenter, "New Scope:")
    .toast_type(ToastType::Error)
    .keep_on(1);
```

| `ToastPreset` | Layout | Separator | Align | Highlight |
|---------------|--------|-----------|-------|-------------|
| `MessageOnly` | — | — | — | — |
| `CompactPlainStart` | compact | — | start | no |
| `CompactHighlightStart` | compact | — | start | yes |
| `CompactPlainCenter` | compact | — | center | no |
| `CompactHighlightCenter` | compact | — | center | yes |
| `GappedDotStart` | gapped | dot | start | no |
| `GappedLineStart` | gapped | line | start | no |
| `GappedEmptyStart` | gapped | empty | start | no |
| `GappedDotHighlightCenter` | gapped | dot | center | yes |

`CompactHighlightStart` extends the highlight band through the left border column so it meets the side rail with no gray gap.

### 🧱 Toast Borders

Toasts now support two border modes:

- `ToastBorderMode::SideRails` keeps the original left/right look
- `ToastBorderMode::Full` renders a full box border for stronger separation

You can set this globally:

```rust
use ratatui_comfy_toaster::{ToastBorderMode, ToastEngineBuilder};

let engine = ToastEngineBuilder::new(area)
    .default_border_mode(ToastBorderMode::Full)
    .build();
```

Or override it per toast:

```rust
use ratatui_comfy_toaster::{ToastBorderMode, ToastBuilder};

ToastBuilder::new("Centered message".into())
    .border_mode(ToastBorderMode::Full);
```

### ⏳ Timed Toast Progress Bar

Timed toasts can show a one-row progress bar that depletes as the toast approaches expiration.
Sticky toasts ignore the progress bar automatically.

Available styles:

- `ToastProgressBarStyle::FullBlock` uses `█`
- `ToastProgressBarStyle::HalfBlock` uses `▄`
- `ToastProgressBarStyle::Minimal` uses `_`

Set it globally:

```rust
use ratatui_comfy_toaster::{ToastEngineBuilder, ToastProgressBarStyle};

let engine = ToastEngineBuilder::new(area)
    .default_progress_bar(true)
    .default_progress_bar_style(ToastProgressBarStyle::HalfBlock)
    .build();
```

Or override it per toast:

```rust
use ratatui_comfy_toaster::{ToastBuilder, ToastProgressBarStyle};

ToastBuilder::new("Saved successfully".into())
    .show_progress_bar(true)
    .progress_bar_style(ToastProgressBarStyle::Minimal);
```

### 📍 Placement API

Convenient `placement()` method to set both position and offset in one call:

```rust
use ratatui_comfy_toaster::{ToastPlacement, ToastPosition};

let placement = ToastPlacement {
    position: ToastPosition::TopRight,
    offset: (-2, 1),
};

ToastBuilder::new("Saved".into()).placement(placement)
```

### 📏 Text Wrapping

Long messages are automatically wrapped instead of clipped, ensuring content is always readable.

### 📬 Toast Queue

Toasts are now queued rather than overwritten. Multiple messages can be pending at once:

- A FIFO queue holds up to `max_queue_depth` toasts (default: **4**, configurable)
- **Timed toasts** drain automatically from the front as each expires or is dismissed
- **Sticky toasts** block the queue — the next toast only becomes visible after the sticky one is dismissed
- When the queue is full, incoming timed toasts are **silently dropped**
- When the queue is full and an incoming toast is **sticky**, the oldest timed toast is displaced to make room; if all slots are sticky, the new one is dropped

```rust
let mut engine: ToastEngine<()> = ToastEngineBuilder::new(area)
    .max_queue_depth(6)
    .build();

engine.show_toast(ToastBuilder::new("Step 1 complete".into()).toast_type(ToastType::Success));
engine.show_toast(ToastBuilder::new("Step 2 complete".into()).toast_type(ToastType::Success));
engine.show_toast(ToastBuilder::new("Build failed!".into()).toast_type(ToastType::Error).keep_on(1));
// All three are queued; the error toast will block until dismissed
```

### 🚫 Area Avoidance

Toasts can avoid overlapping with other UI elements:

```rust
engine.set_area_avoiding(area, &[blocker_rect, another_rect]);
```

Perfect for ensuring toasts don't cover important UI components like dialogs or menus.

---

## Installation

From your project's directory:
```
cargo add ratatui-comfy-toaster
```

Or...

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-comfy-toaster = "0.3.3"
```

### Features

- **`tokio`** — Enable async timer support for automatic toast dismissal

```toml
ratatui-comfy-toaster = { version = "0.3.3", features = ["tokio"] }
```

---

## Quick Start

### Basic Timed Toast

```rust
use ratatui::layout::Rect;
use ratatui_comfy_toaster::{ToastBuilder, ToastEngine, ToastEngineBuilder};

// Create engine with display area
let mut engine: ToastEngine<()> = ToastEngineBuilder::new(Rect::new(0, 0, 120, 40))
    .default_duration(Duration::from_secs(3))
    .build();

// Show a simple toast
engine.show_toast(ToastBuilder::new("Saved successfully".into()));

// In your render loop:
engine.render(area, buf);

// Tick to handle auto-dismissal (without tokio feature)
if engine.tick() {
    // Toast was dismissed
}
```

### Sticky Error Toast

```rust
use ratatui_comfy_toaster::ToastType;

// Show sticky error that user must dismiss
engine.show_toast(
    ToastBuilder::new("Target key missing in Cargo.toml".into())
        .title("Build Failed")
        .toast_type(ToastType::Error)
        .keep_on(1),  // Sticky!
);
```

### Interactive Toast with Clipboard

```rust
use ratatui_comfy_toaster::{ToastInteraction, ToastShortcut};

// Show sticky toast
engine.show_toast(
    ToastBuilder::new("Error details: connection timeout".into())
        .toast_type(ToastType::Error)
        .keep_on(1),
);

// Handle shortcuts
match engine.handle_shortcut(ToastShortcut::Copy) {
    ToastInteraction::CopyRequested(text) => {
        // Copy text to clipboard
        copy_to_clipboard(text);
        // Show confirmation
        engine.show_toast(ToastBuilder::new("Copied to clipboard".into()));
    }
    ToastInteraction::Dismissed => {
        // Toast was dismissed
    }
    _ => {}
}
```

---

## API Reference

### Toast Types

| Type | Border Color | Use Case |
|------|--------------|----------|
| `ToastType::Info` | Blue | General information |
| `ToastType::Success` | Green | Success confirmations |
| `ToastType::Warning` | Yellow | Warnings, cautions |
| `ToastType::Error` | Red | Errors, failures |

### Toast Positions

- `ToastPosition::TopLeft`
- `ToastPosition::TopRight`
- `ToastPosition::BottomLeft`
- `ToastPosition::BottomRight` (default)
- `ToastPosition::Center`

### Builder Methods

| Method | Description |
|--------|-------------|
| `new(message)` | Create builder with message |
| `title(text)` | Add an optional title line above the message |
| `toast_type(type)` | Set toast type |
| `toast_bg(color)` | Override background color |
| `position(pos)` | Set position |
| `offset(x, y)` | Set offset from position |
| `placement(p)` | Set position + offset together |
| `duration(d)` | Set display duration |
| `keep_on(1)` | Make sticky (no auto-dismiss) |
| `constraint(c)` | Set size constraints |

### Engine Builder Methods

| Method | Description |
|--------|-------------|
| `new(area)` | Create builder with display area |
| `default_duration(d)` | Set default toast duration (default: 3s) |
| `max_queue_depth(n)` | Set max queued toasts (default: 4, min: 1) |
| `action_tx(tx)` | Set tokio channel sender *(tokio feature only)* |

### Engine Methods

| Method | Description |
|--------|-------------|
| `show_toast(builder)` | Enqueue a toast |
| `hide_toast()` / `dismiss()` | Dismiss front toast and advance queue |
| `has_toast()` | Check if any toast is queued |
| `queue_len()` | Number of toasts currently queued |
| `is_keep_on()` | Check if front toast is sticky |
| `toast_area()` | Get front toast rectangle |
| `contains(col, row)` | Check if point is inside front toast |
| `handle_click(col, row, button)` | Handle mouse click |
| `handle_shortcut(shortcut)` | Handle keyboard shortcut |
| `set_area(rect)` | Update display area |
| `set_area_avoiding(rect, occupied)` | Update area with overlap avoidance |
| `tick()` | Advance queue if front toast has expired |

### Constants

```rust
pub const DEFAULT_POSITION: ToastPlacement = ToastPlacement {
    position: ToastPosition::BottomRight,
    offset: (0, -1),
};

pub const DEFAULT_BG: Color = Color::DarkGray;
```

---

## Use Cases

### 1. Simple Confirmations

```rust
// Brief success messages
engine.show_toast(ToastBuilder::new("Project saved".into()));
```

### 2. Persistent Errors

```rust
// Errors that must be acknowledged
engine.show_toast(
    ToastBuilder::new("Version write failed in Cargo.toml".into())
        .title("Release Failed")
        .toast_type(ToastType::Error)
        .keep_on(1),
);
```

### 3. Copyable Diagnostics

```rust
// Technical details users can copy
engine.show_toast(
    ToastBuilder::new("Path: /very/long/path/to/file.txt".into())
        .toast_type(ToastType::Info)
        .keep_on(1),
);
// User right-clicks or presses Copy shortcut to copy path
```

### 4. Queued Progress Steps

```rust
// Queue multiple messages — each shown in order as the previous expires
engine.show_toast(ToastBuilder::new("Fetching…".into()));
engine.show_toast(ToastBuilder::new("Compiling…".into()));
engine.show_toast(ToastBuilder::new("Done.".into()).toast_type(ToastType::Success));
```

### 5. Non-Overlapping Notifications

```rust
// Ensure toast doesn't cover dialog
let dialog_rect = Rect::new(10, 5, 60, 20);
engine.set_area_avoiding(full_area, &[dialog_rect]);
engine.show_toast(ToastBuilder::new("Background task started".into()));
```

---

## Tokio Integration

Enable the `tokio` feature for automatic toast dismissal via async timers:

```toml
[dependencies]
ratatui-comfy-toaster = { version = "0.3.3", features = ["tokio"] }
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time"] }
```

```rust
use tokio::sync::mpsc;
use ratatui_comfy_toaster::ToastMessage;

enum Action {
    ShowToast(ToastMessage),
    // ... other variants
}

impl From<ToastMessage> for Action {
    fn from(msg: ToastMessage) -> Self {
        Action::ShowToast(msg)
    }
}

let (tx, mut rx) = mpsc::channel::<Action>(32);

let mut engine: ToastEngine<Action> = ToastEngineBuilder::new(area)
    .action_tx(tx.clone())
    .build();

// Toasts automatically send `Hide` action when duration expires
tokio::spawn(async move {
    while let Some(action) = rx.recv().await {
        match action {
            Action::ShowToast(msg) => match msg {
                ToastMessage::Show { .. } => { /* handle show */ }
                ToastMessage::Hide => engine.hide_toast(),
            },
            _ => {}
        }
    }
});
```

---

## Animations

While ratatui-comfy-toaster doesn't include built-in animations, you can integrate with libraries like [tachyonfx](https://github.com/ratatui/tachyonfx):

```rust
use tachyonfx::{fx, Effect, EffectRenderer};

// Get toast area for animation
let toast_area = engine.toast_area();

// Apply shader effect when showing/hiding
if engine.has_toast() {
    buf.render_effect(fx::sweep_in(toast_area, 300), area, shader);
}
```

---

## License

This project is licensed under the **SA-PS:DA** (Source-Available Public Software with Display Attribution) License.

See [LICENSE.md](LICENSE.md) for full terms.

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

<p align="center">
  <sub>Made with ❤️ by <a href="https://comfyhome.io">ComfyHome™</a></sub>
</p>
