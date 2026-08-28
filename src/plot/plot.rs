//! `Plot`: the retained chart description, and its resolve → layout → rasterize
//! pipeline.

use super::frame::Frame;
use super::layout::Layout;
use super::mapping::Mapping;
use super::resolve::{Kind, Reduce, ResolvedLayer};
use crate::mark::{LineStyle, Mark};
use crate::render::Surface;
use crate::scale::Scale;

static DEFAULT_CATEGORICAL_PALETTE: crate::scale::Palette = crate::scale::Palette::OKABE_ITO;

/// A retained chart description: layers of marks plus furniture.
///
/// A plot is a plain value — build it anywhere, clone it, send it across threads,
/// render it many times. Rendering is a pure function of the plot and a [`Frame`]:
/// no global state, no terminal access, no panics (undersized frames shed furniture
/// instead of failing).
///
/// ```
/// use malevich::{Frame, Line, Plot};
///
/// let plot = Plot::new()
///     .layer(Line::xy(&[0.0, 1.0, 2.0][..], &[1.0, 3.0, 2.0][..]))
///     .title("example");
/// let text = plot.render(&Frame::plain(40, 10));
/// assert!(text.contains("example"));
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plot<'a> {
    #[cfg_attr(feature = "serde", serde(default))]
    layers: Vec<Mark<'a>>,
    #[cfg_attr(feature = "serde", serde(default))]
    title: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    x: Scale,
    #[cfg_attr(feature = "serde", serde(default))]
    y: Scale,
    #[cfg_attr(feature = "serde", serde(default))]
    x_label: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    y_label: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    x_domain: Option<(f64, f64)>,
    #[cfg_attr(feature = "serde", serde(default))]
    y_domain: Option<(f64, f64)>,
    #[cfg_attr(feature = "serde", serde(default))]
    colorbar: bool,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    palette: Option<crate::scale::Palette>,
}

/// The few choices that genuinely differ between cell and device-pixel targets.
#[derive(Clone, Copy)]
struct TargetPolicy {
    density: (usize, usize),
    downsample: bool,
    cycle_markers: bool,
    pixel_lines: bool,
    sample_width_name: &'static str,
}

impl TargetPolicy {
    fn cells(frame: &Frame, downsample: bool) -> Self {
        Self {
            density: frame.charset.pixels_per_cell(),
            downsample,
            cycle_markers: frame.color == crate::render::ColorMode::Plain,
            pixel_lines: false,
            sample_width_name: "cell sample width",
        }
    }

    #[cfg(feature = "pixel")]
    fn pixels(density: (usize, usize)) -> Self {
        Self {
            density,
            downsample: true,
            cycle_markers: false,
            pixel_lines: true,
            sample_width_name: "pixel sample width",
        }
    }
}

/// Target-independent output of resolve → probe → layout → final resolution.
struct PreparedRender<'p> {
    layout: Layout<'p>,
    layers: Vec<ResolvedLayer<'p>>,
}

