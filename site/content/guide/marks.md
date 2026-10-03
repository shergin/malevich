# The eight marks

A mark is a family of geometric primitives that draw data. There are eight, joined under the closed `Mark` enum. The family is declared complete. A chart type is a composition of marks. It is not a peer of them. Each mark has position channels (its constructor arguments) and constant channels (builder methods). This page shows every one with a plate.

Marks draw onto a subpixel surface — 2×4 dots per cell in braille, 2×4 blocks in octants, 2×2 in quadrants, 1×1 in ASCII — and a charset codec turns each cell into a glyph. Text shares the grid and wins over pixels. Drawing does not fail. Off the surface, it clips. A non-finite coordinate draws nothing.

## Line

[`Line` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/struct.Line.html).

Points in order, paired series, or a sampled function. `Line::y(values)` plots against the index. `Line::xy(x, y)` pairs two series. `Line::function(domain, f)` samples a closure once per subpixel column, so there is no resolution to choose.

{{figure mark_line_styles}}

`LineStyle::Pixels` is the subpixel default. `LineStyle::Corners` is the asciichart look — box-drawing corners, one glyph per column — with real axes underneath, which the original never had. `dash` takes `Dashed` or `Dotted`. `glow` thickens a line. `color` sets a constant color, and `label` puts the layer in the legend.

{{figure mark_line_function}}

`grade` colors the line by a third series through a colormap — a route by its altitude, a run by its temperature — and adds a colorbar when the plot asks for one.

{{figure mark_line_grade}}

Large lines reduce automatically. Past four points per rendered column, the plot inserts M4 — first, last, minimum, and maximum per column — which is pixel-identical to drawing every point ([the full draw is the oracle](../../principles/full-draw-oracle/)). A `NaN` breaks the line, at every reduction level. That break is path topology.

## Points

[`Points` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/struct.Points.html).

A scatter. `Points::y` and `Points::xy` mirror `Line`. `style` picks a marker. `opacity` fades dense clouds. `density`, on the pixel canvas, shades by count.

{{figure mark_points_styles}}

The five styles are also the shapes `color_by` cycles through when the output has no color, so a grouped scatter piped into a log keeps its groups apart.

{{figure mark_points_color_by}}

## Bars

[`Bars` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/struct.Bars.html).

Bars rise from the zero baseline, or from a per-bar `base`. Four placements: bands (one bar per category), contiguous spans on a numeric axis, free positions, and explicit intervals.

{{figure mark_bars_bands}}

{{figure mark_bars_spans}}

`Bars::spans(start, width, values)` is the histogram's geometry: bin *i* runs from `start + i·width`. `Bars::intervals(starts, ends, values)` draws each bar between its own edges — bins of unequal width, or calendar months of their true length.

{{figure mark_bars_intervals}}

`base` is the y2-style channel. With it, stacked bars, grouped bars, and waterfalls are plain compositions. They are not modes. Stack by giving the second layer the first layer's values as its base. Group by placing layers side by side with `Bars::at`, at positions `stat::dodge` computes ([the statistics layer](../stats/#stack-and-dodge)).

{{figure mark_bars_base}}

`horizontal` turns any placement sideways — the bands run down the y axis in reading order and the values along x, the `barh` of the catalog. Long category names then take the measured label gutter. A band's width does not hold them.

{{figure mark_bars_horizontal}}

`color_by` on bars colors each bar by a category and builds the legend, the same channel as on points and lines.

## Area

[`Area` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/struct.Area.html).

A fill. `Area::y` and `Area::xy` fill from the baseline. `Area::between(x, low, high)` fills a band between two series — a confidence band, a p10–p90 envelope, one layer of a stacked area. `opacity` is a pixel-target channel. On a sixel, kitty, or iTerm2 panel it scales the fill's coverage, so the background and the layers beneath read through. On cells the fill stays solid, so a wash under a line is a dark explicit color there.

{{figure mark_area}}

