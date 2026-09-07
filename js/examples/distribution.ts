/** The statistical set: histogram, density, ECDF, box plot, violin. */

import { Frame, boxPlot, density, ecdf, hist, violin } from "../dist/index.js";

const samples = Float64Array.from(
  { length: 4000 },
  (_, i) => (Math.sin(i * 0.731) + Math.sin(i * 1.13) + Math.sin(i * 2.71)) * 2 + 10,
);

const frame = Frame.detect();
console.log(`${hist(samples).title("histogram, automatic bins").render(frame)}\n`);
console.log(`${density(samples).title("gaussian KDE").render(frame)}\n`);
console.log(`${ecdf(samples).title("empirical CDF").render(frame)}\n`);

const group = (offset: number, n: number) =>
  Float64Array.from({ length: n }, (_, i) => offset + ((i * 17) % 23) * 0.4);
console.log(
  `${boxPlot(["a", "b", "c"], [group(2, 80), group(5, 60), group(3, 90)]).title("box plots").render(frame)}\n`,
);
console.log(
  violin(["a", "b", "c"], [group(2, 80), group(5, 60), group(3, 90)])
    .title("violins")
    .render(frame),
);
