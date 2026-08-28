//! The ratatui adapter: a plot as a widget, and the interaction controller
//! (feature `ratatui`).
//!
//! Depends only on `ratatui-core` — the stable trait-and-buffer layer — so any app
//! in the ratatui ecosystem can embed charts without version lockstep. The plot
//! rasterizes straight into the `Buffer`: no ANSI round-trip, colors map one to one
//! onto ratatui styles, and the terminal stays entirely the host application's.
//!
//! Two ways to render. [`Widget`] is fire-and-forget: the plot draws into its
//! area, nothing is remembered. [`StatefulWidget`] threads a [`PlotState`]
//! through the render, and that state is the whole interaction story: it caches
//! the [`Mapping`] of what was drawn (so mouse coordinates become data
//! coordinates), holds the [`Viewport`] the widget applies on the next draw,
//! and runs the default mouse gestures — hover crosshair, wheel zoom anchored
//! under the cursor, left-drag pan, right-drag rubber-band zoom — from
//! coordinates the host feeds it via [`PlotState::on_mouse`]. The widget never
//! reads the terminal: the host owns the event loop, mouse capture, and key
//! policy, and can bypass the gestures entirely by driving [`Viewport`] and
//! [`Mapping`] itself.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color as RatColor;
use ratatui_core::style::Style;
use ratatui_core::widgets::{StatefulWidget, Widget};

use crate::data::Series;
use crate::mark::Mark;
use crate::plot::{Frame, Mapping, Plot, Viewport};
use crate::render::{Charset, Color, ColorMode, display_width};
use crate::theme::Theme;

impl Plot<'_> {
    /// Wraps the plot as a ratatui widget rendering into its area.
    ///
    /// ```no_run
    /// # fn draw(frame: &mut ratatui_core::terminal::Frame, area: ratatui_core::layout::Rect) {
    /// let chart = malevich::line(&[1.0, 4.0, 2.0][..]).title("demo");
    /// frame.render_widget(chart.widget(), area);
    /// # }
    /// ```
    ///
    /// For interaction — hit-testing, zoom, pan, crosshair — render it stateful
    /// instead, with a [`PlotState`] the host keeps between frames:
    ///
    /// ```no_run
    /// # fn draw(frame: &mut ratatui_core::terminal::Frame, area: ratatui_core::layout::Rect,
    /// #         state: &mut malevich::PlotState) {
    /// let chart = malevich::line(&[1.0, 4.0, 2.0][..]).title("demo");
    /// frame.render_stateful_widget(chart.widget(), area, state);
    /// # }
    /// ```
    pub fn widget(&self) -> PlotWidget<'_> {
        PlotWidget {
            plot: self,
            charset: Charset::Quadrants,
            theme: Theme::DARK,
            crosshair: true,
            readout: true,
            snap: true,
            #[cfg(feature = "pixel")]
            graphics: None,
        }
    }
}

/// A [`Plot`] rendering into a ratatui `Buffer`.
///
/// Created by [`Plot::widget`]; size comes from the render area, colors go straight
/// into cell styles (the host backend owns color depth).
#[derive(Debug, Clone, Copy)]
pub struct PlotWidget<'a> {
    plot: &'a Plot<'a>,
    charset: Charset,
    theme: Theme,
    crosshair: bool,
    readout: bool,
    snap: bool,
    #[cfg(feature = "pixel")]
    graphics: Option<crate::pixel::Graphics>,
}

