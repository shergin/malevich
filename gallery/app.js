import init, {
  expand_preset,
  mapping_columns,
  raster_columns,
  render_pixels_columns,
} from "./wasm/malevich_js.js";
import { CHARTS, waveDocument } from "./charts.js";

const NAMED = [
  "#000000",
  "#cd0000",
  "#00cd00",
  "#cdcd00",
  "#0000ee",
  "#cd00cd",
  "#00cdcd",
  "#e5e5e5",
  "#7f7f7f",
  "#ff0000",
  "#00ff00",
  "#ffff00",
  "#5c5cff",
  "#ff00ff",
  "#00ffff",
  "#ffffff",
];

const ASCII = {
  width: 72,
  height: 16,
  charset: "Ascii",
  color: "TrueColor",
  theme: { palette: ["Cyan", "Yellow", "Green", "Magenta", "Blue", "Red"] },
};

function cssColor(bytes, offset) {
  const tag = bytes[offset] ?? 0;
  if (tag === 1) {
    return NAMED[bytes[offset + 1] ?? 0] ?? null;
  }
  if (tag === 2) {
    return ansi256(bytes[offset + 1] ?? 0);
  }
  if (tag === 3) {
    const r = bytes[offset + 1] ?? 0;
    const g = bytes[offset + 2] ?? 0;
    const b = bytes[offset + 3] ?? 0;
    return `#${hex(r)}${hex(g)}${hex(b)}`;
  }
  return null;
}

function hex(value) {
  return value.toString(16).padStart(2, "0");
}

function ansi256(index) {
  if (index < 16) {
    return NAMED[index];
  }
  if (index >= 232) {
    const level = 8 + (index - 232) * 10;
    return `rgb(${level}, ${level}, ${level})`;
  }
  const cube = index - 16;
  const ramp = [0, 95, 135, 175, 215, 255];
  const r = ramp[Math.floor(cube / 36) % 6];
  const g = ramp[Math.floor(cube / 6) % 6];
  const b = ramp[cube % 6];
  return `rgb(${r}, ${g}, ${b})`;
}

function escapeHtml(glyph) {
  if (glyph === "&") return "&amp;";
  if (glyph === "<") return "&lt;";
  if (glyph === ">") return "&gt;";
  return glyph;
}

function rasterHtml(packed) {
  const width = packed.width;
  const height = packed.height;
  const glyphs = [...packed.glyphs];
  const fg = packed.fg;
  const bg = packed.bg;
  const columns = packed.columns;
  const rows = [];
  for (let row = 0; row < height; row++) {
    let html = "";
    let current = "";
    let run = "";
    const flush = () => {
      if (!run) {
        return;
      }
      if (current) {
        html += `<span style="${current}">${run}</span>`;
      } else {
        html += run;
      }
      run = "";
    };
    for (let column = 0; column < width; column++) {
      const i = row * width + column;
      if ((columns[i] ?? 1) === 0) {
        continue;
      }
      const glyph = glyphs[i] ?? " ";
      const foreground = cssColor(fg, i * 4);
      const background = cssColor(bg, i * 4);
      const style = [
        foreground && glyph !== " " ? `color:${foreground}` : "",
        background ? `background-color:${background}` : "",
      ]
        .filter(Boolean)
        .join(";");
      if (style !== current) {
        flush();
        current = style;
      }
      run += escapeHtml(glyph);
    }
    flush();
    rows.push(html.replace(/(?:&nbsp;| )+$/g, "").replace(/ +$/g, ""));
  }
  return rows.join("\n");
}

function itermPngUrl(payload) {
  const match = payload.match(/\x1b]1337;File=[^:]*:([A-Za-z0-9+/=\n]+)\x07/);
  if (!match) {
    throw new Error("pixel render did not carry an iTerm2 PNG");
  }
  const binary = atob(match[1].replaceAll("\n", ""));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
}

function frameFor(chart) {
  return {
    ...ASCII,
    width: chart.width,
    height: chart.height,
  };
}

function listing(chart) {
  const rustId = `${chart.kicker.replaceAll(" ", "-").toLowerCase()}-rust`;
  const tsId = `${chart.kicker.replaceAll(" ", "-").toLowerCase()}-ts`;
  const tablist = `${chart.kicker.replaceAll(" ", "-").toLowerCase()}-tabs`;
  return `<div class="listing">
      <div class="tabs" role="tablist" aria-label="Source language">
        <button type="button" role="tab" id="${tablist}-rust" aria-controls="${rustId}" aria-selected="true">Rust</button>
        <button type="button" role="tab" id="${tablist}-ts" aria-controls="${tsId}" aria-selected="false">TypeScript</button>
      </div>
      <pre id="${rustId}" role="tabpanel" aria-labelledby="${tablist}-rust"><code>${escapeHtml(chart.rust)}</code></pre>
      <pre id="${tsId}" role="tabpanel" aria-labelledby="${tablist}-ts" hidden><code>${escapeHtml(chart.typescript)}</code></pre>
    </div>`;
}

