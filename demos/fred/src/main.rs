//! FRED — a Federal Reserve economic-data browser in the terminal, built on
//! malevich and ratatui.
//!
//! The binary is only the shell: state, keys, and layout. All parsing and
//! transforms live in `malevich_demos::fred::data`, and every chart is built by a
//! pure function in `malevich_demos::fred::views` — the same plots render in the
//! TUI, in headless `--render` mode, and under test.
//!
//! Run with `cargo run -p fred`. Headless:
//! `cargo run -p fred -- --render [view] [SERIES]`.

use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::Duration;

use fred::data::{Catalog, parse_csv};
use fred::views::{self, Transform, View};
use malevich::{Charset, LineStyle, Mouse, MouseButton, PlotState};
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
    MouseButton as CtButton, MouseEvent, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line as TextLine;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs};

/// The outcome of a background fetch, delivered to the event loop by channel.
enum Fetch {
    Done {
        index: usize,
        id: &'static str,
        dates: Vec<f64>,
        values: Vec<f64>,
    },
    Failed {
        id: &'static str,
        error: String,
    },
}

struct App {
    catalog: Catalog,
    view: View,
    selected: usize,
    transform: Transform,
    line_style: LineStyle,
    charset: Charset,
    shade_recessions: bool,
    status: String,
    /// An in-flight live fetch, if any. Fetches run on their own thread — a slow
    /// or stalled network must never freeze the event loop.
    fetch: Option<Receiver<Fetch>>,
    /// The series chart's interaction state: the malevich widget caches its
    /// cell↔data mapping here and applies the zoom/pan viewport from it, while
    /// this app stays the owner of the event loop and the key policy.
    series_state: PlotState,
}

/// Crossterm's mouse event into malevich's backend-neutral vocabulary.
fn mouse(event: MouseEvent) -> Option<Mouse> {
    let button = |b: CtButton| match b {
        CtButton::Left => MouseButton::Left,
        CtButton::Right => MouseButton::Right,
        CtButton::Middle => MouseButton::Middle,
    };
    let (column, row) = (event.column, event.row);
    Some(match event.kind {
        MouseEventKind::Moved => Mouse::Moved { column, row },
        MouseEventKind::Down(b) => Mouse::Down {
            button: button(b),
            column,
            row,
        },
        MouseEventKind::Drag(b) => Mouse::Drag {
            button: button(b),
            column,
            row,
        },
        MouseEventKind::Up(b) => Mouse::Up {
            button: button(b),
            column,
            row,
        },
        MouseEventKind::ScrollUp => Mouse::ScrollUp { column, row },
        MouseEventKind::ScrollDown => Mouse::ScrollDown { column, row },
        _ => return None,
    })
}

