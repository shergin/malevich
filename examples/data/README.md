# Vendored datasets

Real, openly licensed data for the gallery — per the project rule: story-bearing
examples, never synthetic noise where reality is available. Retrieved 2026-08-02.

- `co2_monthly.csv` — the Keeling curve: monthly mean atmospheric CO₂ at Mauna Loa,
  1958–present. Source: NOAA Global Monitoring Laboratory
  (<https://gml.noaa.gov/ccgg/trends/data.html>, `co2_mm_mlo.txt`, monthly average
  column; months flagged missing dropped). US government work, public domain;
  credit: Dr. Xin Lan, NOAA/GML, and Dr. Ralph Keeling, Scripps Institution of
  Oceanography.
- `penguins.csv` — Palmer Archipelago penguin measurements (Gorman, Williams &
  Fraser 2014), 342 complete records. Source: the `palmerpenguins` dataset via
  vega-datasets (<https://github.com/vega/vega-datasets>). License: CC0.
- `topos_loss.csv` — a real training log: per-step minibatch cross-entropy of
  topos's makemore bigram model (a 27×27 logit table on the 32k-name corpus),
  1,000 steps, captured 2026-08-02 by running the `makemore_bigram` example with
  per-step logging. Generated data from the author's own crate
  (<https://github.com/shergin> · topos, née poorgrad); the names corpus follows makemore.
