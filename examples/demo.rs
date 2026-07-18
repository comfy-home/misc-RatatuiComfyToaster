//! Copyright © 2026 ComfyHome™
//! All rights reserved.
//!
//! Licensed under the ComfyGit SA-PS:DA License
//!
//! For details, see the LICENSE file in the repository root.

//! Interactive demo for ratatui-comfy-toaster.
//!
//! Run: `cargo run --example demo`
//!
//! A selectable list of toast configurations is shown in the left pane.
//! Press Enter or click to fire the selected toast. The right pane shows
//! the code that produces it. The footer lists keyboard shortcuts.

use std::io::stdout;
use std::time::Duration;

use ratatui::{
    Frame,
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
            MouseButton, MouseEventKind,
        },
        execute,
    },
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, WidgetRef, Wrap},
};

use ratatui_comfy_toaster::{
    ToastBorderMode, ToastBuilder, ToastEngine, ToastEngineBuilder, ToastInteraction,
    ToastPosition, ToastPreset, ToastProgressBarStyle, ToastShortcut, ToastType, ToastUpdate,
};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Toast demo entries
// ---------------------------------------------------------------------------

struct ToastDemo {
    name: &'static str,
    description: &'static str,
    code: &'static str,
    build: fn() -> Vec<ToastBuilder>,
    update_after_secs: Option<u64>,
    update_builder: Option<fn() -> ToastUpdate>,
}

