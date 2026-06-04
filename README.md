# nils-dylint

A collection of custom Rust lints, distributed as
[dylint](https://github.com/trailofbits/dylint) libraries. Consumers point
`cargo dylint` at this repo and it clones, builds, and runs the lints
locally — no binaries are checked in.

## Lints

### `blank_line_before_return`

Flags the trailing **implicit** tail expression of a block when it fits on a
single line and isn't preceded by a blank line. Explicit `return x;`,
multi-line tails, and single-expression bodies are left alone. The
suggestion is `MachineApplicable`, so `--fix` will apply it for you.

```rust
// ⚠️  warned
fn foo() -> i32 {
    let x = compute();
    x
}

// ✅  preferred
fn foo() -> i32 {
    let x = compute();

    x
}

// ✅  also fine — explicit `return` is out of scope
fn bar() -> i32 {
    let x = compute();
    return x;
}
```

## Usage

### 1. Install the dylint runners (one-time, per machine)

```bash
cargo install cargo-dylint dylint-link
```

`cargo-dylint` drives the lint run; `dylint-link` is the linker shim that
each lint is built with.

### 2. Register the lint in your project

Add to your project's root `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { git = "https://github.com/nilswloewen/nils-dylint", branch = "master", pattern = "blank_line_before_return" },
]
```

`pattern` is the lint crate's directory name inside this repo. The default
branch of this repo is `master` (not `main`). To pin to a specific revision
instead (recommended for reproducibility — `rustc_private` APIs drift
between nightlies):

```toml
{ git = "https://github.com/nilswloewen/nils-dylint", rev = "<commit-sha>", pattern = "blank_line_before_return" },
# or
{ git = "https://github.com/nilswloewen/nils-dylint", tag = "v0.1.0",      pattern = "blank_line_before_return" },
```

A non-workspace project uses `[package.metadata.dylint]` instead of
`[workspace.metadata.dylint]`.

### 3. Run

```bash
cargo dylint --all              # run every registered lint
cargo dylint --all --fix        # apply autofixes in place
cargo dylint blank_line_before_return   # run just this one
```

First run clones the repo into `~/.dylint_drivers/` and builds the lint
against the nightly pinned in this repo's `rust-toolchain` (rustup fetches
that toolchain automatically). Subsequent runs are cached.

## Toolchain

Lints are built against a specific Rust nightly (currently
`nightly-2026-04-16`, pinned in [`rust-toolchain`](rust-toolchain)).
`cargo dylint` uses the pinned nightly to compile the lint and your
project's own toolchain to compile your code — you don't need to switch
nightlies yourself.

## Repo layout

```
.
├── blank_line_before_return/   # one lint = one cdylib crate
│   ├── src/lib.rs              # the LateLintPass impl
│   └── ui/                     # UI test fixtures (*.rs + *.stderr)
├── Cargo.toml                  # workspace root
├── rust-toolchain              # pinned nightly
└── .cargo/config.toml          # forces `dylint-link` as the linker
```

See [`CLAUDE.md`](CLAUDE.md) for development notes (toolchain coupling,
fixture-bless workflow, the shape to copy when adding a new lint).

## License

No license declared yet.
