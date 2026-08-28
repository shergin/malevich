# Serde compatibility

The `serde` feature supports two related formats:

- `Document` is the persistent, versioned format for files, caches, and network
  messages.
- Raw `Plot`, `Grid`, mark, scale, frame, and theme serde implementations remain
  available for source compatibility and short-lived interchange. A raw payload has
  no version discriminator, so new persistent data should not use it directly.

## Version 1

A document is a small envelope around an owned plot or grid:

```json
{
  "version": 1,
  "kind": "plot",
  "spec": { "layers": [] }
}
```

Constructing or decoding a `Document` validates the complete payload. Unknown schema
versions, zero-column grids, ragged channels, invalid mark/scale combinations, and
other malformed states are errors rather than documents that fail later at render
time. Unknown additive JSON fields are ignored, and omitted plot fields take their
documented defaults. Gaps remain `null`; function-backed lines still refuse to
serialize because closures have no honest data representation.

```rust
# #[cfg(feature = "serde")] {
use malevich::{Document, Frame};

let document = Document::plot(malevich::line([1.0, 3.0, 2.0]))?;
let json = serde_json::to_string_pretty(&document)?;
let decoded: Document = serde_json::from_str(&json)?;
assert_eq!(decoded.version(), 1);
assert!(!decoded.try_render(&Frame::portable(40, 10))?.is_empty());
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

The committed fixtures under `tests/fixtures/serde/` are the compatibility contract.
Every supported version must continue to decode, validate, and render. Encoder tests
also compare canonical documents to those fixtures, catching accidental field,
variant, or tagging changes. JSON whitespace and object-key order are not part of the
contract.

To migrate a legacy raw payload, decode it as a `Plot` or `Grid`, pass it through
`Document::plot` or `Document::grid`, and serialize the returned document. Keep the
old decoder until all stored raw payloads have been migrated.

Future incompatible schemas will use a new envelope version and an explicit
conversion into runtime `Plot`/`Grid` values. A reader never silently interprets an
unknown version as the current one.
