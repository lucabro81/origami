# Origami — CLAUDE.md

## Project

Origami is a Rust compiler for `.ori` DSL files that emits Nuxt/Vue 3 SFCs. It enforces design system compliance at compile time. One CLI binary. Closed vocabulary. No JS tooling in the hot path.

---

## Context loading

Do not load `design/framework-spec.md` as active context — it is too long and will degrade output quality.

**For any coding session:**
```
design/opencode/quick-ref.md       ← always load this
design/opencode/blocks/XX-name.md  ← load the block you are working on
relevant source files               ← load the files you are changing
```

Do not load two block docs at the same time unless one is a direct dependency of the other.

---

## Design documents

- `design/quick-ref.md` — compact framework reference (use during coding)
- `design/blocks/01-router.md` — file-based routing, `page`/`layout` keywords
- `design/blocks/02-data-layer.md` — `endpoints.toml`, OpenAPI, typed handles
- `design/blocks/03-dev-server.md` — `origami dev`, file watcher, Nuxt process
- `design/blocks/04-i18n.md` — `t()`, locale validation
- `design/blocks/05-testing.md` — `test`/`e2e` blocks, visual app, Playwright
- `design/blocks/06-accessibility.md` — compile-time a11y rules
- `design/blocks/07-cli.md` — `origami init`, `build`, `check`, `unsafe-report`, release
- `design/framework-spec.md` — full spec, source of truth (load only when needed)
- `design/backlog.md` — deferred items and future work (review after completing any block)

---

## Compiler pipeline — current state

`grammar.y` (workspace root) is the formal Bison grammar for the Origami template language. It is the source of truth for the parser.

**Lexer ✓ complete** (`origami-lexer`):
- `preprocess(input, filename) → Result<PreprocessResult, PreprocessorError>`: replaces opaque zones with `__LOGIC__` / `__UNSAFE__` placeholders, validates `----` position.
- `lex(preprocessed) → Result<Vec<Token>, LexError>`: logos-based tokeniser, rehydrates `LogicBlock` tokens, corrects spans via `offset_map`.
- Key token decisions: `RawBlock(String)` for all identifiers (disambiguation is the parser's job), `KwComponent`/`KwPage`/`KwLayout` as unit variants, `CloseTag(String)` carries tag name only.

**Parser — next** (`origami-parser`, `chumsky` v0.9): `Vec<Token> → AST`. Start with TDD. Consult memory for locked AST decisions (structured `Expr` tree, etc.).

---

## Repository layout

- `crates/` — all Rust crates (library + one binary: `origami-cli`)
- `fixtures/` — shared `.ori` test fixtures, used by integration tests
- `design/` — design documents
- `grammar.y` — formal grammar for the `.ori` template language

---

## Crate responsibilities

Each crate does one thing. No business logic in `origami-cli`.

| Crate | Role |
|-------|------|
| `origami-runtime` | Shared types + error types. No deps on other origami crates. |
| `origami-lexer` | `String → Vec<Token>` |
| `origami-parser` | `Vec<Token> → AST`, depends on `origami-runtime` |
| `origami-analyzer` | AST validation, depends on `origami-parser` |
| `origami-router` | Filesystem → `RouteTable`, depends on `origami-analyzer` |
| `origami-data` | `endpoints.toml` + OpenAPI → `DataManifest`, depends on `origami-router` |
| `origami-codegen` | AST + manifests → `.vue` files, depends on analyzer + router + data |
| `origami-i18n` | Locale validation, `t()` resolution |
| `origami-a11y` | Compile-time a11y checks |
| `origami-test` | Test block compiler, visual preview app, Playwright codegen |
| `origami-dev` | File watcher + Nuxt process management. Only crate allowed to use `tokio`. |
| `origami-cli` | Orchestrator only. Parses args, calls library crates, renders errors. |

---

## TDD

RULE 1: Write a failing test BEFORE writing any implementation code. No exceptions.
RULE 2: After writing tests, ask: edge cases covered? assertions meaningful? realistic usage? Iterate if not.
RULE 3: Implement only the minimum code to make the tests pass.
RULE 4: For bugs: discuss root cause with the user before writing any fix.
RULE 5: Write a test that reproduces the bug — it must fail before the fix, pass after.
RULE 6: Add a comment to regression tests explaining what bug they guard against.

---

## Code generation

- Generate one logical unit at a time.
- Before writing each unit, write one sentence describing what will be written and why.

---

## Rust — NEVER

- NEVER: `unwrap()` or `expect()` outside `#[cfg(test)]`
- NEVER: `tokio` as a dependency in any crate except `origami-dev`
- NEVER: define error types outside `origami-runtime`
- NEVER: introduce a dependency cycle between crates
- NEVER: commit with clippy warnings — `cargo clippy -- -D warnings` must be clean
- NEVER: commit without rustfmt formatting

## Rust — ALWAYS

- ALWAYS: use `thiserror` for error type definitions
- ALWAYS: use `miette` for CLI-facing error output
- ALWAYS: prefer `&str` over `String` in function signatures unless ownership is required
- ALWAYS: unit tests in `src/tests.rs`, declared as `#[cfg(test)] mod tests;` in `lib.rs`
- ALWAYS: integration tests in `crates/origami-cli/tests/`, fixtures from `fixtures/`
- ALWAYS: `.ori` fixtures in `fixtures/` at workspace root, shared across crates
- ALWAYS: all public types and functions must be coherent with the relevant block document

---

## Git commits

- One commit per logical unit of work. Small and atomic.
- `Co-Authored-By: Claude Sonnet 4.6` (no email address).
