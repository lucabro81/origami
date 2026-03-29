# Origami Framework Specification

**Version**: 0.2.0-draft
**Status**: In Review
**Relation to prior spec**: Extends compiler spec `0.1.0`. All pipeline decisions (Lexer→Parser→Analyzer→Codegen, arena allocation, `tokens.json`, `unsafe` with required `reason`) remain valid. This document adds the layers above: workspace, routing, data binding, mutations, testing, i18n, and accessibility.

---

## Table of Contents

1. [Guiding Principles](#1-guiding-principles)
2. [Workspace Structure](#2-workspace-structure)
3. [Configuration — `origami.toml`](#3-configuration--cluttertoml)
4. [Language Extensions](#4-language-extensions)
5. [Routing](#5-routing)
6. [Data Layer](#6-data-layer)
7. [Mutations](#7-mutations)
8. [Testing](#8-testing)
9. [Internationalization](#9-internationalization)
10. [Accessibility](#10-accessibility)
11. [CLI Reference](#11-cli-reference)
12. [Dev Server](#12-dev-server)
13. [Code Generation](#13-code-generation)
14. [Module Architecture](#14-module-architecture)
15. [Error Codes](#15-error-codes)
16. [Future Modules](#16-future-modules)
17. [Out of Scope](#17-out-of-scope)

---

## 1. Guiding Principles

These are constraints, not guidelines.

**One interface.** A single CLI binary is the only entry point for all development operations. No Vite config, no Jest config, no separate processes to orchestrate.

**Convention over configuration.** The filesystem structure *is* the configuration. `origami.toml` exists only for things that genuinely cannot be inferred from structure.

**Compiler as control plane.** All critical logic runs in Rust. Nothing load-bearing is delegated to JavaScript tooling.

**Closed vocabulary.** The design system is the type system. A component that violates it cannot compile.

**Type safety at the boundary.** The frontend–backend interface is a compile-time contract, not a runtime assumption.

**Explicit over implicit.** All values available in a page or component are declared in its signature. No magic variables, no ambient globals. What you see is what you have — for humans and LLMs alike.

**LLM-first authoring.** Deterministic format, closed vocabulary, no ambiguity. The compiler's type errors are the feedback loop.

**Transparent runtime.** The developer writes `.ori`. The runtime target is an implementation detail. Changing it requires zero application-code changes.

**Modular progression.** Subsystems (data layer, routing, i18n, a11y, testing, middleware) are independent crates with defined interfaces. Each is developed, tested, and stabilized independently.

---

## 2. Workspace Structure

```
my-app/
├── origami.toml              # project configuration (minimal)
├── tokens.json               # design system tokens (single source of truth)
├── endpoints.toml            # data binding: page → endpoint → type
├── locales/
│   ├── en.json               # base locale (required if i18n module enabled)
│   └── it.json
├── apps/
│   ├── web/
│   │   ├── pages/
│   │   │   ├── _layout.ori        # root layout (optional)
│   │   │   ├── index.ori
│   │   │   ├── books/
│   │   │   │   ├── _layout.ori    # nested layout (optional)
│   │   │   │   ├── index.ori
│   │   │   │   └── [id].ori
│   │   │   └── 404.ori
│   │   └── components/
│   └── admin/
│       ├── pages/
│       └── components/
└── packages/
    └── ui/
        └── components/
```

Files starting with `_` are framework-reserved. `locales/` lives at workspace root, shared across all apps.

---

## 3. Configuration — `origami.toml`

```toml
[project]
name = "my-app"
version = "0.1.0"

[compiler]
target = "vue"                  # currently the only supported target
openapi = "./api/openapi.json"  # path or HTTP URL to OpenAPI spec
default_locale = "en"

# Enabled subsystems. All are opt-in.
modules = ["data", "i18n", "a11y", "test"]

# ── Environment sections ───────────────────────────────────────────
# Each [env.*] block defines variables for a named environment.
# Activate with: origami dev --env staging, origami build --env prod

[env.dev]
api_base_url = "http://localhost:8080"
port = 3000

[env.staging]
api_base_url = "https://api.staging.example.com"
port = 3000

[env.prod]
api_base_url = "https://api.example.com"

[env.test]
api_base_url = "http://localhost:9090"  # test fixture server or MSW

# ── Auto-imports ───────────────────────────────────────────────────
# Components from listed packages are available everywhere without import statements.
[imports]
auto = ["packages/ui"]
```

**Environment variable rules:**

- `api_base_url` is the only field with compiler-level meaning. It is injected into all generated fetching code.
- `port` applies only in dev mode.
- Additional keys in any `[env.*]` section are accessible in logic sections via a reserved `env` object (read-only, inlined at compile time for non-secret values).
- `[env.test]` is used automatically by `origami test --e2e` unless overridden with `--env`.

**Secrets and `.env`:**

`origami.toml` is committed to version control — it must not contain secrets. For secrets (API keys, tokens), a `.env` file is gitignored and loaded automatically in dev mode:

```
# .env — gitignored, never committed
OPENAI_API_KEY=sk-...
STRIPE_SECRET=sk_test_...
```

Any OS environment variable takes precedence over `origami.toml` values of the same name. In production, secrets are set by the deployment environment — no file, no special tooling. The `.env` file is only for values that cannot be committed; all structured config (URLs, ports, feature flags) lives in `origami.toml`.

**What is NOT in `origami.toml`:** routing, component registration, CSS configuration, test configuration, i18n keys.

---

## 4. Language Extensions

This section extends `docs/language.md`.

### 4.1 The `page` Keyword

A page is a component bound to a route and one or more data sources. It uses the `page` keyword.

The page signature may declare typed parameters for data (`Response<T>`) and route params (`Params<P>`). Both are optional. **The parameter names are chosen by the developer** — only the types are validated by the compiler.

```
page BookList(resp: Response<BookListResponse>) {
const books = resp.data?.books ?? []
----
<if condition={resp.isLoading}>
  <Text value={t('common.loading')} color="secondary" />
<else>
  <each collection={books} as="book">
    <BookCard book={book} />
  </each>
</else>
</if>
}
```

```
page BookDetail(book: Response<BookResponse>, params: Params<{ id: string }>) {
const bookId = params.id
----
<Text value={book.data.title} size="xl" weight="bold" />
}
```

**`Response<T>`** is a compiler-generated generic type:

```typescript
// generated: __generated__/types/response.ts
interface Response<T> {
  data: T | null
  isLoading: boolean
  error: QueryError | null
}
```

**`Params<P>`** is a typed accessor for route dynamic segments:

```typescript
interface Params<P extends Record<string, string>> {
  [K in keyof P]: string
}
```

**Compiler validation:**

- The `T` in `Response<T>` must match the `type` field in `endpoints.toml` for this page. Mismatch → CLT306.
- The `P` in `Params<P>` keys must match the dynamic segments in the file path. Extra or missing keys → CLT307.
- A page with no `Response<T>` parameter but a query entry in `endpoints.toml` emits W201 (unused data binding).
- A page may declare `Params<P>` without `Response<T>` (static page with route params, no server data).

### 4.2 On-Demand Queries (`useQueryXxx`)

Every entry in `[queries]` in `endpoints.toml` generates **both** a `Response<T>` type (usable in the page signature for eager loading) **and** a `useQueryXxx` handle (always available in the logic section for deferred or manual use). There is no separate declaration for deferred queries.

The behavior is determined entirely by the page signature:

- Endpoint in `[queries]` + `Response<T>` in signature → **eager**: runs on page mount.
- Endpoint in `[queries]` + no signature param, `useQueryXxx` used in logic section → **deferred**: runs when `.fetch()` is called.
- Both → eager load on mount, manual refresh also available (e.g. pull-to-refresh).

```
page BookDetail(book: Response<BookResponse>, params: Params<{ id: string }>) {
const { data } = book

// useQueryReviewList is always available — deferred because it's not in the signature
const onLoadReviews = () => useQueryReviewList.fetch()
----
<Column gap="md">
  <Text value={data.book.title} />
  <Button variant="outline" @click={onLoadReviews} />
  <if condition={useQueryReviewList.data}>
    <each collection={useQueryReviewList.data.items} as="review">
      <ReviewCard review={review} />
    </each>
  </if>
</Column>
}
```

**`QueryHandle<T>`** type:

```typescript
interface QueryHandle<T> {
  data: T | null
  isLoading: boolean
  error: QueryError | null
  fetch: (params?: Record<string, string | number>) => Promise<void>
}
```

The optional `params` argument to `.fetch()` appends query string parameters to the endpoint URL and is included in the cache key automatically. This covers pagination, filtering, and any other parameterized GET without special `endpoints.toml` syntax:

```
const onNextPage = () => useQueryBookList.fetch({ page: currentPage + 1, limit: 20 })
```

There is no `LazyResponse<T>` and no `[deferred]` section. The distinction between eager and deferred is in the page signature, not in the data declaration.

### 4.3 Layout Files

A `_layout.ori` file wraps all pages in its directory and subdirectories.

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

Rules:
- Uses the `layout` keyword.
- Exactly one `<slot />` required (CLT203 / CLT204).
- No `Response<T>` or `Params<P>` — layouts have no data binding.
- Layouts nest from root inward: page → nearest `_layout` → parent `_layout` → root `_layout`.
- A logic section is allowed for UI-level state (open/closed nav, etc.).

### 4.4 Test Blocks

Test blocks are inline fixture declarations. Their syntax is the same regardless of whether they appear on a `component` or a `page`. See section 8 for the full test system.

---

## 5. Routing

### 5.1 File-to-Route Mapping

| File path | Route |
|-----------|-------|
| `pages/index.ori` | `/` |
| `pages/books/index.ori` | `/books` |
| `pages/books/[id].ori` | `/books/:id` |
| `pages/books/[id]/reviews.ori` | `/books/:id/reviews` |
| `pages/404.ori` | not-found fallback |
| `pages/_layout.ori` | (not a route) |

### 5.2 Route Conflicts

If two files produce the same URL pattern, the compiler fails with CLT201, listing both files. No implicit precedence rules — ambiguity is always an error.

### 5.3 Nested Layouts

Resolved at compile time. The generated router uses nested route definitions.

```
pages/
├── _layout.ori         → wraps: index, books/*, books/[id]/*
└── books/
    ├── _layout.ori     → wraps: books/*, books/[id]/*
    └── [id].ori        → wrapped by BooksLayout, then RootLayout
```

### 5.4 Route Middleware

Route middleware (guards, authentication, permission checks) is defined in `_middleware.ts` files. This feature is part of the `origami-middleware` module, which is specified in section 16 (Future Modules) and not implemented in the initial release.

---

## 6. Data Layer

### 6.1 The Problem

Types between frontend and backend desynchronize silently. A backend field rename does not break the build — it breaks users at runtime. Origami makes this a compile-time failure.

### 6.2 `endpoints.toml` Format

```toml
# GET requests. Each entry generates both a Response<T> type (eager, if declared in page
# signature) and a useQueryXxx handle (deferred, always available in logic section).
[queries]
"books/index"        = { endpoint = "/api/books",                 type = "BookListResponse" }
"books/[id]"         = { endpoint = "/api/books/:id",             type = "BookResponse" }
"books/[id]/reviews" = { endpoint = "/api/books/:id/reviews",     type = "ReviewListResponse" }

# POST / PUT / PATCH / DELETE. Generate useMutationMethodNoun handles.
[mutations]
"books/index"   = [
  { method = "POST",   endpoint = "/api/books",     body = "BookCreateRequest", response = "BookResponse" },
]
"books/[id]"    = [
  { method = "PUT",    endpoint = "/api/books/:id", body = "BookUpdateRequest", response = "BookResponse" },
  { method = "DELETE", endpoint = "/api/books/:id", body = null,               response = null },
]
```

**`[queries]`** — `type` must match an OpenAPI schema name (CLT301). Each entry generates both a `Response<T>` type (for eager use in the page signature) and a `useQueryXxx` handle (for deferred use in the logic section). Whether a query runs eagerly or on demand is determined by the page signature — not by `endpoints.toml`.

**`[mutations]`** — `method` must be `POST`, `PUT`, `PATCH`, or `DELETE`. `body` and `response` are OpenAPI schema names (or `null`). Both validated at compile time.

### 6.3 API Change Handling

When an OpenAPI schema is renamed or removed, all references to it in `endpoints.toml` become invalid. The compiler emits CLT301 for every affected binding — not just the first — and suggests likely replacements based on name similarity.

```
error[CLT301] — endpoints.toml: 3 bindings reference unknown type 'BookResponse'

  [queries] "books/[id]"         type = "BookResponse"
  [mutations] "books/[id]" PUT   response = "BookResponse"
  [mutations] "books/[id]" DELETE response = "BookResponse"  ← null, skipped

  Available types matching 'Book*':
    BookDetailResponse, BookSummaryResponse, BookPayload
```

All affected lines are shown in a single error block. No silent failures, no hunting through the codebase.

### 6.4 Custom Binding Names (`as`)

When the auto-generated handle name is awkward or conflicts with project naming, `as` overrides it. Valid for both `[deferred]` and `[mutations]` entries. Emits W201 — same principle as `unsafe`, makes the deviation auditable.

```toml
[deferred]
"books/[id]/reviews" = { endpoint = "/api/books/:id/reviews", type = "ReviewListResponse", as = "loadReviews" }

[mutations]
"books/[id]" = [
  { method = "PUT", endpoint = "/api/books/:id", body = "BookUpdateRequest", response = "BookResponse", as = "saveBook" },
]
```

Generated handles: `loadReviews` (instead of `useQueryReviewList`) and `saveBook` (instead of `useMutationPutBook`).

`as` is not available for `[queries]`. Those bind to user-named `Response<T>` parameters in the page signature — the name is already under developer control.

### 6.5 Generated Fetching Code

For a page bound to `GET /api/books` returning `BookListResponse`:

```typescript
// generated inside the page's <script setup>
const { data, isLoading, error } = useQuery({
  queryKey: ['books'],
  queryFn: (): Promise<BookListResponse> =>
    fetch(`${env.api_base_url}/api/books`).then(r => r.json()),
})
```

`env.api_base_url` is the value from the active `[env.*]` section, inlined at compile time.

The fetcher is injected via a generated abstraction (`origamiFetch`), not hardcoded to `fetch()`. This keeps the data layer swappable for future SSR or WASM targets. See section 13.3.

---

## 7. Queries and Mutations

### 7.1 Query Handle Naming Convention

Every entry in `[queries]` generates a `useQueryXxx` handle named: `useQuery` + PascalCase noun from the response type, stripping the `Response` suffix.

| Response type | Generated handle |
|--------------|-----------------|
| `ReviewListResponse` | `useQueryReviewList` |
| `BookResponse` | `useQueryBook` |
| `UserSummaryResponse` | `useQueryUserSummary` |
| `ReviewListResponse`, `as = "loadReviews"` | `loadReviews` (W201) |

Query handles are available implicitly in the page logic section and template. Whether they run eagerly or on demand depends on the page signature (see section 4.2), not on their declaration.

**`QueryHandle<T>`:**

```typescript
interface QueryHandle<T> {
  data: T | null
  isLoading: boolean
  error: QueryError | null
  fetch: () => Promise<void>
}
```

### 7.2 Mutation Naming Convention

Mutation handles are auto-named: `useMutation` + PascalCase(method) + noun derived from response type (stripping the `Response` suffix). If `response` is null, the body type is used. If both are null, the name derives from the last path segment.

| Method | Response type | Generated handle |
|--------|--------------|-----------------|
| `POST` | `BookResponse` | `useMutationPostBook` |
| `PUT` | `BookResponse` | `useMutationPutBook` |
| `DELETE` | null | `useMutationDeleteBook` |
| `PATCH` | `BookResponse` | `useMutationPatchBook` |
| `PUT` | `BookResponse`, `as = "saveBook"` | `saveBook` (W201) |

### 7.3 Using Queries and Mutations in Pages

All handles — deferred queries and mutations — are available implicitly in the page logic section. They are named, typed, and inspectable. The generated `__generated__/types/response.ts` file lists every handle available on each page, so the developer (or an LLM) knows what exists without checking the spec.

```
page BookDetail(book: Response<BookResponse>, params: Params<{ id: string }>) {
const { data } = book

// Available implicitly (from endpoints.toml for this page):
// useQueryReviewList: QueryHandle<ReviewListResponse>    ← [deferred]
// useMutationPutBook: MutationHandle<BookUpdateRequest, BookResponse>
// useMutationDeleteBook: MutationHandle<null, null>

const onDelete = async () => {
  await useMutationDeleteBook.mutate()
  router.push('/books')
}
const onLoadReviews = () => useQueryReviewList.fetch()
----
<Column gap="md">
  <Text value={data.book.title} size="xl" />
  <Button variant="ghost" @click={onLoadReviews} />
  <if condition={useQueryReviewList.data}>
    <each collection={useQueryReviewList.data.items} as="review">
      <ReviewCard review={review} />
    </each>
  </if>
  <Row gap="sm" mainAxis="end">
    <Button variant="outline" @click={useMutationPutBook.mutate} />
    <Button variant="danger" @click={onDelete} />
    <if condition={useMutationDeleteBook.isLoading}>
      <Text value={t('common.deleting')} color="secondary" />
    </if>
  </Row>
</Column>
}
```

**MutationHandle type:**

```typescript
interface MutationHandle<BodyType, ResponseType> {
  mutate: (body?: BodyType) => Promise<ResponseType>
  isLoading: boolean
  error: MutationError | null
  reset: () => void
}
```

Mutations are always deferred — they do nothing until `.mutate()` is called. This is the natural behavior.

### 7.4 Naming Conflicts

If two handles on the same page produce the same auto-generated name, the compiler fails with CLT501 and lists all conflicting entries. Resolution: use `as` on one of them. There is no implicit disambiguation.

---

## 8. Testing

### 8.1 Philosophy

Testing is a structural property of the code, not a separate phase. The goal is total coverage by design: every component has visual states, every page has data states, every user flow has an executable path. Coverage gaps are visible by inspection, not by running a tool.

### 8.2 Test Tiers

All tests share a single unified `steps` vocabulary. What differs is the **keyword**, **execution context**, and **what gets generated**.

| Keyword | Where | Steps present | Visual preview | Playwright |
|---------|-------|--------------|----------------|------------|
| `test` | `component` or `page` | no | ✓ static render | — |
| `test` | `component` or `page` | yes | ✓ render + step results | ✓ component test |
| `e2e` | `page` only | always | — | ✓ full-browser test |

**The rule:**
- A `test` block always appears in the visual preview app.
- A `test` block with `steps` additionally generates a Playwright component test (isolated render, no router, no real network). If you write steps, you're writing a test — the compiler executes it.
- An `e2e` block generates only a Playwright full-browser test. It does not appear in the visual preview because it requires a real router and network-level API interception — it cannot be rendered in isolation meaningfully.

The presence of `steps` is the only discriminator for `test` blocks. No configuration, no extra keyword.

### 8.3 Test Blocks on Components

```
component BookCard(book: Book) {
  <Box bg="surface" padding="md" radius="md">
    <Text value={book.title} weight="bold" testId="bookTitle" />
    <Text value={book.author} color="secondary" />
  </Box>

  test "default state" {
    props: { book: { title: "Dune", author: "Frank Herbert" } }
  }

  test "long title" {
    props: { book: { title: "The Lord of the Rings: The Fellowship of the Ring", author: "Tolkien" } }
  }

  test "interaction: title is visible" {
    props: { book: { title: "Dune", author: "Herbert" } }
    steps: [
      { assert: "bookTitle", visible: true },
      { assert: "bookTitle", text: "Dune" }
    ]
  }
}
```

The `props` body is a JSON-compatible object matching the component's declared props type. The `steps` array is optional — tests without steps are pure visual states.

### 8.4 The `steps` Vocabulary

`steps` is the unified interaction and assertion language. It is the only way to express interactions in both component tests, integration tests, and E2E tests.

```
steps: [
  // Actions
  { trigger: "<testId>", event: "click" },
  { trigger: "<testId>", event: "input", value: "some text" },
  { trigger: "<testId>", event: "change", value: "option-value" },
  { navigate: "<path>" },            // e2e only

  // Assertions
  { assert: "<testId>", visible: true },
  { assert: "<testId>", visible: false },
  { assert: "<testId>", disabled: true },
  { assert: "<testId>", disabled: false },
  { assert: "<testId>", text: "expected text" },
  { assert: "<testId>", count: 3 },
  { assert: "<testId>", checked: true },
]
```

`testId` refers to a `testId` prop set on a built-in layout component. See section 8.7.

**`navigate` is valid only in `e2e` blocks.** Using it in a `test` block is CLT705.

Additional step types will be added as needed, guided by real test requirements. The vocabulary is intentionally minimal to start.

### 8.5 Test Blocks on Pages (Integration)

Pages declare test blocks with a mock `Response<T>` fixture. The compiler validates the fixture shape against the generated `Response<T>` type.

```
page BookList(resp: Response<BookListResponse>) {
const books = resp.data?.books ?? []
----
<Column gap="md" testId="bookList">
  <each collection={books} as="book">
    <BookCard book={book} />
  </each>
</Column>

test "populated list" {
  resp: {
    data: { books: [
      { id: "1", title: "Dune", author: "Herbert" },
      { id: "2", title: "Foundation", author: "Asimov" }
    ]},
    isLoading: false,
    error: null
  }
}

test "loading state" {
  resp: { data: null, isLoading: true, error: null }
}

test "empty state" {
  resp: { data: { books: [] }, isLoading: false, error: null }
  steps: [
    { assert: "bookList", count: 0 }
  ]
}

test "interaction: form disables save after submit" {
  resp: { data: { books: [...] }, isLoading: false, error: null }
  steps: [
    { trigger: "saveButton", event: "click" },
    { assert: "saveButton", disabled: true }
  ]
}
}
```

The fixture key name (`resp` above) must match the parameter name declared in the page signature. The compiler validates this.

### 8.6 E2E Blocks

E2E blocks test full user flows in a real browser. API calls are intercepted at the network level before they reach the real backend.

```
page BookList(resp: Response<BookListResponse>) {
  ...

  e2e "user sees the book list" {
    api_mock: {
      "GET /api/books": { books: [{ id: "1", title: "Dune", author: "Herbert" }] }
    }
    steps: [
      { navigate: "/books" },
      { assert: "bookList", visible: true },
      { assert: "bookCard", count: 1 }
    ]
  }

  e2e "user navigates to book detail" {
    api_mock: {
      "GET /api/books": { books: [{ id: "1", title: "Dune", author: "Herbert" }] },
      "GET /api/books/:id": { book: { id: "1", title: "Dune", author: "Herbert" } }
    }
    steps: [
      { navigate: "/books" },
      { trigger: "bookCard", event: "click" },
      { assert: "bookDetail", visible: true },
      { assert: "bookTitle", text: "Dune" }
    ]
  }
}
```

`api_mock` intercepts HTTP requests matching the pattern. Patterns follow `METHOD /path` format. Dynamic segments (`:id`) match any value.

The compiler generates Playwright TypeScript from `e2e` blocks. The developer never writes Playwright directly.

### 8.7 `testId` Prop

`testId` is a valid prop on all built-in layout components (`Box`, `Row`, `Column`) and leaf components (`Button`, `Input`, `Text`, `Select`). It emits `data-testid="..."` on the generated element and is not validated against design tokens.

```
<Column gap="md" testId="bookList">
  <BookCard testId="bookCard" book={book} />
</Column>
```

`testId` values must be unique within a page's rendered output. Duplicate `testId` values in a static template are CLT708. Duplicate values that can only occur dynamically (inside `<each>`) are a warning, not an error.

### 8.8 Visual Test App

`origami test --preview` builds and serves a standalone app rendering every test state of every component and page.

- Sidebar: all files with test blocks.
- Per file: one panel per test state, labeled by name.
- Component tests: isolated render.
- Integration tests: isolated render with mocked Response<T>.
- A11y results per state (see section 10.3).

`origami test --build-preview` outputs a static site suitable for deployment as a visual review environment.

### 8.9 Snapshot Tests

`origami test --snapshot` captures an HTML snapshot of each test state and diffs on subsequent runs. `--update-snapshots` re-records all snapshots — deliberate, not automatic.

---

## 9. Internationalization

### 9.1 Design Constraints

`origami-i18n` is a built-in module, not a library. Minimal interface covering what applications need: key-based lookup, typed interpolation, plural forms, and runtime locale switching. Nothing else.

**Out of scope for this module:** date/time/number formatting (use platform locale APIs), RTL layout (handle via `tokens.json` CSS variables), complex CLDR plural rules beyond `one`/`other`.

### 9.2 Locale Files

Locale files live in `locales/` at workspace root, shared across all apps.

**Key naming convention:** maximum 3 segments, dot-separated.

```
{context}.{component}.{label}
```

- `context`: page name or `common` for shared strings (`books`, `admin`, `common`)
- `component`: component or feature name, PascalCase if compound (`bookCard`, `filterForm`)
- `label`: specific string identifier (`title`, `saveButton`, `emptyState`)

Examples: `common.loading`, `books.bookCard.title`, `books.filterForm.placeholder`, `admin.userTable.emptyState`.

The compiler enforces maximum 3 segments (CLT403). Keys with more than 3 segments are a compile error.

Keys are flat strings, not nested objects. The 3-segment convention is enforced by the compiler at the key level, not by JSON nesting.

```json
// locales/en.json
{
  "common.loading":           "Loading...",
  "common.error":             "Something went wrong.",
  "common.save":              "Save",
  "common.delete":            "Delete",
  "common.deleting":          "Deleting...",
  "books.bookCard.title":     "Book",
  "books.bookCard.author":    "by {author}",
  "books.list.title":         "My Books",
  "books.list.empty":         "No books found.",
  "books.list.count":         "{count, plural, one {# book} other {# books}}"
}
```

**Interpolation:** `{variableName}` for substitution, `{variableName, plural, one {…} other {…}}` for plurals.

Flat keys are easier to search (grep, LLM lookup), avoid structural duplication, and are immediately readable without navigating nested objects.

`en.json` (or the configured `default_locale`) is the canonical key source. Other locale files are validated against it: missing keys → W101, extra keys → W102.

### 9.3 Using `t()`

`t()` is a compiler built-in available in all templates and logic sections without import.

```
<Text value={t('books.list.title')} size="xl" />
<Text value={t('books.list.count', { count: resp.data.books.length })} color="secondary" />
<Text value={t('books.bookCard.author', { author: book.author })} color="secondary" />
```

**Compile-time validation:**
- Key must exist in the default locale. Unknown key → CLT401.
- Interpolation variables must match placeholders. Missing variable → CLT402. Extra variable → W103.

### 9.4 Locale Selection

```bash
origami dev                    # loads all locales, runtime switching available
origami dev --locale it        # loads only Italian (faster startup, useful for focused work)
origami build                  # bundles all locales by default
origami build --locale it      # bundles Italian only (smaller bundle for locale-specific deploy)
```

The default in both dev and build is `all` — all available locale files are loaded and runtime switching is enabled. `--locale <specific>` is an optimization for production deploys targeting a single market, or for dev sessions focused on a specific locale.

---

## 10. Accessibility

### 10.1 Design Constraints

`origami-a11y` is a built-in module. Because the vocabulary is closed, accessibility violations can be detected at compile time. Target: WCAG 2.1 AA.

### 10.2 Compile-time Checks

| Code | Rule |
|------|------|
| CLT601 | `Button` has no accessible label (no text child and no `aria-label` prop) |
| CLT602 | `Input` has no associated `Label` component |
| CLT603 | `Image` (future component) missing `alt` prop |
| CLT604 | Foreground/background color token combination fails 4.5:1 contrast ratio |

**CLT604 detail:** since color token values are declared in `tokens.json` as CSS custom properties, the compiler computes contrast ratios at build time. A `<Text color="secondary">` on a `<Box bg="surface">` is checked statically. No runtime check needed.

These are errors (not warnings) when the a11y module is enabled. See D06 for the discussion on making this configurable.

### 10.3 Runtime Checks in Tests

Each test state in the visual test app is scanned with axe-core. Results are shown inline in the test preview. A11y violations fail `origami test --a11y`.

```bash
origami test --a11y     # runs axe-core on all test states
```

---

## 11. CLI Reference

### `origami dev`

```
origami dev [--app <name>] [--env <name>] [--port <n>] [--host <host>]
```

Starts the dev server with HMR. Resolves workspace, fetches OpenAPI, validates `endpoints.toml`, compiles, watches for changes.

### `origami build`

```
origami build [--app <name>] [--env <name>] [--locale <locale|all>] [--out <dir>]
```

Production build. Exits non-zero on any error. CI-ready.

### `origami check`

```
origami check [--app <name>]
```

Validates without emitting output. Checks compilation, endpoint bindings, i18n keys, route conflicts. Use in pre-merge CI.

### `origami test`

```
origami test [options]

Options:
  --preview             Serve the visual test app (does not exit)
  --build-preview       Build the visual test app as a static site
  --snapshot            Run HTML snapshot comparison
  --update-snapshots    Re-record all snapshots
  --e2e                 Run E2E blocks via Playwright
  --a11y                Run axe-core on all test states
  --filter <pattern>    Run only matching test names
  --env <name>          Use a specific env section (default: [env.test] if present)
  --watch               Re-run on file changes
```

### `origami init`

```
origami init <project-name> [--app <name>] [--no-example]
```

Scaffolds a new workspace: standard directory structure, minimal `origami.toml`, sample `tokens.json`, starter `locales/en.json`.

### `origami unsafe-report`

```
origami unsafe-report [--app <name>] [--format json]
```

Lists all `<unsafe>` blocks, `unsafe()` prop values, and `as` overrides (W201) with reasons. Technical debt audit tool.

---

## 12. Dev Server

### 12.1 Current Implementation (v0.2)

`origami dev` orchestrates two processes under the hood, transparent to the developer:

1. **Origami compiler (Rust)** — watches `.ori` source files, recompiles incrementally to `__generated__/` on change. Runs synchronously via `spawn_blocking`. File watching uses `notify` with a 50 ms debounce.
2. **Nuxt dev server (Bun)** — watches `__generated__/`, handles Vue SFC compilation, HMR, and serves the browser. Bun is preferred over Node; the binary falls back to Node if Bun is not found.

The developer runs one command. The process management (spawn, pipe logs, kill on exit) is handled by the Origami binary. The Nuxt process is a child process; its stdout/stderr are forwarded to the terminal unchanged.

**Bun installation:** if neither Bun nor Node is found on `PATH`, `origami dev` prints a one-line install instruction and exits. It does not auto-install.

**API proxy:** `origami dev` generates a `nuxt.config.ts` in `__generated__/` that sets `nitro.devProxy` to forward `/api/*` to `api_base_url` from the active `[env.*]` section. The developer never writes Nuxt config.

### 12.2 Dev Server Roadmap

The current implementation is the pragmatic starting point. The progression toward a fully embedded dev server follows the compilation target roadmap:

| Phase | Dev server | JS runtime required |
|-------|-----------|---------------------|
| v0.2 (current) | Rust compiler + Nuxt child process (Bun preferred) | Bun or Node |
| Farm milestone | Farm replaces Nuxt dev server (Rust-native, Vite-compatible) | None |
| Owned bundling | axum + Rust SFC compiler embedded in binary | None |
| SSR | axum serves SSR responses; Nuxt SSR via `nuxt build` | Bun or Node |
| WASM | Full WASM target, runtime-agnostic | None |

The Farm milestone is the first point where `origami dev` becomes a true single-binary experience. It is not a prerequisite for a usable framework — Nuxt+Bun is sufficient and fast for SPA development.

---

## 13. Code Generation

### 13.1 Current Target: Nuxt (Intermediate)

The compiler emits Vue 3 SFCs targeting Nuxt's file-based conventions. The developer never writes Vue or Nuxt config. Generated files are artifacts — git-ignored, never touched by hand.

**Key design decision:** Nuxt's file-based router handles route resolution automatically. The compiler does not emit a `router.ts`. Pages go to `__generated__/pages/`, layouts to `__generated__/layouts/`. Nuxt picks them up at startup.

```
apps/web/
├── pages/               ← source (.ori)
├── components/          ← source (.ori)
└── __generated__/       ← compiler output (git-ignored)
    ├── components/
    ├── pages/           ← Nuxt file-based pages
    ├── layouts/         ← Nuxt layouts (from _layout.ori files)
    ├── nuxt.config.ts   ← generated by compiler (proxy, i18n, auto-imports)
    ├── types/
    │   ├── api.ts       ← TypeScript types from OpenAPI
    │   └── response.ts  ← Response<T>, Params<P>
    ├── locales/         ← processed locale bundles
    └── origami.css
```

Layout assignment is via `definePageMeta({ layout: '...' })` in each generated page. The layout name is derived by the compiler's layout chain algorithm: it walks the filesystem upward from the page file, finds the nearest `_layout.ori`, and derives the Nuxt layout name from the directory path.

### 13.2 Compilation Targets Roadmap

**Now — Nuxt SFC + Bun**
The compiler emits `.vue` files into `__generated__/pages/` and `__generated__/layouts/`. Nuxt (running on Bun) handles routing, SFC compilation, HMR, and serves the browser. `origami dev` manages both processes. The developer never writes Vue, Nuxt config, or a router. Nuxt is the current runtime target because it gives SSR as an almost-free addition — one `nuxt build` command away.

**Intermediate — Rust-native dev server**
As the framework matures, the Nuxt layer is progressively replaced. First candidate: Farm v1.0 (Rust-native build tool, Vite-compatible plugins, no JS runtime required). When Origami owns the JS transformation step entirely, the dev server is fully embedded using axum + notify + tokio.

**SSR — Nuxt SSR**
SSR is near-term with Nuxt as the target: `nuxt build` already produces a server bundle. The `origamiFetch` abstraction (section 13.3) is the only structural requirement — it must work on both client and server. No separate codegen path needed for this milestone: the same generated SFCs work for both SPA and SSR.

**North star — WASM**
The compiler targets WASM directly. No JS runtime dependency at any layer. Application source (`.ori` files) is identical across all targets.

The progression is additive: each new target is a new codegen path, existing targets unchanged.

### 13.3 The Fetcher Abstraction

The generated fetching code uses a `origamiFetch` function instead of a direct `fetch()` call:

```typescript
// generated: __generated__/fetch.ts
export const origamiFetch = <T>(url: string): Promise<T> =>
  fetch(url).then(r => r.json())
```

This indirection makes the data layer swappable. Future SSR or WASM targets replace `origamiFetch` without touching any generated page code. This is a lightweight constraint with significant future value.

### 13.4 Future Target: WASM

When maturity permits, the compiler gains a WASM codegen target eliminating the Vue runtime. Source files are unchanged; only the codegen crate changes.

### 13.5 Architectural Note: SSR and the Long-Term Vision

SSR is not in scope for the current release. However, certain architectural choices made now determine whether SSR can be added later without pain:

**What would make SSR hard to add later:**
- Components that assume `window` or `document` at render time (use `<unsafe>` for those, which already marks them as escape hatches)
- Hardcoded `fetch()` in generated code (addressed by `origamiFetch` in 13.3)
- A routing layer that is inherently client-only

**What already makes SSR tractable:**
- `Response<T>` is a typed data container with no inherent browser dependency. The same type works whether data was fetched server-side or client-side.
- The closed vocabulary means no arbitrary DOM access in templates.
- The compiler generates all code — SSR support is a codegen concern, not an application concern.

**The north star:** if the compiler eventually targets WASM, and the WASM module can run both in the browser and on a server, the same `.ori` source could compile to a full-stack application where the server handles data fetching and SSR, and the browser handles hydration and interaction. At that point, the only external dependency would be a database. This is a viable long-term direction that the current architecture does not preclude.

**Constraint for current development:** do not make choices that assume a browser environment in any layer that `origami-data` or `origami-router` touches. These modules should work equally well on a server.

---

## 14. Module Architecture

Each module is an independent crate with defined interfaces, developable and stabilizable in isolation.

```
origami-cli           ← orchestrator
├── origami-lexer     ← String → Vec<Token>                   (stable, 0.1.0)
├── origami-parser    ← Vec<Token> → AST                      (stable, 0.1.0)
├── origami-analyzer  ← AST + VocabularyMap validation        (stable, 0.1.0)
├── origami-codegen   ← AST → output files                    (active, 0.1.0)
├── origami-runtime   ← shared types                          (stable, 0.1.0)
├── origami-router    ← file-based route table                (new)
├── origami-data      ← OpenAPI parsing, endpoints.toml, type gen (new)
├── origami-i18n      ← locale validation, t() resolution     (new)
├── origami-a11y      ← compile-time a11y rules               (new)
├── origami-test      ← test block compiler, visual app, Playwright gen (new)
└── origami-dev       ← HMR server, file watcher, proxy       (new)
```

**Integration contract:** each crate receives the AST and workspace manifest as read-only input, returns validated output or typed errors. All error types live in `origami-runtime` for uniform rendering by the CLI.

**Recommended development order:**
1. `origami-router` — file-based routing without data
2. `origami-data` — OpenAPI integration and typed Response<T>
3. `origami-dev` — HMR dev server
4. `origami-i18n` — `t()` validation
5. `origami-test` — test blocks, visual app, Playwright codegen
6. `origami-a11y` — compile-time a11y rules

---

## 15. Error Codes

Codes CLT101–CLT107 from `0.1.0` remain valid.

### Routing (CLT2xx)

| Code | Meaning |
|------|---------|
| CLT201 | Route conflict: two files resolve to the same URL |
| CLT202 | Dynamic segment in `endpoints.toml` does not match route params |
| CLT203 | `_layout.ori` contains more than one `<slot />` |
| CLT204 | `_layout.ori` contains no `<slot />` |
| CLT205 | `page` keyword used outside `pages/` directory |
| CLT206 | `layout` keyword used outside a `_layout.ori` file |

### Data layer (CLT3xx)

| Code | Meaning |
|------|---------|
| CLT301 | Type name in `endpoints.toml` not found in OpenAPI schema |
| CLT302 | Endpoint path in `endpoints.toml` not found in OpenAPI paths |
| CLT303 | HTTP method not declared in OpenAPI for that path |
| CLT304 | Body type not found in OpenAPI schema |
| CLT305 | OpenAPI spec could not be fetched or parsed |
| CLT306 | `Response<T>` type parameter does not match `endpoints.toml` declaration |
| CLT307 | `Params<P>` keys do not match file path dynamic segments |
| CLT308 | `Response<T>` parameter name shadows a built-in identifier |

### i18n (CLT4xx)

| Code | Meaning |
|------|---------|
| CLT401 | `t()` key not found in default locale file |
| CLT402 | Interpolation variable missing from `t()` call |
| CLT403 | Locale key exceeds 3 segments |

### Mutations (CLT5xx)

| Code | Meaning |
|------|---------|
| CLT501 | Naming conflict: two mutations produce the same handle name |
| CLT502 | Mutation handle referenced in a page with no matching `[mutations]` entry |

### Accessibility (CLT6xx)

| Code | Meaning |
|------|---------|
| CLT601 | `Button` missing accessible label |
| CLT602 | `Input` missing associated `Label` |
| CLT603 | `Image` missing `alt` prop |
| CLT604 | Color token combination fails WCAG 4.5:1 contrast ratio |

### Testing (CLT7xx)

| Code | Meaning |
|------|---------|
| CLT701 | Duplicate test name within a component or page |
| CLT702 | Test fixture body contains non-JSON-compatible values |
| CLT703 | Page test fixture does not match `Response<T>` shape |
| CLT704 | `navigate` step used in a `test` block (only valid in `e2e`) |
| CLT705 | `api_mock` used in a `test` block (only valid in `e2e`) |
| CLT706 | `testId` value is not unique in a static template |
| CLT707 | `e2e` block declared on a component (only valid on pages) |
| CLT708 | Test fixture parameter name does not match page signature parameter name |

### Warnings (Wxxx)

| Code | Meaning |
|------|---------|
| W101 | Locale key missing from a non-default locale file |
| W102 | Extra key in a non-default locale file (no corresponding key in default) |
| W103 | Extra interpolation variable passed to `t()` |
| W201 | Handle name overridden via `as` on a query or mutation entry (deviation from convention) |

---

## 16. Future Modules

These subsystems are architecturally planned but not implemented in the initial release.

### `origami-middleware`

Route middleware enables logic that runs before or after route navigation — authentication checks, permission guards, analytics, logging.

**Planned convention:** `_middleware.ts` files in any `pages/` directory, discovered by the compiler, wired into the generated router without manual registration. Middleware stacks from root to leaf. A redirect in a parent short-circuits all children.

```typescript
// pages/books/_middleware.ts  (planned)
import type { RouteContext, GuardResult } from 'origami/router'

export const guard = async (ctx: RouteContext): Promise<GuardResult> => {
  if (!ctx.auth.isAuthenticated) {
    return { redirect: '/login', query: { from: ctx.path } }
  }
  return { proceed: true }
}
```

The function signature is fixed; the implementation is opaque TypeScript. The compiler validates the signature at build time.

**Auth and permissions** are the primary use case. A future companion module (`origami-auth`) will provide `ctx.auth` with a standard interface covering token management, session state, and role-based access. This has historically been painful to integrate in every project; it belongs in the framework.

The middleware module is deferred because it has no meaningful implementation without an auth provider to wire into.

### `origami fix` (codemod)

When an OpenAPI type is renamed, the compile-time error already lists every affected binding in a single block with suggestions. A future `origami fix rename-type <old> <new>` command will apply the rename automatically across `endpoints.toml` and all page signatures in one pass, with a diff preview before applying. Not required for v0.2 — the compile error is sufficient for manual correction.

---

## 17. Out of Scope

**Server-side rendering.** Not in scope for current release. The architecture is SSR-compatible (see 13.5) — this is an intentional constraint, not an oversight.

**Authentication and authorization.** Deferred to `origami-middleware` + `origami-auth`. Route guards will be the integration point.

**Backend framework.** The only coupling to the backend is the OpenAPI spec.

**Third-party component libraries.** All external components must be wrapped via `<unsafe>`. Intentional — makes non-system components visible and auditable.

**Custom CSS.** Developers do not write CSS. `tokens.json` is the only styling interface.

**Complex ICU plural rules.** Only `one`/`other` in this version.

**Date/time/number formatting in i18n.** Delegated to platform locale APIs.

---

*End of Specification — v0.2.0-draft*