function bindTabs(figure) {
  const buttons = [...figure.querySelectorAll('[role="tab"]')];
  const panels = [...figure.querySelectorAll('[role="tabpanel"]')];
  for (const button of buttons) {
    button.addEventListener("click", () => {
      for (const other of buttons) {
        other.setAttribute("aria-selected", String(other === button));
      }
      for (const panel of panels) {
        panel.hidden = panel.id !== button.getAttribute("aria-controls");
      }
    });
  }
}

function paintPlate(figure, packed, pixels, alt) {
  figure.querySelector(".chart").innerHTML = rasterHtml(packed);
  const img = figure.querySelector("img");
  const previous = img.src;
  img.alt = alt;
  img.src = itermPngUrl(pixels);
  if (previous.startsWith("blob:")) {
    URL.revokeObjectURL(previous);
  }
}

function formatMs(ms) {
  if (ms < 10) {
    return `${ms.toFixed(2)} ms`;
  }
  if (ms < 100) {
    return `${ms.toFixed(1)} ms`;
  }
  return `${Math.round(ms)} ms`;
}

function pairOf(value) {
  if (!value) {
    return undefined;
  }
  if (Array.isArray(value) && value.length >= 2) {
    return [Number(value[0]), Number(value[1])];
  }
  return undefined;
}

function fillWave(n) {
  const y = new Float64Array(n);
  for (let i = 0; i < n; i++) {
    y[i] = Math.sin(i * 0.0002) * Math.cos(i * 0.000013) * 8;
  }
  return y;
}

