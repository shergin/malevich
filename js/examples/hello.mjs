import { Frame, Line, Plot, bar, hist, line } from "../dist/index.js";

console.log(line([1, 5, 2, 8]));

console.log(
  new Plot()
    .layer(Line.y([1, 5, 2, 8]).label("loss").style("Corners"))
    .title("training")
    .render(Frame.portable(60, 14)),
);

console.log(bar(["mon", "tue", "wed", "thu", "fri"], [3, 7, 4.5, 8, 6]));
console.log(hist([1, 2, 2, 3, 3, 3, 4, 8, 9]));
