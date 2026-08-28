//! The view layer: pure functions from history snapshots to `malevich::Plot`s.
//!
//! Every chart takes plain vectors (ring snapshots) and returns an owned plot —
//! the sampler thread keeps pushing while these run, and the same functions power
//! the TUI and the headless `--render` mode.

use malevich::{Area, Color, Line, Plot};

use crate::data::History;

/// The app's screens, in tab order.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// CPU, memory, and network stacked — the classic monitor.
    Dashboard,
    /// Per-core utilization over time as a heatmap, current load as bars.
    Cores,
}

impl View {
    pub const ALL: [View; 2] = [View::Dashboard, View::Cores];

    pub fn title(self) -> &'static str {
        match self {
            View::Dashboard => "dashboard",
            View::Cores => "cores",
        }
    }

    pub fn next(self) -> View {
        match self {
            View::Dashboard => View::Cores,
            View::Cores => View::Dashboard,
        }
    }
}

/// A "seconds ago" x column for `values` sampled every `interval` seconds: the
/// newest sample sits at 0, the oldest at `-(len-1) * interval`. Time flows left
/// to right into "now", the way every monitor reads.
fn seconds_ago(len: usize, interval: f64) -> Vec<f64> {
    (0..len)
        .map(|index| (index as f64 - (len.saturating_sub(1)) as f64) * interval)
        .collect()
}

/// Total CPU utilization as a filled area.
///
/// malevich notes: two layers share one dataset — `Area::xy` fills from the
/// values down to zero (a translucent wash on pixel targets, solid ink in
/// cells), and a `Line::xy` over it draws a crisp glowing edge the fill alone
/// can't give. `.y_domain(0, 100)` pins the axis: utilization must not
/// autoscale, or an idle machine would look busy.
pub fn cpu_chart(cpu: &[f64], interval: f64) -> Plot<'static> {
    let x = seconds_ago(cpu.len(), interval);
    let latest = cpu.last().copied().unwrap_or(0.0);
    Plot::new()
        .layer(
            Area::xy(x.clone(), cpu.to_vec())
                .color(Color::Cyan)
                .opacity(0.4),
        )
        .layer(Line::xy(x, cpu.to_vec()).color(Color::BrightCyan).glow())
        .y_domain(0.0, 100.0)
        .title(format!("cpu  {latest:>5.1}%"))
        .y_label("%")
        .x_label("seconds ago")
}

/// Memory in use as a filled area, in GiB, pinned to the machine's total.
pub fn mem_chart(mem_bytes: &[f64], total_bytes: f64, interval: f64) -> Plot<'static> {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    let mem: Vec<f64> = mem_bytes.iter().map(|b| b / GIB).collect();
    let x = seconds_ago(mem.len(), interval);
    let latest = mem.last().copied().unwrap_or(0.0);
    let total = total_bytes / GIB;
    Plot::new()
        .layer(
            Area::xy(x.clone(), mem.clone())
                .color(Color::Green)
                .opacity(0.4),
        )
        .layer(Line::xy(x, mem).color(Color::BrightGreen).glow())
        .y_domain(0.0, total)
        .title(format!("memory  {latest:.1} / {total:.0} GiB"))
        .y_label("GiB")
        .x_label("seconds ago")
}

/// Network receive and transmit rates as two labeled lines.
///
/// malevich notes: the y axis carries raw bytes per second and the tick engine
/// picks one SI prefix for the whole axis — labels come out as `2.5M`, `100k` —
/// so the chart never needs manual unit switching as traffic scales. Labeling the
/// layers is what makes the legend appear.
pub fn net_chart(rx: &[f64], tx: &[f64], interval: f64) -> Plot<'static> {
    let x = seconds_ago(rx.len(), interval);
    Plot::new()
        .layer(
            Line::xy(x.clone(), rx.to_vec())
                .label("rx")
                .color(Color::Cyan),
        )
        .layer(Line::xy(x, tx.to_vec()).label("tx").color(Color::Magenta))
        .title("network")
        .y_label("B/s")
        .x_label("seconds ago")
}

/// Per-core utilization over time as a heatmap: one row per core, one column per
/// sample, colorbar legending the percent scale.
///
/// malevich notes: `Cells::matrix` takes the row-major grid with row 0 at the
/// bottom (core 0 lowest, like every core listing), and `.extents` maps the
/// column index onto a "seconds ago" x axis and rows onto core numbers. Cells
/// normalize their colormap to the grid's own finite extent, so the `.colorbar()`
/// is what keeps the picture honest — it always shows which values the shades
/// span. Cells that have no sample yet are `NaN` and stay blank.
pub fn cores_heatmap(history: &History) -> Plot<'static> {
    let (columns, grid) = history.core_grid();
    let rows = history.cores.len();
    if columns == 0 || rows == 0 {
        return Plot::new().title("cores: sampling…");
    }
    let span = columns as f64 * history.interval;
    Plot::new()
        .layer(malevich::Cells::matrix(columns, grid).extents((-span, 0.0), (0.0, rows as f64)))
        .colorbar()
        .title(format!("per-core utilization %  ({rows} cores)"))
        .x_label("seconds ago")
        .y_label("core")
}

/// The instantaneous per-core load as bars, pinned to 0–100%.
pub fn cores_bars(history: &History) -> Plot<'static> {
    let current: Vec<f64> = history
        .cores
        .iter()
        .map(|ring| ring.snapshot().last().copied().unwrap_or(0.0))
        .collect();
    let labels: Vec<String> = (0..current.len()).map(|core| core.to_string()).collect();
    malevich::bar(labels, current)
        .y_domain(0.0, 100.0)
        .title("now")
        .y_label("%")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{History, Sample};

    fn synthetic_history() -> History {
        let history = History::new(32, 4, 16.0e9, 0.5);
        for step in 0..16 {
            let phase = step as f64;
            history.push(&Sample {
                cpu_total: 30.0 + phase,
                per_core: (0..4).map(|c| 20.0 * c as f64 + phase).collect(),
                mem_used: 6.0e9 + phase * 1.0e8,
                mem_total: 16.0e9,
                rx_rate: 1.0e6 * phase,
                tx_rate: 2.0e5 * phase,
            });
        }
        history
    }

    #[test]
    fn every_chart_renders_from_a_live_history() {
        let history = synthetic_history();
        let frame = malevich::Frame::plain(70, 14);
        let charts = [
            cpu_chart(&history.cpu.snapshot(), history.interval),
            mem_chart(&history.mem.snapshot(), history.mem_total, history.interval),
            net_chart(
                &history.rx.snapshot(),
                &history.tx.snapshot(),
                history.interval,
            ),
            cores_heatmap(&history),
            cores_bars(&history),
        ];
        for chart in charts {
            assert!(!chart.render(&frame).is_empty());
        }
    }

    #[test]
    fn the_network_axis_uses_si_prefixes() {
        let history = synthetic_history();
        let rendered = net_chart(
            &history.rx.snapshot(),
            &history.tx.snapshot(),
            history.interval,
        )
        .render(&malevich::Frame::plain(70, 14));
        assert!(
            rendered.contains('M'),
            "megabyte rates get an SI prefix:\n{rendered}"
        );
    }

    #[test]
    fn the_cpu_axis_is_pinned_to_one_hundred() {
        let history = synthetic_history();
        let rendered = cpu_chart(&history.cpu.snapshot(), history.interval)
            .render(&malevich::Frame::plain(70, 14));
        assert!(rendered.contains("100"), "axis reaches 100:\n{rendered}");
    }
}
