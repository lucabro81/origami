# Block 02 — Data Layer

**Version:** 0.1 — draft

## Contents

- [Scope](#scope)
- [Expected output](#expected-output)
- [Design principles](#design-principles)
- [endpoints.toml](#endpointstoml)
- [OpenAPI integration](#openapi-integration)
- [Compiler pipeline extensions](#compiler-pipeline-extensions)
  - [Lexer — new tokens](#lexer--new-tokens)
  - [Parser — new AST nodes](#parser--new-ast-nodes)
  - [Analyzer — new validation rules](#analyzer--new-validation-rules)
- [origami-data](#origami-data)
  - [Input](#input)
  - [Output: generated types](#output-generated-types)
  - [Query handle naming convention](#query-handle-naming-convention)
  - [Mutation handle naming convention](#mutation-handle-naming-convention)
  - [The origamiFetch abstraction](#the-origamifetch-abstraction)
- [origami-codegen extension](#origami-codegen-extension)
- [Error codes](#error-codes)
- [CLI contribution](#cli-contribution)
- [Tests](#tests)

---

## Scope

This block implements the data layer: the compile-time contract between the frontend and the backend API. It includes the `origami-data` crate (OpenAPI parsing, `endpoints.toml` validation, typed handle generation), compiler pipeline extensions for `Response<T>` and `Params<P>` in page signatures, and the `origamiFetch` abstraction that decouples HTTP from the runtime target.

Part of M1 (Compilable SPA). Together with Block 01 (Router), it completes the first end-to-end compilable application.

The data layer is **SPA-first**. Fetching is client-side, handled by **TanStack Vue Query** (`@tanstack/vue-query`). This is an explicit choice over Nuxt's `useAsyncData`: Vue Query is decoupled from Nuxt, portable to any future runtime target, and aligns with the primary SPA use case. When SSR becomes a first-class target (M5), `origamiFetch` is the single abstraction layer that changes — the rest of the generated data code is unaffected.

---

## Expected output

- `Response<T>`, `Params<P>`, `useQueryXxx`, and `useMutationXxx` syntax recognised and validated by the compiler
- `origami-data`: parses `endpoints.toml`, validates against OpenAPI spec, produces a typed `DataManifest`
- `origami-codegen` extended: from the `DataManifest`, emits `__generated__/types/` and Vue Query composables
- `origamiFetch` abstraction emitted into `__generated__/fetch.ts`
- Error codes CLT301–CLT308, CLT501–CLT502, W201 implemented with `miette` messages
- Unit tests for naming conventions, conflict detection, and OpenAPI validation
- `.ori` fixtures with pages using queries and mutations

---

## Design principles

**Type safety at the boundary.** The frontend–backend interface is a compile-time contract. A backend type rename that breaks `endpoints.toml` is a build error, not a runtime surprise. The compiler reports every affected binding in a single block — not just the first one.

**Explicit over implicit.** Every handle available on a page (`useQueryXxx`, `useMutationXxx`) is declared in `endpoints.toml` and visible in `__generated__/types/response.ts`. Nothing is magic. Developers and LLMs can inspect the generated file to know exactly what is available on any given page.

**SPA-first, SSR-compatible.** Vue Query runs on the client. The `origamiFetch` wrapper is the only layer that would change for SSR. The rest of the generated data code — types, handles, composables — is identical across targets.

**Stable interface, swappable implementation.** The compiler generates code against a stable contract: `QueryHandle<T>` and `MutationHandle<T,R>`. Vue Query is the first implementation of that contract — not the permanent one. Before reaching the WASM target, a custom `origami-query` crate will replace Vue Query, implementing only the features Origami actually uses: keyed caching, eager/deferred toggle, and the mutation pattern. When that happens, only the codegen templates change — `.ori` source files are unaffected. The scope of `origami-query` is deliberately narrow: there is no value in reimplementing the full complexity of a general-purpose query library when the vocabulary is closed and the use cases are known in advance.

---

## endpoints.toml

`endpoints.toml` lives at the workspace root. It maps pages to API endpoints and declares the TypeScript types for request and response bodies. All types must exist in the OpenAPI spec.

```toml
# GET requests — each entry generates both a Response<T> type (eager, if declared
# in the page signature) and a useQueryXxx handle (deferred, always available).
[queries]
"books/index"        = { endpoint = "/api/books",            type = "BookListResponse" }
"books/[id]"         = { endpoint = "/api/books/:id",        type = "BookResponse" }
"books/[id]/reviews" = { endpoint = "/api/books/:id/reviews", type = "ReviewListResponse" }

# POST / PUT / PATCH / DELETE — each entry generates a useMutationMethodNoun handle.
[mutations]
"books/index" = [
  { method = "POST", endpoint = "/api/books", body = "BookCreateRequest", response = "BookResponse" }
]
"books/[id]" = [
  { method = "PUT",    endpoint = "/api/books/:id", body = "BookUpdateRequest", response = "BookResponse" },
  { method = "DELETE", endpoint = "/api/books/:id", body = null, response = null }
]
```

The key in both `[queries]` and `[mutations]` is a route pattern that must match a page file path (relative to `pages/`). This is how the compiler knows which handles are available on which page.

**The `as` override** renames a generated handle. It is a deliberate escape hatch — marked with warning W201 so deviations from the naming convention are auditable:

```toml
[queries]
"books/[id]/reviews" = { endpoint = "/api/books/:id/reviews", type = "ReviewListResponse", as = "loadReviews" }
```

`as` is not available for `[queries]` entries used as eager `Response<T>` — those are named by the developer in the page signature. `as` applies to deferred query handles and all mutation handles.

---

## OpenAPI integration

The OpenAPI spec is the source of truth for all type names. The path to the spec (local file or HTTP URL) is declared in `origami.toml`:

```toml
[compiler]
openapi = "./api/openapi.json"   # or "https://api.example.com/openapi.json"
```

`origami-data` fetches and parses the spec at the start of every compilation. The fetch is synchronous (`ureq`) — it happens once, before the pipeline begins. If the spec cannot be fetched or parsed, the compiler fails with CLT305 before processing any `.ori` files.

All `type`, `body`, and `response` values in `endpoints.toml` must match schema names defined in the OpenAPI spec's `components/schemas`. The compiler validates every binding and reports all violations in a single error block — not one at a time.

**API change handling:** when an OpenAPI schema is renamed, the compiler emits CLT301 for every affected binding and suggests likely replacements based on name similarity:

```
error[CLT301] — endpoints.toml: 3 bindings reference unknown type 'BookResponse'

  [queries] "books/[id]"       type = "BookResponse"
  [mutations] "books/[id]" PUT response = "BookResponse"

  Available types matching 'Book*':
    BookDetailResponse, BookSummaryResponse
```

---

## Compiler pipeline extensions

### Lexer — new tokens

No new tokens are required. `Response`, `Params`, and the type parameter syntax are part of the page signature, which is treated as opaque TypeScript by the lexer (same as `ComponentDef` signatures).

### Parser — new AST nodes

`PageDef` already holds the signature as an opaque string (from Block 01). The parser does not need to change — the signature is parsed by `origami-data`, not by the core parser.

The `origami-data` crate performs a targeted parse of the page signature string to extract `Response<T>` and `Params<P>` type parameters. This is intentionally isolated: changes to the signature parsing do not affect the core parser or AST.

### Analyzer — new validation rules

The analyzer delegates data-layer validation to `origami-data`. New error codes surfaced here:

- CLT306: `Response<T>` type parameter does not match the `endpoints.toml` declaration for this page
- CLT307: `Params<P>` keys do not match the dynamic segments in the file path
- CLT308: `Response<T>` parameter name shadows a built-in identifier
- W201: handle name overridden via `as`

---

## origami-data

New crate. Owns all data-layer logic: reading `endpoints.toml`, fetching and parsing the OpenAPI spec, validating bindings, and producing the `DataManifest` consumed by `origami-codegen`.

### Input

```
origami.toml
  └── compiler.openapi: String     ← path or URL to OpenAPI spec

endpoints.toml
  ├── queries: Map<RouteKey, QueryEntry>
  └── mutations: Map<RouteKey, Vec<MutationEntry>>

RouteTable                         ← from origami-router (Block 01)
  └── routes: Vec<Route>           ← used to validate route keys and params
```

### Output: generated types

`origami-data` produces a `DataManifest` that `origami-codegen` uses to emit three files:

```
__generated__/
  ├── fetch.ts          ← origamiFetch abstraction
  └── types/
      ├── api.ts        ← TypeScript types derived from OpenAPI schemas
      └── response.ts   ← Response<T>, Params<P>, QueryHandle<T>, MutationHandle<T,R>
                          + one entry per page listing its available handles
```

`response.ts` is the "manifest" file for developers and LLMs: it shows exactly which handles are available on each page without having to inspect the source.

**Core generated types:**

```typescript
// Response<T> — eager data binding in page signature
interface Response<T> {
  data: T | null
  isLoading: boolean
  error: QueryError | null
}

// Params<P> — typed route params in page signature
type Params<P extends Record<string, string>> = P

// QueryHandle<T> — deferred query, always available in logic section
interface QueryHandle<T> {
  data: T | null
  isLoading: boolean
  error: QueryError | null
  fetch: (params?: Record<string, string | number>) => Promise<void>
}

// MutationHandle<Body, Response> — mutation handle
interface MutationHandle<BodyType, ResponseType> {
  mutate: (body?: BodyType) => Promise<ResponseType>
  isLoading: boolean
  error: MutationError | null
  reset: () => void
}
```

### Query handle naming convention

Every entry in `[queries]` generates a `useQueryXxx` handle named `useQuery` + PascalCase noun from the response type, stripping the `Response` suffix:

| Response type | Generated handle |
|--------------|-----------------|
| `BookListResponse` | `useQueryBookList` |
| `BookResponse` | `useQueryBook` |
| `ReviewListResponse` | `useQueryReviewList` |
| `ReviewListResponse` with `as = "loadReviews"` | `loadReviews` (W201) |

Query handles are always available in the page logic section. Whether they run eagerly or on demand is determined solely by the page signature — not by `endpoints.toml`:

- `Response<T>` in signature + endpoint in `[queries]` → **eager**: runs on page mount
- No `Response<T>` in signature + `useQueryXxx` used in logic section → **deferred**: runs when `.fetch()` is called
- Both → eager on mount, manual refresh also available

### Mutation handle naming convention

Mutation handles are named `useMutation` + PascalCase(method) + noun from the response type (stripping `Response`). If `response` is null, the body type is used. If both are null, the name derives from the last path segment:

| Method | Response type | Generated handle |
|--------|--------------|-----------------|
| `POST` | `BookResponse` | `useMutationPostBook` |
| `PUT` | `BookResponse` | `useMutationPutBook` |
| `DELETE` | null | `useMutationDeleteBook` |
| `PUT` with `as = "saveBook"` | — | `saveBook` (W201) |

If two handles on the same page produce the same auto-generated name, the compiler fails with CLT501 listing all conflicts. Resolution: use `as` on one of them.

Mutations are always deferred — they do nothing until `.mutate()` is called.

### The origamiFetch abstraction

All generated fetching code calls `origamiFetch` instead of `fetch()` directly. This indirection is the single seam that makes the data layer portable across runtime targets:

```typescript
// __generated__/fetch.ts
// SPA implementation — replaced wholesale for SSR or WASM targets
export const origamiFetch = <T>(url: string): Promise<T> =>
  fetch(`${import.meta.env.VITE_API_BASE_URL}${url}`).then(r => r.json())
```

`api_base_url` from the active `[env.*]` section in `origami.toml` is inlined as a Vite environment variable at build time. This keeps the fetcher simple and avoids runtime config lookups.

Vue Query wraps `origamiFetch` in the generated composable:

```typescript
// generated inside a page's <script setup>
const { data, isLoading, error } = useQuery({
  queryKey: ['books'],
  queryFn: (): Promise<BookListResponse> => origamiFetch('/api/books')
})
```

The `queryKey` includes any runtime params passed to `.fetch()` automatically, making pagination and filtering cache-correct without special configuration.

---

## origami-codegen extension

`origami-codegen` receives the `DataManifest` and emits the generated type files and Vue Query composables. The composables are injected into the `<script setup>` block of each page's generated `.vue` file.

**Composable injection schema for a page with one eager query and one mutation:**

```
<script setup>
import { useQuery, useMutation } from '@tanstack/vue-query'
import { origamiFetch } from '../fetch'
import type { BookResponse, BookUpdateRequest } from '../types/api'

// Eager query — runs on mount (Response<T> declared in page signature)
const { data: book, isLoading, error } = useQuery({
  queryKey: ['books', route.params.id],
  queryFn: () => origamiFetch<BookResponse>(`/api/books/${route.params.id}`)
})

// Deferred query — runs on .fetch() call
const useQueryReviewList = useQuery({
  queryKey: ['books', route.params.id, 'reviews'],
  queryFn: () => origamiFetch('/api/books/' + route.params.id + '/reviews'),
  enabled: false
})

// Mutation handle
const useMutationPutBook = useMutation({
  mutationFn: (body: BookUpdateRequest) =>
    origamiFetch<BookResponse>(`/api/books/${route.params.id}`, { method: 'PUT', body })
})
</script>
```

The developer never writes or reads this code directly — it is generated and git-ignored. The `response.ts` manifest file is the inspectable surface.

---

## Error codes

| Code | Condition | Phase |
|------|-----------|-------|
| CLT301 | Type name in `endpoints.toml` not found in OpenAPI schema | data validation |
| CLT302 | Endpoint path in `endpoints.toml` not found in OpenAPI paths | data validation |
| CLT303 | HTTP method not declared in OpenAPI for that path | data validation |
| CLT304 | Body type not found in OpenAPI schema | data validation |
| CLT305 | OpenAPI spec could not be fetched or parsed | startup, before pipeline |
| CLT306 | `Response<T>` type parameter does not match `endpoints.toml` | analyzer |
| CLT307 | `Params<P>` keys do not match file path dynamic segments | analyzer |
| CLT308 | `Response<T>` parameter name shadows a built-in identifier | analyzer |
| CLT501 | Two handles on the same page produce the same auto-generated name | data validation |
| CLT502 | Handle referenced in page logic with no matching `endpoints.toml` entry | analyzer |
| W201 | Handle name overridden via `as` | data validation |

All errors use `miette` with source spans. CLT301 reports all affected bindings in one block and suggests type name alternatives. The compiler does not stop at the first error — it collects all data-layer violations and reports them together.

---

## CLI contribution

With this block, `origami check` additionally validates:
- `endpoints.toml` parse and structure
- All type names against the OpenAPI spec
- `Response<T>` / `Params<P>` coherence in page signatures
- Handle naming conflicts

`origami build` is extended to invoke `origami-data` and write `__generated__/fetch.ts` and `__generated__/types/`.

---

## Tests

**Unit tests in `origami-data/src/tests.rs`:**
- Query handle naming: all convention cases including `as` override
- Mutation handle naming: all method/type combinations
- Conflict detection: two handles producing the same name on the same page
- OpenAPI validation: unknown type, unknown path, wrong method
- `Params<P>` key matching against route dynamic segments

**Integration fixtures in `fixtures/`:**

```
fixtures/
  └── data/
      ├── basic/          ← one page with one eager query
      ├── deferred/       ← page with deferred query (no Response<T> in signature)
      ├── mutations/      ← page with POST + PUT + DELETE
      ├── params/         ← page with Params<P> and dynamic route
      ├── as-override/    ← handle renamed via `as` (W201)
      └── api-change/     ← endpoints.toml references a removed OpenAPI type (CLT301)
```

Integration tests compile each fixture and verify the contents of `__generated__/types/response.ts` and the injected composable blocks in the generated `.vue` files.
