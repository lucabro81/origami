# Origami — Quick Reference

Rust compiler for `.ori` DSL files → Nuxt/Vue 3 SFCs. One CLI binary. Closed vocabulary. Design system compliance enforced at compile time.

---

## Workspace structure

```
my-app/
├── origami.toml
├── tokens.json          ← design system tokens (source of truth)
├── endpoints.toml       ← API bindings: page → endpoint → type
├── locales/
│   ├── en.json          ← default locale (flat keys, max 3 segments)
│   └── it.json
├── apps/
│   └── web/
│       ├── pages/       ← .ori pages and layouts
│       └── components/  ← .ori components
└── packages/
    └── ui/components/
```

`__generated__/` is the compiler output directory. Git-ignored. Never edit by hand.

---

## origami.toml

```toml
[project]
name = "my-app"
version = "0.1.0"

[compiler]
target = "vue"
openapi = "./api/openapi.json"   # path or HTTP URL
default_locale = "en"
modules = ["data", "i18n", "a11y", "test"]

[env.dev]
api_base_url = "http://localhost:8080"
port = 3000

[env.prod]
api_base_url = "https://api.example.com"

[env.test]
api_base_url = "http://localhost:9090"

[imports]
auto = ["packages/ui"]
```

---

## .ori syntax

### Component

```
component BookCard(book: { title: string, author: string }) {
const label = book.title
----
<Box padding="md">
  <Text value={label} size="lg" weight="bold" />
  <Text value={book.author} color="secondary" />
</Box>
}
```

### Page

```
page BookList(resp: Response<BookListResponse>) {
const books = resp.data?.books ?? []
----
<Column gap="md">
  <if condition={resp.isLoading}>
    <Text value={t('common.loading')} color="secondary" />
  <else>
    <each collection={books} as="book">
      <BookCard book={book} />
    </each>
  </else>
  </if>
</Column>
}
```

### Layout

```
layout RootLayout {
  <Column gap="none">
    <AppHeader />
    <Box padding="lg">
      <slot />
    </Box>
  </Column>
}
```

Rules: `page` only in `pages/`, `layout` only in `_layout.ori`, exactly one `<slot />` per layout.

---

## Key generated types

```typescript
// Response<T> — eager data, declared in page signature
interface Response<T> { data: T | null; isLoading: boolean; error: QueryError | null }

// Params<P> — typed route params in page signature
type Params<P extends Record<string, string>> = P

// QueryHandle<T> — deferred query, always available in logic section
interface QueryHandle<T> {
  data: T | null; isLoading: boolean; error: QueryError | null
  fetch: (params?: Record<string, string | number>) => Promise<void>
}

// MutationHandle<Body, Response>
interface MutationHandle<B, R> {
  mutate: (body?: B) => Promise<R>; isLoading: boolean
  error: MutationError | null; reset: () => void
}
```

Eager vs deferred is determined by the **page signature**, not by `endpoints.toml`:
- `Response<T>` in signature + endpoint in `[queries]` → **eager** (runs on mount)
- No `Response<T>` + `useQueryXxx` in logic section → **deferred** (runs on `.fetch()`)

---

## endpoints.toml

```toml
[queries]
"books/index"        = { endpoint = "/api/books",            type = "BookListResponse" }
"books/[id]"         = { endpoint = "/api/books/:id",        type = "BookResponse" }

[mutations]
"books/[id]" = [
  { method = "PUT",    endpoint = "/api/books/:id", body = "BookUpdateRequest", response = "BookResponse" },
  { method = "DELETE", endpoint = "/api/books/:id", body = null, response = null }
]
```

Key = route pattern (relative to `pages/`). Must match a page file path. All `type`, `body`, `response` values must exist in the OpenAPI spec.

### Handle naming

| Source type | Generated handle |
|-------------|-----------------|
| `BookListResponse` (query) | `useQueryBookList` |
| `BookResponse` (query) | `useQueryBook` |
| `PUT` → `BookResponse` (mutation) | `useMutationPutBook` |
| `DELETE` → null (mutation) | `useMutationDeleteBook` |
| `as = "saveBook"` | `saveBook` (W201) |

---

## File-to-route mapping

| File path | Route |
|-----------|-------|
| `pages/index.ori` | `/` |
| `pages/books/index.ori` | `/books` |
| `pages/books/[id].ori` | `/books/:id` |
| `pages/books/[id]/reviews.ori` | `/books/:id/reviews` |
| `pages/404.ori` | not-found fallback |
| `pages/_layout.ori` | (not a route — layout only) |

---

