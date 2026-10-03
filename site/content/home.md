# malevich

<section class="hero">

<div class="hero-text">

<h1>Terminal plotting for Rust.</h1>

<p class="lede">A small grammar of marks, honest axes, millions of points. The chart is a plain value. Give it a frame and it draws. Give it another frame and it draws again. A pipe gets clean text. A bad terminal still gets a chart.</p>

```sh
cargo add malevich
```

<p class="trust"><code>CRATE_VERSION</code> · <code>forbid(unsafe_code)</code> · Rust RUST_VERSION · CRATE_DEPENDENCIES</p>

<p><a class="cta solid" href="guide/start/">Start here</a> <a class="cta" href="guide/choose/">Which chart</a> <a class="cta red" href="playground/">Open the playground</a> <a class="cta" href="gallery/">See the gallery</a></p>

</div>

{{figure hero nocode | A training run: two lines, a fill, a target, a note at data coordinates.}}

</section>

## What it draws

<div class="tiles">

<div class="tile">

{{figure stat_kde nocode | A density from a real kernel, three bandwidths, one vocabulary.}}

<h3>A real statistics layer</h3>

<p>Type-7 quartiles and Tukey whiskers. A density from a real kernel. A fit you can stream, with R² and a band. One <code>Reducer</code> word in a bin, a group, or a rolling window.</p>

</div>

<div class="tile">

{{figure scale_time_years nocode | Sixty-seven years on a calendar axis.}}

<h3>Axes that are actually good</h3>

<p>Ticks from the extended Wilkinson algorithm. Labels you can parse back to the number. One SI prefix on an axis. Calendar time that says <code>14:05</code>, or <code>Aug 2</code>, or <code>2027</code>.</p>

</div>

<div class="tile">