impl PlotWidget<'_> {
    /// Sets the charset; quadrants by default. Dense tiers are explicit because the
    /// host application knows its terminal and configured font better than we do.
    #[must_use]
    pub fn charset(mut self, charset: Charset) -> Self {
        self.charset = charset;
        self
    }

    /// Sets the theme (palette) used for unstyled layers.
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Whether a stateful render tints the cursor's row and column (on by
    /// default). Only glyph backgrounds still unset get the tint, so data —
    /// heatmap cells carry their color in the background — is never repainted.
    #[must_use]
    pub fn crosshair(mut self, on: bool) -> Self {
        self.crosshair = on;
        self
    }

    /// Whether a stateful render writes the cursor's data coordinates —
    /// formatted the way the axes format their own labels — in the plot
    /// panel's top-right corner (on by default). With snapping on, the
    /// readout shows the values of the data under the cursor instead; see
    /// [`PlotWidget::snap`].
    #[must_use]
    pub fn readout(mut self, on: bool) -> Self {
        self.readout = on;
        self
    }

    /// Whether the cursor snaps to the data (on by default). For every point-
    /// backed [`Line`](crate::Line) and [`Points`](crate::Points) layer, a
    /// stateful render finds the datum nearest the cursor's x inside the
    /// visible window, highlights its cell, and the readout lists the datum's
    /// value — `label: value` for labeled layers — instead of the cursor's
    /// own coordinates. A gap at the snapped x reads as `—`, never an
    /// interpolation; function-backed lines and other marks do not
    /// participate. With `snap(false)` the readout returns to plain cursor
    /// coordinates.
    #[must_use]
    pub fn snap(mut self, on: bool) -> Self {
        self.snap = on;
        self
    }

    /// Draws the plot panel as a real image (feature `pixel`): sixel, kitty
    /// graphics, or iTerm2, in the protocol and cell geometry `graphics`
    /// names. Requires a stateful render — the widget reserves its area in
    /// the buffer (spaces, skipped so ratatui's diff leaves the image alone)
    /// and stores the encoded block in the [`PlotState`]; the host emits it
    /// after `terminal.draw` with [`present_pixels`]. Interaction chrome
    /// becomes annotation marks rendered into the image itself: anti-aliased
    /// crosshair rules, snap markers, and the readout as in-panel text.
    ///
    /// Detect capabilities *before* `ratatui::init()` — the probe reads
    /// terminal replies that a raw-mode event loop would swallow:
    ///
    /// ```no_run
    /// let graphics = malevich::pixel::Capabilities::detect_for(&std::io::stdout()).best();
    /// // let mut terminal = ratatui::init();
    /// // ... chart.widget().graphics(graphics.unwrap()) when Some ...
    /// ```
    ///
    /// A stateless render ignores this setting and stays glyphs — it has no
    /// state to carry the image block in.
    #[cfg(feature = "pixel")]
    #[must_use]
    pub fn graphics(mut self, graphics: crate::pixel::Graphics) -> Self {
        self.graphics = Some(graphics);
        self
    }

    fn frame(&self, area: Rect) -> Frame {
        Frame {
            width: area.width as usize,
            height: area.height as usize,
            charset: self.charset,
            // The mode only governs string encoding, which the adapter bypasses.
            color: ColorMode::TrueColor,
            theme: self.theme,
        }
    }

    /// The fire-and-forget render both `Widget` impls delegate to.
    fn draw(&self, area: Rect, buffer: &mut Buffer) {
        let surface = self.plot.rasterize(&self.frame(area));
        blit(&surface, area, buffer);
    }

    /// The interactive render both `StatefulWidget` impls delegate to: apply
    /// the state's viewport, cache the mapping, draw the overlays — as buffer
    /// cells, or as a reserved rectangle plus a pending image block when a
    /// pixel protocol is configured.
    fn draw_stateful(&self, area: Rect, buffer: &mut Buffer, state: &mut PlotState) {
        #[cfg(feature = "pixel")]
        if let Some(graphics) = self.graphics
            && self.draw_stateful_pixels(&graphics, area, buffer, state)
        {
            return;
        }
        let plot = self.plot.clone().viewport(&state.view);
        let (surface, mapping) = plot.rasterize_mapped(&self.frame(area));
        blit(&surface, area, buffer);
        state.area = area;
        state.mapping = Some(mapping);
        self.overlays(buffer, state);
    }

    /// The pixel render: the whole area becomes a post-frame block — chrome
    /// text and image woven by the hybrid renderer — and the buffer only
    /// reserves the ground. Interaction chrome rides the clone as annotation
    /// marks (never palette-consuming data layers), with the domains pinned
    /// to the last frame's mapping so hovering cannot jitter an automatic
    /// axis. Returns false when bounded geometry refuses the pixel path, and
    /// the caller degrades to glyphs.
    #[cfg(feature = "pixel")]
    fn draw_stateful_pixels(
        &self,
        graphics: &crate::pixel::Graphics,
        area: Rect,
        buffer: &mut Buffer,
        state: &mut PlotState,
    ) -> bool {
        // Self-pacing: encoding and transmitting an image panel costs
        // milliseconds, and hover motion asks for it hundreds of times a
        // second. Within the pace window — same rectangle, same viewport —
        // this render only re-reserves the ground and keeps the image already
        // on screen: at most one window of crosshair staleness, and a naive
        // one-event-per-frame host loop stops falling behind, because the
        // redundant frames it draws cost nearly nothing. A changed viewport
        // or rectangle always renders: a zoomed window must never show stale.
        let unchanged = state.pixel_area == Some(area) && state.pixel_view == Some(state.view);
        if unchanged
            && state
                .pixel_pace
                .is_some_and(|last| last.elapsed() < PIXEL_PACE)
        {
            for y in area.y..area.bottom() {
                for x in area.x..area.right() {
                    let cell = &mut buffer[(x, y)];
                    cell.reset();
                    cell.set_diff_option(ratatui_core::buffer::CellDiffOption::Skip);
                }
            }
            state.area = area;
            return true;
        }
        let mut plot = self.plot.clone().viewport(&state.view);
        if let Some((cursor_x, cursor_y)) = state.cursor_data()
            && cursor_x.is_finite()
            && cursor_y.is_finite()
            && let Some(mapping) = state.mapping().cloned()
        {
            let (x_low, x_high) = mapping.x_domain();
            let (y_low, y_high) = mapping.y_domain();
            if mapping.x_bands().is_none() {
                plot = plot.x_domain(x_low, x_high);
            }
            if mapping.y_bands().is_none() {
                plot = plot.y_domain(y_low, y_high);
            }
            if self.crosshair {
                plot = plot
                    .layer(
                        crate::Rule::v(cursor_x)
                            .color(Color::BrightBlack)
                            .dash(crate::Dash::Dotted),
                    )
                    .layer(
                        crate::Rule::h(cursor_y)
                            .color(Color::BrightBlack)
                            .dash(crate::Dash::Dotted),
                    );
            }
            let snapped = if self.snap {
                self.snapped(state)
            } else {
                Vec::new()
            };
            let ink = if self.theme == Theme::LIGHT {
                Color::Black
            } else {
                Color::BrightWhite
            };
            // The panel font is printable ASCII (coverage is honest: anything
            // else advances without ink), so the pixel chrome speaks ASCII —
            // an asterisk marker, dashes for the separators and gaps.
            for snap in &snapped {
                if let Some(value) = snap.value {
                    plot = plot.layer(crate::Text::at(snap.x, value, "*").color(ink));
                }
            }
            if self.readout
                && let Some((_, _, columns, _)) = mapping.plot_area()
                && let Some(line) =
                    self.readout_line(state, &snapped, columns.saturating_sub(2) as u16)
            {
                let line = line.replace('·', "-").replace('—', "-");
                let x = x_low + (x_high - x_low) * 0.02;
                let y = y_high - (y_high - y_low) * 0.06;
                plot = plot.layer(crate::Text::at(x, y, line).color(Color::BrightBlack));
            }
        }
        // One stable image id per panel, for life: kitty replaces the
        // on-screen image atomically when a new one arrives under the same
        // id, so repaints never blink through a deleted-but-not-yet-drawn
        // gap. Seeded from the process id so two apps sharing a terminal
        // are unlikely to collide.
        let id = *state.pixel_id.get_or_insert_with(next_pixel_id);
        let kitty_id = (graphics.protocol == crate::pixel::Protocol::Kitty).then_some(id);
        let Ok((block, mapping)) = crate::pixel::try_render_mapped(
            &plot,
            &self.frame(area),
            graphics,
            area.x as usize,
            kitty_id,
        ) else {
            return false;
        };
        // Reserve the ground: spaces, skipped so ratatui's diff never writes
        // under the image — except the first frame at this rectangle, whose
        // real spaces clear whatever the cells held before (transparent
        // panels would otherwise show it through their alpha).
        let fresh = state.pixel_area != Some(area);
        let diff = if fresh {
            ratatui_core::buffer::CellDiffOption::None
        } else {
            ratatui_core::buffer::CellDiffOption::Skip
        };
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let cell = &mut buffer[(x, y)];
                cell.reset();
                cell.set_diff_option(diff);
            }
        }
        state.area = area;
        state.mapping = Some(mapping);
        state.pixel_area = Some(area);
        state.pixel_pace = Some(std::time::Instant::now());
        state.pixel_view = Some(state.view);
        // Raw mode: LF moves down without returning the carriage; the block's
        // rows need explicit carriage returns. Image payloads never contain a
        // raw newline, so this touches only the row separators.
        let block = block.replace('\n', "\r\n");
        let hash = block_hash(&block);
        // What is on screen already matches: nothing to queue, nothing to
        // transmit — an unchanged panel is free at every protocol.
        if state.pixel_sent != Some((area, hash)) {
            state.pixels = Some((area, block, hash));
        }
        true
    }
}