fn main() -> std::io::Result<()> {
    let mut app = App {
        catalog: Catalog::load(),
        view: View::Overview,
        selected: 0,
        transform: Transform::Level,
        line_style: LineStyle::Pixels,
        charset: Charset::Braille,
        shade_recessions: true,
        status: String::from("vendored snapshot — press f to refresh live from FRED"),
        fetch: None,
        series_state: PlotState::default(),
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--render") {
        return render_headless(&mut app, &args[1..]);
    }

    let mut terminal = ratatui::init();
    // The mouse drives the series chart: hover crosshair, wheel zoom, drag
    // pan, right-drag zoom. Capture is the host's to enable and release.
    let _ = execute!(std::io::stdout(), EnableMouseCapture);
    let mut list_state = ListState::default();
    let result = loop {
        app.poll_fetch();
        list_state.select(Some(app.selected));
        terminal.draw(|frame| app.draw(frame, &mut list_state))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let key = match event::read()? {
            // The chart's PlotState interprets coordinates; this app only
            // routes them to the pane they landed on.
            Event::Mouse(raw) => {
                if app.view == View::Series
                    && let Some(input) = mouse(raw)
                {
                    app.series_state.on_mouse(input);
                }
                continue;
            }
            Event::Key(key) => key,
            _ => continue,
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
            KeyCode::Char('r') => app.series_state.reset_view(),
            KeyCode::Tab | KeyCode::Right => app.view = app.view.next(),
            KeyCode::BackTab | KeyCode::Left => app.view = app.view.previous(),
            KeyCode::Char(digit @ '1'..='5') => {
                app.view = View::ALL[digit as usize - '1' as usize];
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.selected = (app.selected + 1) % app.catalog.series.len();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = app.catalog.series.len();
                app.selected = (app.selected + count - 1) % count;
            }
            KeyCode::Char('t') => app.transform = app.transform.next(),
            KeyCode::Char('s') => app.shade_recessions = !app.shade_recessions,
            KeyCode::Char('c') => {
                app.line_style = match app.line_style {
                    LineStyle::Pixels => LineStyle::Corners,
                    _ => LineStyle::Pixels,
                };
            }
            KeyCode::Char('g') => {
                app.charset = match app.charset {
                    Charset::Braille => Charset::Octants,
                    Charset::Octants => Charset::Quadrants,
                    Charset::Quadrants => Charset::HalfBlocks,
                    _ => Charset::Braille,
                };
            }
            KeyCode::Char('f') => app.refresh_selected(),
            _ => {}
        }
    };
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

impl App {
    fn draw(&mut self, frame: &mut ratatui::Frame, list_state: &mut ListState) {
        let [tabs_area, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(4),
        ])
        .areas(frame.area());

        let tabs = Tabs::new(View::ALL.iter().map(|v| v.title()))
            .select(View::ALL.iter().position(|v| *v == self.view))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_widget(tabs, tabs_area);

        match self.view {
            View::Overview => self.draw_overview(frame, body),
            View::Series => self.draw_series(frame, body, list_state),
            View::Distribution => {
                self.draw_split(frame, body, views::distribution_charts(self.series()))
            }
            View::Seasonality => {
                let chart =
                    views::seasonality_chart(self.series(), body.height.saturating_sub(4) as usize);
                frame.render_widget(chart.widget().charset(self.charset), body);
            }
            View::Relations => {
                let charts = views::relations_charts(
                    &self.catalog,
                    self.shade_recessions
                        .then_some(self.catalog.recessions.as_slice()),
                );
                self.draw_split(frame, body, charts);
            }
        }

        frame.render_widget(
            Paragraph::new(self.footer()).block(Block::default().borders(Borders::ALL)),
            footer,
        );
    }

    fn draw_overview(&self, frame: &mut ratatui::Frame, area: Rect) {
        let charts = views::overview_charts(&self.catalog);
        let row_areas = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).split(area);
        for (row_area, row_charts) in row_areas.iter().zip(charts.chunks(3)) {
            let cells = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(*row_area);
            for (cell, chart) in cells.iter().zip(row_charts) {
                frame.render_widget(chart.widget().charset(self.charset), *cell);
            }
        }
    }

    fn draw_series(&mut self, frame: &mut ratatui::Frame, area: Rect, list_state: &mut ListState) {
        let [sidebar, chart_area] =
            Layout::horizontal([Constraint::Length(26), Constraint::Fill(1)]).areas(area);
        let items: Vec<ListItem> = self
            .catalog
            .series
            .iter()
            .map(|series| {
                let latest = series
                    .latest()
                    .map(|v| format!("{v:.1}"))
                    .unwrap_or_else(|| "—".into());
                ListItem::new(format!("{:<9} {:>9}", series.id, latest))
            })
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" series "))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, sidebar, list_state);

        let chart = views::series_chart(
            self.series(),
            self.transform,
            self.line_style,
            self.shade_recessions
                .then_some(self.catalog.recessions.as_slice()),
        );
        // Stateful rendering is what makes the chart interactive: the widget
        // caches its cell↔data mapping in `series_state`, applies the state's
        // zoom/pan viewport, and draws the crosshair and readout overlays.
        frame.render_stateful_widget(
            chart.widget().charset(self.charset),
            chart_area,
            &mut self.series_state,
        );
    }

    /// Renders two plots side by side. `plot.widget()` is malevich's ratatui
    /// adapter: it rasterizes into the widget's own `Rect` at draw time — cells
    /// written straight into the ratatui `Buffer`, colors mapped onto cell styles,
    /// no ANSI round-trip — so the same `Plot` value works at any pane size, and
    /// `.charset(...)` picks the glyph tier per widget.
    fn draw_split(
        &self,
        frame: &mut ratatui::Frame,
        area: Rect,
        (left, right): (malevich::Plot<'static>, malevich::Plot<'static>),
    ) {
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);
        frame.render_widget(left.widget().charset(self.charset), left_area);
        frame.render_widget(right.widget().charset(self.charset), right_area);
    }

    fn series(&self) -> &fred::data::Series {
        &self.catalog.series[self.selected]
    }

    fn footer(&self) -> Vec<TextLine<'static>> {
        let series = self.series();
        let stats = TextLine::from(format!(
            " {}  ·  {}  ·  latest {}  ·  1y {}",
            series.title,
            self.transform.label(series.kind),
            series
                .latest()
                .map(|v| format!("{v:.2} {}", series.unit))
                .unwrap_or_else(|| "—".into()),
            series
                .latest_year_change()
                .map(|c| format!("{}{c:.1}", if c >= 0.0 { "+" } else { "" }))
                .unwrap_or_else(|| "—".into()),
        ));
        let keys = TextLine::from(format!(
            " [1-5/tab] view  [jk] series  [t] transform  [c] line  [g] glyphs  [s] recessions: {}  [f] fetch  [r] reset zoom  [q] quit  ·  mouse: wheel zoom, drag pan, right-drag select ",
            if self.shade_recessions { "on" } else { "off" },
        ))
        .style(Style::default().add_modifier(Modifier::DIM));
        let status = TextLine::from(format!(" {} · source: {}", self.status, series.source))
            .style(Style::default().add_modifier(Modifier::DIM));
        vec![stats, keys, status]
    }

    /// Starts fetching the selected series from FRED on a background thread. The
    /// event loop stays live; [`App::poll_fetch`] applies the result when it lands.
    fn refresh_selected(&mut self) {
        if self.fetch.is_some() {
            self.status = String::from("a fetch is already in flight");
            return;
        }
        let index = self.selected;
        let id = self.series().id;
        self.status = format!("fetching {id} from FRED…");
        let (sender, receiver) = channel();
        self.fetch = Some(receiver);
        std::thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(10))
                .build();
            let url = format!("https://fred.stlouisfed.org/graph/fredgraph.csv?id={id}");
            let fetched = agent
                .get(&url)
                .call()
                .map_err(|error| error.to_string())
                .and_then(|response| response.into_string().map_err(|error| error.to_string()));
            let message = match fetched {
                Ok(body) => {
                    let (dates, values) = parse_csv(&body);
                    Fetch::Done {
                        index,
                        id,
                        dates,
                        values,
                    }
                }
                Err(error) => Fetch::Failed { id, error },
            };
            // The receiver may be gone if the app quit; nothing to do then.
            let _ = sender.send(message);
        });
    }

    /// Applies a finished background fetch, if one has landed.
    fn poll_fetch(&mut self) {
        let Some(receiver) = &self.fetch else { return };
        match receiver.try_recv() {
            Ok(Fetch::Done {
                index,
                id,
                dates,
                values,
            }) => {
                let points = values.len();
                if let Some(series) = self.catalog.series.get_mut(index) {
                    series.dates = dates;
                    series.values = values;
                }
                self.status = format!("refreshed {id} live ({points} observations)");
                self.fetch = None;
            }
            Ok(Fetch::Failed { id, error }) => {
                self.status = format!("fetch {id} failed: {error}");
                self.fetch = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.status = String::from("fetch thread died");
                self.fetch = None;
            }
        }
    }
}

