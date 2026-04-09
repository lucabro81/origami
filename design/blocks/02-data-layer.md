# Block 02 — Data Layer

Compile-time contract between frontend and API. Includes `origami-data` crate (OpenAPI parsing, `endpoints.toml` validation, typed handle generation), compiler extensions for `Response<T>`/`Params<P>`, and the `origamiFetch` abstraction.

Part of M1. Together with Block 01 (Router), completes the first end-to-end compilable application.

Data layer is SPA-first. Fetching is client-side via TanStack Vue Query (`@tanstack/vue-query`).

---

## Checklist — done when all of these are true

- [ ] `Response<T>`, `Params<P>`, `useQueryXxx`, `useMutationXxx` recognised and validated by compiler
- [ ] `origami-data`: parses `endpoints.toml`, validates all types against OpenAPI, produces `DataManifest`
- [ ] `origami-codegen` extended: emits `__generated__/types/` and composables injected into page `.vue` files
- [ ] `origamiFetch` emitted into `__generated__/fetch.ts`
- [ ] Error codes CLT301–CLT308, CLT501–CLT502, W201 implemented with `miette` messages
- [ ] Unit tests pass: naming conventions, conflict detection, OpenAPI validation, `Params<P>` matching
- [ ] Integration fixtures exist: basic, deferred, mutations, params, as-override, api-change
- [ ] `cargo clippy -- -D warnings` clean, `rustfmt` applied

---

## Error codes

| Code | Condition | Phase |
|------|-----------|-------|
| CLT301 | Type name in `endpoints.toml` not found in OpenAPI schema | data validation |
| CLT302 | Endpoint path not found in OpenAPI paths | data validation |
| CLT303 | HTTP method not declared in OpenAPI for that path | data validation |
| CLT304 | Body type not found in OpenAPI schema | data validation |
| CLT305 | OpenAPI spec could not be fetched or parsed | startup, before pipeline |
| CLT306 | `Response<T>` type parameter does not match `endpoints.toml` | analyzer |
| CLT307 | `Params<P>` keys do not match file path dynamic segments | analyzer |
| CLT308 | `Response<T>` parameter name shadows a built-in identifier | analyzer |
| CLT501 | Two handles on the same page produce the same auto-generated name | data validation |
| CLT502 | Handle used in page logic with no matching `endpoints.toml` entry | analyzer |
| W201 | Handle name overridden via `as` | data validation |

CLT301 must report **all** affected bindings in one block and suggest type name alternatives.
The compiler does not stop at the first error — collect all data-layer violations, report together.

---

## endpoints.toml format

```toml
[queries]
"books/index"        = { endpoint = "/api/books",             type = "BookListResponse" }
"books/[id]"         = { endpoint = "/api/books/:id",         type = "BookResponse" }
"books/[id]/reviews" = { endpoint = "/api/books/:id/reviews", type = "ReviewListResponse" }

[mutations]
"books/index" = [
  { method = "POST", endpoint = "/api/books", body = "BookCreateRequest", response = "BookResponse" }
]
"books/[id]" = [
  { method = "PUT",    endpoint = "/api/books/:id", body = "BookUpdateRequest", response = "BookResponse" },
  { method = "DELETE", endpoint = "/api/books/:id", body = null, response = null }
]
```

Key = route pattern relative to `pages/`. Must match a page file path. All `type`, `body`, `response` must exist in OpenAPI `components/schemas`.

`as` override: renames a generated handle. Emits W201. Valid for deferred queries and mutations. Not valid for `[queries]` eager entries (those are named by the developer in the page signature).

```toml
[queries]
"books/[id]/reviews" = { endpoint = "...", type = "ReviewListResponse", as = "loadReviews" }
```

---

## Compiler pipeline extensions

### Lexer
No new tokens required. `Response`, `Params`, and type parameter syntax are part of the page signature, treated as opaque TypeScript.

### Parser
`PageDef.signature` is already an opaque string from Block 01. No parser changes needed.

### Analyzer
Delegates data-layer validation to `origami-data`. New validations:
- CLT306: `Response<T>` type does not match `endpoints.toml` for this page
- CLT307: `Params<P>` keys do not match dynamic segments in the file path
- CLT308: `Response<T>` parameter name shadows a built-in
- W201: handle name overridden via `as`

`origami-data` performs a targeted parse of the `PageDef.signature` string to extract `Response<T>` and `Params<P>`. This is isolated — changes do not affect the core parser or AST.

---

## origami-data crate

New crate. Input: `origami.toml` + `endpoints.toml` + OpenAPI spec + `RouteTable`. Output: `DataManifest`.

### Input types

```rust
// from origami.toml
compiler.openapi: String    // path or URL to OpenAPI spec

// endpoints.toml
queries:   Map<RouteKey, QueryEntry>
mutations: Map<RouteKey, Vec<MutationEntry>>

// from Block 01
RouteTable { routes: Vec<Route> }
```

OpenAPI spec is fetched synchronously with `ureq` at the start of every compilation — once, before the pipeline begins. Failure → CLT305, compilation aborts.