impl Widget for PlotWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.draw(area, buffer);
    }
}

impl Widget for &PlotWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.draw(area, buffer);
    }
}

impl StatefulWidget for PlotWidget<'_> {
    type State = PlotState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut PlotState) {
        self.draw_stateful(area, buffer, state);
    }
}

impl StatefulWidget for &PlotWidget<'_> {
    type State = PlotState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut PlotState) {
        self.draw_stateful(area, buffer, state);
    }
}

/// Writes the rasterized cells into the buffer, clipped to the area.
fn blit(surface: &crate::render::Surface, area: Rect, buffer: &mut Buffer) {
    for (column, row, glyph, foreground, background) in surface.cells() {
        let x = area.x + column as u16;
        let y = area.y + row as u16;
        if x >= area.right() || y >= area.bottom() {
            continue;
        }
        let cell = &mut buffer[(x, y)];
        let mut symbol = [0u8; 4];
        cell.set_symbol(glyph.encode_utf8(&mut symbol));
        if let Some(fg) = convert(foreground) {
            cell.set_fg(fg);
        }
        if let Some(bg) = convert(background) {
            cell.set_bg(bg);
        }
    }
}

/// Backend-neutral mouse input, in terminal cell coordinates.
///
/// `ratatui-core` defines no event types and this crate imports no backend's,
/// so the controller speaks this minimal vocabulary instead; mapping a
/// backend's mouse event onto it is one short `match`. For crossterm:
///
/// ```
/// use crossterm::event::{MouseButton as Ct, MouseEvent, MouseEventKind as Kind};
/// use malevich::{Mouse, MouseButton};
///
/// fn mouse(event: MouseEvent) -> Option<Mouse> {
///     let button = |b: Ct| match b {
///         Ct::Left => MouseButton::Left,
///         Ct::Right => MouseButton::Right,
///         Ct::Middle => MouseButton::Middle,
///     };
///     let (column, row) = (event.column, event.row);
///     Some(match event.kind {
///         Kind::Moved => Mouse::Moved { column, row },
///         Kind::Down(b) => Mouse::Down { button: button(b), column, row },
///         Kind::Drag(b) => Mouse::Drag { button: button(b), column, row },
///         Kind::Up(b) => Mouse::Up { button: button(b), column, row },
///         Kind::ScrollUp => Mouse::ScrollUp { column, row },
///         Kind::ScrollDown => Mouse::ScrollDown { column, row },
///         _ => return None,
///     })
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mouse {
    /// The cursor moved with no button held.
    Moved {
        /// Terminal column.
        column: u16,
        /// Terminal row.
        row: u16,
    },
    /// A button was pressed.
    Down {
        /// The pressed button.
        button: MouseButton,
        /// Terminal column.
        column: u16,
        /// Terminal row.
        row: u16,
    },
    /// The cursor moved with a button held.
    Drag {
        /// The held button.
        button: MouseButton,
        /// Terminal column.
        column: u16,
        /// Terminal row.
        row: u16,
    },
    /// A button was released.
    Up {
        /// The released button.
        button: MouseButton,
        /// Terminal column.
        column: u16,
        /// Terminal row.
        row: u16,
    },
    /// The wheel scrolled up (away from the user).
    ScrollUp {
        /// Terminal column.
        column: u16,
        /// Terminal row.
        row: u16,
    },
    /// The wheel scrolled down (toward the user).
    ScrollDown {
        /// Terminal column.
        column: u16,
        /// Terminal row.
        row: u16,
    },
}

