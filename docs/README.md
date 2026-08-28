# Docs

What to read when.

- **What is this, and why is it shaped this way?**
  [vision.md](vision.md) — the argument and the five rules.
- **Why is this decision the way it is?**
  [principles/](principles/) — one file per constraint: the failure mode it
  avoids, the idea, the consequences, the rejected alternatives. Files where
  the claim is visual carry a generated witness chart, spliced and verified
  by CI like every chart in these docs.
- **What does this word mean?**
  [terminology.md](terminology.md) — the vocabulary contract, updated in the
  same change as the code.
- **How do I…**
  - meet any terminal honestly — [terminal.md](terminal.md)
  - draw real pixels in a terminal — [pixels.md](pixels.md)
  - plot in a Jupyter notebook — [notebooks.md](notebooks.md)
  - understand the speed story — [performance.md](performance.md)
  - persist and interchange specs — [serde.md](serde.md)
- **What does it look like?**
  [../EXAMPLES.md](../EXAMPLES.md) — the gallery, every chart real program
  output. `cargo run --example showcase` renders a colored tour.
- **API reference** — [docs.rs/malevich](https://docs.rs/malevich).