{{figure scale_colormap_log nocode | Decades of weight, and the mask's zeros as gaps.}}

<h3>The ML set</h3>

<p>Attention maps, confusion matrices, decision boundaries, images, a loss landscape with the optimizer's path on it. A few marks put together. None of them got its own chart type.</p>

</div>

<div class="tile">

{{figure stat_m4 nocode | M4 keeps a spike one sample wide.}}

<h3>Millions of points, measured</h3>

<p>A long line is reduced with M4, one bucket per column, and the pixels match drawing every point. Ten million of them take tens of milliseconds on the <a href="benchmarks/">recorded baseline</a>.</p>

</div>

<div class="tile">

{{figure stat_describe nocode | The summary, as a table lined up on the decimal.}}

<h3>The first look is sometimes a table</h3>

<p><code>describe</code> prints count, mean, sd, min, quartiles, and max. Text on two band axes, each column formatted like a tiny axis. A gap is <code>—</code>.</p>

</div>

<div class="tile">

{{figure mark_bars_horizontal nocode | Bars on their side, long names in the gutter.}}

<h3>Sideways, stacked, grouped</h3>

<p><code>Bars::horizontal</code> turns a bar on its side. <code>Bars::base</code> stacks the next one. <code>stat::dodge</code> sets them shoulder to shoulder. The <a href="gallery/">gallery</a> spells the rest. They never graduated to presets.</p>

</div>

</div>

<div class="shots">

![Loss curves, a calendar time axis, and smoothing: cell rendering beside pixel rendering](examples/showcase-lines.png)

![A 2D density, contour lines, and a vector field: cell rendering beside pixel rendering](examples/showcase-2d.png)

</div>

<p class="shot-caption">The same plot value, twice. Cells on the left. Real pixels on the right, where the terminal speaks sixel, kitty, or iTerm2. Title, axes, and legend stay text.</p>

## One call, then the lid comes off

The front door is a preset. You can walk through it without knowing the grammar. <code>line(&values)</code> and <code>Plot::new().layer(Line::y(&values))</code> are the same call, the second one with the lid off, and moving from one to the other doesn't change a byte of the picture.

```rust
println!("{}", malevich::line(&[1.0, 5.0, 2.0, 8.0][..]));
```

{{figure start_line nocode | Four values. That is the whole program.}}

```rust
use malevich::{Frame, Line, Plot, Rule};

let steps: Vec<f64> = (0..100).map(f64::from).collect();
let loss: Vec<f64> = steps.iter().map(|s| 4.0 * (-0.05 * s).exp() + 0.4).collect();
let chart = Plot::new()
    .layer(Line::xy(&steps[..], &loss[..]).label("loss"))
    .layer(Rule::h(0.5).label("target"))
    .title("training");
println!("{}", chart.render(&Frame::plain(60, 14)));
```

{{figure start_layers nocode | The same chart with the lid off: a second layer, a title, a frame you chose.}}

`Plot::render` never fails. When the frame is too small for a title or a legend, those go first and the data stays. `Frame::plain` gives a test the same string every time. `Frame::detect()` colors it and sizes it for the terminal you are actually in.

<section class="manifesto">

<div>

<h3>Eight marks, and that's the catalog</h3>

<p>Line, Points, Bars, Area, Cells, Range, Rule, Text. Give them a statistics layer and shared scales, and a histogram, a box plot, a density are words you spell. A preset is the short spelling. A test checks that both spellings print the same bytes.</p>

</div>

<div>

<h3>Every brag has a test behind it</h3>

<p>The true chart draws every point. The fast one has to land on the same pixels, including a spike one sample wide. A published number has a benchmark behind it. A chart in the docs was drawn by the program, and the build diffs it.</p>

</div>

<div>

<h3>A bad terminal still gets a chart</h3>

<p>Real pixels when the terminal can draw them. Otherwise octants, quadrants, then ASCII, which always works. It never fails, never sends a probe where escapes aren't safe, and never takes over the screen. A pipe gets text a log, a diff, or a language model can read.</p>

</div>

</section>

## Three doors

<div class="doors">

<section>

<h3>Rust</h3>

<p>A plot is a value. Printing it looks at stdout and picks a frame. The <a href="playground/">playground</a> writes this program for the numbers you paste, and <code>kaz --emit-code</code> writes it from a pipe.</p>

{{figure start_bar nocode | Categories and a number. `bar` is the short spelling.}}

<p><a href="guide/start/">Getting started</a></p>

</section>

<section>

<h3>kaz</h3>

<p>A column of numbers in, a chart on stderr, the data still flowing on stdout. <code>--emit-code</code> prints the Rust that would draw the same picture.</p>

{{figure start_hist nocode | A column of numbers through `hist`. `kaz hist` bins it the same way.}}

```sh
cargo install malevich-cli
cat loss.tsv | kaz line --emit-code
```

<p><a href="cli/">kaz, the command line</a></p>

</section>

<section>

<h3>JavaScript</h3>

<p>The same engine, compiled to wasm, on npm. The Ink widget shares the ratatui adapter's gestures: zoom, pan, and crosshairs, without owning the terminal.</p>

{{figure start_scatter nocode | Two columns through `scatter`. The npm package exports the same preset.}}

<p><a href="js/">The npm package</a></p>

</section>

</div>

## Five things people do

<ul class="tasks">
<li><a href="cli/">Pipe a column</a> through <code>kaz</code>. The plot goes to stderr.</li>
<li><a href="guide/interaction/">Drop a widget in ratatui</a>. The plot does not own the screen.</li>
<li><a href="guide/streaming/">Repaint a chart</a> as the numbers keep arriving.</li>
<li><a href="guide/grammar/">Color by a category</a>. The legend and, without color, the marker shapes.</li>
<li><a href="guide/scales/">Put time on an axis</a>. Hours, days, or years, from the span.</li>
</ul>

<p>If you have a shape of data and want the spelling, <a href="guide/choose/">which chart</a> is the plate for each preset.</p>

## The name

<section class="name">

<div class="black-square" aria-hidden="true"></div>

<div>

<p>Kazimir Malevich painted a black square on a plain ground and meant it: a small vocabulary of geometric forms, composed deliberately. That is the design budget of this library — and, as it happens, a fair description of a terminal, which draws everything it will ever draw from a grid of small rectangles.</p>

</div>

</section>

![A compact Suprematist composition: cells beside real pixels](examples/suprematist-composition.png)

<p class="shot-caption">One plot, two fidelities. Rectangles, composed. The cells are on the left. The pixels are on the right.</p>
