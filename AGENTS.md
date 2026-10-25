# feed-rs

Rust library for parsing RSS 0.9x/1.0/2.0, RDF, Atom and JSON Feed (including badly malformed feeds). Workspace members:

- `feed-rs/` — the library crate (edition 2024).
- `tools/` — dev-only helper binaries under `tools/src/bin/` (edition 2021). Nothing here ships with the library.

## Commands

- `cargo test` — full suite; runs offline against fixtures in `feed-rs/fixture/`.
- Single test: `cargo test -p feed-rs <test_name>` (append `-- --nocapture` for prints).
- `cargo clippy -- -D warnings` and `cargo fmt --all -- --check` — CI quality gates; run both before committing.
- rustfmt uses `max_width = 160` (`.rustfmt.toml`); don't reformat with default settings.

## CI constraints

- Both GitHub Actions and GitLab CI run fmt, clippy (warnings = errors) and tests on stable **and pinned Rust 1.96.1** (tracks freedesktop-sdk). Treat 1.96.1 as the effective MSRV — no newer std/cargo features. Beta/nightly failures are allowed.
- The pinned toolchain builds run `--all-features`; keep the default `sanitize` feature compiling.

## Fixtures and the serde regression test (important)

- `feed-rs/fixture/**` are sample feeds (rss0, rss1, rss2, atom, jsonfeed); each has a committed `*.serde.json` baseline.
- `serde_regression` in `feed-rs/src/parser/tests.rs` parses every fixture with `Builder::default().sanitize_content(false)` and compares the serde output against the baseline.
- **Never hand-edit `*.serde.json`.** Any change to the model, serialization, or parsing behavior requires regenerating them — `cargo run --bin fixture_serde` — and committing the diff, or CI fails.

## Testing notes

- Load fixtures through the helpers in `feed-rs/src/util/test.rs` (paths resolve relative to `CARGO_MANIFEST_DIR/fixture`); tests must not use the network. To fetch and parse live URLs, use `tools/src/bin/testurls.rs` (URLs on stdin).
- `id_generator_default` (in `feed-rs/tests/id_generator.rs`) pins stable IDs (siphash over title/link); changing ID generation breaks it.
- `sanitize` is a default cargo feature (uses `ammonia`); it sanitizes HTML but skips `text/plain` content. Many tests use `sanitize_content(false)` for exact comparisons.

## Layout pointers

- Parsing: `feed-rs/src/parser/` — `mod.rs` holds `Builder`/`Parser`, format-specific modules do the mapping, `rss1/` handles RDF, `tokenizer.rs` tolerates broken XML.
- Public data model: `feed-rs/src/model/` — `module.rs` (structs), `value.rs` (`Text`, `Link`, ...). Exposing a new feed element = add model field + parser mapping + fixture coverage.
- Integration tests: `feed-rs/tests/` (examples, public API, ID generators). `fuzz-target/` is a separate cargo project for fuzzing the parser.
