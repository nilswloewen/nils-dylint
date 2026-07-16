# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Cargo workspace of [dylint](https://github.com/trailofbits/dylint) lints — Rust
lint plugins that are compiled as `cdylib`s and loaded dynamically by the
`cargo-dylint` driver against a target crate. Each workspace member is one
distributable lint library.

Currently one member: `blank_line_before_return` (stubbed — the
`LateLintPass` impl in `src/lib.rs` is a TODO).

## Build & test

```
cargo build                            # build every lint cdylib in the workspace
cargo test -p blank_line_before_return # run that lint's UI tests
```

UI tests live under `<lint>/ui/`. The fixture is a `.rs` file and the expected
diagnostic output is a sibling `.stderr`. `dylint_testing` v6 builds its own
`compiletest::Config` and **does not** plumb through `BLESS=1` or accept
`-- --bless`. To regenerate a fixture after changing the lint, run the test —
on mismatch compiletest writes the actual output to `/tmp/<fixture>.stage-id.stderr`
— then copy it into place:

```
cargo test -p blank_line_before_return   # fails; writes /tmp/*.stage-id.stderr
cp /tmp/<fixture>.stage-id.stderr <lint>/ui/<fixture>.stderr
```

Running the lint against a real crate uses the dylint CLI
(`cargo install cargo-dylint dylint-link` once):

```
cargo dylint --path blank_line_before_return -- --manifest-path <target>/Cargo.toml
```

## Toolchain & deps — these are tightly coupled

- `rust-toolchain` pins **nightly-2026-05-29** (`1.98.0-nightly`) with
  `rustc-dev` + `llvm-tools-preview`. `rustc_private` APIs are unstable; this
  exact nightly is what the lints compile against.
- **Why a 1.98 nightly to lint a 1.97.1 target:** the driver nightly's numeric
  version must clear the *target workspace's* `rust-version` (MSRV). Every
  nightly in the 1.97 cycle reports `1.97.0-nightly` → cargo reads `1.97.0`,
  which is `< 1.97.1`, so a target with `rust-version = "1.97.1"` rejects the
  driver during `cargo fix`/`cargo dylint`. Only a `1.98.0-nightly` (post
  branch-cut, `>= 1.97.1`) clears it. nightly-2026-05-29 is the earliest 1.98
  nightly, so drift from the 1.97.1 target is minimal. (Alternatively the
  consumer passes `--ignore-rust-version`, but that relaxes MSRV checking for
  the whole resolve — pinning a 1.98 nightly keeps the check intact and needs
  no flag.)
- `clippy_utils` is pinned to a **git rev** in each lint's `Cargo.toml` (rev
  `e8fdfcc0…`, `0.1.98`). That rev must be compatible with the pinned nightly —
  bump them together, never one without the other. The rev's own
  `rust-toolchain.toml` self-documents which nightly it needs (crates.io also
  publishes `clippy_utils`, but the published version does *not* record its
  nightly, which is why the git rev is preferred here). Note the ~1-day dist
  skew: a rev's declared `nightly-YYYY-MM-DD` was built from the *previous*
  day's rustc, so it often needs the *next* day's dist nightly (e.g. rev
  declares `2026-05-28` → build against `nightly-2026-05-29`). Same coupling
  applies to `dylint_linting` / `dylint_testing` when upgrading the nightly.
- `.cargo/config.toml` forces `linker=dylint-link` for all targets. Without
  it, linking the `cdylib` against rustc's private crates fails. Don't remove
  that flag; if you ever need a different linker for a non-lint binary, scope
  the override to `[target.'cfg(...)']` instead of replacing it.

## Adding a new lint to the workspace

1. Create `<lint_name>/` with the same shape as `blank_line_before_return/`:
   `Cargo.toml` (`crate-type = ["cdylib"]`, `publish = false`, the same
   `clippy_utils` + `dylint_linting` + `dylint_testing` deps, and the
   `[package.metadata.rust-analyzer] rustc_private = true` block so RA loads
   the rustc sysroot crates), plus `src/lib.rs` and `ui/`.
2. Add it to `members` in the root `Cargo.toml`.
3. In `src/lib.rs` start with `#![feature(rustc_private)]`, `extern crate` the
   `rustc_*` crates you use, and declare the lint with
   `dylint_linting::declare_late_lint!` (or `declare_early_lint!` /
   `impl_late_lint!` for multi-lint libs). The macro generates the registration
   entry point dylint looks for — don't write that by hand.
4. Add a UI test that calls `dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui")`.

## Editor / rust-analyzer

The `rustc_private = true` metadata block in each lint's `Cargo.toml` is what
lets rust-analyzer resolve `rustc_hir`, `rustc_lint`, etc. — if RA can't find
those crates after adding a new lint, that block is missing.
