# Origami — Milestones

## Contents

- [M0 — Stack](#m0--stack)
- [M1 — Compilable SPA](#m1--compilable-spa-internal-checkpoint)
- [M2 — Developer experience](#m2--developer-experience-first-showable-state)
- [M3 — Feature-complete framework](#m3--feature-complete-framework)
- [M4 — Production-ready](#m4--production-ready)
- [M5 — SSR](#m5--ssr-near-term)
- [North star — WASM](#north-star--wasm-long-term)

Each milestone is an independently verifiable delivery point. Every subsequent block builds on the previous ones, but each milestone produces something working.

The compiler pipeline code (lexer → parser → analyzer → codegen, `origami-runtime` crate) is ported from the Clutter POC at the start of M1. The POC is frozen — its role as a proof of concept is done.

---

## M0 — Stack

**Goal:** working Cargo workspace, green CI, CLI skeleton.

No user-facing functionality. The framework does nothing yet, but the structure is in place and compiles.

- Cargo workspace with all crates declared (even if empty)
- Key dependencies pinned (versions, features)
- `origami-cli` skeleton: `clap` configured, subcommands registered but not implemented
- CI: `cargo check`, `cargo test`, `cargo clippy` on every push
- Codebase conventions documented in `CLAUDE.md`

**Verification:** `cargo check` green across the workspace. `origami --help` shows all subcommands.

---

## M1 — Compilable SPA *(internal checkpoint)*

**Goal:** write an `.ori` app with pages and data, compile it, serve it manually with Nuxt.

Internal milestone: not showable to anyone without the dev server. Serves as an intermediate checkpoint before M2. The "opinionated, closed vocabulary, enforced design system" concept is already demonstrable with the Clutter POC — here we are building the real thing.

- Compiler pipeline ported from the POC (lexer, parser, analyzer, codegen, runtime)
- Block 01 — Router: `page`/`layout` keywords, file-to-route mapping, Nuxt pages/layouts generated
- Block 02 — Data Layer: `endpoints.toml`, OpenAPI parsing, `Response<T>`, `useQueryXxx`/`useMutationXxx`, type gen, `origamiFetch`
- `origami build` produces `__generated__/` with Nuxt pages, layouts, and types

**Verification:** sample app with 2-3 pages, dynamic routing, one query and one mutation compiled and running in the browser via `nuxt dev` launched manually.

---

## M2 — Developer experience *(first showable state)*

**Goal:** `origami dev` is the only command a developer needs to know.

- Block 03 — Dev Server: file watcher, spawn Nuxt dev server, proxy to `api_base_url`
- Incremental recompilation on save
- Compiler errors printed in the terminal via `miette`

**Verification:** edit an `.ori` file → browser updated in < 1s without manual reload. Compilation error → readable message in the terminal.

---

## M3 — Feature-complete framework

**Goal:** all modules active. The compiler enforces i18n correctness, test coverage by design, and a11y at compile time.

- Block 04 — i18n: `t()` validated, keys enforced to max 3 segments, locale file validation
- Block 05 — Testing: `test`/`e2e` block parsing, visual preview app, Playwright codegen, snapshot
- Block 06 — Accessibility: CLT6xx compile-time checks, color contrast from `tokens.json`, axe-core in preview

**Verification:** sample app with complete i18n, visual and E2E test suite generated automatically, zero a11y violations at compile time.

---

## M4 — Production-ready

**Goal:** the framework is distributable and usable from scratch by someone who has never seen the repo.

- Block 07 — CLI: `origami init`, `origami build` (production), `origami check` (CI), `origami unsafe-report`
- Optimized production build, correct exit codes for CI
- GitHub Actions: build binaries for macOS arm64 + Linux x86_64 on tag `v*`

**Verification:** `origami init my-app && cd my-app && origami dev` works on a clean machine. `origami check` passes in CI on a green PR.

---

## M5 — SSR *(near-term)*

**Goal:** production SSR deployment.

Since the codegen target is Nuxt, SSR is architecturally available from M2 — Nuxt handles it by default. This milestone is about making it a first-class, documented, production-supported mode rather than an accidental capability.

- Nuxt SSR mode enabled and tested end-to-end
- `origamiFetch` verified isomorphic (same behaviour server-side and client-side)
- Deployment docs: Bun as production runtime for SSR output

Note: this is considerably closer than originally planned. Nuxt's SSR support means there is no new codegen path to build — only validation, testing, and documentation.

---

## North star — WASM *(long-term)*

Compiler with a WASM target. No dependency on any JS runtime at any layer. Same `.ori` source, completely different target. This is the point of full independence from the JS ecosystem.