/// A mouse button, in the vocabulary of [`Mouse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// The primary button: drags pan the view.
    Left,
    /// The secondary button: drags select a rubber-band zoom window.
    Right,
    /// The wheel button; the default gestures ignore it.
    Middle,
}

/// A drag in progress.
#[derive(Debug, Clone, Copy)]
enum Drag {
    /// Left button held: the view pans with the cursor.
    Pan { last: (u16, u16) },
    /// Right button held: a rubber-band selection grows toward `current`.
    Select {
        anchor: (u16, u16),
        current: (u16, u16),
    },
}

/// The interaction state of one plot pane, threaded through
/// `render_stateful_widget` — the ratatui idiom for widget state the host
/// queries and feeds (`ListState`, `TableState`, …).
///
/// The state holds the cached [`Mapping`] of the last render (making
/// [`PlotState::data_at`] answer against exactly what is on screen), the
/// [`Viewport`] the widget applies on the next render, the hover cursor, and
/// any drag in progress. Application state — what a selection *means*, which
/// key resets — stays in the host.
///
/// The default gesture grammar, fed by [`PlotState::on_mouse`]: hover tracks
/// the cursor for the crosshair and readout; the wheel zooms x around the data
/// under the cursor; a left drag pans; a right drag selects a rectangle and
/// zooms to it on release. Everything ignores coordinates outside the plot
/// rectangle, and axes with no continuous window (bands) stay untouched. A
/// host wanting different gestures skips `on_mouse` and drives
/// [`PlotState::set_viewport`] with its own [`Viewport`] arithmetic.
#[derive(Debug, Clone, Default)]
pub struct PlotState {
    area: Rect,
    mapping: Option<Mapping>,
    view: Viewport,
    cursor: Option<(u16, u16)>,
    drag: Option<Drag>,
    /// The pending image block of the last pixel render — rectangle, encoded
    /// block, content hash — consumed by [`present_pixels`].
    #[cfg(feature = "pixel")]
    pixels: Option<(Rect, String, u64)>,
    /// The rectangle whose ground was last cleared for an image; a change
    /// triggers a fresh blanking frame.
    #[cfg(feature = "pixel")]
    pixel_area: Option<Rect>,
    /// This panel's stable kitty image id: retransmitting under it replaces
    /// the on-screen image atomically, which is what makes repaints
    /// flicker-free. Assigned on the first pixel render, kept for life.
    #[cfg(feature = "pixel")]
    pixel_id: Option<u32>,
    /// What the last [`present_pixels`] put on screen (rectangle, content
    /// hash): an identical re-render is not queued again — unchanged panels
    /// cost no bandwidth and cause no repaint.
    #[cfg(feature = "pixel")]
    pixel_sent: Option<(Rect, u64)>,
    /// When the last full pixel render ran — the self-pacing clock that keeps
    /// redundant re-encodes (hover floods, tick redraws) nearly free.
    #[cfg(feature = "pixel")]
    pixel_pace: Option<std::time::Instant>,
    /// The viewport the last pixel render drew; a change bypasses the pace —
    /// a zoomed or panned window must never show stale.
    #[cfg(feature = "pixel")]
    pixel_view: Option<Viewport>,
}

impl PlotState {
    /// The data coordinates under a terminal cell — hit-testing against the
    /// last stateful render. `None` before the first render or outside the
    /// plot rectangle. Band axes answer in band-index space, time axes in
    /// unix seconds.
    pub fn data_at(&self, column: u16, row: u16) -> Option<(f64, f64)> {
        let mapping = self.mapping.as_ref()?;
        let local_column = column.checked_sub(self.area.x)? as usize;
        let local_row = row.checked_sub(self.area.y)? as usize;
        mapping.data_at(local_column, local_row)
    }

    /// The plot rectangle of the last stateful render, in terminal
    /// coordinates — where overlays and tooltips can anchor.
    pub fn plot_area(&self) -> Option<Rect> {
        let (left, top, columns, rows) = self.mapping.as_ref()?.plot_area()?;
        Some(Rect::new(
            self.area.x.saturating_add(left as u16),
            self.area.y.saturating_add(top as u16),
            columns.min(u16::MAX as usize) as u16,
            rows.min(u16::MAX as usize) as u16,
        ))
    }

