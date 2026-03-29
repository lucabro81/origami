# Block 00 — Stack

**Version:** 0.1 — draft

## Contents

- [Scope](#scope)
- [Expected output](#expected-output)
- [Toolchain](#toolchain)
- [Workspace structure](#workspace-structure)
- [Dependencies](#dependencies)
- [Crate dependency graph](#crate-dependency-graph)
- [CLI skeleton](#cli-skeleton)
- [CI](#ci)
- [Porting from the POC](#porting-from-the-poc)
- [Codebase conventions](#codebase-conventions)
- [Workspace versioning](#workspace-versioning)

## Scope

This block defines everything that must exist before any other block can begin: Rust toolchain, Cargo workspace structure, pinned dependencies, CLI skeleton, and CI pipeline. It produces no user-facing functionality. It produces the foundation everything else is built on.

## Expected output

- Cargo workspace compiling with all crates declared (even if empty)
- `origami --help` working with all subcommands registered but not implemented
- Green CI on every push
- `CLAUDE.md` in the repo with codebase conventions

---

## Toolchain

**Rust edition:** 2021. Do not move to 2024 until the tooling (rust-analyzer, clippy) is stable on it — the risk is not worth it at this stage.

**MSRV (Minimum Supported Rust Version):** pin to the current stable version at bootstrap time. Declared in the root `Cargo.toml` (`rust-version = "..."`) and enforced in CI.

**Required components:**
- `rustfmt` — automatic formatting, non-negotiable
- `clippy` — linting, treated as errors in CI (`-D warnings`)

---

## Workspace structure

```
origami/
├── Cargo.toml              ← workspace root (manifest only, no src/)
├── Cargo.lock              ← committed (binary crate project)
├── CLAUDE.md
├── .github/
│   └── workflows/
│       └── ci.yml
├── crates/
│   ├── origami-runtime/    ← shared types across all crates
│   ├── origami-lexer/
│   ├── origami-parser/
│   ├── origami-analyzer/
│   ├── origami-codegen/
│   ├── origami-router/
│   ├── origami-data/
│   ├── origami-dev/
│   ├── origami-i18n/
│   ├── origami-test/
│   ├── origami-a11y/
│   └── origami-cli/        ← binary crate, entry point
└── fixtures/               ← shared .ori test files
```

Each crate under `crates/` has the minimal structure:

```
crates/origami-foo/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── tests.rs    ← #[cfg(test)] mod tests — inline unit tests
```

`origami-cli` is the only binary crate. All others are library crates. There is no `src/main.rs` at the workspace root — the root `Cargo.toml` is a workspace manifest only.

Integration tests (end-to-end on real `.ori` files) live in `crates/origami-cli/tests/` and use fixtures from `fixtures/`.

---

## Dependencies

Versions are to be pinned at bootstrap time. This table defines the choices, not the exact numbers.

| Crate | Dependency | Rationale |
|-------|-----------|-----------|
| all | `origami-runtime` | shared types, error types |
| `origami-cli` | `clap` (v4, derive) | CLI argument parsing |
| `origami-cli` | `miette` (v5) | human-readable error reporting |
| `origami-lexer` | — | no external dependencies |
| `origami-parser` | `typed-arena` | AST node allocation without lifetime complexity |
| `origami-analyzer` | — | uses only runtime + parser output |
| `origami-codegen` | — | uses only runtime + analyzer output |
| `origami-router` | — | uses only runtime |
| `origami-data` | `serde`, `serde_json` | parsing `tokens.json`, OpenAPI types |
| `origami-data` | `toml` | parsing `endpoints.toml` |
| `origami-data` | `ureq` | fetching OpenAPI spec from HTTP URL |
| `origami-dev` | `notify` | cross-platform file watcher |
| `origami-dev` | `tokio` | async runtime for process orchestration |
| `origami-i18n` | `serde_json` | parsing locale files |
| `origami-test` | `serde_json` | fixture serialization and Playwright output |
| `origami-a11y` | — | uses only runtime |

**Deliberate choices:**

`ureq` over `reqwest` for OpenAPI fetching: it is synchronous, has zero async dependencies, and is small. The OpenAPI spec is fetched once at the start of compilation — async is not needed here.

`notify` for the file watcher: the de facto standard in Rust for this purpose, used by cargo itself.

`tokio` only in `origami-dev`: the only crate that needs real concurrency (spawning processes, handling stdout/stderr in parallel, killing processes on Ctrl-C). All other crates are synchronous.

---

## Crate dependency graph

The dependency graph must remain acyclic and respect this direction:

```
origami-cli
  ├── origami-router
  ├── origami-data
  ├── origami-dev
  ├── origami-i18n
  ├── origami-test
  ├── origami-a11y
  └── origami-codegen
        └── origami-analyzer
              └── origami-parser
                    └── origami-lexer
                          └── origami-runtime

(all crates depend on origami-runtime)
```

No crate may depend on `origami-cli`. No "leaf" crate may depend on a crate higher in the graph. If a new dependency would create a cycle, the solution is to move the shared type into `origami-runtime` — not to violate the graph.

---

## CLI skeleton

At the end of this block, `origami-cli` exposes all subcommands defined in the spec, each with their flags, but every handler is `unimplemented!()` or prints a placeholder.

```
origami
  ├── dev     [--app] [--env] [--port] [--host]
  ├── build   [--app] [--env] [--locale] [--out]
  ├── check   [--app]
  ├── test    [--preview] [--build-preview] [--snapshot]
  │           [--update-snapshots] [--e2e] [--a11y]
  │           [--filter] [--env] [--watch]
  ├── init    <project-name> [--app] [--no-example]
  └── unsafe-report [--app] [--format]
```

The `clap` structure uses the derive API (`#[derive(Parser, Subcommand)]`). Each subcommand is a separate struct in `crates/origami-cli/src/commands/`.

Errors are handled with `miette` from the start: every `Result` that reaches main is wrapped and printed with miette's error formatter.

---

## CI

GitHub Actions pipeline on every push and every PR targeting `main`.

**Job: `check`**
```
cargo fmt --check
cargo clippy -- -D warnings
cargo check --workspace
```

**Job: `test`**
```
cargo test --workspace
```

Both jobs run in parallel. PRs are not mergeable if either fails.

**Release** (separate, on tag `v*`): build binaries for macOS arm64 and Linux x86_64. Defined here, implemented in Block 07 (CLI).

---

## Porting from the POC

The stable crates from the Clutter POC (`origami-runtime`, `origami-lexer`, `origami-parser`, `origami-analyzer`, `origami-codegen`) are copied — not imported — at the start of M1, not in this block. This block declares the crates as empty placeholders in the workspace.

**What NOT to do:** do not import the POC as a Git dependency or submodule. The physical copy is deliberate: the POC is frozen, Origami is the real project.

---

## Codebase conventions

To be documented in the repo `CLAUDE.md`:

- Unit tests in `src/tests.rs` per crate (`#[cfg(test)] mod tests;` declared in `lib.rs`)
- Integration tests in `crates/origami-cli/tests/` with real fixtures
- `.ori` fixtures in `fixtures/` at workspace root, shared across crates
- Error types defined in `origami-runtime`, not in individual crates
- No `unwrap()` or `expect()` outside tests — all error cases are explicit
- `clippy` treated as errors: zero warnings tolerated

---

## Workspace versioning

Cargo 1.64+ supports `[workspace.package]` in the root `Cargo.toml`: all crates inherit the same version with `version.workspace = true`. A single bump updates the entire workspace.

The standard tool for bump management is **`cargo-release`**: reads configuration from the workspace, creates the bump commit, tags the release, and optionally publishes to crates.io (disabled for now). It is the Rust equivalent of `npm version` + changeset combined.

Typical workflow:

```
cargo release minor      # bump 0.1.0 → 0.2.0, commit, tag
cargo release patch      # bump 0.1.0 → 0.1.1, commit, tag
```

All crates share a single workspace-level version. With 11 internal crates that are not published separately, shared versioning is the only sane choice. Crates are not published to crates.io at this stage — `cargo-release` is used only for local bumps and Git tags. The CI pipeline builds binaries on `v*` tags.
