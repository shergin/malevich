# Releasing malevich

Releases are cut from a clean `main` after every CI job is green. Replace the example
previous tag below when the next release is not based on `v1.14.3`.

## 1. Prepare the release commit

- Choose the core version and, independently, whether `malevich-cli` changed enough
  to require a CLI release.
- Update the relevant `Cargo.toml` versions, the CLI's minimum `malevich` dependency,
  `Cargo.lock`, the README dependency snippet, and any versioned examples.
- Rename `Unreleased` in `CHANGELOG.md` to the version and date, then add a fresh empty
  `Unreleased` section.
- Confirm `BENCHMARKS.md` names the code revision behind every current performance
  claim. Rerun Criterion on an otherwise idle machine when render code or a claim
  changed; never substitute a shared-runner wall-clock result.
- Review `git diff` and require a clean worktree after committing the release changes.

## 2. Reproduce the local gates

Run the same core, adapter, CLI, documentation, and MSRV checks used by CI:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo test --all-features --all-targets
cargo test --doc
cargo run --example regen_docs -- --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo clippy -p malevich-cli --all-targets -- -D warnings
cargo test -p malevich-cli
rustup run 1.88 cargo check --lib --all-features
rustup run 1.88 cargo bench --bench alloc -- --check
cargo deny check
cargo semver-checks check-release --baseline-rev v1.14.3 --all-features
```

Do not replace the allocation check with a timing gate. Confirm the native Windows CI
job and the scheduled parser-fuzz workflow separately; local Unix runs do not exercise
their platform boundaries.

## 3. Inspect the packages

The crate archive is part of the API. Inspect it before any publish:

```sh
cargo package --list -p malevich
cargo package -p malevich
cargo package --list -p malevich-cli
```

The core archive must include source, examples, fixtures, README, changelog,
terminology, serde policy, benchmark record, licenses, and this checklist; it must not
include `private/`, fuzz corpora, screenshots, or demo applications. Check the CLI
archive independently. A CLI package that depends on new core API can be fully
verified only after that core version reaches the registry index.

## 4. Publish and verify in dependency order

1. Run `cargo publish -p malevich` and wait until the new version resolves from the
   registry.
2. If the CLI changed, run `cargo package -p malevich-cli`, then
   `cargo publish -p malevich-cli`.
3. Build the docs.rs page and both README quick starts from the published versions,
   not the workspace paths.
4. Tag the exact release commit (`v<version>` for the library and
   `cli-v<version>` for a CLI release), push the commit and tags, and verify the
   GitHub release/CI state.
5. Leave the worktree clean and record any post-release issue in the current private
   backlog rather than editing a historical release entry.

## 5. npm (`malevich@0.x`)

The JS rim is versioned independently of crates.io until the API is 1.x. Publish
from `js/` after the wasm build is in `js/generated` (gitignored; produced by
`npm run build`).

First time on a machine:

```sh
# Create an account at https://www.npmjs.com/signup if you do not have one.
npm login
npm whoami
```

Then:

```sh
cd js
npm run build
npm test
npm pack --dry-run          # licenses in, dist/test out, ~240 KB
npm publish --access public
```

The package is unscoped ESM (`import { line } from "malevich"`). `prepublishOnly`
rebuilds wasm and reruns tests. 2FA is required by npm for publish. Bump
`js/package.json` independently of the crate (0.x until the JS API settles);
keep `engineVersion` equal to the crate it was built against. Record the
release in `js/CHANGELOG.md`.