impl<'a> Plot<'a> {
    /// An empty plot with no layers and no furniture.
    pub fn new() -> Plot<'a> {
        Plot {
            layers: Vec::new(),
            title: None,
            x: Scale::Auto,
            y: Scale::Auto,
            x_label: None,
            y_label: None,
            x_domain: None,
            y_domain: None,
            colorbar: false,
            palette: None,
        }
    }

    /// Replaces the categorical color scale `color_by` channels draw from;
    /// [`Palette::OKABE_ITO`](crate::scale::Palette::OKABE_ITO) by default.
    #[must_use]
    pub fn palette(mut self, palette: crate::scale::Palette) -> Plot<'a> {
        self.palette = Some(palette);
        self
    }

    /// Fixes the x axis to `[min, max]` instead of fitting the data — matplotlib's
    /// `xlim`. Data outside clips honestly. Ignored on a bands axis.
    ///
    /// # Panics
    ///
    /// Panics if the bounds are not finite.
    #[must_use]
    pub fn x_domain(mut self, min: f64, max: f64) -> Plot<'a> {
        assert!(
            min.is_finite() && max.is_finite(),
            "Plot::x_domain requires finite bounds"
        );
        self.x_domain = Some((min.min(max), max.max(min)));
        self
    }

    /// Fixes the y axis to `[min, max]` instead of fitting the data — matplotlib's
    /// `ylim`. Data outside clips honestly.
    ///
    /// # Panics
    ///
    /// Panics if the bounds are not finite.
    #[must_use]
    pub fn y_domain(mut self, min: f64, max: f64) -> Plot<'a> {
        assert!(
            min.is_finite() && max.is_finite(),
            "Plot::y_domain requires finite bounds"
        );
        self.y_domain = Some((min.min(max), max.max(min)));
        self
    }

    /// Sets the x axis scale. Under [`Scale::Auto`] (the default) a bars or
    /// band-range layer makes the axis categorical; any scale set here is honored
    /// as-is, so an explicit choice is never overridden by a categorical layer.
    #[must_use]
    pub fn x_scale(mut self, scale: Scale) -> Plot<'a> {
        self.x = scale;
        self
    }

    /// Sets the y axis scale.
    ///
    /// [`Scale::Bands`] labels the rows: continuous marks position y against
    /// band indices (0 is the top band), and a Cells matrix maps row k onto
    /// band k — the confusion-matrix and attention-map axis.
    #[must_use]
    pub fn y_scale(mut self, scale: Scale) -> Plot<'a> {
        self.y = scale;
        self
    }

    /// Sugar for [`Plot::x_scale`] with [`Scale::Time`]: unix seconds (UTC) with
    /// calendar-aligned, multi-scale tick labels (`14:05`, `Aug 2`, `2027`).
    #[must_use]
    pub fn time_x(self) -> Plot<'a> {
        self.x_scale(Scale::Time)
    }

    /// Sugar for [`Plot::x_scale`] with [`Scale::Log`]: decade ticks, and values at
    /// or below zero become gaps — a log axis cannot place them honestly.
    #[must_use]
    pub fn log_x(self) -> Plot<'a> {
        self.x_scale(Scale::Log)
    }

    /// Sugar for [`Plot::y_scale`] with [`Scale::Log`]: decade ticks, and values at
    /// or below zero become gaps — a log axis cannot place them honestly.
    #[must_use]
    pub fn log_y(self) -> Plot<'a> {
        self.y_scale(Scale::Log)
    }

    /// Shows a colorbar: a vertical strip of the colormap down the right edge,
    /// labeled with the value range it encodes. Applies to the plot's first
    /// [`Cells`](crate::Cells) layer (heatmaps, 2D histograms); ignored when there is
    /// none, or when the frame is too narrow to spare the room.
    #[must_use]
    pub fn colorbar(mut self) -> Plot<'a> {
        self.colorbar = true;
        self
    }

    /// Titles the x axis, centered under its tick labels.
    #[must_use]
    pub fn x_label(mut self, label: impl Into<String>) -> Plot<'a> {
        self.x_label = Some(label.into());
        self
    }

    /// Titles the y axis, written vertically along the left edge.
    #[must_use]
    pub fn y_label(mut self, label: impl Into<String>) -> Plot<'a> {
        self.y_label = Some(label.into());
        self
    }

    /// The retained layers, in draw order. In-crate presentation (the ratatui
    /// widget's snap readout) reads series data through this.
    #[cfg(feature = "ratatui")]
    pub(crate) fn layers(&self) -> &[Mark<'a>] {
        &self.layers
    }

    /// Adds a mark as the next layer. Layers share scales: domains are the union of
    /// all layers' data, resolved at render time. A [`crate::mark::Bars`] layer puts
    /// a band scale on the x axis; other layers then position x against category
    /// indices (0 is the first band's center).
    #[must_use]
    pub fn layer(mut self, mark: impl Into<Mark<'a>>) -> Plot<'a> {
        self.layers.push(mark.into());
        self
    }

    /// Sets the title, shown centered above the plot (shed first when space runs out).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Plot<'a> {
        self.title = Some(title.into());
        self
    }

    /// Applies a [`Viewport`](crate::Viewport): a fixed window overrides that
    /// axis's domain, an automatic axis leaves the plot's own setting — data
    /// fit, or a domain the plot already fixed — untouched. Sugar over
    /// [`Plot::x_domain`]/[`Plot::y_domain`], which is the point: an
    /// interactive view is a scale option, not a render mode.
    #[must_use]
    pub fn viewport(mut self, viewport: &crate::plot::Viewport) -> Plot<'a> {
        if let Some(window) = viewport.x() {
            self.x_domain = Some(window);
        }
        if let Some(window) = viewport.y() {
            self.y_domain = Some(window);
        }
        self
    }

    /// The resolved geometry of this plot in `frame`, without rendering: the
    /// plot rectangle and the cell ↔ data mapping, from the same resolve →
    /// layout pass rendering runs. Pure — same plot and frame, same mapping.
    ///
    /// This is the physics interactive hosts build on: hit-test a cursor with
    /// [`Mapping::data_at`], anchor an overlay with [`Mapping::cell_at`], seed
    /// zoom and pan with [`Mapping::viewport`]. A frame too small to draw a
    /// plot panel yields a mapping whose positional queries return `None`.
    pub fn mapping(&self, frame: &Frame) -> Mapping {
        if frame.width == 0 || frame.height == 0 {
            return Mapping::empty();
        }
        match self.prepare_render(frame, TargetPolicy::cells(frame, true)) {
            Ok(prepared) => Mapping::new(&prepared.layout, &self.x, &self.y),
            Err(_) => Mapping::empty(),
        }
    }

    /// Detaches from any borrowed storage, making the plot `'static`.
    pub fn into_owned(self) -> Plot<'static> {
        Plot {
            layers: self.layers.into_iter().map(Mark::into_owned).collect(),
            title: self.title,
            x: self.x,
            y: self.y,
            x_label: self.x_label,
            y_label: self.y_label,
            x_domain: self.x_domain,
            y_domain: self.y_domain,
            colorbar: self.colorbar,
            palette: self.palette,
        }
    }

    /// Renders into a string according to the frame's charset and color mode.
    pub fn render(&self, frame: &Frame) -> String {
        self.try_render_unvalidated(frame).unwrap_or_default()
    }

    /// Renders the complete plot as a self-contained HTML terminal card.
    ///
    /// The cell grid is placed in a styled `<pre>` element: default-colored
    /// chrome inherits the card foreground, while mark colors become concrete RGB
    /// spans. The frame's color mode is ignored because HTML always carries RGB;
    /// its size, charset, and theme still apply. Rendering is pure and deterministic
    /// for a given plot and frame.
    #[cfg(feature = "evcxr")]
    pub fn to_html(&self, frame: &Frame) -> String {
        use std::fmt::Write as _;

        let content = self.rasterize(frame).encode_html();
        let (background, foreground) = crate::evcxr::card_colors(frame.theme);
        let mut html = String::with_capacity(content.len() + 320);
        let _ = write!(
            html,
            "<pre style=\"margin:0;padding:12px 16px;border:0;border-radius:8px;box-sizing:border-box;display:inline-block;max-width:100%;overflow-x:auto;white-space:pre;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;line-height:1.1;font-variant-ligatures:none;font-feature-settings:\"liga\" 0,\"calt\" 0;background-color:{background};color:{foreground}\">{content}</pre>"
        );
        html
    }

    /// Displays this plot when it is the last expression in an Evcxr cell.
    ///
    /// Emits two representations and lets the frontend pick the richest it can
    /// draw: an HTML card (100×26 quadrants, dark theme) for Jupyter, and a terminal
    /// plot (80×24) for the terminal REPL, which cannot render HTML and would
    /// otherwise show nothing. With the `pixel` feature also enabled, the terminal
    /// block becomes a real sixel/kitty/iTerm2 image in a graphics-capable terminal
    /// (detected explicitly for Evcxr's stdout destination; its pipe prevents an
    /// active tty probe, so environment sniffing supplies the fallback) and stays
    /// cells everywhere else. Use [`Plot::to_html`] with a custom [`Frame`] for
    /// explicit size, charset, or theme control.
    #[cfg(feature = "evcxr")]
    pub fn evcxr_display(&self) {
        let html = self.to_html(&Frame::portable(100, 26));
        let terminal = Frame::portable(80, 24);
        #[cfg(feature = "pixel")]
        let plain = self.render_with_capabilities(
            &terminal,
            &crate::pixel::Capabilities::detect_for(&std::io::stdout()),
        );
        #[cfg(not(feature = "pixel"))]
        let plain = self.render(&terminal);
        println!(
            "{}",
            crate::evcxr::mime_bundle(&[("text/html", &html), ("text/plain", &plain)])
        );
    }

    /// Checks the spec against the invariants the constructors enforce — paired
    /// channel lengths, rectangular grids, valid colormaps — plus finite manual
    /// domains and scale/domain compatibility, without rendering. Returns the first
    /// problem as an [`Error`](crate::Error).
    ///
    /// [`Plot::render`] never fails: it sheds whatever it cannot draw. `validate` is
    /// the strict counterpart for a spec that arrived by deserialization or
    /// configuration, where you want a typed error rather than a quietly dropped
    /// mark. [`Plot::try_render`] does both in one call.
    ///
    /// ```
    /// let plot = malevich::line(&[1.0, 2.0, 3.0][..]);
    /// assert!(plot.validate().is_ok());
    /// ```
    pub fn validate(&self) -> crate::Result<()> {
        for layer in &self.layers {
            layer.validate()?;
        }
        if let Some(palette) = &self.palette {
            palette.validate()?;
        }
        for scale in [&self.x, &self.y] {
            if let Scale::Bands(categories) = scale
                && categories.is_empty()
            {
                return Err(crate::Error::EmptyDimension {
                    what: "Bands categories",
                });
            }
        }
        // Categorical layers must agree on one ordered set of bands, and a numeric x
        // scale cannot host them — `Auto` adapts, but an explicit numeric choice is a
        // conflict, not an override.
        let mut bands: Option<&[String]> = match &self.x {
            Scale::Bands(bands) => Some(bands.as_slice()),
            _ => None,
        };
        for layer in &self.layers {
            let layer_bands = match layer {
                Mark::Bars(bars) => match &bars.placement {
                    crate::mark::Placement::Bands(bands) => Some(bands.as_slice()),
                    _ => None,
                },
                Mark::Range(range) => match &range.placement {
                    crate::mark::RangePlacement::Bands(bands) => Some(bands.as_slice()),
                    _ => None,
                },
                _ => None,
            };
            let Some(layer_bands) = layer_bands else {
                continue;
            };
            if matches!(self.x, Scale::Linear | Scale::Log | Scale::Time) {
                return Err(crate::Error::IncompatibleScale {
                    detail: "a categorical layer needs an Auto or Bands x scale",
                });
            }
            match bands {
                Some(existing) if existing != layer_bands => {
                    return Err(crate::Error::IncompatibleScale {
                        detail: "categorical layers disagree on their bands",
                    });
                }
                _ => bands = Some(layer_bands),
            }
        }
        let categorical_x = match &self.x {
            Scale::Bands(_) => true,
            Scale::Auto => bands.is_some_and(|categories| !categories.is_empty()),
            _ => false,
        };
        for layer in &self.layers {
            match layer {
                Mark::Bars(bars) => {
                    if matches!(self.y, Scale::Log) {
                        return Err(crate::Error::IncompatibleScale {
                            detail: "Bars has a zero baseline and cannot use a log y axis",
                        });
                    }
                    if matches!(self.y, Scale::Bands(_)) {
                        return Err(crate::Error::IncompatibleScale {
                            detail: "Bars encode a numeric length and cannot use a Bands y axis",
                        });
                    }
                    if categorical_x
                        && matches!(bars.placement, crate::mark::Placement::Spans { .. })
                    {
                        return Err(crate::Error::IncompatibleScale {
                            detail: "numeric-span Bars needs a continuous x scale",
                        });
                    }
                }
                Mark::Area(area) if area.low.is_none() && !area.horizontal => {
                    if matches!(self.y, Scale::Log) {
                        return Err(crate::Error::IncompatibleScale {
                            detail: "a zero-baseline Area cannot use a log y axis",
                        });
                    }
                }
                Mark::Area(area) if area.low.is_none() && area.horizontal => {
                    if matches!(self.x, Scale::Log) {
                        return Err(crate::Error::IncompatibleScale {
                            detail: "a horizontal zero-baseline Area cannot use a log x axis",
                        });
                    }
                }
                Mark::Cells(cells) => {
                    let rows = cells.rows();
                    // On a band axis the grid index is the band index, so the
                    // counts must agree and data-coordinate extents cannot apply.
                    if categorical_x {
                        if cells.extents.is_some() {
                            return Err(crate::Error::IncompatibleScale {
                                detail: "Cells on a Bands x axis maps columns to bands and cannot take extents",
                            });
                        }
                        if bands.map_or(0, <[String]>::len) != cells.columns {
                            return Err(crate::Error::IncompatibleScale {
                                detail: "Cells columns must match the x axis bands",
                            });
                        }
                    }
                    if let Scale::Bands(categories) = &self.y {
                        if cells.extents.is_some() {
                            return Err(crate::Error::IncompatibleScale {
                                detail: "Cells on a Bands y axis maps rows to bands and cannot take extents",
                            });
                        }
                        if categories.len() != rows {
                            return Err(crate::Error::IncompatibleScale {
                                detail: "Cells rows must match the y axis bands",
                            });
                        }
                    }
                    let (x, y) = cells
                        .extents
                        .unwrap_or(((0.0, cells.columns as f64), (0.0, rows as f64)));
                    if matches!(self.x, Scale::Log) && (x.0 <= 0.0 || x.1 <= 0.0) {
                        return Err(crate::Error::IncompatibleScale {
                            detail: "Cells on a log x axis needs positive x extents",
                        });
                    }
                    if matches!(self.y, Scale::Log) && (y.0 <= 0.0 || y.1 <= 0.0) {
                        return Err(crate::Error::IncompatibleScale {
                            detail: "Cells on a log y axis needs positive y extents",
                        });
                    }
                }
                _ => {}
            }
        }
        for (axis, domain) in [("x", self.x_domain), ("y", self.y_domain)] {
            if let Some((lo, hi)) = domain {
                if !(lo.is_finite() && hi.is_finite()) {
                    return Err(crate::Error::NonFiniteDomain { axis });
                }
                if lo > hi {
                    return Err(crate::Error::InvalidParameter {
                        detail: "manual axis domains must be ascending",
                    });
                }
            }
        }
        if matches!(self.x, Scale::Log)
            && let Some((lo, hi)) = self.x_domain
            && (lo <= 0.0 || hi <= 0.0)
        {
            return Err(crate::Error::IncompatibleScale {
                detail: "a log x axis needs a positive domain",
            });
        }
        if matches!(self.y, Scale::Log)
            && let Some((lo, hi)) = self.y_domain
            && (lo <= 0.0 || hi <= 0.0)
        {
            return Err(crate::Error::IncompatibleScale {
                detail: "a log y axis needs a positive domain",
            });
        }
        Ok(())
    }

    /// [`Plot::validate`] followed by fallible rasterization and encoding: a
    /// rendered string, or the first spec, geometry, or allocation error.
    pub fn try_render(&self, frame: &Frame) -> crate::Result<String> {
        self.validate()?;
        self.try_render_unvalidated(frame)
    }

    pub(crate) fn try_render_unvalidated(&self, frame: &Frame) -> crate::Result<String> {
        self.try_rasterize(frame)?.try_encode(frame.color)
    }

    /// Renders into `frame` at the best graphics tier the terminal offers: with
    /// the `pixel` feature enabled and a protocol detected
    /// ([`Graphics::detect`](crate::pixel::Graphics::detect)), the plot panel
    /// becomes a real image; everywhere else — pipes, unknown terminals, tmux,
    /// or without the feature — exactly [`Plot::render`]. The one-call top of
    /// the resolution ladder for CLIs that already know their frame.
    ///
    /// Unlike [`Plot::render`] this consults the environment, so it is not
    /// deterministic across terminals; keep `render` for tests and snapshots.
    ///
    /// ```no_run
    /// let plot = malevich::line(&[1.0, 3.0, 2.0][..]);
    /// println!("{}", plot.render_best(&malevich::Frame::detect()));
    /// ```
    pub fn render_best(&self, frame: &Frame) -> String {
        #[cfg(feature = "pixel")]
        return self.render_with_capabilities(frame, &crate::pixel::Capabilities::detect());
        #[cfg(not(feature = "pixel"))]
        self.render(frame)
    }

    /// The fallible counterpart of [`Plot::render_best`]. It validates the plot
    /// and returns geometry/allocation errors instead of degrading to empty output.
    pub fn try_render_best(&self, frame: &Frame) -> crate::Result<String> {
        #[cfg(feature = "pixel")]
        return self.try_render_with_capabilities(frame, &crate::pixel::Capabilities::detect());
        #[cfg(not(feature = "pixel"))]
        self.try_render(frame)
    }

    /// Renders at the best tier allowed by an explicit capability context.
    ///
    /// The first advertised pixel protocol is used at the detected cell size;
    /// when `capabilities` contains no pixel protocol this is exactly
    /// [`Plot::render`]. Unlike [`Plot::render_best`], this method never reads the
    /// process environment or touches a terminal. It is the auto-render path for
    /// stderr, tests, and applications managing more than one terminal.
    #[cfg(feature = "pixel")]
    pub fn render_with_capabilities(
        &self,
        frame: &Frame,
        capabilities: &crate::pixel::Capabilities,
    ) -> String {
        match capabilities.best() {
            Some(graphics) => self.render_pixels(frame, &graphics),
            None => self.render(frame),
        }
    }

    /// The fallible counterpart of [`Plot::render_with_capabilities`].
    #[cfg(feature = "pixel")]
    pub fn try_render_with_capabilities(
        &self,
        frame: &Frame,
        capabilities: &crate::pixel::Capabilities,
    ) -> crate::Result<String> {
        match capabilities.best() {
            Some(graphics) => self.try_render_pixels(frame, &graphics),
            None => self.try_render(frame),
        }
    }

    /// Renders with the plot panel as a real image (feature `pixel`): chrome —
    /// title, axes, tick labels, legend — as text cells exactly like
    /// [`Plot::render`], and the plot rectangle as device-pixel graphics in the
    /// protocol `graphics` names. The returned string weaves both with relative
    /// cursor movement; print it to a terminal that speaks the protocol.
    ///
    /// Deterministic like every render path: no terminal is touched, and the
    /// same plot, frame, and graphics always produce the same string.
    #[cfg(feature = "pixel")]
    pub fn render_pixels(&self, frame: &Frame, graphics: &crate::pixel::Graphics) -> String {
        crate::pixel::render(self, frame, graphics, 0)
    }

    /// Validates and renders a hybrid pixel plot, returning bounded geometry or
    /// allocation failures as typed errors.
    #[cfg(feature = "pixel")]
    pub fn try_render_pixels(
        &self,
        frame: &Frame,
        graphics: &crate::pixel::Graphics,
    ) -> crate::Result<String> {
        self.validate()?;
        crate::pixel::try_render(self, frame, graphics, 0)
    }

    /// [`Plot::render_pixels`], anchored `column` cells from the left edge:
    /// every text row and the image cursor walk start with an absolute-column
    /// jump, so printing the block leaves anything to its left untouched. For
    /// hosts pasting plots side by side — print the left content, move the
    /// cursor back to the block's top row, print this. Rows stay relative;
    /// only columns are absolute, so scrollback is safe.
    #[cfg(feature = "pixel")]
    pub fn render_pixels_at(
        &self,
        frame: &Frame,
        graphics: &crate::pixel::Graphics,
        column: usize,
    ) -> String {
        crate::pixel::render(self, frame, graphics, column)
    }

    /// The fallible counterpart of [`Plot::render_pixels_at`].
    #[cfg(feature = "pixel")]
    pub fn try_render_pixels_at(
        &self,
        frame: &Frame,
        graphics: &crate::pixel::Graphics,
        column: usize,
    ) -> crate::Result<String> {
        self.validate()?;
        crate::pixel::try_render(self, frame, graphics, column)
    }

    /// Runs the target-independent render orchestration once. The returned
    /// layers still borrow the retained plot; layout borrows only retained band
    /// labels, never the temporary probe or final layer vector.
    fn prepare_render<'p>(
        &'p self,
        frame: &Frame,
        policy: TargetPolicy,
    ) -> crate::Result<PreparedRender<'p>> {
        let sample_width =
            frame
                .width
                .checked_mul(policy.density.0)
                .ok_or(crate::Error::DimensionTooLarge {
                    what: policy.sample_width_name,
                    requested: usize::MAX,
                    limit: crate::render::MAX_DEVICE_PIXELS,
                })?;
        let title = self.title.is_some();
        let scales = (&self.x, &self.y);
        let labels = (self.x_label.as_deref(), self.y_label.as_deref());
        let domains = (self.x_domain, self.y_domain);
        let layer_palette = &frame.theme.palette;
        let categorical = self
            .palette
            .as_ref()
            .unwrap_or(&DEFAULT_CATEGORICAL_PALETTE);

        // M4 must bucket by the rendered column, but that column mapping comes
        // from layout. Probe with independent channel extents, retain its layout,
        // then resolve the final scene in exactly that raster space.
        let extent = Reduce::Extent {
            x_positive: matches!(&self.x, Scale::Log),
            y_positive: matches!(&self.y, Scale::Log),
        };
        let probe = policy.downsample.then(|| {
            super::resolve::resolve(
                &self.layers,
                sample_width,
                layer_palette,
                categorical,
                policy.cycle_markers,
                extent,
            )
        });
        let probed_layout = probe.as_ref().map(|probe| {
            Layout::compute(
                frame,
                policy.density,
                probe,
                title,
                scales,
                labels,
                domains,
                self.colorbar,
            )
        });
        let reduce = match &probed_layout {
            Some(layout) => Reduce::Mapped {
                map: layout.x_scale,
                columns: layout.plot_sub_w,
            },
            None => Reduce::None,
        };

        let mut layers = super::resolve::resolve(
            &self.layers,
            sample_width,
            layer_palette,
            categorical,
            policy.cycle_markers,
            reduce,
        );
        if policy.pixel_lines {
            // Corners is cell-glyph art; on a pixel canvas the honest line is
            // the line itself.
            for layer in &mut layers {
                if let ResolvedLayer::Series { kind, .. } = layer
                    && let Kind::Line { style, .. } = kind
                    && *style == LineStyle::Corners
                {
                    *style = LineStyle::Pixels;
                }
            }
        }
        let layout = probed_layout.unwrap_or_else(|| {
            Layout::compute(
                frame,
                policy.density,
                &layers,
                title,
                scales,
                labels,
                domains,
                self.colorbar,
            )
        });
        Ok(PreparedRender { layout, layers })
    }

    /// Rasterizes for hybrid pixel output: chrome on a cell surface, marks on a
    /// device-pixel canvas at `cell` pixels per cell, one shared layout — so the
    /// scales map into device pixels and M4 buckets per pixel column.
    #[cfg(feature = "pixel")]
    pub(crate) fn try_rasterize_hybrid(
        &self,
        frame: &Frame,
        cell: (usize, usize),
        stroke: Option<u8>,
    ) -> crate::Result<(Surface, crate::pixel::PixelCanvas, crate::render::PlotRect)> {
        use crate::render::PlotRect;

        let mut surface = Surface::try_new(frame.width, frame.height, frame.charset)?;
        let canvas = crate::pixel::PixelCanvas::try_new(frame.width, frame.height, cell, stroke)?;
        let empty = PlotRect {
            gutter: 0,
            top: 0,
            columns: 0,
            rows: 0,
        };
        if frame.width == 0 || frame.height == 0 || cell.0 == 0 || cell.1 == 0 {
            return Ok((surface, canvas, empty));
        }
        let mut canvas = canvas;
        let labels = (self.x_label.as_deref(), self.y_label.as_deref());
        let PreparedRender { layout, layers } =
            self.prepare_render(frame, TargetPolicy::pixels(cell))?;
        super::chrome::draw(
            &mut surface,
            &layout,
            self.title.as_deref(),
            labels,
            &layers,
        );
        super::draw::layers(&mut canvas, &layout, &layers);
        let rect = PlotRect {
            gutter: layout.gutter,
            top: layout.plot_top,
            columns: layout.plot_cols,
            rows: layout.plot_rows,
        };
        Ok((surface, canvas, rect))
    }

    #[cfg_attr(
        not(any(test, feature = "evcxr", feature = "ratatui")),
        allow(dead_code)
    )]
    #[cfg(any(feature = "evcxr", feature = "ratatui"))]
    pub(crate) fn rasterize(&self, frame: &Frame) -> Surface {
        self.try_rasterize(frame)
            .unwrap_or_else(|_| Surface::new(0, 0, frame.charset))
    }

    /// Rasterizes and returns the resolved [`Mapping`] from the same pass — the
    /// one-render path the ratatui stateful widget caches its geometry from.
    #[cfg(feature = "ratatui")]
    pub(crate) fn rasterize_mapped(&self, frame: &Frame) -> (Surface, Mapping) {
        self.try_rasterize_with(frame, true)
            .unwrap_or_else(|_| (Surface::new(0, 0, frame.charset), Mapping::empty()))
    }

    pub(crate) fn try_rasterize(&self, frame: &Frame) -> crate::Result<Surface> {
        self.try_rasterize_with(frame, true)
            .map(|(surface, _)| surface)
    }

    /// Rasterizes with M4 line downsampling optionally disabled. With `downsample`
    /// false, large line layers draw every point — the raw raster that M4 must
    /// reproduce, used as a test oracle for the aggregate-to-raster claim.
    #[cfg(test)]
    pub(crate) fn rasterize_with(&self, frame: &Frame, downsample: bool) -> Surface {
        self.try_rasterize_with(frame, downsample)
            .map(|(surface, _)| surface)
            .unwrap_or_else(|_| Surface::new(0, 0, frame.charset))
    }

    fn try_rasterize_with(
        &self,
        frame: &Frame,
        downsample: bool,
    ) -> crate::Result<(Surface, Mapping)> {
        let mut surface = Surface::try_new(frame.width, frame.height, frame.charset)?;
        if frame.width == 0 || frame.height == 0 {
            return Ok((surface, Mapping::empty()));
        }
        let labels = (self.x_label.as_deref(), self.y_label.as_deref());
        let PreparedRender { layout, layers } =
            self.prepare_render(frame, TargetPolicy::cells(frame, downsample))?;
        let mapping = Mapping::new(&layout, &self.x, &self.y);
        super::chrome::draw(
            &mut surface,
            &layout,
            self.title.as_deref(),
            labels,
            &layers,
        );
        super::draw::layers(&mut surface, &layout, &layers);
        Ok((surface, mapping))
    }
}

impl std::fmt::Display for Plot<'_> {
    /// Renders with [`Frame::detect`]: the one-line `println!("{plot}")` path.
    /// Detection assumes stdout; for full control use [`Plot::render`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render(&Frame::detect()))
    }
}

#[cfg(test)]
#[path = "tests/plot_tests.rs"]
mod tests;