const DEMOS: &[ToastDemo] = &[
    ToastDemo {
        name: "Basic Info",
        description: "Simple timed info toast with default settings (3s, bottom-right).",
        code: r#"ToastBuilder::new("File saved".into())"#,
        build: || vec![ToastBuilder::new("File saved".into())],
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Success",
        description: "Green success toast confirming an operation completed.",
        code: r#"ToastBuilder::new("Build succeeded".into())
    .toast_type(ToastType::Success)"#,
        build: || vec![ToastBuilder::new("Build succeeded".into()).toast_type(ToastType::Success)],
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Warning",
        description: "Yellow warning toast for non-critical issues.",
        code: r#"ToastBuilder::new("Low disk space".into())
    .toast_type(ToastType::Warning)"#,
        build: || vec![ToastBuilder::new("Low disk space".into()).toast_type(ToastType::Warning)],
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Sticky Error",
        description: "Red sticky error toast — stays until user dismisses it (left-click or Enter).",
        code: r#"ToastBuilder::new("Cargo.toml: missing [dependencies]".into())
    .toast_type(ToastType::Error)
    .keep_on(1)"#,
        build: || {
            vec![
                ToastBuilder::new("Cargo.toml: missing [dependencies]".into())
                    .toast_type(ToastType::Error)
                    .keep_on(1),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Compact Title",
        description: "Toast with a compact title on the first row, message below.",
        code: r#"ToastBuilder::new("target path cannot be empty".into())
    .title("New Scope:")
    .toast_type(ToastType::Error)"#,
        build: || {
            vec![
                ToastBuilder::new("target path cannot be empty".into())
                    .title("New Scope:")
                    .toast_type(ToastType::Error),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Gapped Title",
        description: "Toast with a gapped title — separator row between title and message.",
        code: r#"ToastBuilder::new("target path cannot be empty".into())
    .title_gapped("New Scope:")
    .toast_type(ToastType::Error)"#,
        build: || {
            vec![
                ToastBuilder::new("target path cannot be empty".into())
                    .title_gapped("New Scope:")
                    .toast_type(ToastType::Error),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Highlight Title",
        description: "Title with type-colored highlight band and contrasting text.",
        code: r#"ToastBuilder::new("target path cannot be empty".into())
    .title("New Scope:")
    .title_highlight()
    .toast_type(ToastType::Error)"#,
        build: || {
            vec![
                ToastBuilder::new("target path cannot be empty".into())
                    .title("New Scope:")
                    .title_highlight()
                    .toast_type(ToastType::Error),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Preset: GappedDotHighlightCenter",
        description: "Gapped title, dot separator, centered, highlighted — all in one preset call.",
        code: r#"ToastBuilder::new("target path cannot be empty".into())
    .preset(ToastPreset::GappedDotHighlightCenter, "New Scope:")
    .toast_type(ToastType::Error)"#,
        build: || {
            vec![
                ToastBuilder::new("target path cannot be empty".into())
                    .preset(ToastPreset::GappedDotHighlightCenter, "New Scope:")
                    .toast_type(ToastType::Error),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Preset: CompactHighlightStart",
        description: "Compact title, highlighted, start-aligned. Highlight extends through left border.",
        code: r#"ToastBuilder::new("target path cannot be empty".into())
    .preset(ToastPreset::CompactHighlightStart, "New Scope:")
    .toast_type(ToastType::Error)"#,
        build: || {
            vec![
                ToastBuilder::new("target path cannot be empty".into())
                    .preset(ToastPreset::CompactHighlightStart, "New Scope:")
                    .toast_type(ToastType::Error),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Progress Bar",
        description: "Timed toast with a depleting progress bar at the bottom.",
        code: r#"ToastBuilder::new("Saving...".into())
    .show_progress_bar(true)
    .duration(Duration::from_secs(5))
    .toast_type(ToastType::Info)"#,
        build: || {
            vec![
                ToastBuilder::new("Saving...".into())
                    .show_progress_bar(true)
                    .duration(Duration::from_secs(5))
                    .toast_type(ToastType::Info),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Progress Bar (HalfBlock)",
        description: "Progress bar using half-block glyphs for a finer look.",
        code: r#"ToastBuilder::new("Uploading...".into())
    .show_progress_bar(true)
    .progress_bar_style(ToastProgressBarStyle::HalfBlock)
    .duration(Duration::from_secs(5))"#,
        build: || {
            vec![
                ToastBuilder::new("Uploading...".into())
                    .show_progress_bar(true)
                    .progress_bar_style(ToastProgressBarStyle::HalfBlock)
                    .duration(Duration::from_secs(5)),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Full Border",
        description: "Toast with a full box border instead of the default side rails.",
        code: r#"ToastBuilder::new("Encrypted transfer complete".into())
    .border_mode(ToastBorderMode::Full)
    .toast_type(ToastType::Success)"#,
        build: || {
            vec![
                ToastBuilder::new("Encrypted transfer complete".into())
                    .border_mode(ToastBorderMode::Full)
                    .toast_type(ToastType::Success),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Top-Left Position",
        description: "Toast positioned at the top-left corner of the screen.",
        code: r#"ToastBuilder::new("Notification".into())
    .position(ToastPosition::TopLeft)"#,
        build: || vec![ToastBuilder::new("Notification".into()).position(ToastPosition::TopLeft)],
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Center Position",
        description: "Toast centered on screen — useful for important alerts.",
        code: r#"ToastBuilder::new("Press any key to continue".into())
    .position(ToastPosition::Center)
    .duration(Duration::from_secs(2))"#,
        build: || {
            vec![
                ToastBuilder::new("Press any key to continue".into())
                    .position(ToastPosition::Center)
                    .duration(Duration::from_secs(2)),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Custom Offset",
        description: "Toast with a custom offset from its default position.",
        code: r#"ToastBuilder::new("Shifted toast".into())
    .offset(-5, 3)"#,
        build: || vec![ToastBuilder::new("Shifted toast".into()).offset(-5, 3)],
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Long Message Wrap",
        description: "Long messages are automatically wrapped to fit the toast width.",
        code: r#"ToastBuilder::new(
    "This is a very long message that will \
     be automatically wrapped".into(),
)"#,
        build: || {
            vec![
            ToastBuilder::new(
                "This is a very long message that will be automatically wrapped to fit the toast width and not be clipped".into(),
            ),
        ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Sticky + Progress",
        description: "Sticky toasts ignore progress bar requests — progress is for timed toasts only.",
        code: r#"// show_progress_bar(true) is ignored for keep_on toasts
ToastBuilder::new("Waiting for input...".into())
    .show_progress_bar(true)
    .keep_on(1)
    .toast_type(ToastType::Warning)"#,
        build: || {
            vec![
                ToastBuilder::new("Waiting for input...".into())
                    .show_progress_bar(true)
                    .keep_on(1)
                    .toast_type(ToastType::Warning),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Custom Duration",
        description: "Toast with a custom 1-second duration for quick flash messages.",
        code: r#"ToastBuilder::new("Quick flash".into())
    .duration(Duration::from_secs(1))"#,
        build: || vec![ToastBuilder::new("Quick flash".into()).duration(Duration::from_secs(1))],
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "Sticky + Title + Highlight",
        description: "Sticky error with highlighted title — requires manual dismissal.",
        code: r#"ToastBuilder::new("Connection refused: localhost:8080".into())
    .title_gapped("Network Error")
    .title_highlight()
    .title_align(ToastTitleAlign::Center)
    .toast_type(ToastType::Error)
    .keep_on(1)"#,
        build: || {
            vec![
                ToastBuilder::new("Connection refused: localhost:8080".into())
                    .title_gapped("Network Error")
                    .title_highlight()
                    .toast_type(ToastType::Error)
                    .keep_on(1),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "★ Presentation: All Types",
        description: "Fires all 4 toast types with highlighted titles — showcases contrasting_fg readability fix.",
        code: r#"// All four types with highlighted titles
// to demonstrate contrast fix (2.4):
//   Yellow/Green → black text
//   Blue/Red     → white text

vec![
    ToastBuilder::new("System information updated".into())
        .title_gapped("Info")
        .title_highlight()
        .toast_type(ToastType::Info)
        .duration(Duration::from_secs(6)),

    ToastBuilder::new("All tests passed (37/37)".into())
        .title_gapped("Success")
        .title_highlight()
        .toast_type(ToastType::Success)
        .duration(Duration::from_secs(6)),

    ToastBuilder::new("Deprecated API usage detected".into())
        .title_gapped("Warning")
        .title_highlight()
        .toast_type(ToastType::Warning)
        .duration(Duration::from_secs(6)),

    ToastBuilder::new("Failed to connect to database".into())
        .title_gapped("Error")
        .title_highlight()
        .toast_type(ToastType::Error)
        .duration(Duration::from_secs(6)),
]"#,
        build: || {
            vec![
                ToastBuilder::new("System information updated".into())
                    .title_gapped("Info")
                    .title_highlight()
                    .toast_type(ToastType::Info)
                    .duration(Duration::from_secs(6)),
                ToastBuilder::new("All tests passed (37/37)".into())
                    .title_gapped("Success")
                    .title_highlight()
                    .toast_type(ToastType::Success)
                    .duration(Duration::from_secs(6)),
                ToastBuilder::new("Deprecated API usage detected".into())
                    .title_gapped("Warning")
                    .title_highlight()
                    .toast_type(ToastType::Warning)
                    .duration(Duration::from_secs(6)),
                ToastBuilder::new("Failed to connect to database".into())
                    .title_gapped("Error")
                    .title_highlight()
                    .toast_type(ToastType::Error)
                    .duration(Duration::from_secs(6)),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
    ToastDemo {
        name: "★ Live Update: Info → Success",
        description: "Shows an info toast with progress bar, then after 5s updates it to success with 2s expiry.",
        code: r#"// 1. Show info toast with 20s timeout + progress bar
let id = engine.show_toast_with_id(
    ToastBuilder::new("command: running...".into())
        .toast_type(ToastType::Info)
        .duration(Duration::from_secs(20))
        .show_progress_bar(true),
);

// 2. After 5s, update to success with 2s expiry
engine.update_toast_by_id(
    id,
    ToastUpdate::new()
        .toast_type(ToastType::Success)
        .message("command: SUCCESS")
        .duration(Some(Duration::from_secs(2)))
        .show_progress_bar(false),
);"#,
        build: || {
            vec![
                ToastBuilder::new("command: running...".into())
                    .toast_type(ToastType::Info)
                    .duration(Duration::from_secs(20))
                    .show_progress_bar(true),
            ]
        },
        update_after_secs: Some(5),
        update_builder: Some(|| {
            ToastUpdate::new()
                .toast_type(ToastType::Success)
                .message("command: SUCCESS")
                .duration(Some(Duration::from_secs(2)))
                .show_progress_bar(false)
        }),
    },
    ToastDemo {
        name: "★ Dedup Counter",
        description: "Fires 5 identical toasts rapidly — dedup consolidates them into one with a [4x] prefix.",
        code: r#"// 5 identical toasts → dedup keeps 1, counter shows [4x]
for _ in 0..5 {
    engine.show_toast(
        ToastBuilder::new("git: SUCCESS".into())
            .toast_type(ToastType::Success)
            .duration(Duration::from_secs(5)),
    );
}"#,
        build: || {
            vec![
                ToastBuilder::new("git: SUCCESS".into())
                    .toast_type(ToastType::Success)
                    .duration(Duration::from_secs(5)),
                ToastBuilder::new("git: SUCCESS".into())
                    .toast_type(ToastType::Success)
                    .duration(Duration::from_secs(5)),
                ToastBuilder::new("git: SUCCESS".into())
                    .toast_type(ToastType::Success)
                    .duration(Duration::from_secs(5)),
                ToastBuilder::new("git: SUCCESS".into())
                    .toast_type(ToastType::Success)
                    .duration(Duration::from_secs(5)),
                ToastBuilder::new("git: SUCCESS".into())
                    .toast_type(ToastType::Success)
                    .duration(Duration::from_secs(5)),
            ]
        },
        update_after_secs: None,
        update_builder: None,
    },
];

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

type PendingUpdate = Option<(u64, Instant, fn() -> ToastUpdate)>;

struct App {
    list_state: ListState,
    engine: ToastEngine<()>,
    last_toast_name: &'static str,
    last_interaction: String,
    list_area: Rect,
    progress_bar_style: ToastProgressBarStyle,
    pending_update: PendingUpdate,
}

impl Default for App {
    fn default() -> Self {
        Self {
            list_state: ListState::default(),
            engine: ToastEngineBuilder::new(Rect::new(0, 0, 120, 40))
                .default_duration(Duration::from_secs(3))
                .default_progress_bar(true)
                .default_progress_bar_style(ToastProgressBarStyle::HalfBlock)
                .build(),
            last_toast_name: "",
            last_interaction: String::new(),
            list_area: Rect::default(),
            progress_bar_style: ToastProgressBarStyle::HalfBlock,
            pending_update: None,
        }
    }
}

impl App {
    fn selected(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    fn fire_selected(&mut self) {
        let idx = self.selected();
        if idx >= DEMOS.len() {
            return;
        }
        let demo = &DEMOS[idx];
        let builders = (demo.build)();
        let mut last_id = None;
        for builder in builders {
            last_id = Some(self.engine.show_toast_with_id(builder));
        }
        if let (Some(id), Some(secs), Some(update_fn)) =
            (last_id, demo.update_after_secs, demo.update_builder)
        {
            self.pending_update = Some((id, Instant::now() + Duration::from_secs(secs), update_fn));
        } else {
            self.pending_update = None;
        }
        self.last_toast_name = demo.name;
        self.last_interaction.clear();
    }

    fn run(mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        self.list_state.select(Some(0));

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Some((id, when, update_fn)) = self.pending_update
                && Instant::now() >= when
            {
                let update = update_fn();
                self.engine.update_toast_by_id(id, update);
                self.pending_update = None;
            }

            if !event::poll(Duration::from_millis(50))? {
                self.engine.tick();
                continue;
            }

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
                    KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
                    KeyCode::Home => self.list_state.select(Some(0)),
                    KeyCode::End => self.list_state.select(Some(DEMOS.len() - 1)),
                    KeyCode::Enter => self.fire_selected(),
                    KeyCode::Char('d') => {
                        if self.engine.is_keep_on() {
                            self.engine.dismiss();
                            self.last_interaction = "Dismissed".to_string();
                        }
                    }
                    KeyCode::Char('c') => {
                        let interaction = self.engine.handle_shortcut(ToastShortcut::Copy);
                        match interaction {
                            ToastInteraction::CopyRequested(text) => {
                                self.last_interaction = format!("Copied: {text}");
                            }
                            ToastInteraction::Dismissed => {
                                self.last_interaction = "Dismissed".to_string();
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('b') => {
                        let next = match self.progress_bar_style {
                            ToastProgressBarStyle::FullBlock => ToastProgressBarStyle::HalfBlock,
                            ToastProgressBarStyle::HalfBlock => ToastProgressBarStyle::Minimal,
                            ToastProgressBarStyle::Minimal => ToastProgressBarStyle::FullBlock,
                        };
                        self.progress_bar_style = next;
                        self.engine.set_default_progress_bar_style(next);
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => {
                    if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                        self.handle_click(mouse.column, mouse.row);
                    } else if mouse.kind == MouseEventKind::Down(MouseButton::Right) {
                        let interaction = self.engine.handle_click(
                            mouse.column,
                            mouse.row,
                            ratatui_comfy_toaster::ToastMouseButton::Right,
                        );
                        if let ToastInteraction::CopyRequested(text) = interaction {
                            self.last_interaction = format!("Copied: {text}");
                        }
                    }
                }
                _ => {}
            }

            self.engine.tick();
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let len = DEMOS.len();
        let current = self.selected();
        let new = (current as i32 + delta).rem_euclid(len as i32) as usize;
        self.list_state.select(Some(new));
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        if self.engine.contains(col, row) {
            let interaction =
                self.engine
                    .handle_click(col, row, ratatui_comfy_toaster::ToastMouseButton::Left);
            if interaction == ToastInteraction::Dismissed {
                self.last_interaction = "Dismissed (click)".to_string();
            }
            return;
        }

        if col >= self.list_area.x
            && col < self.list_area.x + self.list_area.width
            && row >= self.list_area.y
            && row < self.list_area.y + self.list_area.height
        {
            let inner_y = row - self.list_area.y - 1;
            if inner_y < DEMOS.len() as u16 * 2 {
                let idx = (inner_y / 2) as usize;
                if idx < DEMOS.len() {
                    self.list_state.select(Some(idx));
                    self.fire_selected();
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let bg = Color::Rgb(20, 20, 40);

        Block::new()
            .style(Style::new().bg(bg))
            .render(area, frame.buffer_mut());

        let [header, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas(area);

        Line::from(vec![
            Span::styled(
                "ratatui-comfy-toaster",
                Style::new().fg(Color::LightBlue).bold(),
            ),
            Span::raw(" "),
            Span::styled("demo", Style::new().fg(Color::DarkGray)),
        ])
        .centered()
        .render(header, frame.buffer_mut());

        self.draw_body(frame, body);
        self.draw_footer(frame, footer);

        self.engine.set_area(body);
        self.engine.render_ref(body, frame.buffer_mut());
    }

    fn draw_body(&mut self, frame: &mut Frame, area: Rect) {
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(65), Constraint::Fill(1)]).areas(area);

        self.draw_list(frame, left);
        self.draw_code(frame, right);
    }

    fn draw_list(&mut self, frame: &mut Frame, area: Rect) {
        self.list_area = area;

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(Color::Rgb(60, 60, 100)))
            .title(" Toast Configurations ")
            .style(Style::new().bg(Color::Rgb(20, 20, 40)));

        let items: Vec<ListItem> = DEMOS
            .iter()
            .map(|demo| {
                ListItem::new(vec![
                    Line::from(Span::styled(
                        demo.name,
                        Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        demo.description,
                        Style::new().fg(Color::DarkGray),
                    )),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::new()
                    .fg(Color::LightYellow)
                    .bg(Color::Rgb(40, 40, 70))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_code(&self, frame: &mut Frame, area: Rect) {
        let idx = self.selected();
        let demo = &DEMOS[idx];

        let mut lines = vec![
            Line::from(Span::styled(
                demo.name,
                Style::new().fg(Color::LightCyan).bold(),
            )),
            Line::from(""),
        ];

        for code_line in demo.code.lines() {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::new()),
                Span::styled(code_line, Style::new().fg(Color::LightGreen)),
            ]));
        }

        lines.push(Line::from(""));
        if !self.last_toast_name.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Last fired: ", Style::new().fg(Color::DarkGray)),
                Span::styled(self.last_toast_name, Style::new().fg(Color::LightYellow)),
            ]));
        }
        if !self.last_interaction.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Interaction: ", Style::new().fg(Color::DarkGray)),
                Span::styled(&self.last_interaction, Style::new().fg(Color::LightCyan)),
            ]));
        }

        let queue_info = format!(
            "Queue: {} toast(s){}",
            self.engine.queue_len(),
            if self.engine.is_keep_on() {
                " (sticky front)"
            } else {
                ""
            },
        );
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            queue_info,
            Style::new().fg(Color::DarkGray),
        )));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(Color::Rgb(60, 60, 100)))
            .title(" Code ")
            .style(Style::new().bg(Color::Rgb(20, 20, 40)));

        let paragraph = Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let key = |s: &'static str| Span::styled(s, Style::new().fg(Color::Yellow));
        let dim = |s: &'static str| Span::styled(s, Style::new().fg(Color::DarkGray));

        let bar_style_label = match self.progress_bar_style {
            ToastProgressBarStyle::FullBlock => "full",
            ToastProgressBarStyle::HalfBlock => "half",
            ToastProgressBarStyle::Minimal => "minimal",
        };

        let segments: Vec<Vec<Span<'static>>> = vec![
            vec![key("↑"), dim("/"), key("↓"), dim(" select")],
            vec![key("Enter"), dim(" fire toast")],
            vec![key("d"), dim(" dismiss sticky")],
            vec![key("c"), dim(" copy")],
            vec![
                key("B"),
                dim(" bar style ("),
                Span::styled(bar_style_label, Style::new().fg(Color::DarkGray)),
                dim(")"),
            ],
            vec![key("Click"), dim(" fire / dismiss")],
            vec![key("Right-Click"), dim(" copy toast")],
            vec![key("q"), dim(" quit")],
        ];

        let separator = vec![Span::styled(" | ", Style::new().fg(Color::DarkGray))];
        let sep_width: usize = separator.iter().map(Span::width).sum();

        let mut current = Vec::new();
        let mut current_width = 0;
        let mut lines: Vec<Line<'static>> = Vec::new();
        let max_width = area.width as usize;

        for segment in segments {
            let seg_width: usize = segment.iter().map(Span::width).sum();
            let added = if current.is_empty() {
                seg_width
            } else {
                sep_width + seg_width
            };

            if !current.is_empty() && current_width + added > max_width {
                lines.push(Line::from(std::mem::take(&mut current)));
                current = segment;
                current_width = seg_width;
            } else {
                if !current.is_empty() {
                    current.extend(separator.clone());
                    current_width += sep_width;
                }
                current.extend(segment);
                current_width += seg_width;
            }
        }
        if !current.is_empty() {
            lines.push(Line::from(current));
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .alignment(Alignment::Center)
            .style(Style::new().bg(Color::Rgb(20, 20, 40)));

        frame.render_widget(paragraph, area);
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    ratatui::run(|terminal| {
        execute!(stdout(), EnableMouseCapture)?;
        let result = App::default().run(terminal);
        let _ = execute!(stdout(), DisableMouseCapture);
        result
    })?;

    Ok(())
}