## Generated output structure

```
__generated__/
  ├── pages/            ← Nuxt file-based pages (.vue)
  ├── layouts/          ← Nuxt layouts from _layout.ori
  ├── components/       ← shared components
  ├── nuxt.config.ts    ← generated (proxy, i18n, imports)
  ├── fetch.ts          ← origamiFetch abstraction
  └── types/
      ├── api.ts        ← TypeScript types from OpenAPI
      └── response.ts   ← Response<T>, handles per page (inspect this to know what's available)
```

---

## Crate pipeline

```
origami-cli (orchestrator)
  │
  ├─ origami-lexer     String → Vec<Token>
  ├─ origami-parser    Vec<Token> → AST
  ├─ origami-analyzer  AST validation → AnalyzedFile[]
  ├─ origami-router    AnalyzedFile[] + filesystem → RouteTable
  ├─ origami-data      RouteTable + OpenAPI + endpoints.toml → DataManifest
  ├─ origami-i18n      AnalyzedFile[] + locales/ → LocaleManifest
  ├─ origami-a11y      AnalyzedFile[] + tokens.json → a11y errors
  ├─ origami-codegen   all manifests → __generated__/
  └─ origami-dev       file watcher + Nuxt child process (tokio)
```

All error types in `origami-runtime`. All public API must match the relevant block document.

---

## Error codes

### Routing (CLT2xx)
| Code | Condition |
|------|-----------|
| CLT201 | Two files produce the same URL pattern |
| CLT202 | Dynamic segment in `endpoints.toml` does not match route params |
| CLT203 | `_layout.ori` has more than one `<slot />` |
| CLT204 | `_layout.ori` has no `<slot />` |
| CLT205 | `page` keyword outside `pages/` |
| CLT206 | `layout` keyword outside `_layout.ori` |

### Data layer (CLT3xx)
| Code | Condition |
|------|-----------|
| CLT301 | Type name in `endpoints.toml` not in OpenAPI schema |
| CLT302 | Endpoint path not in OpenAPI paths |
| CLT303 | HTTP method not declared in OpenAPI for that path |
| CLT304 | Body type not in OpenAPI schema |
| CLT305 | OpenAPI spec could not be fetched or parsed |
| CLT306 | `Response<T>` type does not match `endpoints.toml` |
| CLT307 | `Params<P>` keys do not match file path dynamic segments |
| CLT308 | `Response<T>` parameter name shadows a built-in |

### i18n (CLT4xx)
| Code | Condition |
|------|-----------|
| CLT401 | `t()` key not in default locale |
| CLT402 | Interpolation variable missing from `t()` call |
| CLT403 | Locale key exceeds 3 segments |

### Mutations (CLT5xx)
| Code | Condition |
|------|-----------|
| CLT501 | Two mutation handles produce the same auto-generated name |
| CLT502 | Mutation handle used in page logic with no `endpoints.toml` entry |

### Accessibility (CLT6xx)
| Code | Condition |
|------|-----------|
| CLT601 | `Button` missing accessible label |
| CLT602 | `Input` missing associated `Label` |
| CLT603 | `Image` missing `alt` prop |
| CLT604 | Color token pair fails WCAG 4.5:1 contrast ratio |

### Testing (CLT7xx)
| Code | Condition |
|------|-----------|
| CLT701 | Duplicate test name in component or page |
| CLT703 | Test fixture does not match `Response<T>` shape |
| CLT704 | `navigate` step in a `test` block (only valid in `e2e`) |
| CLT705 | `api_mock` in a `test` block (only valid in `e2e`) |
| CLT706 | `testId` not unique in static template |
| CLT707 | `e2e` block on a component (only valid on pages) |
| CLT708 | Test fixture param name does not match page signature param name |

### Warnings
| Code | Condition |
|------|-----------|
| W101 | Locale key missing from a non-default locale file |
| W102 | Extra key in non-default locale (no counterpart in default) |
| W103 | Extra interpolation variable passed to `t()` |
| W201 | Handle name overridden via `as` |

---

## CLI commands

| Command | Purpose |
|---------|---------|
| `origami dev [--app] [--env] [--port] [--host]` | Start dev server with HMR |
| `origami build [--app] [--env] [--locale] [--out]` | Production build, exits non-zero on error |
| `origami check [--app]` | Validate without emitting output — CI gate |
| `origami test [--preview\|--e2e\|--a11y\|--snapshot]` | Run test suite |
| `origami init <name> [--app] [--no-example]` | Scaffold new workspace |
| `origami unsafe-report [--format json]` | List all escape hatches |
