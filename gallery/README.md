# Gallery

A small gallery of charts, rendered in the browser from the same wasm the npm
package uses. The last plate is live: a long line through M4, with a render
clock.
Ascii cells on the left, the plot panel as a PNG on the right. Rust and
TypeScript for each figure.

```sh
./gallery/build.sh
python3 -m http.server 4173 --directory gallery
```

Then open <http://localhost:4173>. Live: <https://shergin.github.io/malevich/>.
GitHub Pages builds from `main` (`.github/workflows/pages.yml`).