    /// The [`Mapping`] cached by the last stateful render.
    pub fn mapping(&self) -> Option<&Mapping> {
        self.mapping.as_ref()
    }

    /// The hover cursor, in terminal coordinates, when it is over the plot
    /// rectangle.
    pub fn cursor(&self) -> Option<(u16, u16)> {
        self.cursor
    }

    /// The data coordinates under the hover cursor.
    pub fn cursor_data(&self) -> Option<(f64, f64)> {
        let (column, row) = self.cursor?;
        self.data_at(column, row)
    }

    /// The viewport the next stateful render applies.
    pub fn viewport(&self) -> Viewport {
        self.view
    }

    /// Replaces the viewport — the escape hatch for hosts running their own
    /// gesture policy over [`Viewport`]'s arithmetic.
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.view = viewport;
    }

    /// Forgets pixel-presentation state: the next pixel render paints fresh
    /// ground and the next [`present_pixels`] re-emits. Call when something
    /// outside the widget disturbed the screen — a view switch away and back,
    /// an overlay that covered the panel.
    #[cfg(feature = "pixel")]
    pub fn invalidate_pixels(&mut self) {
        self.pixels = None;
        self.pixel_area = None;
        self.pixel_pace = None;
        self.pixel_view = None;
        self.pixel_sent = None;
    }

    /// Back to the automatic view on both axes — the host's reset binding.
    pub fn reset_view(&mut self) {
        self.view = self.view.reset();
    }

    /// Zooms in around the center of the visible x window — a keyboard
    /// binding's zoom. Returns whether the view changed.
    pub fn zoom_in(&mut self) -> bool {
        self.zoom_center(ZOOM_IN)
    }

    /// Zooms out around the center of the visible x window.
    pub fn zoom_out(&mut self) -> bool {
        self.zoom_center(ZOOM_OUT)
    }

    /// Pans left by a tenth of the visible x window — a keyboard binding's
    /// pan. Returns whether the view changed; a bands axis never moves.
    pub fn pan_left(&mut self) -> bool {
        self.pan_step(-PAN_STEP)
    }

    /// Pans right by a tenth of the visible x window.
    pub fn pan_right(&mut self) -> bool {
        self.pan_step(PAN_STEP)
    }

    fn pan_step(&mut self, fraction: f64) -> bool {
        let Some(mapping) = self.mapping.clone() else {
            return false;
        };
        if mapping.x_bands().is_some() {
            return false;
        }
        self.view = self.seed_x(&mapping).pan_x(fraction);
        true
    }

    /// Feeds one mouse input through the default gesture grammar. Returns
    /// whether any state changed — `false` means the host can skip a redraw.
    pub fn on_mouse(&mut self, mouse: Mouse) -> bool {
        match mouse {
            Mouse::Moved { column, row } => self.hover(column, row),
            Mouse::Down {
                button: MouseButton::Left,
                column,
                row,
            } => {
                let hovered = self.hover(column, row);
                if self.inside(column, row) {
                    self.drag = Some(Drag::Pan {
                        last: (column, row),
                    });
                    return true;
                }
                hovered
            }
            Mouse::Drag {
                button: MouseButton::Left,
                column,
                row,
            } => {
                let hovered = self.hover(column, row);
                let Some(Drag::Pan { last }) = self.drag else {
                    return hovered;
                };
                self.drag = Some(Drag::Pan {
                    last: (column, row),
                });
                self.pan_by(last, (column, row)) || hovered
            }
            Mouse::Up {
                button: MouseButton::Left,
                ..
            } => {
                let panning = matches!(self.drag, Some(Drag::Pan { .. }));
                if panning {
                    self.drag = None;
                }
                panning
            }
            Mouse::Down {
                button: MouseButton::Right,
                column,
                row,
            } => {
                if self.inside(column, row) {
                    self.drag = Some(Drag::Select {
                        anchor: (column, row),
                        current: (column, row),
                    });
                    return true;
                }
                false
            }
            Mouse::Drag {
                button: MouseButton::Right,
                column,
                row,
            } => {
                let hovered = self.hover(column, row);
                let Some(Drag::Select { anchor, .. }) = self.drag else {
                    return hovered;
                };
                let Some(rect) = self.plot_area() else {
                    return hovered;
                };
                self.drag = Some(Drag::Select {
                    anchor,
                    current: clamp_into(rect, column, row),
                });
                true
            }
            Mouse::Up {
                button: MouseButton::Right,
                ..
            } => {
                let Some(Drag::Select { anchor, current }) = self.drag else {
                    return false;
                };
                self.drag = None;
                self.apply_selection(anchor, current);
                true
            }
            Mouse::ScrollUp { column, row } => self.wheel(ZOOM_IN, column, row),
            Mouse::ScrollDown { column, row } => self.wheel(ZOOM_OUT, column, row),
            Mouse::Down { .. } | Mouse::Drag { .. } | Mouse::Up { .. } => false,
        }
    }

    /// Updates the hover cursor; the cursor exists only over the plot rectangle.
    fn hover(&mut self, column: u16, row: u16) -> bool {
        let cursor = self.inside(column, row).then_some((column, row));
        let changed = cursor != self.cursor;
        self.cursor = cursor;
        changed
    }

    fn inside(&self, column: u16, row: u16) -> bool {
        self.plot_area().is_some_and(|rect| {
            column >= rect.x && column < rect.right() && row >= rect.y && row < rect.bottom()
        })
    }

    /// The wheel: an x zoom anchored at the data under the cursor.
    fn wheel(&mut self, factor: f64, column: u16, row: u16) -> bool {
        if !self.inside(column, row) {
            return false;
        }
        let Some((anchor, _)) = self.data_at(column, row) else {
            return false;
        };
        let Some(mapping) = &self.mapping else {
            return false;
        };
        if mapping.x_bands().is_some() {
            return false;
        }
        self.view = self.seed_x(mapping).zoom_x(factor, anchor);
        true
    }

    fn zoom_center(&mut self, factor: f64) -> bool {
        let Some(mapping) = self.mapping.clone() else {
            return false;
        };
        let Some((left, top, columns, rows)) = mapping.plot_area() else {
            return false;
        };
        if mapping.x_bands().is_some() {
            return false;
        }
        let Some((anchor, _)) = mapping.data_at(left + columns / 2, top + rows / 2) else {
            return false;
        };
        self.view = self.seed_x(&mapping).zoom_x(factor, anchor);
        true
    }

    /// Pans so the data under the cursor follows it, on every continuous axis.
    fn pan_by(&mut self, from: (u16, u16), to: (u16, u16)) -> bool {
        let Some(mapping) = &self.mapping else {
            return false;
        };
        let Some((_, _, columns, rows)) = mapping.plot_area() else {
            return false;
        };
        let dx = f64::from(to.0) - f64::from(from.0);
        let dy = f64::from(to.1) - f64::from(from.1);
        if dx == 0.0 && dy == 0.0 {
            return false;
        }
        // Dragging right pulls the data right, so the window slides left;
        // dragging down pulls the data down, so the window slides up (y grows
        // upward while rows grow downward).
        let mut view = mapping.viewport();
        view = view.pan_x(-dx / columns.max(1) as f64);
        view = view.pan_y(dy / rows.max(1) as f64);
        self.view = view;
        true
    }

    /// Zooms to a released rubber-band selection; a selection under two cells
    /// on a side is a click, not a window, and is discarded.
    fn apply_selection(&mut self, anchor: (u16, u16), current: (u16, u16)) {
        let Some(mapping) = &self.mapping else {
            return;
        };
        if anchor.0.abs_diff(current.0) < 2 || anchor.1.abs_diff(current.1) < 2 {
            return;
        }
        let Some((ax, ay)) = self.data_at(anchor.0, anchor.1) else {
            return;
        };
        let Some((cx, cy)) = self.data_at(current.0, current.1) else {
            return;
        };
        let mut view = mapping.viewport();
        if mapping.x_bands().is_none() {
            view = view.with_x(ax, cx);
        }
        if mapping.y_bands().is_none() {
            view = view.with_y(ay, cy);
        }
        self.view = view;
    }

    /// The current view with x fixed — to the view's own window when one is
    /// already set (so gestures stacked between renders compound instead of
    /// re-reading a stale mapping), otherwise to the mapping's rendered
    /// window — and y left exactly as the host's view holds it, so an x
    /// gesture never silently freezes a still-automatic y axis.
    fn seed_x(&self, mapping: &Mapping) -> Viewport {
        let mut seeded = mapping.viewport();
        if let Some((low, high)) = self.view.x() {
            seeded = seeded.with_x(low, high);
        }
        match self.view.y() {
            Some((low, high)) => seeded.with_y(low, high),
            None => seeded.reset_y(),
        }
    }
}

