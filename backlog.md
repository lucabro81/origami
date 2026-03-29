# Backlog — cross-cutting and deferred items

Ideas and future work that emerged during block design. Items are grouped by area. They do not belong to the current block scope but must not be lost.

When a block is completed during a session, review this file: some items may have been resolved incidentally (mark or remove them), and new ideas may be worth adding.

---

## Block 01 — Router

| Item | Detail |
|------|--------|
| Multi-app routing | `WorkspaceManifest` holds `Vec<App>` to support multiple apps (`apps/web/`, `apps/admin/`). Block 01 implements and tests single-app routing only. The `--app` flag in `origami dev` and `origami build` is wired up but routes to a single default app. Full multi-app support is a follow-up once single-app structure is stable. |
| Route middleware | `_middleware.ts` files for guards and auth checks (`origami-middleware` crate, spec section 16). Deferred because it has no useful implementation without an auth provider. The generated `definePageMeta` calls already include a `middleware: []` slot — no structural changes needed when this is implemented. |

---

## Block 02 — Data Layer

| Item | Detail |
|------|--------|
| `origami-query` crate | Custom query/mutation library to replace TanStack Vue Query. Implements only the features Origami uses: keyed caching, eager/deferred toggle, mutation pattern. Scope is deliberately narrow — closed vocabulary means use cases are known in advance. This is an intermediate step before the WASM target. When swapped, only codegen templates change; `.ori` source files are unaffected. |
| `origami fix rename-type` | Codemod: `origami fix rename-type <old> <new>` applies a type rename across `endpoints.toml` and all page signatures in one pass, with a diff preview before applying. The compile error already gives all the context; this automates the fix. Not required for M4 — the error message is sufficient for manual correction. |
| `origami-auth` | Auth provider module with a standard `ctx.auth` interface: token management, session state, role-based access. Historically painful to integrate per-project — belongs in the framework. Depends on `origami-middleware` being in place first. |

---

## Block 03 — Dev Server

| Item | Detail |
|------|--------|
| Incremental compilation | Recompile only files affected by a change, rather than the full workspace. Belongs in `origami-codegen` and `origami-router`, not in `origami-dev`. Full recompilation is acceptable at current project sizes; this becomes relevant as apps grow. |
| Farm as Nuxt/Vite replacement | Farm is a Rust-native build tool with Vite-compatible plugins and no JS runtime requirement. Replacing the Nuxt child process with Farm is the first step toward a true single-binary dev experience. Not a prerequisite — Nuxt is sufficient and fast. |
| Embedded Rust dev server | axum + Rust SFC compiler (e.g. Vize) fully embedded in the `origami-dev` binary. Eliminates all JS runtime dependency from the dev loop. Follows Farm milestone. |
| Dev server integration tests | Full integration tests for `origami dev`: spawn process, verify browser update on file change, verify clean shutdown. Currently deferred — slow and flaky in CI, require a real Nuxt installation. Candidate for a dedicated `e2e/` test suite once the framework is stable enough to run against itself. |

---

## Block 04 — i18n

| Item | Detail |
|------|--------|
| `origami-i18n` runtime | Custom i18n runtime to replace `@nuxtjs/i18n`. Simpler than `origami-query` — only needs key lookup, `{variable}` interpolation, and `one`/`other` plural forms. Same principle: closed vocabulary means use cases are fully known. When swapped, only the codegen templates and the `nuxt.config.ts` emission change; `.ori` source files and locale JSON files are unaffected. |
| Complex ICU plural rules | Only `one`/`other` plural forms are supported in this version. CLDR rules (`few`, `many`, `zero`) are deferred. Required for languages like Polish, Arabic, Russian. Add when there is a real use case. |

---

## Block 05 — Testing

| Item | Detail |
|------|--------|
| Layout preview | Preview of `_layout.ori` files in isolation (with placeholder slot content). Low value without real page content — deferred until there is a concrete use case. |

---

## Packages — shared UI library

| Item | Detail |
|------|--------|
| `packages/ui` structure and conventions | The workspace supports a `packages/ui/` directory for shared components (auto-imported via `origami.toml`). Open questions: how opinionated should the framework be about its structure? Should `packages/ui` components be plain `.ori` files (same compiler pipeline) or allow escape hatches? How are package components versioned relative to the app? How does the preview app handle package components vs. app components? Address once a stable single-app baseline exists. |

---

## Future modules (spec section 16)

| Item | Detail |
|------|--------|
| `origami-middleware` | Route middleware via `_middleware.ts` files. Fixed function signature validated by the compiler; opaque TypeScript implementation. Prerequisite for `origami-auth`. |
| SSR target | Nuxt SSR mode as a first-class documented deployment. Architecturally available from M2 (Nuxt handles it); M5 is about making it tested, documented, and production-supported. `origamiFetch` is the only abstraction layer that needs verifying for isomorphic behaviour. |
| WASM target | Full WASM target: no JS runtime dependency at any layer. Same `.ori` source, completely different codegen path. North star — not a near-term milestone. When reached, `origami-query` is also replaced by a WASM-native data layer. |
