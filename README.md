# Origami

A fullstack opinionated framework with a closed-vocabulary DSL (`.ori` files) that enforces design system compliance at compile time.

---

## Contents

- [What is Origami](#what-is-origami)
- [Status](#status)
- [Getting started](#getting-started)
- [Development](#development)
- [Versioning](#versioning)

---

## What is Origami

Origami is built around a single idea: the design system is the type system. Components that violate it cannot compile.

You write `.ori` files. The compiler (written in Rust) validates them, enforces the design system, and generates a Nuxt/Vue application. You never touch the generated code.

Key properties:

- **Closed vocabulary** — only the components and props defined in `tokens.json` are valid
- **Compile-time type safety** — the frontend–backend API contract is validated at build time via OpenAPI
- **One command** — `origami dev` is the only entry point for local development
- **LLM-first authoring** — deterministic format, no ambiguity; compiler errors are the feedback loop

## Status

Early development. Not usable yet.

## Getting started

> Not ready for use. This section will be filled in once the framework is usable.

## Development

**Requirements:** Rust 1.89.0+

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Run the CLI locally:

```sh
cargo run -p origami-cli -- --help
```

## Versioning

All crates share a single workspace version. Releases are managed with [`cargo-release`](https://github.com/crate-ci/cargo-release):

```sh
cargo release minor   # 0.1.0 → 0.2.0
cargo release patch   # 0.1.0 → 0.1.1
```
