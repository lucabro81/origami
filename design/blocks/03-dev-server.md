# Block 03 — Dev Server

Implements `origami dev`: the single entry point for the local development loop. New crate `origami-dev` orchestrates the Origami compiler and a Nuxt child process with file watching and unified log output.

Part of M2 (first showable milestone).

---

## Checklist — done when all of these are true

- [ ] `origami dev` starts the full development environment with a single command
- [ ] `.ori` file changes trigger recompilation and browser update without manual intervention
- [ ] Compiler errors appear in terminal with `miette` formatting; Nuxt keeps running
- [ ] `Ctrl-C` cleanly terminates Nuxt child process (no orphaned processes)
- [ ] Unified log stream: compiler output prefixed `[origami]`, Nuxt output prefixed `[nuxt]`
- [ ] Bun/Node detection: Bun preferred, Node fallback, clear message if neither found
- [ ] Unit tests pass: debounce logic, env resolution, runtime detection
- [ ] `cargo clippy -- -D warnings` clean, `rustfmt` applied

---

## Process model

`origami dev` runs two concurrent tasks under a single `tokio` runtime:

```
origami dev
  │
  ├── Compiler loop  (sync — tokio::task::spawn_blocking)
  │     watch .ori files + origami.toml + tokens.json + endpoints.toml
  │     on change → recompile → write __generated__/
  │
  └── Nuxt process  (async — tokio::process::Command)
        spawn: nuxt dev --rootDir __generated__/
        pipe stdout/stderr → terminal (prefixed [nuxt])
        Nuxt's own HMR updates the browser on __generated__/ change
```

The compiler loop is entirely synchronous. `tokio::task::spawn_blocking` bridges it into the async runtime. `origami-dev` is the **only** crate allowed to use `tokio`.

---

## File watcher

Uses `notify` crate (same as cargo). Debounced to avoid redundant recompilations on rapid saves.

```rust
// watcher configuration
recursive: true
paths: ["apps/", "packages/", "tokens.json", "endpoints.toml", "origami.toml"]
debounce: 50ms
events: [Create, Write, Remove, Rename]
```

`tokens.json` and `endpoints.toml` changes trigger full recompilation — they are compile-time inputs.

Full recompilation on every change is acceptable for the initial implementation. Incremental compilation is a future optimisation — belongs in `origami-codegen` and `origami-router`, not here.

---

## Nuxt process management

Before spawning Nuxt:

```
1. Check PATH for `bun` → use Bun (preferred: faster startup, single binary)
2. If not found, check PATH for `node` → use Node as fallback
3. If neither found → print one-line install instruction, exit non-zero
```

Nuxt is spawned pointing at `__generated__/` as its root directory. The generated `nuxt.config.ts` (emitted by `origami-codegen`) configures pages, layouts, components, and the API proxy.

**Signal handling:** `Ctrl-C` → send `SIGTERM` to Nuxt child process before exiting. Use `tokio`'s signal handling. On Windows: `CTRL_C_EVENT`. Child must not be orphaned.

---

## API proxy

`origami-dev` passes `ORIGAMI_API_BASE_URL` to the Nuxt process as an environment variable, sourced from the active `[env.*]` section in `origami.toml`. The generated `nuxt.config.ts` configures the proxy:

```typescript
// __generated__/nuxt.config.ts (generated, never user-visible)
export default defineNuxtConfig({
  devServer: { proxy: { '/api': { target: process.env.ORIGAMI_API_BASE_URL } } }
})
```

Developer never configures this manually.

---

## Error reporting

Compiler errors are printed via `miette`, prefixed `[origami]`. Nuxt keeps running after a compiler error — last valid `__generated__/` remains live.

```
[origami] error[CLT401] — apps/web/pages/books/index.ori
  ...
[nuxt]    ✓ Nuxt 3.x.x ready
[nuxt]    ➜ Local: http://localhost:3000
```

On next successful recompilation: `[origami] ✓ compiled`.

---

## origami.toml — relevant sections

```toml
[compiler]
openapi = "./api/openapi.json"

[env.dev]
api_base_url = "http://localhost:8080"
port = 3000
```

`origami dev` uses `[env.dev]` by default. `--env <name>` switches sections. `--port` overrides port. `--host` sets bind address (default `localhost`).

---

## CLI command

```
origami dev [--app <name>] [--env <name>] [--port <n>] [--host <host>]
```

- `--app`: select app in multi-app workspace (default: sole app, or `web` if multiple)
- `--env`: env section from `origami.toml` (default: `dev`)
- `--port`: override port (default: from `[env.dev]`, or 3000)
- `--host`: bind address (default: `localhost`)

---

## Runtime target roadmap

`origami-dev`'s process model is designed to evolve. The architecture of the file watcher, compiler loop, signal handling, and log forwarding is **unchanged** across all phases — only the child process changes.

| Phase | Dev server | JS runtime required |
|-------|-----------|---------------------|
| Current | Rust compiler + Nuxt child process (Bun preferred) | Bun or Node |
| Next | Farm replaces Nuxt/Vite (Rust-native, Vite-compatible) | None |
| Later | axum + Rust SFC compiler embedded in binary | None |
| SSR | axum serves SSR; Nuxt SSR via `nuxt build` | Bun or Node |
| WASM | Full WASM target | None |

---

## Tests

### Unit tests — `origami-dev/src/tests.rs`

- Debounce: rapid file events produce a single recompilation trigger, not multiple
- Env resolution: correct `api_base_url` selected for each `[env.*]` section
- Runtime detection: Bun found → use Bun; Bun not found → Node; neither → error with message

### Integration tests

Full dev server integration tests (spawn process, verify browser update, send Ctrl-C) are **deferred** — slow, flaky in CI, require a real Nuxt installation.

Primary validation is manual: `origami dev` on M1 fixture app.
- Edit `.ori` file → browser updates without manual reload
- Introduce syntax error → error appears in terminal, Nuxt keeps running
- Fix error → browser updates, `[origami] ✓ compiled` appears
- `Ctrl-C` → clean shutdown, no orphaned processes