/// Wheel and keyboard zoom steps: one notch in, its exact inverse out.
const ZOOM_IN: f64 = 0.8;
const ZOOM_OUT: f64 = 1.25;
/// The keyboard pan step, as a fraction of the visible window.
const PAN_STEP: f64 = 0.1;
/// The pixel render's self-pacing window (~30 full renders per second):
/// within it, an unchanged-view render reuses the image already on screen.
#[cfg(feature = "pixel")]
const PIXEL_PACE: std::time::Duration = std::time::Duration::from_millis(33);

/// Presents the pending pixel blocks of a frame — call after
/// `terminal.draw` with every pixel-rendered [`PlotState`] of the current
/// view. The blocks land in one synchronized write (DEC 2026), and each
/// kitty image travels under its panel's stable id, so the terminal swaps
/// the old image for the new atomically — no delete, no blank gap, no
/// flicker, however large the payload. Each block is consumed: a state
/// presents once per render, and a panel whose content is already on
/// screen queues nothing at all.
///
/// This function writes exactly what it is told to the handle it is given —
/// like [`stream::Live`](crate::stream::Live), it never owns the terminal.
/// The host decides *when*: emit on state changes (input handled, data
/// arrived, resize), not on a timer, and the previous transmission stays on
/// screen through quiet frames.
#[cfg(feature = "pixel")]
pub fn present_pixels(
    out: &mut impl std::io::Write,
    graphics: &crate::pixel::Graphics,
    states: &mut [&mut PlotState],
) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let _ = graphics;
    let mut blocks = String::new();
    for state in states.iter_mut() {
        if let Some((rect, block, hash)) = state.pixels.take() {
            let _ = write!(blocks, "\x1b[{};{}H", rect.y + 1, rect.x + 1);
            blocks.push_str(&block);
            state.pixel_sent = Some((rect, hash));
        }
    }
    if blocks.is_empty() {
        return Ok(());
    }
    let mut swap = String::with_capacity(blocks.len() + 32);
    swap.push_str("\x1b[?2026h");
    swap.push_str(&blocks);
    swap.push_str("\x1b[?2026l");
    out.write_all(swap.as_bytes())?;
    out.flush()
}

