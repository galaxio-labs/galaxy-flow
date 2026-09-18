# Repository Guidelines

## Project Structure & Module Organization
Galaxy Flow is a Rust workspace that builds a **single binary**, `gx` (`app/gx/main.rs`), with all logic in the `galaxy_flow` library crate (`src/`).

Runtime chain: `parser -> model -> ability -> runner`.
- `src/parser/` — parses GXL text into `model` objects (`abilities/`, `inner/`, `stc_*`, `gxl_fun/`)
- `src/model/` — execution data model (`components/`, `execution/`, `task_report/`)
- `src/ability/` — built-in `gx.*` abilities (cmd, tpl, read, patch_file, download/upload, sn, ...)
- `src/evaluator/` and `src/runner.rs` — environment expression rendering and flow scheduling
- `src/cli.rs` + `src/cmd/` — clap command surface (`gx_cmd.rs`, `gxl_cmd.rs`)
- `src/self_update/` — `gx self` check/update/rollback
- `src/util/`, `src/conf/`, `src/friendly/` — helpers, configuration, output formatting

Workspace crates: `crates/orion_parse` (parser-combinator helpers) and `crates/orion_cond` (condition evaluation).

Reference diagrams live under `docs/structure/` (structural facts are in `docs/structure/*-actual.md`), guides in `docs/guidle/`, GXL docs in `docs/gxl/`. Runnable samples live in `examples/`, integration tests in `tests/`.

## Build, Test, and Development Commands
- `cargo check --workspace` — fast validation; run before committing.
- `cargo build --workspace` — builds the `gx` binary.
- `cargo run --bin gx -- --help` — sanity-check CLI wiring after argument changes.
- `cargo fmt --all` / `cargo fmt --all -- --check` — apply and verify formatting.
- `cargo clippy --all-targets --all-features -- -D warnings` — the lint gate CI enforces; it must be clean.
- `cargo test --all-features -- --test-threads=1` — run suites the way CI does; serial execution is required because tests change the process working directory.
- Optional features: `res_depend_test`, `network_test`.

## Coding Style & Naming Conventions
Adopt `rustfmt` defaults (4-space indent, trailing commas in multiline literals). Use snake_case for modules, files, and functions; CamelCase for types and traits. Keep `app/gx/main.rs` thin by delegating orchestration to the library crate.

Error handling is centralized on `orion-error`: business failures are `RunResult`/`RunReason`, and IO/HTTP/parse boundaries use `source_err`/`source_raw_err` so underlying sources survive into reports. Return errors instead of panicking — avoid introducing new `unwrap()`, `expect()`, or `panic!` on non-test paths. Document any new public API surface or ability so flow authors can discover it via `gx doc`.

## Documentation
Markdown under `docs/` is **not documentation-only**. `src/help.rs` embeds `docs/guidle/index.md`, `docs/guidle/cli/gx.md`, and `docs/gxl/**` via `include_str!` and serves them through `gx doc <topic>`. Editing those files changes shipped CLI output, and every path listed in `src/help.rs` must keep existing or the build breaks.

`docs/gxl/**` is mirrored into the `operator-docs` repository, which publishes it as the GXL section of its mdBook. This repository is the source of truth for those pages, with one exception: **`docs/gxl/example/*.md` is owned by `operator-docs`** and rewritten there in an operator-facing style, so never sync over it.

Use `scripts/sync-gxl-docs.sh sync|check` to regenerate or verify the mirror; the script header documents the exact boundary. `operator-docs` CI runs `check` on push, on pull requests, and daily against this repository's `main`.

## Testing Guidelines
Colocate unit tests inside `#[cfg(test)]` modules and name integration files `tests/<feature>_test.rs`. Leverage `rstest` for parameterized coverage and `tempfile` when touching the filesystem. `tests/example_test.rs` runs the runnable samples under `examples/` (read, shell, fun, assert, template, transaction, dryrun; the AI sample is `#[ignore]`d), so new samples are expected to be runnable end to end. New abilities need positive and failure assertions plus a CLI smoke run (`cargo run --bin gx -- ...`). Keep tests deterministic: mock through `mockall`, or gate network-dependent cases behind the `network_test` feature instead of making external calls.

## Commit & Pull Request Guidelines
Commit subjects stay short and imperative (e.g. `Add gx.sn serial number ability`, `Preserve structured error sources`) and cover one logical change. Record user-visible changes in both `CHANGELOG.md` and `CHANGELOG_CN.md`. Pull requests should summarize workflow impact and list the manual test commands that were run; attach screenshots for CLI UX changes. Wait for CI green before merge.

## Configuration & Security Tips
There is currently **no AI backend wired in**: `orion-ai` is commented out in `Cargo.toml` and `ai_diagnose` is a no-op, so do not add code that assumes AI credentials exist. Never hardcode secrets or vendor URLs. Validate `.gxl` workflow samples with the parser before publishing. When introducing configuration keys, update `src/conf/` defaults and note deployment expectations in `docs/` to keep environments reproducible.
