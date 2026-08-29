//! FRED — a Federal Reserve economic-data browser in the terminal, built on
//! malevich and ratatui.
//!
//! The binary is only the shell: state, keys, and layout. All parsing and
//! transforms live in `malevich_demos::fred::data`, and every chart is built by a
//! pure function in `malevich_demos::fred::views` — the same plots render in the
//! TUI, in headless `--render` mode, and under test.
//!
//! Run with `cargo run -p fred` — add `--release` in a pixel-capable
//! terminal (sixel, kitty, iTerm2): every view renders as real images at
//! native device-pixel density — glowing lines over translucent washes,
//! accumulated-ink scatters, a bilinear heatmap — and every pane answers
//! the mouse: hover crosshairs with snapped readouts, wheel zoom, drag
//! pan, rubber-band zoom. `p` toggles pixel drawing live (the glyph/image
//! comparison switch), `--fast` halves the image density for slow links,
//! `--cells` forces glyphs from the start. Headless:
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
    /// The year-over-year context strip's state — a second pane linked to the
    /// main chart by mirroring the x window after every event.
    strip_state: PlotState,
    /// Whether the strip was drawn last frame; a hidden pane receives nothing.
    strip_shown: bool,
    /// Which pane a drag started in (`true` = strip): drags stay with their
    /// pane even when the cursor crosses into the other one.
    drag_in_strip: Option<bool>,
    /// The detected pixel protocol, when the terminal speaks one: the series
    /// charts render as real images through the same stateful widgets.
    graphics: Option<malevich::pixel::Graphics>,
    /// The live opt-out: `p` flips between pixel panels and glyph charts
    /// without restarting — the comparison switch.
    pixels_on: bool,
    /// One interaction state per pane, per view: every chart in the app
    /// renders stateful, so every view gets pixel panels, hover, and zoom.
    overview_states: [PlotState; 6],
    distribution_states: [PlotState; 2],
    seasonality_state: PlotState,
    relations_states: [PlotState; 2],
    /// Which pane of the current (non-series) view a drag started in.
    drag_pane: Option<usize>,
}

/// The widget dressing every pane shares: the chosen charset and — when the
/// terminal speaks a protocol and pixels are on — the image renderer.
fn dress(
    widget: malevich::PlotWidget<'_>,
    charset: Charset,
    graphics: Option<malevich::pixel::Graphics>,
) -> malevich::PlotWidget<'_> {
    let widget = widget.charset(charset);
    match graphics {
        Some(graphics) => widget.graphics(graphics),
        None => widget,
    }
}

/// Whether a terminal cell lies inside a pane's plot rectangle.
fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x && column < rect.right() && row >= rect.y && row < rect.bottom()
}

/// The cell position an input names, or `None` for vocabulary this app
/// cannot route (the `Mouse` enum is non-exhaustive).
fn position(input: Mouse) -> Option<(u16, u16)> {
    match input {
        Mouse::Moved { column, row }
        | Mouse::ScrollUp { column, row }
        | Mouse::ScrollDown { column, row }
        | Mouse::ScrollLeft { column, row }
        | Mouse::ScrollRight { column, row }
        | Mouse::Down { column, row, .. }
        | Mouse::Drag { column, row, .. }
        | Mouse::Up { column, row, .. } => Some((column, row)),
        _ => None,
    }
}

