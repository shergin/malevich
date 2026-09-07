/** Categorical bars — analog of `cargo run --example languages`. */

import { Frame, bar } from "../dist/index.js";

console.log(
  bar(
    ["rust", "go", "python", "typescript", "zig"],
    [68, 41, 55, 62, 12],
  )
    .title("admired languages, % (synthetic)")
    .render(Frame.detect()),
);