/// Prints one view's chart(s) to stdout and exits — the whole pipeline without a
/// terminal. Arguments: an optional view name and an optional series id.
fn render_headless(app: &mut App, args: &[String]) -> std::io::Result<()> {
    for argument in args {
        if let Some(view) = View::ALL
            .iter()
            .find(|v| v.title() == argument.to_lowercase())
        {
            app.view = *view;
        } else if let Some(index) = app
            .catalog
            .series
            .iter()
            .position(|s| s.id.eq_ignore_ascii_case(argument))
        {
            app.selected = index;
            if app.view == View::Overview {
                app.view = View::Series;
            }
        }
    }
    let frame = malevich::Frame::plain(110, 28);
    let charts: Vec<malevich::Plot<'static>> = match app.view {
        View::Overview => views::overview_charts(&app.catalog),
        View::Series => vec![views::series_chart(
            app.series(),
            app.transform,
            app.line_style,
            app.shade_recessions
                .then_some(app.catalog.recessions.as_slice()),
        )],
        View::Distribution => {
            let (histogram, boxes) = views::distribution_charts(app.series());
            vec![histogram, boxes]
        }
        View::Seasonality => vec![views::seasonality_chart(app.series(), 24)],
        View::Relations => {
            let (phillips, spread) = views::relations_charts(
                &app.catalog,
                app.shade_recessions
                    .then_some(app.catalog.recessions.as_slice()),
            );
            vec![phillips, spread]
        }
    };
    for chart in charts {
        println!("{}\n", chart.render(&frame));
    }
    Ok(())
}