/// Retires these panels' kitty images — call when leaving a view whose
/// charts were pixel-rendered: kitty images live on their own layer, and the
/// next view's cell repaints cannot cover them. Deletion is by each panel's
/// own id, so images other applications put on the terminal are never
/// touched. Other protocols paint into cells and need nothing; every state's
/// presentation memory is reset either way, so a return to the view starts
/// from fresh ground and a fresh emission.
#[cfg(feature = "pixel")]
pub fn clear_pixels(
    out: &mut impl std::io::Write,
    graphics: &crate::pixel::Graphics,
    states: &mut [&mut PlotState],
) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let mut goodbye = String::new();
    if graphics.protocol == crate::pixel::Protocol::Kitty {
        for state in states.iter() {
            if let Some(id) = state.pixel_id
                && state.pixel_sent.is_some()
            {
                let _ = write!(goodbye, "\x1b_Ga=d,d=I,i={id},q=2\x1b\\");
            }
        }
    }
    for state in states.iter_mut() {
        state.invalidate_pixels();
    }
    if goodbye.is_empty() {
        return Ok(());
    }
    out.write_all(goodbye.as_bytes())?;
    out.flush()
}

/// A content fingerprint for already-on-screen suppression.
#[cfg(feature = "pixel")]
fn block_hash(block: &str) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    block.hash(&mut hasher);
    hasher.finish()
}

/// The next stable kitty image id: unique within the process, seeded from
/// the process id so two applications sharing a terminal are unlikely to
/// collide. Never zero — the protocol reserves it.
#[cfg(feature = "pixel")]
fn next_pixel_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let base = std::process::id().wrapping_mul(0x9E37_79B1);
    let id = base.wrapping_add(COUNTER.fetch_add(1, Ordering::Relaxed));
    if id == 0 { 1 } else { id }
}

/// Clamps a terminal coordinate into a rectangle.
fn clamp_into(rect: Rect, column: u16, row: u16) -> (u16, u16) {
    (
        column.clamp(rect.x, rect.right().saturating_sub(1).max(rect.x)),
        row.clamp(rect.y, rect.bottom().saturating_sub(1).max(rect.y)),
    )
}

