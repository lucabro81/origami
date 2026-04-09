# Block 07 — CLI

Completes the CLI. Blocks 01–06 each contribute partial subcommand implementations. This block fills the remaining gaps: `origami init`, production `origami build`, `origami check` as a standalone CI gate, `origami unsafe-report`, and the GitHub Actions release pipeline.

Part of M4 (Production-ready). After this block, the framework is usable from scratch on a clean machine.

---

## Checklist — done when all of these are true

- [ ] `origami init <name>` scaffolds a complete workspace and passes `origami check`
- [ ] `origami build` runs full pipeline, emits optimised bundle, exits non-zero on any error
- [ ] `origami check` validates without writing any files to `__generated__/`, exits non-zero on CLT errors
- [ ] `origami unsafe-report` lists all escape hatches; `--format json` produces valid JSON
- [ ] GitHub Actions release workflow builds `macos-arm64` and `linux-x86_64` binaries on tag `v*`
- [ ] `setup.sh` attached to release, tested on clean macOS and Linux
- [ ] Unit tests pass: exit codes, `check` writes nothing, `unsafe-report` JSON structure
- [ ] Integration tests pass: init→check, build valid, build invalid, check valid, unsafe-report
- [ ] `cargo clippy -- -D warnings` clean, `rustfmt` applied

---

## origami-cli architecture

`origami-cli` is the orchestrator. It owns no business logic. Pattern per subcommand:

```rust
parse args (clap)
  → load origami.toml
  → call origami-{module}::run(config)
  → collect Result<(), Vec<OriError>>
  → render errors with miette
  → exit(0) or exit(1)
```

`OriError` is defined in `origami-runtime`. The CLI's only job: parse, orchestrate, render.

### Internal module call order for `origami build` and `origami check`

```
origami-data      ← fetch + parse OpenAPI spec (once, shared)
  ↓
origami-lexer → origami-parser → origami-analyzer   ← per file
  ↓
origami-router    ← route table from analyzed files
origami-data      ← data manifest from analyzed files + OpenAPI
origami-i18n      ← locale manifest
origami-a11y      ← a11y errors from analyzed files + token values
  ↓
origami-codegen   ← emit __generated__/ (build only — SKIPPED in check)
```

All validation modules run after the core pipeline. Errors from all modules collected and printed together.

---

## origami init

```
origami init <project-name> [--app <name>] [--no-example]
```

Scaffolds:

```
<project-name>/
├── origami.toml
├── tokens.json          ← minimal token set (enough to compile starter page)
├── endpoints.toml       ← empty
├── locales/
│   └── en.json
└── apps/
    └── <app-name>/      ← default: "web"
        ├── pages/
        │   └── index.ori    ← starter page (unless --no-example)
        └── components/
```

Generated `origami.toml`:

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

`--no-example`: scaffolds structure and config files only, no starter `.ori` files.

After scaffolding, print:

```
✓ Project created: my-app/

Next steps:
  cd my-app
  origami dev
```

No `npm install` — developer does not interact with the JS toolchain directly.

---

## origami build

```
origami build [--app <name>] [--env <name>] [--locale <locale|all>] [--out <dir>]
```

Pipeline:

1. Run full validation (same as `origami check`) — fail immediately if any CLT error
2. If errors: print all with `miette`, exit non-zero
3. Invoke `origami-codegen` → emit `__generated__/`
4. Run `nuxt build` in `__generated__/` → produce production bundle in `--out` (default: `dist/`)
5. Exit zero

`--locale <locale>`: bundle only the specified locale (smaller bundle for locale-specific deploy).
`--out <dir>`: output directory, default `dist/`. Created if missing, existing contents replaced.

Production build uses `nuxt build` with `ssr: false` (static SPA). SSR mode (`ssr: true`) is M5.

---

## origami check

```
origami check [--app <name>]
```

Validates without emitting any output files. CI-ready.

**Validation order:**

1. `origami.toml` and `endpoints.toml` parse and structure
2. OpenAPI spec fetch and parse (CLT305)
3. Full compiler pipeline on all `.ori` files (lexer → parser → analyzer)
4. Router: route conflicts, slot rules, keyword placement (CLT201–CLT206)
5. Data layer: type bindings, `Response<T>`/`Params<P>` coherence (CLT301–CLT308, CLT501–CLT502)
6. i18n: locale consistency, key segment limits (CLT401–CLT403, W101–W103)
7. A11y: compile-time checks (CLT601–CLT604)

Warnings (W-codes): printed, do not cause non-zero exit.
Errors (CLT-codes): all collected, printed together, non-zero exit.
Success: no output.

RULE: `origami check` must NEVER write any file to `__generated__/`.

---

## origami unsafe-report

```
origami unsafe-report [--app <name>] [--format json]
```

Lists all escape hatches in a structured format:

```
Unsafe report — my-app

<unsafe> blocks: 2
  apps/web/components/MapEmbed.ori:12
    reason: "Third-party map component, no .ori equivalent"

`as` overrides (W201): 1
  endpoints.toml:8 — loadReviews (was: useQueryReviewList)

Total escape hatches: 3
```

`--format json`: machine-readable output for dashboards or tooling.

This command is informational only — **never exits non-zero**. It is a periodic review tool, not a CI gate.

---

## Release pipeline

GitHub Actions at `.github/workflows/release.yml`. Trigger: tag push matching `v*`.

**Build matrix:**
- `macos-latest` (arm64) → `origami-macos-arm64`
- `ubuntu-latest` (x86_64) → `origami-linux-x86_64`

**Steps per target:**

1. Checkout
2. Install Rust stable
3. `cargo build --release`
4. Strip binary (Linux only: `strip target/release/origami`)
5. Upload as GitHub Release asset

**`setup.sh`** — attached to release. Downloads correct binary for OS/arch, places in `~/.local/bin/` (or `/usr/local/bin/` with sudo), verifies install:

```bash
#!/bin/sh
ORIGAMI_VERSION="v0.1.0"  # injected at release time
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
# resolve URL, download, chmod +x
echo "✓ origami $ORIGAMI_VERSION installed"
echo "  Run: origami init my-app"
```

Version string and download URL injected at release time. Script is intentionally short and auditable.

---

## Tests

### Unit tests — `origami-cli/src/tests.rs`

- Exit code zero on valid project, non-zero on any CLT error
- `origami check` does not write any file to `__generated__/`
- `origami unsafe-report --format json` produces valid JSON with correct structure

### Integration tests — `crates/origami-cli/tests/`

```
tests/
  ├── init/           ← origami init produces correct structure; origami check on result exits 0
  ├── build_valid/    ← origami build on valid fixture exits 0, produces dist/
  ├── build_invalid/  ← origami build on fixture with CLT errors exits 1
  ├── check_valid/    ← origami check exits 0, writes nothing to __generated__/
  └── unsafe_report/  ← origami unsafe-report lists correct escape hatches
```

The `init` integration test runs `origami init`, then immediately `origami check` on the scaffolded project — verifies that generated starter files are themselves valid.
