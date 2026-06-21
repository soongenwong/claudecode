//! A dynamic, split-pane terminal dashboard for the agent, built on `ratatui`.
//!
//! Layout:
//! ```text
//! ┌───────────────────────┬───────────────────────────────┐
//! │  Agent Pipeline        │  Workspace                     │
//! │  (live step log)       │  (file tree, colored by state) │
//! │                        ├───────────────────────────────┤
//! │                        │  (legend / progress)           │
//! ├────────────────────────┴───────────────────────────────┤
//! │  Live Diff  (syntax-highlighted, streamed char-by-char) │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! The UI is driven entirely by [`AgentEvent`]s read from an `mpsc` channel, so
//! it is agnostic to whether the events come from the [`demo`] producer or the
//! real agent loop.

mod events;
mod highlight;
mod demo;

pub use events::{AgentEvent, DiffKind, FileStatus, LogEntry};

use std::io;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Gauge, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Wrap,
};
use ratatui::Frame;

/// How often we force a repaint even with no new events (drives the spinner).
const TICK: Duration = Duration::from_millis(33); // ~30 fps

/// A single, fully-revealed diff line held in the app state.
struct DiffLine {
    kind: DiffKind,
    text: String,
}

/// Mutable UI state, updated from the event stream each tick.
#[derive(Default)]
struct App {
    status: String,
    progress: u16,
    logs: Vec<LogEntry>,
    files: Vec<(String, FileStatus)>,
    diff_file: String,
    diff_lang: String,
    diff: Vec<DiffLine>,
    frame: usize,
    finished: bool,
    quit: bool,
}

impl App {
    fn apply(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::Log(entry) => {
                self.logs.push(entry);
                // Keep memory bounded on long sessions.
                if self.logs.len() > 500 {
                    self.logs.drain(0..self.logs.len() - 500);
                }
            }
            AgentEvent::Status(text) => self.status = text,
            AgentEvent::Progress(value) => self.progress = value.min(100),
            AgentEvent::File { path, status } => {
                if let Some(slot) = self.files.iter_mut().find(|(p, _)| *p == path) {
                    slot.1 = status;
                } else {
                    self.files.push((path, status));
                }
            }
            AgentEvent::DiffBegin { file, language } => {
                self.diff_file = file;
                self.diff_lang = language;
                self.diff.clear();
            }
            AgentEvent::DiffNewLine(kind) => self.diff.push(DiffLine {
                kind,
                text: String::new(),
            }),
            AgentEvent::DiffPush(ch) => {
                if let Some(last) = self.diff.last_mut() {
                    last.text.push(ch);
                } else {
                    self.diff.push(DiffLine {
                        kind: DiffKind::Context,
                        text: ch.to_string(),
                    });
                }
            }
            AgentEvent::Done => self.finished = true,
        }
    }

    /// Drain everything currently pending without blocking.
    fn drain(&mut self, rx: &Receiver<AgentEvent>) {
        loop {
            match rx.try_recv() {
                Ok(event) => self.apply(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.finished = true;
                    break;
                }
            }
        }
    }

    fn spinner(&self) -> char {
        const FRAMES: [char; 8] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠇'];
        FRAMES[(self.frame / 3) % FRAMES.len()]
    }
}

/// Run the dashboard against an arbitrary event stream. Returns when the user
/// quits (`q` / `Esc` / `Ctrl-C`).
pub fn run(rx: &Receiver<AgentEvent>) -> io::Result<()> {
    // Make sure a panic inside the draw loop still restores the terminal.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original_hook(info);
    }));

    let mut terminal = ratatui::init();
    let mut app = App::default();
    if app.status.is_empty() {
        app.status = "Initializing agent…".to_string();
    }

    let result = event_loop(&mut terminal, &mut app, rx);

    ratatui::restore();
    result
}