impl PlotWidget<'_> {
    /// Draws the interaction chrome over the blitted plot: selection band,
    /// crosshair, readout. Overlays live in the buffer only — the plot value
    /// and its rendering stay byte-identical with or without them.
    fn overlays(&self, buffer: &mut Buffer, state: &PlotState) {
        let Some(rect) = state.plot_area() else {
            return;
        };
        let light = self.theme == Theme::LIGHT;
        let tint = if light {
            RatColor::Gray
        } else {
            RatColor::DarkGray
        };
        if let Some(Drag::Select { anchor, current }) = state.drag {
            let (x0, x1) = (anchor.0.min(current.0), anchor.0.max(current.0));
            let (y0, y1) = (anchor.1.min(current.1), anchor.1.max(current.1));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    buffer[(x, y)].set_bg(tint);
                }
            }
        }
        let cursor = state.cursor().filter(|&(column, row)| {
            column >= rect.x && column < rect.right() && row >= rect.y && row < rect.bottom()
        });
        if self.crosshair
            && let Some((column, row)) = cursor
        {
            for x in rect.x..rect.right() {
                let cell = &mut buffer[(x, row)];
                if cell.bg == RatColor::Reset {
                    cell.set_bg(tint);
                }
            }
            for y in rect.y..rect.bottom() {
                let cell = &mut buffer[(column, y)];
                if cell.bg == RatColor::Reset {
                    cell.set_bg(tint);
                }
            }
        }
        if cursor.is_none() {
            return;
        }
        let snapped = if self.snap {
            self.snapped(state)
        } else {
            Vec::new()
        };
        // The snapped data get their cells marked brighter than the crosshair
        // tint, so the eye lands on the datum the readout is describing.
        let highlight = if light {
            RatColor::DarkGray
        } else {
            RatColor::Gray
        };
        for snap in &snapped {
            if let Some(value) = snap.value
                && let Some(mapping) = state.mapping()
                && let Some((column, row)) = mapping.cell_at(snap.x, value)
            {
                let x = state.area.x.saturating_add(column as u16);
                let y = state.area.y.saturating_add(row as u16);
                if x < rect.right() && y < rect.bottom() {
                    buffer[(x, y)].set_bg(highlight);
                }
            }
        }
        if self.readout
            && let Some(text) = self.readout_line(state, &snapped, rect.width.saturating_sub(2))
        {
            let width = display_width(&text) as u16;
            buffer.set_string(
                rect.right() - width - 1,
                rect.y,
                &text,
                Style::new().fg(highlight),
            );
        }
    }

    /// The readout text, shed to `budget` cells: the x position, then one
    /// `label: value` entry per snapped series (or the plain cursor
    /// coordinates when nothing snapped). One snapped series speaks for the x
    /// part with its own datum's position; several share the cursor's.
    /// Trailing series drop until the line fits; `None` when even the
    /// two-part minimum cannot.
    fn readout_line(&self, state: &PlotState, snapped: &[Snapped], budget: u16) -> Option<String> {
        let (cursor_x, cursor_y) = state.cursor_data()?;
        let mapping = state.mapping()?;
        let x_part = match snapped {
            [only] => mapping.format_x(only.x),
            [] | [..] => mapping.format_x(cursor_x),
        };
        let mut parts = vec![x_part];
        if snapped.is_empty() {
            parts.push(mapping.format_y(cursor_y));
        }
        for snap in snapped {
            let value = snap
                .value
                .map_or_else(|| "—".to_string(), |value| mapping.format_y(value));
            parts.push(match &snap.label {
                Some(label) => format!("{label}: {value}"),
                None => value,
            });
        }
        loop {
            let text = parts.join(" · ");
            if display_width(&text) as u16 <= budget {
                return Some(text);
            }
            if parts.len() <= 2 {
                return None;
            }
            parts.pop();
        }
    }

    /// The datum nearest the cursor's x on every point-backed Line and Points
    /// layer, restricted to the visible x window — the snap targets.
    fn snapped(&self, state: &PlotState) -> Vec<Snapped> {
        let Some((cursor_x, _)) = state.cursor_data() else {
            return Vec::new();
        };
        let Some(mapping) = state.mapping() else {
            return Vec::new();
        };
        let window = mapping.x_domain();
        let mut snapped = Vec::new();
        for layer in self.plot.layers() {
            let (x, y, label) = match layer {
                Mark::Line(line) => match line.channels() {
                    Some((x, y)) => (x, y, line.label.as_ref()),
                    None => continue,
                },
                Mark::Points(points) => (points.x.as_ref(), &points.y, points.label.as_ref()),
                _ => continue,
            };
            let Some(index) = nearest_visible(x.map(Series::as_slice), y.len(), cursor_x, window)
            else {
                continue;
            };
            snapped.push(Snapped {
                label: label.cloned(),
                x: x.map_or(index as f64, |series| series.as_slice()[index]),
                value: y.as_slice().get(index).copied().filter(|v| v.is_finite()),
            });
        }
        snapped
    }
}

/// One snap target: the datum nearest the cursor's x on one layer. A `value`
/// of `None` is a gap at the snapped position — shown as `—`, never
/// interpolated away.
struct Snapped {
    label: Option<String>,
    x: f64,
    value: Option<f64>,
}

/// The index of the datum nearest `target` whose x lies inside the visible
/// `window`: explicit x scans like [`crate::stat::nearest`], implicit indices
/// round in place.
fn nearest_visible(
    x: Option<&[f64]>,
    len: usize,
    target: f64,
    window: (f64, f64),
) -> Option<usize> {
    match x {
        Some(values) => {
            let mut best: Option<(usize, f64)> = None;
            for (index, &value) in values.iter().enumerate() {
                if !value.is_finite() || value < window.0 || value > window.1 {
                    continue;
                }
                let distance = (value - target).abs();
                if best.is_none_or(|(_, nearest)| distance < nearest) {
                    best = Some((index, distance));
                }
            }
            best.map(|(index, _)| index)
        }
        None => {
            let mut index = target.round();
            if index < window.0 {
                index = target.ceil();
            } else if index > window.1 {
                index = target.floor();
            }
            if index < window.0.max(0.0) || index > window.1 {
                return None;
            }
            let index = index as usize;
            (index < len).then_some(index)
        }
    }
}

/// Our color into ratatui's; `Default` keeps the cell's existing style.
fn convert(color: Color) -> Option<RatColor> {
    Some(match color {
        Color::Default => return None,
        Color::Black => RatColor::Black,
        Color::Red => RatColor::Red,
        Color::Green => RatColor::Green,
        Color::Yellow => RatColor::Yellow,
        Color::Blue => RatColor::Blue,
        Color::Magenta => RatColor::Magenta,
        Color::Cyan => RatColor::Cyan,
        Color::White => RatColor::White,
        Color::BrightBlack => RatColor::DarkGray,
        Color::BrightRed => RatColor::LightRed,
        Color::BrightGreen => RatColor::LightGreen,
        Color::BrightYellow => RatColor::LightYellow,
        Color::BrightBlue => RatColor::LightBlue,
        Color::BrightMagenta => RatColor::LightMagenta,
        Color::BrightCyan => RatColor::LightCyan,
        Color::BrightWhite => RatColor::White,
        Color::Ansi256(index) => RatColor::Indexed(index),
        Color::Rgb(r, g, b) => RatColor::Rgb(r, g, b),
    })
}

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod tests;