function mountInstrument(figure, chart) {
  const sizes = [100_000, 1_000_000, 10_000_000];
  const state = {
    n: 1_000_000,
    domain: null,
    series: new Map(),
    mapping: undefined,
    pixelTimer: 0,
  };
  const frame = frameFor(chart);
  const frameJson = JSON.stringify(frame);
  const controls = document.createElement("div");
  controls.className = "instrument";
  controls.innerHTML = `
    <div class="sizes" role="group" aria-label="Series length">
      ${sizes
        .map(
          (n) =>
            `<button type="button" data-n="${n}" aria-pressed="${n === state.n}">${n === 100_000 ? "100k" : n === 1_000_000 ? "1M" : "10M"}</button>`,
        )
        .join("")}
    </div>
    <button type="button" data-reset>reset view</button>
    <p class="hint">wheel zooms · drag pans</p>
  `;
  const readout = document.createElement("p");
  readout.className = "readout";
  figure.querySelector("figcaption").after(controls);
  controls.after(readout);

  const series = (n) => {
    let values = state.series.get(n);
    if (!values) {
      values = fillWave(n);
      state.series.set(n, values);
    }
    return values;
  };

  const setMapping = (documentJson, columns) => {
    state.mapping?.free();
    state.mapping = mapping_columns(documentJson, frameJson, columns);
  };

  const paintPixels = (documentJson, columns) => {
    const t0 = performance.now();
    const pixels = render_pixels_columns(
      documentJson,
      frameJson,
      columns,
      "iterm2",
      8,
      16,
    );
    const pixelMs = performance.now() - t0;
    const img = figure.querySelector("img");
    const previous = img.src;
    img.src = itermPngUrl(pixels);
    if (previous.startsWith("blob:")) {
      URL.revokeObjectURL(previous);
    }
    return pixelMs;
  };

  const draw = (withPixels) => {
    const values = series(state.n);
    const document = waveDocument(state.n, state.domain);
    const documentJson = JSON.stringify(document);
    const columns = [values];
    const t0 = performance.now();
    const packed = raster_columns(documentJson, frameJson, columns);
    const cellMs = performance.now() - t0;
    figure.querySelector(".chart").innerHTML = rasterHtml(packed);
    setMapping(documentJson, columns);
    let pixelMs = state.lastPixels;
    if (withPixels) {
      pixelMs = paintPixels(documentJson, columns);
      state.lastPixels = pixelMs;
    } else {
      window.clearTimeout(state.pixelTimer);
      state.pixelTimer = window.setTimeout(() => draw(true), 90);
    }
    const windowLabel = state.domain
      ? `${state.mapping.formatX(state.domain[0])} – ${state.mapping.formatX(state.domain[1])}`
      : "full";
    const pixelBit =
      pixelMs === undefined ? "" : ` · pixels ${formatMs(pixelMs)}`;
    readout.textContent = `${state.n.toLocaleString("en-US")} points · cells ${formatMs(cellMs)}${pixelBit} · ${windowLabel}`;
  };

  const dataXAt = (clientX) => {
    if (!state.mapping) {
      return undefined;
    }
    const pre = figure.querySelector(".chart");
    const rect = pre.getBoundingClientRect();
    if (rect.width <= 0) {
      return undefined;
    }
    const column = Math.max(
      0,
      Math.min(frame.width - 1, Math.floor(((clientX - rect.left) / rect.width) * frame.width)),
    );
    const pair = pairOf(state.mapping.dataAt(column, Math.floor(frame.height / 2)));
    return pair?.[0];
  };

  const applyViewport = (next) => {
    const x = pairOf(next.x);
    next.free();
    state.domain = x ?? null;
    draw(false);
  };

  figure.querySelector(".plate").addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      if (!state.mapping) {
        return;
      }
      const anchor = dataXAt(event.clientX);
      if (!Number.isFinite(anchor)) {
        return;
      }
      const factor = event.deltaY > 0 ? 1.18 : 1 / 1.18;
      const viewport = state.mapping.viewport();
      const zoomed = viewport.zoomX(factor, anchor);
      viewport.free();
      applyViewport(zoomed);
    },
    { passive: false },
  );

  let drag = null;
  figure.querySelector(".plate").addEventListener("pointerdown", (event) => {
    if (event.button !== 0) {
      return;
    }
    drag = { x: event.clientX };
    event.currentTarget.setPointerCapture(event.pointerId);
  });
  figure.querySelector(".plate").addEventListener("pointerup", () => {
    drag = null;
  });
  figure.querySelector(".plate").addEventListener("pointermove", (event) => {
    if (!drag || !state.mapping) {
      return;
    }
    const width = figure.querySelector(".plate").getBoundingClientRect().width;
    const fraction = (drag.x - event.clientX) / Math.max(width, 1);
    drag.x = event.clientX;
    if (Math.abs(fraction) < 1e-6) {
      return;
    }
    const viewport = state.mapping.viewport();
    const panned = viewport.panX(fraction);
    viewport.free();
    applyViewport(panned);
  });

  controls.querySelector("[data-reset]").addEventListener("click", () => {
    state.domain = null;
    draw(true);
  });
  for (const button of controls.querySelectorAll("[data-n]")) {
    button.addEventListener("click", () => {
      state.n = Number(button.dataset.n);
      state.domain = null;
      for (const other of controls.querySelectorAll("[data-n]")) {
        other.setAttribute("aria-pressed", String(other === button));
      }
      readout.textContent = "sampling…";
      window.setTimeout(() => draw(true), 20);
    });
  }

  draw(true);
}

async function main() {
  const status = document.getElementById("status");
  const root = document.getElementById("figures");
  const engine = document.getElementById("engine");
  const fail = (error) => {
    const message = error instanceof Error ? error.message : String(error);
    const node = status ?? document.createElement("p");
    node.className = "status error";
    node.textContent =
      message.includes("Failed to fetch") || /wasm/i.test(message)
        ? "WebAssembly is missing. From the repo root: ./gallery/build.sh then serve this directory."
        : message;
    if (!status) {
      root.prepend(node);
    }
    console.error(error);
  };
  try {
    await init();
    engine.textContent = "1.21.0";
    const api = { expand_preset, raster_columns, render_pixels_columns };
    for (const chart of CHARTS) {
      const built = chart.build(api);
      const figure = document.createElement("figure");
      figure.innerHTML = `
        <div class="plate">
          <div class="pane">
            <p class="label">ascii</p>
            <pre class="chart"></pre>
          </div>
          <div class="pane">
            <p class="label">pixels</p>
            <img alt="${chart.kicker}: pixel panel" />
          </div>
        </div>
        <figcaption>${chart.caption}</figcaption>
        ${listing(chart)}
      `;
      bindTabs(figure);
      if (chart.interactive) {
        figure.id = "live";
      }
      root.append(figure);
      if (chart.interactive) {
        mountInstrument(figure, chart);
        continue;
      }
      const documentJson = JSON.stringify(built.document);
      const frameJson = JSON.stringify(frameFor(chart));
      const packed = raster_columns(documentJson, frameJson, built.columns);
      const pixels = render_pixels_columns(
        documentJson,
        frameJson,
        built.columns,
        "iterm2",
        8,
        16,
      );
      paintPlate(figure, packed, pixels, `${chart.kicker}: pixel panel`);
    }
    status.remove();
  } catch (error) {
    fail(error);
  }
}

main();
