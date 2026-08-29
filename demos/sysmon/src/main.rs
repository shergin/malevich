//! sysmon — a live system monitor in the terminal, built on malevich and ratatui.
//!
//! The streaming story end to end: a sampler thread reads the machine twice a
//! second and `push`es into `malevich::stream::Ring` windows; the UI thread
//! `snapshot`s them and rebuilds plots every frame. No shared state beyond the
//! rings' own lock — the sampler never blocks the UI, the UI never blocks the
//! sampler.
//!
//! Run with `cargo run -p sysmon`. Headless: `cargo run -p sysmon -- --render`
//! samples for ~2 seconds and prints every chart once.
//!
//! Keys: `tab`/`1`/`2` switch view · `g` cycles the glyph charset · `p` pauses ·
//! `q` quits.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use malevich::Charset;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line as TextLine;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use sysmon::data::{History, Sampler};
use sysmon::views::{self, View};

/// Seconds between samples. CPU deltas need a steady cadence; half a second is
/// smooth without being busywork.
const INTERVAL: f64 = 0.5;
/// Ring capacity: two minutes of history at the sampling interval.
const CAPACITY: usize = 240;

struct App {
    history: History,
    view: View,
    charset: Charset,
    paused: Arc<AtomicBool>,
}

fn main() -> std::io::Result<()> {
    // First contact with the machine: one sample fixes the core count and total
    // memory, which size the history.
    let mut sampler = Sampler::new();
    let first = sampler.sample();
    let history = History::new(CAPACITY, first.per_core.len(), first.mem_total, INTERVAL);

    if std::env::args().any(|argument| argument == "--render") {
        return render_headless(sampler, &history);
    }

    let paused = Arc::new(AtomicBool::new(false));
    let app = App {
        history: history.clone(),
        view: View::Dashboard,
        charset: Charset::Braille,
        paused: paused.clone(),
    };

    // The sampler thread: read, push, sleep. `History` clones share their rings,
    // so this handle and the UI's handle observe the same windows.
    std::thread::spawn(move || {
        loop {
            if !paused.load(Ordering::Relaxed) {
                history.push(&sampler.sample());
            }
            std::thread::sleep(Duration::from_secs_f64(INTERVAL));
        }
    });

    run(app)
}

fn run(mut app: App) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = loop {
        terminal.draw(|frame| app.draw(frame))?;
        // A short poll keeps charts flowing even with no input: every timeout is
        // a redraw from fresh ring snapshots.
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
            KeyCode::Tab | KeyCode::Right | KeyCode::Left | KeyCode::BackTab => {
                app.view = app.view.next();
            }
            KeyCode::Char('1') => app.view = View::Dashboard,
            KeyCode::Char('2') => app.view = View::Cores,
            KeyCode::Char('p') | KeyCode::Char(' ') => {
                let paused = app.paused.load(Ordering::Relaxed);
                app.paused.store(!paused, Ordering::Relaxed);
            }
            KeyCode::Char('g') => {
                app.charset = match app.charset {
                    Charset::Braille => Charset::Octants,
                    Charset::Octants => Charset::Quadrants,
                    Charset::Quadrants => Charset::HalfBlocks,
                    _ => Charset::Braille,
                };
            }
            _ => {}
        }
    };
    ratatui::restore();
    result
}

impl App {
    fn draw(&self, frame: &mut ratatui::Frame) {
        let [tabs_area, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas(frame.area());

        let tabs = Tabs::new(View::ALL.iter().map(|view| view.title()))
            .select(View::ALL.iter().position(|view| *view == self.view))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_widget(tabs, tabs_area);

        match self.view {
            View::Dashboard => self.draw_dashboard(frame, body),
            View::Cores => self.draw_cores(frame, body),
        }

        let paused = self.paused.load(Ordering::Relaxed);
        let keys = TextLine::from(format!(
            " [1-2/tab] view  [g] glyphs  [p] pause: {}  [q] quit ",
            if paused { "paused" } else { "live" },
        ))
        .style(Style::default().add_modifier(Modifier::DIM));
        frame.render_widget(
            Paragraph::new(keys).block(Block::default().borders(Borders::ALL)),
            footer,
        );
    }

    /// CPU, memory, network — stacked thirds, each a fresh snapshot of its ring.
    fn draw_dashboard(&self, frame: &mut ratatui::Frame, area: Rect) {
        let [cpu_area, mem_area, net_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(area);
        let history = &self.history;
        // Follow the stream: pin every x axis to the full two-minute window
        // from the first frame, so the filling rings draw into a stable axis
        // instead of rescaling it on every sample.
        let window = views::tail_window(CAPACITY, history.interval);
        let charts = [
            (
                views::cpu_chart(&history.cpu.snapshot(), history.interval),
                cpu_area,
            ),
            (
                views::mem_chart(&history.mem.snapshot(), history.mem_total, history.interval),
                mem_area,
            ),
            (
                views::net_chart(
                    &history.rx.snapshot(),
                    &history.tx.snapshot(),
                    history.interval,
                ),
                net_area,
            ),
        ];
        for (chart, chart_area) in charts {
            frame.render_widget(
                chart.viewport(window).widget().charset(self.charset),
                chart_area,
            );
        }
    }

    /// The per-core heatmap over a strip of instantaneous bars.
    fn draw_cores(&self, frame: &mut ratatui::Frame, area: Rect) {
        let [heat_area, bars_area] =
            Layout::vertical([Constraint::Fill(2), Constraint::Length(9)]).areas(area);
        let window = views::tail_window(CAPACITY, self.history.interval);
        frame.render_widget(
            views::cores_heatmap(&self.history)
                .viewport(window)
                .widget()
                .charset(self.charset),
            heat_area,
        );
        frame.render_widget(
            views::cores_bars(&self.history)
                .widget()
                .charset(self.charset),
            bars_area,
        );
    }
}

/// Samples for about two seconds, then prints every chart once — the whole
/// pipeline without a terminal, for piping and screenshots.
fn render_headless(mut sampler: Sampler, history: &History) -> std::io::Result<()> {
    for _ in 0..4 {
        std::thread::sleep(Duration::from_secs_f64(INTERVAL));
        history.push(&sampler.sample());
    }
    let frame = malevich::Frame::plain(100, 18);
    let charts = [
        views::cpu_chart(&history.cpu.snapshot(), history.interval),
        views::mem_chart(&history.mem.snapshot(), history.mem_total, history.interval),
        views::net_chart(
            &history.rx.snapshot(),
            &history.tx.snapshot(),
            history.interval,
        ),
        views::cores_heatmap(history),
        views::cores_bars(history),
    ];
    for chart in charts {
        println!("{}\n", chart.render(&frame));
    }
    Ok(())
}