/// The linked-panes pattern: the x view is a value, so sharing it is
/// assignment. Each pane keeps its own y.
fn mirror_x(window: Option<(f64, f64)>, to: &mut PlotState) {
    let view = to.viewport();
    to.set_viewport(match window {
        Some((low, high)) => view.with_x(low, high),
        None => view.reset_x(),
    });
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
        MouseEventKind::ScrollLeft => Mouse::ScrollLeft { column, row },
        MouseEventKind::ScrollRight => Mouse::ScrollRight { column, row },
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
        strip_state: PlotState::default(),
        strip_shown: false,
        drag_in_strip: None,
        graphics: None,
        pixels_on: true,
        overview_states: std::array::from_fn(|_| PlotState::default()),
        distribution_states: std::array::from_fn(|_| PlotState::default()),
        seasonality_state: PlotState::default(),
        relations_states: std::array::from_fn(|_| PlotState::default()),
        drag_pane: None,
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--render") {
        return render_headless(&mut app, &args[1..]);
    }

    // Real pixels where the terminal speaks them. Probe before raw mode: the
    // capability query reads terminal replies, which the event loop below
    // would swallow as input. `--cells` opts out.
    if !args.iter().any(|argument| argument == "--cells") {
        app.graphics = malevich::pixel::Capabilities::detect_for(&std::io::stdout()).best();
    }
    // Native density by default: one transmitted pixel maps onto one device
    // pixel, which is what crisp means — and the widget's pacing plus
    // unchanged-panel suppression keep it interactive (a fred-sized kitty
    // frame measures ~22 ms at Retina density in release). `--fast` halves
    // the density for slow links — the placement rectangle scales the image
    // back over the panel, trading sharpness for a quarter of the bytes —
    // with a heavier stroke so the ink weight survives the upscale. Sixel
    // has no placement scaling and always stays native.
    if args.iter().any(|argument| argument == "--fast")
        && let Some(graphics) = &mut app.graphics
        && graphics.protocol != malevich::pixel::Protocol::Sixel
    {
        let (w, h) = graphics.cell_size;
        if w > 10 && h > 20 {
            graphics.cell_size = (w / 2, h / 2);
            let derived = (usize::from(graphics.cell_size.1) + 8) / 16;
            graphics.stroke = Some((derived + 1).min(4) as u8);
        }
    }

    let mut terminal = ratatui::init();
    // The mouse drives the series chart: hover crosshair, wheel zoom, drag
    // pan, right-drag zoom. Capture is the host's to enable and release.
    let _ = execute!(std::io::stdout(), EnableMouseCapture);
    let mut list_state = ListState::default();
    // Repaint the image on state change, not on a timer: when nothing
    // changed, the previous transmission stays on screen through the
    // 250 ms poll frames.
    let mut emit_needed = true;
    let mut was_view = app.view;
    let result = 'ui: loop {
        if app.poll_fetch() {
            emit_needed = true;
        }
        list_state.select(Some(app.selected));
        terminal.draw(|frame| app.draw(frame, &mut list_state))?;

        if let Some(graphics) = app.graphics {
            if app.view != was_view {
                // Kitty images live on their own layer; the new view's cell
                // repaints cannot cover them. Retiring also resets the
                // states, so a return starts from fresh ground.
                let mut previous = app.pixel_states(was_view);
                graphics.retire(&mut std::io::stdout(), &mut previous[..])?;
                emit_needed = true;
            }
            if app.pixels_on && emit_needed {
                let mut panes = app.pixel_states(app.view);
                graphics.present(&mut std::io::stdout(), &mut panes[..])?;
                emit_needed = false;
            }
        }
        was_view = app.view;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        // Drain the whole queue before redrawing: mouse motion arrives far
        // faster than a pixel frame renders and transmits, so handling one
        // event per frame would grow an unbounded lag queue — the cursor
        // trailing seconds behind the hand. A burst collapses into one
        // repaint of the final state; that coalescing is the throttle, and
        // it adds no latency of its own.
        loop {
            match event::read()? {
                // The charts' PlotStates interpret coordinates; this app only
                // routes them to the pane they landed on and keeps the two
                // panes' x windows mirrored (docs/interaction.md).
                Event::Mouse(raw) => {
                    if let Some(input) = mouse(raw) {
                        let changed = if app.view == View::Series {
                            app.route_mouse(input)
                        } else {
                            app.route_view(input)
                        };
                        if changed {
                            emit_needed = true;
                        }
                    }
                }
                Event::Resize(..) => emit_needed = true,
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    emit_needed = true;
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break 'ui Ok(()),
                        KeyCode::Char('r') => {
                            for state in app.pixel_states(app.view) {
                                state.reset_view();
                            }
                        }
                        // The pixel/glyph switch: retiring the images lets the
                        // next cell frame actually show (kitty images sit on
                        // their own layer above text).
                        KeyCode::Char('p') => {
                            app.pixels_on = !app.pixels_on;
                            if !app.pixels_on
                                && let Some(graphics) = app.graphics
                            {
                                let mut panes = app.pixel_states(app.view);
                                let _ = graphics.retire(&mut std::io::stdout(), &mut panes[..]);
                            }
                        }
                        // h/l pan the series chart (arrows switch views); the
                        // strip follows through the same mirroring the mouse
                        // path uses.
                        KeyCode::Char('h') if app.view == View::Series => {
                            app.series_state.pan_left();
                            mirror_x(app.series_state.viewport().x(), &mut app.strip_state);
                        }
                        KeyCode::Char('l') if app.view == View::Series => {
                            app.series_state.pan_right();
                            mirror_x(app.series_state.viewport().x(), &mut app.strip_state);
                        }
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
                }
                _ => {}
            }
            if !event::poll(Duration::ZERO)? {
                break;
            }
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
                let charts = views::distribution_charts(self.series());
                self.draw_split(frame, body, charts, View::Distribution);
            }
            View::Seasonality => {
                let chart =
                    views::seasonality_chart(self.series(), body.height.saturating_sub(4) as usize);
                let (charset, ink) = (self.charset, self.ink());
                frame.render_stateful_widget(
                    dress(chart.widget(), charset, ink),
                    body,
                    &mut self.seasonality_state,
                );
            }
            View::Relations => {
                let charts = views::relations_charts(
                    &self.catalog,
                    self.shade_recessions
                        .then_some(self.catalog.recessions.as_slice()),
                );
                self.draw_split(frame, body, charts, View::Relations);
            }
        }

        frame.render_widget(
            Paragraph::new(self.footer()).block(Block::default().borders(Borders::ALL)),
            footer,
        );
    }

    fn draw_overview(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let charts = views::overview_charts(&self.catalog);
        let (charset, ink) = (self.charset, self.ink());
        let row_areas = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).split(area);
        let cells: Vec<Rect> = row_areas
            .iter()
            .flat_map(|row| {
                Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ])
                .split(*row)
                .to_vec()
            })
            .collect();
        for ((chart, cell), state) in charts
            .iter()
            .zip(cells)
            .zip(self.overview_states.iter_mut())
        {
            frame.render_stateful_widget(dress(chart.widget(), charset, ink), cell, state);
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
        // zoom/pan viewport, and draws the crosshair, snap, and readout
        // overlays. When there is room (and the main chart is not already the
        // year-over-year view), a linked context strip renders below it from
        // its own state — `route_mouse` keeps the two x windows mirrored.
        self.strip_shown = chart_area.height >= 22 && self.transform != Transform::YearOverYear;
        let main_area = if self.strip_shown {
            let [main_area, strip_area] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(8)]).areas(chart_area);
            let strip = views::context_strip(self.series());
            let (charset, ink) = (self.charset, self.ink());
            frame.render_stateful_widget(
                dress(strip.widget(), charset, ink),
                strip_area,
                &mut self.strip_state,
            );
            main_area
        } else {
            chart_area
        };
        let (charset, ink) = (self.charset, self.ink());
        frame.render_stateful_widget(
            dress(chart.widget(), charset, ink),
            main_area,
            &mut self.series_state,
        );
    }

    /// The image renderer to dress widgets with, when pixels are on.
    fn ink(&self) -> Option<malevich::pixel::Graphics> {
        self.graphics.filter(|_| self.pixels_on)
    }

    /// Every pane state of one view, in layout order — what presents,
    /// retires, and routes as a unit.
    fn pixel_states(&mut self, view: View) -> Vec<&mut PlotState> {
        match view {
            View::Overview => self.overview_states.iter_mut().collect(),
            View::Series => vec![&mut self.series_state, &mut self.strip_state],
            View::Distribution => self.distribution_states.iter_mut().collect(),
            View::Seasonality => vec![&mut self.seasonality_state],
            View::Relations => self.relations_states.iter_mut().collect(),
        }
    }

    /// Routes an input to the pane of the current view it belongs to — the
    /// generic single-pane routing every non-series view uses (hover, wheel
    /// zoom, drags; each pane's state machine is independent). Drags stay
    /// with the pane they started in.
    fn route_view(&mut self, input: Mouse) -> bool {
        let Some((column, row)) = position(input) else {
            return false;
        };
        let dragging = self.drag_pane;
        let mut states = self.pixel_states(self.view);
        let hit = states
            .iter()
            .position(|state| state.plot_area().is_some_and(|r| contains(r, column, row)));
        let target = match input {
            Mouse::Drag { .. } | Mouse::Up { .. } => dragging.or(hit),
            _ => hit,
        };
        let mut changed = false;
        if let Some(index) = target {
            changed = states[index].on_mouse(input);
        }
        if matches!(input, Mouse::Moved { .. }) {
            // A hover belongs to one pane; every other pane's cursor clears.
            for (index, state) in states.iter_mut().enumerate() {
                if Some(index) != target {
                    changed |= state.on_mouse(Mouse::Moved { column: 0, row: 0 });
                }
            }
        }
        drop(states);
        match input {
            Mouse::Down { .. } => self.drag_pane = target,
            Mouse::Up { .. } => self.drag_pane = None,
            _ => {}
        }
        changed
    }

    /// Routes one mouse input to the pane it belongs to and mirrors the x
    /// window onto the other pane — the linked-panes pattern from
    /// docs/interaction.md. Drags stay with the pane they started in; a hover
    /// entering one pane clears the other's cursor. True when any pane's
    /// state changed.
    fn route_mouse(&mut self, input: Mouse) -> bool {
        fn contains(rect: Rect, column: u16, row: u16) -> bool {
            column >= rect.x && column < rect.right() && row >= rect.y && row < rect.bottom()
        }
        let (column, row) = match input {
            Mouse::Moved { column, row }
            | Mouse::ScrollUp { column, row }
            | Mouse::ScrollDown { column, row }
            | Mouse::ScrollLeft { column, row }
            | Mouse::ScrollRight { column, row }
            | Mouse::Down { column, row, .. }
            | Mouse::Drag { column, row, .. }
            | Mouse::Up { column, row, .. } => (column, row),
            // The vocabulary is non-exhaustive; inputs this app cannot
            // position are not routed.
            _ => return false,
        };
        let positional = self.strip_shown
            && self
                .strip_state
                .plot_area()
                .is_some_and(|rect| contains(rect, column, row));
        let in_strip = match input {
            Mouse::Drag { .. } | Mouse::Up { .. } => self.drag_in_strip.unwrap_or(positional),
            _ => positional,
        };
        let (pane, other) = if in_strip {
            (&mut self.strip_state, &mut self.series_state)
        } else {
            (&mut self.series_state, &mut self.strip_state)
        };
        let mut changed = pane.on_mouse(input);
        if let Mouse::Moved { .. } = input {
            // (0, 0) is the tab row — outside any plot — so this clears.
            changed |= other.on_mouse(Mouse::Moved { column: 0, row: 0 });
        }
        match input {
            Mouse::Down { .. } => self.drag_in_strip = Some(in_strip),
            Mouse::Up { .. } => self.drag_in_strip = None,
            _ => {}
        }
        mirror_x(pane.viewport().x(), other);
        changed
    }

    /// Renders two plots side by side. `plot.widget()` is malevich's ratatui
    /// adapter: it rasterizes into the widget's own `Rect` at draw time — cells
    /// written straight into the ratatui `Buffer`, colors mapped onto cell styles,
    /// no ANSI round-trip — so the same `Plot` value works at any pane size, and
    /// `.charset(...)` picks the glyph tier per widget.
    fn draw_split(
        &mut self,
        frame: &mut ratatui::Frame,
        area: Rect,
        (left, right): (malevich::Plot<'static>, malevich::Plot<'static>),
        view: View,
    ) {
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);
        let (charset, ink) = (self.charset, self.ink());
        let states = match view {
            View::Relations => &mut self.relations_states,
            _ => &mut self.distribution_states,
        };
        frame.render_stateful_widget(
            dress(left.widget(), charset, ink),
            left_area,
            &mut states[0],
        );
        frame.render_stateful_widget(
            dress(right.widget(), charset, ink),
            right_area,
            &mut states[1],
        );
    }

    fn series(&self) -> &fred::data::Series {
        &self.catalog.series[self.selected]
    }

    fn footer(&self) -> Vec<TextLine<'static>> {
        let series = self.series();
        // Zoomed in, the stats line describes what is on screen instead of the
        // whole series — the selection → statistics pattern from
        // docs/interaction.md: the rubber-band window is the selection, and
        // the visible data summarizes with the ordinary stat vocabulary.
        let stats = if self.view == View::Series
            && let Some((low, high)) = self.series_state.viewport().x()
        {
            let year_over_year;
            let values: &[f64] = match self.transform {
                Transform::Level | Transform::Log => &series.values,
                Transform::YearOverYear => {
                    year_over_year = series.year_over_year();
                    &year_over_year
                }
            };
            let mut visible = malevich::stat::Moments::new();
            for (&date, &value) in series.dates.iter().zip(values) {
                if (low..=high).contains(&date) && value.is_finite() {
                    visible.add(value);
                }
            }
            let range = self
                .series_state
                .mapping()
                .map(|m| format!("{} – {}", m.format_x(low), m.format_x(high)))
                .unwrap_or_default();
            match (visible.mean(), visible.min(), visible.max()) {
                (Some(mean), Some(min), Some(max)) => TextLine::from(format!(
                    " {}  ·  visible {range}  ·  {} obs  ·  mean {mean:.2}  ·  min {min:.2}  ·  max {max:.2}",
                    series.title,
                    visible.count(),
                )),
                _ => TextLine::from(format!(
                    " {}  ·  visible {range}  ·  no data in view",
                    series.title
                )),
            }
        } else {
            TextLine::from(format!(
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
            ))
        };
        let keys = TextLine::from(format!(
            " [1-5/tab] view  [jk] series  [t] transform  [c] line  [g] glyphs  [s] recessions: {}{}  [f] fetch  [hl] pan  [r] reset zoom  [q] quit  ·  mouse: wheel zoom, drag pan, right-drag select ",
            if self.shade_recessions { "on" } else { "off" },
            match self.graphics {
                Some(_) if self.pixels_on => "  [p] pixels: on",
                Some(_) => "  [p] pixels: off",
                None => "",
            },
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
    /// Applies a finished background fetch; true when anything changed.
    fn poll_fetch(&mut self) -> bool {
        let Some(receiver) = &self.fetch else {
            return false;
        };
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
                true
            }
            Ok(Fetch::Failed { id, error }) => {
                self.status = format!("fetch {id} failed: {error}");
                self.fetch = None;
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                self.status = String::from("fetch thread died");
                self.fetch = None;
                true
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
