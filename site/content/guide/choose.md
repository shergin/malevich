# Which chart

You have a shape of data. The preset is the short spelling. The plate is that spelling, drawn. The link is the page that takes the lid off.

## One series

Numbers in order, against the index. The preset is `line`.

{{figure start_line nocode | `line` over four values.}}

[`Line`](../marks/#line) is the mark. A second series, a title, or a reference line is the [grammar](../grammar/), and it prints the same bytes as the preset it grew from.

## Categories

A name and a number. The preset is `bar`. Bars rise from zero on a band axis.

{{figure start_bar nocode | `bar` over a week.}}

Sideways, stacked, and grouped are [the bar mark](../marks/#bars), not new presets.

## A distribution

One column of measurements, and the shape it makes. The preset is `hist`. It chooses the bins. The count axis does not label a half.

{{figure start_hist nocode | `hist` over 342 penguin body masses.}}

Bins, percent, and the cumulative climb are the [statistics layer](../stats/).

## A density

The same column, smoothed. The preset is `density`. The bandwidth is the knob.

{{figure stat_kde nocode | One sample, three bandwidths. Silverman's rule is the middle one.}}

## A box plot

A number per category, and the spread. The preset is `box_plot`: type-7 quartiles, Tukey whiskers, outliers.

{{figure mark_range_over nocode | The box plot, spelled with `Range`: whiskers, a body from q1 to q3, a marker at the median.}}

## A violin

The shape per category, instead of the five numbers. The preset is `violin`. It is two horizontal areas around a kernel density.

{{figure mark_area_horizontal nocode | One violin, spelled as two horizontal areas.}}

## Two columns

An x and a y. The preset is `scatter`.

{{figure start_scatter nocode | `scatter` over two columns.}}

## Categories on a scatter

The same two columns, and a name for each point. `color_by` paints the groups, names them in the legend, and cycles marker shapes when the output has no color.

{{figure mark_points_color_by nocode | Three species, one `color_by` channel.}}

The [grammar](../grammar/) builds this up one layer at a time.

## A matrix

Rows, columns, and a value in each cell. The preset is `heatmap`.

{{figure mark_cells_matrix nocode | `Cells::matrix` under a colormap. `heatmap` is the short spelling.}}

A logarithmic colormap, for weights that span decades, is [scales](../scales/).

## Calendar time

A timestamp and a number. The axis reads the span and labels hours, days, or years. You do not hand it a format string.

{{figure scale_time_years nocode | Monthly CO₂ at Mauna Loa, 1958 onward, on a calendar axis.}}

## A table first

The summary you usually compute before you plot anything. The preset is `describe`.

{{figure stat_describe nocode | `describe`: count, mean, sd, min, quartiles, max. A gap is an em dash.}}

## More points than pixels

A long line. Past four points per column, the plot inserts M4 — first, last, minimum, and maximum in each column — and the pixels match drawing every point. A spike one sample wide stays.

{{figure stat_m4 nocode | 200,000 points. The three one-sample spikes survive.}}

The measurement is on the [performance](../performance/) page.
