#!/bin/sh
# Compile the JS wasm crate for the browser and drop it in gallery/wasm.
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
out="$root/gallery/wasm"
cd "$root/js"
wasm-pack build native --target web --out-dir "$out" --release
rm -f "$out/.gitignore" "$out/README.md" "$out/.npmignore"
# The glue is ESM; GitHub Pages must not treat it as CommonJS.
printf '%s\n' '{"type":"module"}' > "$out/package.json"
echo "wrote $out"
