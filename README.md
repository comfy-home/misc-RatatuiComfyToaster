# ratatui-comfy-toaster

An advanced toast notification engine for [Ratatui](https://ratatui.rs/) terminal UI applications.

## Origin & Attribution

**ratatui-comfy-toaster** is an enhanced fork of [ratatui-toaster](https://crates.io/crates/ratatui-toaster) by JayanAXHF. While the original provided a solid foundation for toast notifications in terminal UIs, this fork adds significant functionality for production-grade applications requiring interactive and persistent notifications.

> **Attribution**: Originally based on `ratatui-toaster v0.1.2` by JayanAXHF <sunil.chdry@gmail.com>.

---

## What's New

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

### 🚫 Area Avoidance

Toasts can avoid overlapping with other UI elements:

```rust
engine.set_area_avoiding(area, &[blocker_rect, another_rect]);
```

Perfect for ensuring toasts don't cover important UI components like dialogs or menus.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-comfy-toaster = { path = "path/to/comfy-toaster" }
```

_soon to be added to crates.io_

### Features

- **`tokio`** — Enable async timer support for automatic toast dismissal

```toml
ratatui-comfy-toaster = { path = "...", features = ["tokio"] }
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
    ToastBuilder::new("Build failed: target key missing".into())
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
| `toast_type(type)` | Set toast type |
| `toast_bg(color)` | Override background color |
| `position(pos)` | Set position |
| `offset(x, y)` | Set offset from position |
| `placement(p)` | Set position + offset together |
| `duration(d)` | Set display duration |
| `keep_on(1)` | Make sticky (no auto-dismiss) |
| `constraint(c)` | Set size constraints |

### Engine Methods

| Method | Description |
|--------|-------------|
| `show_toast(builder)` | Display a toast |
| `hide_toast()` / `dismiss()` | Hide current toast |
| `has_toast()` | Check if toast is visible |
| `is_keep_on()` | Check if current toast is sticky |
| `toast_area()` | Get toast rectangle |
| `contains(col, row)` | Check if point is inside toast |
| `handle_click(col, row, button)` | Handle mouse click |
| `handle_shortcut(shortcut)` | Handle keyboard shortcut |
| `set_area(rect)` | Update display area |
| `set_area_avoiding(rect, occupied)` | Update area with overlap avoidance |
| `tick()` | Check for auto-dismissal |

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
    ToastBuilder::new("Version write failed".into())
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

### 4. Non-Overlapping Notifications

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
ratatui-comfy-toaster = { path = "...", features = ["tokio"] }
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
