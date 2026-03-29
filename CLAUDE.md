# Origami — CLAUDE.md

## Contents

- [Project](#project)
- [Repository layout](#repository-layout)
- [Design documents](#design-documents)
- [Rust conventions](#rust-conventions)
- [Backlog](#backlog)

## Project

Origami is a fullstack opinionated framework with a closed-vocabulary DSL markup language (`.ori` files) that enforces design system compliance at compile time. Target: general-purpose production applications.

The Clutter POC (`../clutter/`) is the reference implementation for the compiler pipeline. Decisions made there are still valid unless explicitly superseded here.

## Repository layout

- `crates/` — all Rust crates (library + one binary: `origami-cli`)
- `fixtures/` — shared `.ori` test fixtures, used by integration tests
- `design/` — design documents (see below)

## Design documents

All design documents live in `design/`. Before working on any area, consult the relevant document there — they define scope, interface contracts, and design principles that the implementation must respect.

- `design/framework-spec.md` — full framework spec (source of truth)
- `design/milestones.md` — delivery milestones
- `design/backlog.md` — deferred items and future work
- `design/blocks/` — one document per independently deliverable block

## Rust conventions

- No `unwrap()` or `expect()` outside of tests — all error cases are explicit
- `clippy` is treated as errors: `cargo clippy -- -D warnings` must pass clean
- `rustfmt` is non-negotiable: all code must be formatted before commit
- Unit tests live in `src/tests.rs` per crate, declared as `#[cfg(test)] mod tests;` in `lib.rs`
- Integration tests live in `crates/origami-cli/tests/` and use fixtures from `fixtures/`
- `.ori` fixtures live in `fixtures/` at workspace root, shared across crates
- Error types are defined in `origami-runtime`, never in individual crates
- No crate may introduce a dependency cycle — if a type needs to be shared, move it to `origami-runtime`
- `tokio` is only a dependency of `origami-dev` — all other crates are synchronous
- Prefer `&str` over `String` in function signatures where ownership is not required
- Use `thiserror` for error type definitions, `miette` for user-facing error reporting in the CLI
- All public types and functions must be coherent with the interfaces defined in the relevant block document

## Backlog

`design/backlog.md` collects deferred items and future work that emerged during block design. After completing any block, review it: some items may have been resolved incidentally, and new ones may be worth adding.