/// Convenience entry point used by the CLI: spins up the simulated agent and
/// drives the dashboard with it.
pub fn run_demo() -> io::Result<()> {
    let rx = demo::spawn();
    run(&rx)
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    rx: &Receiver<AgentEvent>,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        app.drain(rx);
        terminal.draw(|frame| draw(frame, app))?;

        let timeout = TICK.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let ctrl_c =
                        key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c');
                    if ctrl_c || matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        app.quit = true;
                    }
                }
            }
        }

        if last_tick.elapsed() >= TICK {
            app.frame = app.frame.wrapping_add(1);
            last_tick = Instant::now();
        }

        if app.quit {
            break;
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(8),    // top half (logs + workspace)
            Constraint::Percentage(45), // diff
        ])
        .split(frame.area());

    draw_header(frame, root[0], app);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(root[1]);

    draw_logs(frame, top[0], app);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(6)])
        .split(top[1]);
    draw_files(frame, right[0], app);
    draw_legend(frame, right[1], app);

    draw_diff(frame, root[2], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let spin = if app.finished { '✓' } else { app.spinner() };
    let status_color = if app.finished {
        Color::Green
    } else {
        Color::Yellow
    };
    let title = Line::from(vec![
        Span::styled(" claw ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(
            format!("{spin} "),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if app.status.is_empty() { "working…" } else { &app.status },
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Agent Dashboard — press q to quit ");
    frame.render_widget(Paragraph::new(title).block(block), area);
}

fn draw_logs(frame: &mut Frame, area: Rect, app: &App) {
    let inner_height = area.height.saturating_sub(2) as usize;
    let start = app.logs.len().saturating_sub(inner_height.max(1));
    let items: Vec<ListItem> = app.logs[start..]
        .iter()
        .map(|entry| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", entry.stage),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(entry.detail.clone(), Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Agent Pipeline ");
    frame.render_widget(List::new(items).block(block), area);
}

fn draw_files(frame: &mut Frame, area: Rect, app: &App) {
    let inner_height = area.height.saturating_sub(2) as usize;
    let start = app.files.len().saturating_sub(inner_height.max(1));
    let items: Vec<ListItem> = app.files[start..]
        .iter()
        .map(|(path, status)| {
            let depth = path.matches('/').count();
            let name = path.rsplit('/').next().unwrap_or(path);
            let color = status_color(*status);
            let mut spans = vec![Span::raw("  ".repeat(depth))];
            spans.push(Span::styled(
                format!("{} ", status.glyph()),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            let mut name_style = Style::default().fg(color);
            if *status == FileStatus::Modified {
                name_style = name_style.add_modifier(Modifier::BOLD);
            }
            if *status == FileStatus::Pending {
                name_style = name_style.add_modifier(Modifier::DIM);
            }
            spans.push(Span::styled(name.to_string(), name_style));
            ListItem::new(Line::from(spans))
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Workspace ");
    frame.render_widget(List::new(items).block(block), area);
}

fn draw_legend(frame: &mut Frame, area: Rect, app: &App) {
    let legend = Line::from(
        [
            FileStatus::Scanning,
            FileStatus::Read,
            FileStatus::Modified,
            FileStatus::Pending,
        ]
        .iter()
        .flat_map(|s| {
            vec![
                Span::styled(
                    format!("{} ", s.glyph()),
                    Style::default().fg(status_color(*s)).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{}  ", s.label()), Style::default().fg(Color::Gray)),
            ]
        })
        .collect::<Vec<_>>(),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" System State ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    frame.render_widget(Paragraph::new(legend), rows[0]);

    let label = format!("{}% ", app.progress);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(app.progress)
        .label(label);
    if rows.len() > 2 {
        frame.render_widget(gauge, rows[2]);
    }
}

fn draw_diff(frame: &mut Frame, area: Rect, app: &App) {
    let title = if app.diff_file.is_empty() {
        " Live Diff ".to_string()
    } else {
        format!(" Live Diff — {} ", app.diff_file)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let view_height = inner.height as usize;
    // Auto-scroll to keep the streaming tail in view.
    let scroll = app.diff.len().saturating_sub(view_height);
    let visible = &app.diff[scroll.min(app.diff.len())..];

    let lines: Vec<Line> = visible.iter().map(|line| render_diff_line(line)).collect();

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);

    // Scrollbar so long diffs read as scrollable, not truncated.
    if app.diff.len() > view_height {
        let mut state = ScrollbarState::new(app.diff.len()).position(scroll);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area,
            &mut state,
        );
    }
}

fn render_diff_line(line: &DiffLine) -> Line<'static> {
    match line.kind {
        DiffKind::Hunk => Line::from(Span::styled(
            line.text.clone(),
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )),
        DiffKind::Removed => {
            let base = Style::default().fg(Color::Red).bg(Color::Rgb(40, 16, 16));
            let mut spans = vec![Span::styled("- ", base.add_modifier(Modifier::BOLD))];
            spans.push(Span::styled(line.text.clone(), base));
            Line::from(spans)
        }
        DiffKind::Added => {
            let base = Style::default().bg(Color::Rgb(12, 36, 16));
            let mut spans = vec![Span::styled(
                "+ ",
                Style::default().fg(Color::Green).bg(Color::Rgb(12, 36, 16)).add_modifier(Modifier::BOLD),
            )];
            spans.extend(highlight::highlight_line(&line.text, base));
            Line::from(spans)
        }
        DiffKind::Context => {
            let base = Style::default().add_modifier(Modifier::DIM);
            let mut spans = vec![Span::styled("  ", base)];
            spans.extend(highlight::highlight_line(&line.text, base));
            Line::from(spans)
        }
    }
}

fn status_color(status: FileStatus) -> Color {
    match status {
        FileStatus::Pending => Color::DarkGray,
        FileStatus::Scanning => Color::Yellow,
        FileStatus::Read => Color::Blue,
        FileStatus::Modified => Color::Green,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_status_updates_in_place() {
        let mut app = App::default();
        app.apply(AgentEvent::File {
            path: "src/main.rs".into(),
            status: FileStatus::Scanning,
        });
        app.apply(AgentEvent::File {
            path: "src/main.rs".into(),
            status: FileStatus::Modified,
        });
        assert_eq!(app.files.len(), 1);
        assert_eq!(app.files[0].1, FileStatus::Modified);
    }

    #[test]
    fn diff_chars_stream_into_current_line() {
        let mut app = App::default();
        app.apply(AgentEvent::DiffBegin {
            file: "a.rs".into(),
            language: "rust".into(),
        });
        app.apply(AgentEvent::DiffNewLine(DiffKind::Added));
        for ch in "let x = 1;".chars() {
            app.apply(AgentEvent::DiffPush(ch));
        }
        assert_eq!(app.diff.len(), 1);
        assert_eq!(app.diff[0].kind, DiffKind::Added);
        assert_eq!(app.diff[0].text, "let x = 1;");
    }

    #[test]
    fn diff_begin_resets_previous_diff() {
        let mut app = App::default();
        app.apply(AgentEvent::DiffNewLine(DiffKind::Context));
        app.apply(AgentEvent::DiffPush('x'));
        app.apply(AgentEvent::DiffBegin {
            file: "b.rs".into(),
            language: "rust".into(),
        });
        assert!(app.diff.is_empty());
        assert_eq!(app.diff_file, "b.rs");
    }

    #[test]
    fn progress_is_clamped_and_done_sets_finished() {
        let mut app = App::default();
        app.apply(AgentEvent::Progress(250));
        assert_eq!(app.progress, 100);
        assert!(!app.finished);
        app.apply(AgentEvent::Done);
        assert!(app.finished);
    }

    #[test]
    fn logs_are_capped() {
        let mut app = App::default();
        for i in 0..600 {
            app.apply(AgentEvent::Log(LogEntry {
                stage: "S".into(),
                detail: format!("{i}"),
            }));
        }
        assert_eq!(app.logs.len(), 500);
        // Oldest entries are dropped, newest retained.
        assert_eq!(app.logs.last().unwrap().detail, "599");
    }
}