`Area::horizontal(y, x_low, x_high)` fills along y. A violin is two of these — a density and its mirror — which is exactly how the `violin` preset is expanded.

{{figure mark_area_horizontal}}

## Cells

[`Cells` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/struct.Cells.html).

One geometry, three color readings: a value grid under a colormap, an RGB image, or categorical class regions. A heatmap is not a mark. It is `Cells` under a colormap, and the `heatmap` preset says so.

{{figure mark_cells_matrix}}

`Cells::matrix(columns, values)` takes a row-major grid. Put `Scale::bands` on both axes and the rows are labeled in matrix order — row 0 at the top — which is what a confusion matrix or an attention map needs. `colormap` chooses the ramp. `Plot::colorbar` draws its legend.

{{figure mark_cells_rgb}}

`Cells::rgb(columns, pixels)` takes direct colors: an image, a convolution filter bank, a color-opponency map. In a plain pipe it degrades to a luma shade ramp.

{{figure mark_cells_classes}}

`Cells::classes(columns, labels)` colors each cell by its class through the categorical palette, keeps a stable shade per class, and puts matching swatches in the legend. A decision boundary, a waffle, a land-use map.

{{figure mark_cells_extents}}

`extents` places the grid in data coordinates so it can share axes with points and lines. A grid denser than the raster reduces bucket-exactly. Every screen bucket owns the cells whose centers fall inside it and shows a declared reduction over all of them — the mean box filter by default, `reduce(Reducer::Max)` when the sparse spikes are the point. Nothing is dropped because a sampler stepped over it. `smooth` interpolates on the pixel canvas.

## Range

[`Range` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/struct.Range.html).

An interval per position, with two optional channels inside it: a thick `body` sub-interval and a `marker` crossbar. `Range::xy(x, low, high)` is an error bar at each x. `Range::y(low, high)` uses the index. `Range::over(categories, low, high)` puts one interval per band.

{{figure mark_range_xy}}

Whiskers plus a body from the first to the third quartile plus a marker at the median is a box plot, and that is the whole expansion of the `box_plot` preset — the statistics come from `stat::BoxStats`. Candlesticks are the same mark with `color_by` splitting up days from down days.

{{figure mark_range_over}}

## Rule

[`Rule` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/struct.Rule.html).

A reference line at one value: `Rule::h(y)` or `Rule::v(x)`, optionally dashed and labeled. And a span: `Rule::h_span(y0, y1)` or `Rule::v_span(x0, x1)` washes the band between two values across the plot — a recession, a warm-up phase, a tolerance window — behind the layers drawn after it.

{{figure mark_rule}}

Rules take part in the domain: a target at 0.5 is on the axis even when no data reaches it. On a log axis a span that starts at or below zero washes its visible part from the axis floor up.

## Text

[`Text` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/struct.Text.html).

A string at data coordinates. `Text::at(x, y, text)` starts at the anchor and extends right. `align(Align::Center)` sets it on the anchor. `Align::Right` ends it there. On a `Bands` x axis, the band nearest the anchor becomes the text's box, with exactly the geometry the band's own header label uses — its rounded center, its step-wide budget — so aligned text and band labels land in lockstep. Text wider than its box clips to it with a truncation `.`. Digits from a neighboring column are never mixed into a number.

{{figure mark_text}}

Text is how a stat table is drawn. `describe` and `table` are `Text` marks on two band axes, each column formatted by its own `NumberFormat`. Text is also how a heatmap is annotated. A `Text` over a `Cells` keeps the cell's color as its background. It does not punch a hole in the field. It picks dark or light ink from the luminance underneath.

{{example correlation}}

## What a mark is not

There is no `Heatmap` mark beside `Cells`, no `barh` beside `Bars`, no `Errorbar` beside `Range`, no `Annotation` beside `Text`. Each of those would be a second name for one geometry with a different reading, and a vocabulary that large cannot be learned, only searched. The membership test is two clauses, both required. Real charts demand it. No composition of the rest reproduces its output ([what earns a concept](../../principles/what-earns-a-concept/)).