### Output: DataManifest → generated files

```
__generated__/
  ├── fetch.ts           ← origamiFetch abstraction
  └── types/
      ├── api.ts         ← TypeScript types from OpenAPI schemas
      └── response.ts    ← Response<T>, Params<P>, QueryHandle<T>, MutationHandle<T,R>
                            + per-page listing of available handles (inspect this to know what's available)
```

### Generated core types

```typescript
interface Response<T> { data: T | null; isLoading: boolean; error: QueryError | null }
type Params<P extends Record<string, string>> = P
interface QueryHandle<T> {
  data: T | null; isLoading: boolean; error: QueryError | null
  fetch: (params?: Record<string, string | number>) => Promise<void>
}
interface MutationHandle<B, R> {
  mutate: (body?: B) => Promise<R>; isLoading: boolean
  error: MutationError | null; reset: () => void
}
```

### Query handle naming convention

`useQuery` + PascalCase noun from response type, stripping `Response` suffix:

| Response type | Generated handle |
|--------------|-----------------|
| `BookListResponse` | `useQueryBookList` |
| `BookResponse` | `useQueryBook` |
| `ReviewListResponse` | `useQueryReviewList` |
| `ReviewListResponse` + `as = "loadReviews"` | `loadReviews` (W201) |

### Mutation handle naming convention

`useMutation` + PascalCase(method) + noun from response type (stripping `Response`). If `response` is null, use body type. If both null, use last path segment:

| Method | Response type | Generated handle |
|--------|--------------|-----------------|
| `POST` | `BookResponse` | `useMutationPostBook` |
| `PUT` | `BookResponse` | `useMutationPutBook` |
| `DELETE` | null | `useMutationDeleteBook` |
| `PUT` + `as = "saveBook"` | — | `saveBook` (W201) |

Two handles on the same page producing the same name → CLT501. Resolution: use `as`.

Mutations are always deferred — do nothing until `.mutate()` is called.

### Eager vs deferred (determined by page signature)

- `Response<T>` in signature + entry in `[queries]` → **eager**: runs on page mount
- No `Response<T>`, `useQueryXxx` used in logic section → **deferred**: runs on `.fetch()`
- Both → eager on mount, manual refresh also available

### origamiFetch abstraction

```typescript
// __generated__/fetch.ts — SPA implementation
export const origamiFetch = <T>(url: string): Promise<T> =>
  fetch(`${import.meta.env.VITE_API_BASE_URL}${url}`).then(r => r.json())
```

All generated fetching code calls `origamiFetch`, not `fetch()` directly. This is the single seam for SSR/WASM portability.

---

## origami-codegen extension

Receives `DataManifest`, injects composables into each page's `<script setup>`:

```typescript
// generated for a page with one eager query + one mutation
<script setup>
import { useQuery, useMutation } from '@tanstack/vue-query'
import { origamiFetch } from '../fetch'
import type { BookResponse, BookUpdateRequest } from '../types/api'

// Eager query (Response<T> in page signature)
const { data: book, isLoading, error } = useQuery({
  queryKey: ['books', route.params.id],
  queryFn: () => origamiFetch<BookResponse>(`/api/books/${route.params.id}`)
})

// Deferred query (no Response<T> in signature)
const useQueryReviewList = useQuery({
  queryKey: ['books', route.params.id, 'reviews'],
  queryFn: () => origamiFetch('/api/books/' + route.params.id + '/reviews'),
  enabled: false
})

// Mutation
const useMutationPutBook = useMutation({
  mutationFn: (body: BookUpdateRequest) =>
    origamiFetch<BookResponse>(`/api/books/${route.params.id}`, { method: 'PUT', body })
})
</script>
```

Developer never reads or writes this code — it is generated and git-ignored.

---

## CLI contribution

`origami check` additionally validates:
- `endpoints.toml` parse and structure
- All type names against OpenAPI spec
- `Response<T>` / `Params<P>` coherence in page signatures
- Handle naming conflicts

`origami build` extended to invoke `origami-data` and write `__generated__/fetch.ts` and `__generated__/types/`.

---

## Tests

### Unit tests — `origami-data/src/tests.rs`

- Query handle naming: all convention cases including `as` override
- Mutation handle naming: all method/type combinations
- Conflict detection: two handles producing the same name on the same page → CLT501
- OpenAPI validation: unknown type (CLT301), unknown path (CLT302), wrong method (CLT303)
- `Params<P>` key matching against route dynamic segments (CLT307)

### Integration fixtures — `fixtures/data/`

```
fixtures/data/
  ├── basic/       ← one page with one eager query
  ├── deferred/    ← deferred query (no Response<T> in signature)
  ├── mutations/   ← page with POST + PUT + DELETE
  ├── params/      ← page with Params<P> and dynamic route
  ├── as-override/ ← handle renamed via `as` (W201)
  └── api-change/  ← endpoints.toml references a removed OpenAPI type (CLT301)
```

Integration tests verify contents of `__generated__/types/response.ts` and composable blocks in generated `.vue` files.
