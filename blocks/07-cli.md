# Block 07 — CLI

**Version:** 0.1 — draft

## Contents

- [Scope](#scope)
- [Expected output](#expected-output)
- [Design principles](#design-principles)
- [Commands](#commands)
  - [origami init](#origami-init)
  - [origami build](#origami-build)
  - [origami check](#origami-check)
  - [origami unsafe-report](#origami-unsafe-report)
- [Release pipeline](#release-pipeline)
- [origami-cli architecture](#origami-cli-architecture)
- [Tests](#tests)

---

## Scope

This block completes the CLI. Blocks 01–06 each contribute partial CLI implementations — subcommands that are wired up but not fully implemented. This block fills the remaining gaps: `origami init`, the production `origami build`, `origami check` as a standalone CI gate, and `origami unsafe-report`. It also implements the GitHub Actions release pipeline that produces distributable binaries.

Part of M4 (Production-ready). After this block, the framework is usable from scratch on a clean machine.

Commands already fully implemented in previous blocks (`origami dev` in Block 03, `origami test` in Block 05) are not repeated here — this block only documents what is new or completed.

---

## Expected output

- `origami init <project-name>` scaffolds a complete new workspace
- `origami build` produces an optimised production build and exits non-zero on any error
- `origami check` runs the full validation pipeline without emitting output — CI-ready
- `origami unsafe-report` lists all escape hatches in a structured, auditable format
- GitHub Actions workflow builds macOS arm64 and Linux x86_64 binaries on tag `v*`
- A `setup.sh` script installs the binary and bootstraps a new project on a clean machine

---

## Design principles

**One binary, zero prerequisites for the end user.** `origami init` and `origami build` work after installing the single binary. The only external dependencies are those required at runtime (Bun/Node for `origami dev` — not for `origami build`).

**CI-first exit codes.** Every command that validates the project exits non-zero on failure and zero on success. No partial exits, no "warnings treated as errors" flags — the behaviour is deterministic and scripting-friendly.

**`origami check` is the CI gate.** It runs the full validation pipeline (compiler, router, data layer, i18n, a11y) without emitting any generated output. It is fast and safe to run on every PR. `origami build` implies `check` — it will not produce output if the project does not pass validation.

**`unsafe-report` is a technical debt audit tool.** Escape hatches (`<unsafe>`, `as` overrides, W201 deviations) are intentionally visible and auditable. `origami unsafe-report` surfaces them all in one place — useful for periodic review, not for blocking CI.

---

## Commands

### origami init

```
origami init <project-name> [--app <name>] [--no-example]
```

Scaffolds a complete new Origami workspace:

```
<project-name>/
├── origami.toml
├── tokens.json
├── endpoints.toml
├── locales/
│   └── en.json
└── apps/
    └── <app-name>/          ← default: "web"
        ├── pages/
        │   └── index.ori    ← starter page (unless --no-example)
        └── components/
```

**`origami.toml` generated content:**
```toml
[project]
name = "<project-name>"
version = "0.1.0"

[compiler]
target = "nuxt"
default_locale = "en"

modules = ["data", "i18n", "a11y", "test"]

[env.dev]
api_base_url = "http://localhost:8080"
port = 3000

[env.test]
api_base_url = "http://localhost:9090"
```

**`tokens.json` generated content:** a minimal token set covering the 6 built-in components — enough to compile the starter page. Not a design system, just enough to not fail compilation.

**`--no-example`:** scaffolds the directory structure and config files only. No starter `.ori` files. For projects that want to start from a blank slate.

After scaffolding, `origami init` prints a short getting-started message:

```
✓ Project created: my-app/

Next steps:
  cd my-app
  origami dev
```

No `npm install` or similar — the developer does not interact with the JS toolchain directly.

### origami build

```
origami build [--app <name>] [--env <name>] [--locale <locale|all>] [--out <dir>]
```

Production build. Runs the full pipeline and emits optimised output.

**Pipeline:**
1. Run `origami check` — fail immediately if any validation error
2. Invoke `origami-router`, `origami-data`, `origami-i18n`, `origami-a11y` — collect all errors
3. If any errors: print all of them (miette), exit non-zero
4. Invoke `origami-codegen` — emit `__generated__/`
5. Run `nuxt build` in `__generated__/` — produce the production bundle in `--out` (default: `dist/`)
6. Exit zero

The production build runs `nuxt build`, which produces a static SPA by default (Nuxt's `ssr: false` mode). SSR output (`nuxt build` with `ssr: true`) is the M5 milestone.

**`--locale <locale>`:** bundle only the specified locale. Useful for locale-specific production deploys with smaller bundle size.

**`--out <dir>`:** output directory for the production bundle. Default: `dist/`. The directory is created if it does not exist; existing contents are replaced.

### origami check

```
origami check [--app <name>]
```

Validates the entire project without emitting any generated output. Designed for CI pre-merge gates.

**What it checks (in order):**
1. `origami.toml` and `endpoints.toml` parse and structure
2. OpenAPI spec fetch and parse (CLT305)
3. Full compiler pipeline (lexer → parser → analyzer) on all `.ori` files
4. Router validation: route conflicts, slot rules, keyword placement (CLT201–CLT206)
5. Data layer validation: type bindings, `Response<T>`/`Params<P>` coherence (CLT301–CLT308, CLT501–CLT502)
6. i18n validation: locale file consistency, key segment limits (CLT401–CLT403, W101–W103)
7. A11y compile-time checks (CLT601–CLT604)

Warnings (W-codes) are printed but do not cause a non-zero exit. Errors (CLT-codes) cause a non-zero exit. All errors are collected and printed together — not one at a time.

Output on success: nothing. Output on failure: all errors formatted with `miette`.

### origami unsafe-report

```
origami unsafe-report [--app <name>] [--format json]
```

Lists all escape hatches in the project in a structured, human-readable format:

```
Unsafe report — my-app

<unsafe> blocks: 2
  apps/web/components/MapEmbed.ori:12
    reason: "Third-party map component, no .ori equivalent"

  apps/web/pages/books/[id].ori:34
    reason: "Icon-only button, aria-label provided by parent context"

`as` overrides (W201): 1
  endpoints.toml:8 — loadReviews (was: useQueryReviewList)

Total escape hatches: 3
```

`--format json` emits machine-readable output for integration with dashboards or custom tooling.

The report is informational only — it never exits non-zero. It is a periodic review tool, not a CI gate.

---

## Release pipeline

GitHub Actions workflow at `.github/workflows/release.yml`. Triggered on tag push matching `v*`.

**Build matrix:**
- `macos-latest` (arm64) — produces `origami-macos-arm64`
- `ubuntu-latest` (x86_64) — produces `origami-linux-x86_64`

**Steps per target:**
1. Checkout
2. Install Rust stable
3. `cargo build --release`
4. Strip binary (Linux only — `strip target/release/origami`)
5. Upload binary as GitHub Release asset

**`setup.sh`** — generated and attached to the release. Downloads the correct binary for the current OS/arch, places it in `~/.local/bin/` (or `/usr/local/bin/` with `sudo`), and verifies the install:

```bash
#!/bin/sh
# generated setup.sh
ORIGAMI_VERSION="v0.1.0"
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# resolve download URL for this OS/arch
# download binary
# chmod +x
# echo "✓ origami $ORIGAMI_VERSION installed"
# echo "  Run: origami init my-app"
```

The exact `setup.sh` content is generated by the release workflow — the version string and download URL are injected at release time. The script is intentionally short and auditable.

---

## origami-cli architecture

`origami-cli` is the orchestrator. It owns no business logic — every subcommand delegates to the appropriate library crates. The pattern per subcommand:

```
parse args (clap)
  → load origami.toml
  → call origami-{module}::run(config)
  → collect Result<(), Vec<OriError>>
  → render errors with miette
  → exit(0) or exit(1)
```

`OriError` is defined in `origami-runtime` and is the common error type across all crates. The CLI's only job is parsing, orchestration, and rendering — never business logic.

**Internal module call order for `origami build` and `origami check`:**

```
origami-data     ← fetch + parse OpenAPI (once, shared across modules)
  ↓
origami-lexer → origami-parser → origami-analyzer   ← per file, parallel
  ↓
origami-router   ← route table from analyzed files
origami-data     ← data manifest from analyzed files + OpenAPI
origami-i18n     ← locale manifest
origami-a11y     ← a11y errors from analyzed files + token values
  ↓
origami-codegen  ← emit __generated__/ (build only, skipped in check)
```

All validation modules run after the core pipeline completes. Errors from all modules are collected and printed together.

---

## Tests

**Unit tests in `origami-cli/src/tests.rs`:**
- Exit code: zero on valid project, non-zero on any CLT error
- `origami check` does not write any files to `__generated__/`
- `origami unsafe-report` JSON output structure

**Integration tests in `crates/origami-cli/tests/`:**

```
tests/
  ├── init/             ← origami init produces correct directory structure
  ├── build_valid/      ← origami build on a valid fixture exits 0
  ├── build_invalid/    ← origami build on a fixture with CLT errors exits 1
  ├── check_valid/      ← origami check exits 0, writes nothing
  └── unsafe_report/    ← origami unsafe-report lists correct escape hatches
```

The `init` integration test runs `origami init`, then `origami check` on the scaffolded project, and expects exit 0 — verifying that the generated starter files are themselves valid.
