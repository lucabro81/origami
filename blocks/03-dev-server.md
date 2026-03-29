# Block 03 — Dev Server

**Version:** 0.1 — draft

## Contents

- [Scope](#scope)
- [Expected output](#expected-output)
- [Design principles](#design-principles)
- [origami-dev](#origami-dev)
  - [Process model](#process-model)
  - [File watcher](#file-watcher)
  - [Incremental compilation](#incremental-compilation)
  - [Nuxt process management](#nuxt-process-management)
  - [API proxy](#api-proxy)
  - [Error reporting](#error-reporting)
- [origami.toml — dev configuration](#origamitoml--dev-configuration)
- [Runtime target roadmap](#runtime-target-roadmap)
- [CLI contribution](#cli-contribution)
- [Tests](#tests)

---

## Scope

This block implements the `origami dev` command: the single entry point for the local development loop. It includes the `origami-dev` crate, which orchestrates the Origami compiler and the Nuxt dev server as a unified process, with file watching, incremental recompilation, and transparent process management.

Part of M2 (Developer experience). This is the first showable milestone — the point where someone other than the author can run the framework and experience it.

---

## Expected output

- `origami dev` starts the full development environment with a single command
- File changes to `.ori` files trigger recompilation and browser update without manual intervention
- Compiler errors appear in the terminal with `miette` formatting; the Nuxt process keeps running
- `Ctrl-C` cleanly terminates all child processes
- The developer never sees Nuxt, Vite, or any intermediate tooling — one command, one log stream

---

## Design principles

**One command, transparent internals.** The developer runs `origami dev`. What happens underneath — compiler watch loop, Nuxt process, proxy — is an implementation detail. No config files to touch, no separate terminals, no orchestration scripts.

**Compiler errors do not kill the server.** A compilation error in an `.ori` file is reported in the terminal but does not terminate the Nuxt process. The last valid compiled output remains live. The developer fixes the error and the watch loop recompiles automatically.

**Nuxt is a managed dependency, not a peer.** Nuxt is installed in the generated project's `node_modules`. The developer does not interact with it directly. `origami-dev` spawns it, forwards its output, and kills it on exit. This relationship is intentional: Nuxt is an intermediate runtime, not a permanent architectural commitment.

**Log streams are unified.** Compiler output and Nuxt output are printed to the same terminal, prefixed to distinguish their source. The developer sees one coherent log, not two interleaved streams.

---

## origami-dev

New crate. Owns the full development loop: file watching, compilation dispatch, Nuxt process lifecycle, and signal handling.

### Process model

`origami dev` runs two concurrent tasks under a single `tokio` runtime:

```
origami dev
  │
  ├── Compiler loop (sync, run in tokio::task::spawn_blocking)
  │     watch .ori files
  │     on change → recompile → write __generated__/
  │
  └── Nuxt process (async)
        spawn: nuxt dev --rootDir __generated__/
        forward stdout/stderr to terminal (prefixed)
        restart on __generated__/ change (Nuxt handles HMR internally)
```

The compiler loop runs synchronously on a blocking thread — the compiler pipeline (lexer → parser → analyzer → codegen → router → data) is entirely synchronous. `tokio::task::spawn_blocking` bridges it into the async runtime without blocking the event loop.

The Nuxt process is spawned with `tokio::process::Command`. Its stdout and stderr are piped and forwarded line-by-line to the terminal. Nuxt's own HMR handles browser updates when `__generated__/` changes — `origami-dev` does not implement HMR directly.

### File watcher

`notify` (the same crate used by cargo) watches the `.ori` source files and `origami.toml`. Debouncing is applied to avoid redundant recompilations on rapid successive saves (typical editor behaviour):

```
watcher configuration:
  recursive: true
  paths: [apps/, packages/, tokens.json, endpoints.toml, origami.toml]
  debounce: 50ms
  events: Create, Write, Remove, Rename
```

`tokens.json` and `endpoints.toml` changes also trigger a full recompilation — the design system and data bindings are compile-time inputs.

### Incremental compilation

Full recompilation of the entire workspace on every file change is acceptable for the initial implementation. The compiler is fast (Rust, no JS in the hot path), and the expected project size at this stage makes full recompilation non-problematic.

Incremental compilation — recompiling only the files affected by a change — is a future optimisation, not a requirement for this block. When added, it belongs in `origami-codegen` and `origami-router`, not in `origami-dev`.

### Nuxt process management

Before spawning Nuxt, `origami-dev` verifies that the required runtime is available:

```
check: node or bun on PATH
  if not found:
    print one-line install instruction
    exit with non-zero code
```

Bun is preferred over Node as the JS runtime: faster startup, single binary, drops in as a Node replacement. If Bun is not found, Node is the fallback. If neither is found, the command fails with a clear message.

The Nuxt process is spawned pointing at `__generated__/` as its root directory. The generated `nuxt.config.ts` (emitted by `origami-codegen`) configures Nuxt to look for pages, layouts, and components in the right places.

**Signal handling:** `Ctrl-C` sends `SIGTERM` to the Nuxt child process before exiting. `tokio`'s signal handling ensures the child is not orphaned. On Windows, `CTRL_C_EVENT` is used instead.

### API proxy

API requests from the browser are proxied to `api_base_url` from the active `[env.*]` section in `origami.toml`. In the current implementation (Nuxt + Vite), the proxy is configured in the generated `nuxt.config.ts`:

```typescript
// __generated__/nuxt.config.ts (generated, not user-visible)
export default defineNuxtConfig({
  devServer: { proxy: { '/api': { target: process.env.ORIGAMI_API_BASE_URL } } }
})
```

`ORIGAMI_API_BASE_URL` is passed to the Nuxt process as an environment variable by `origami-dev`, sourced from the active env section. The developer never configures this manually.

When the dev server is replaced with a Rust-native implementation (see Runtime target roadmap), proxying moves into the Rust server directly.

### Error reporting

Compiler errors are printed to stdout using `miette`'s default formatter, prefixed with `[origami]`:

```
[origami] error[CLT401] — apps/web/pages/books/index.ori
  ...
[nuxt]    ✓ Nuxt 3.x.x ready
[nuxt]    ➜ Local: http://localhost:3000
```

The Nuxt process continues running after a compiler error. The last successfully compiled `__generated__/` output remains live. On the next successful recompilation, a `[origami] ✓ compiled` message is printed.

---

## origami.toml — dev configuration

Relevant sections for `origami dev`:

```toml
[compiler]
openapi = "./api/openapi.json"

[env.dev]
api_base_url = "http://localhost:8080"
port = 3000

[env.staging]
api_base_url = "https://api.staging.example.com"
```

`origami dev` uses `[env.dev]` by default. `--env staging` switches to the staging section. `--port` overrides the port for the local server. `--host` sets the bind address (default `localhost`).

---

## Runtime target roadmap

`origami-dev` is designed to evolve as the framework matures toward JS independence. The process model above (compiler loop + Nuxt child process) is the current implementation. Future phases replace the Nuxt layer progressively:

```
Phase        Dev server                            JS runtime required
─────────────────────────────────────────────────────────────────────
Current      origami-dev spawns Nuxt (Vite)        Bun or Node
Next         Farm replaces Nuxt/Vite               None
             (Rust-native, Vite-compatible)
Later        axum + Rust SFC compiler embedded     None
             in origami-dev binary
SSR          axum serves SSR; Bun optional         None
WASM         Full WASM target                      None
```

Each phase replaces only the child process that `origami-dev` spawns. The file watcher, compilation loop, signal handling, and log forwarding remain unchanged across all phases. This is the architectural seam that makes the progression incremental rather than a rewrite.

The "Next" phase (Farm) is the first point where `origami dev` becomes a true single-binary experience with no JS runtime requirement. It is not a prerequisite for a usable framework — Nuxt is sufficient and fast enough for the current stage.

---

## CLI contribution

This block fully implements the `origami dev` subcommand:

```
origami dev [--app <name>] [--env <name>] [--port <n>] [--host <host>]
```

- `--app`: select which app to run in a multi-app workspace (default: the only app, or `web` if multiple exist)
- `--env`: select the env section from `origami.toml` (default: `dev`)
- `--port`: override the local server port (default: from `[env.dev]`, or 3000)
- `--host`: bind address (default: `localhost`)

---

## Tests

The dev server is difficult to test with traditional unit tests — it is inherently a process orchestration concern. Testing strategy:

**Unit tests in `origami-dev/src/tests.rs`:**
- Debounce logic: rapid file events produce a single recompilation trigger
- Environment variable resolution: correct `api_base_url` for each env section
- Runtime detection: Bun found → use Bun; Bun not found → Node; neither → error message

**Integration tests:**
Full dev server integration tests (spawn process, verify output, send Ctrl-C) are deferred — they are slow, flaky in CI, and require a real Nuxt installation. They are candidates for a dedicated `e2e/` test suite once the framework is stable enough to run against itself.

The primary validation for this block is manual: `origami dev` on the M1 fixture app, verify the browser updates on `.ori` file save, verify error messages appear on broken syntax, verify clean shutdown on `Ctrl-C`.
