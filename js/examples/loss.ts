/** Training-loop look: two series, a target rule, an annotation. */

import { Frame, Line, Plot, Rule, Text } from "../dist/index.js";

const steps = Float64Array.from({ length: 120 }, (_, i) => i);
const train = Float64Array.from(
  steps,
  (s) => 3.8 * Math.exp(-0.035 * s) + 0.32 + 0.05 * Math.sin(s * 0.7),
);
const val = Float64Array.from(
  steps,
  (s) => 4.0 * Math.exp(-0.03 * s) + 0.55 + 0.08 * Math.cos(s * 0.35),
);

console.log(
  new Plot()
    .layer(Line.xy(steps, train).label("train"))
    .layer(Line.xy(steps, val).label("val"))
    .layer(Rule.h(0.5).label("target"))
    .layer(Text.at(60, 2, "< converging"))
    .title("loss (synthetic)")
    .xLabel("step")
    .yLabel("loss")
    .render(Frame.detect()),
);
