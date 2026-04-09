# Block 01 — Router

Implements file-based routing: `page`/`layout` keywords, `origami-router` crate (filesystem → `RouteTable`), `origami-codegen` extension (emits Nuxt pages/layouts into `__generated__/`).

Part of M1. Produces the first real user-facing output: navigable pages.

---

## Checklist — done when all of these are true

- [ ] `page` and `layout` tokens recognised by lexer, parsed, validated by analyzer
- [ ] `origami-router`: given a pages filesystem, produces a correct `RouteTable`
- [ ] `origami-codegen` extended: emits `__generated__/pages/` and `__generated__/layouts/`
- [ ] Error codes CLT201–CLT206 implemented with `miette` messages and source spans
- [ ] Unit tests pass: file→route mapping, layout chain, conflict detection, naming convention
- [ ] Integration fixtures exist for: simple, dynamic, nested-layouts, conflict cases
- [ ] `cargo clippy -- -D warnings` clean, `rustfmt` applied

---

## Error codes

| Code | Condition | Phase |
|------|-----------|-------|
| CLT201 | Two files produce the same URL pattern | router, before codegen |
| CLT202 | Dynamic segment in `endpoints.toml` does not match route params | data layer (Block 02) |
| CLT203 | `_layout.ori` contains more than one `<slot />` | analyzer |
| CLT204 | `_layout.ori` contains no `<slot />` | analyzer |
| CLT205 | `page` keyword used outside `pages/` | analyzer |
| CLT206 | `layout` keyword used outside a `_layout.ori` file | analyzer |

CLT201 must list both conflicting source file paths in the error message.

---

## Lexer — new tokens

```
Token::PageKeyword      ← "page"
Token::LayoutKeyword    ← "layout"
```

All other syntax for `page` and `layout` reuses existing tokens. The lexer emits tokens only — no structural understanding required.

---

## Parser — new AST nodes

```
FileNode
  └── definitions: Vec<Definition>

Definition
  ├── Component(ComponentDef)   ← existing
  ├── Page(PageDef)             ← new
  └── Layout(LayoutDef)         ← new

PageDef
  ├── name: String
  ├── signature: String         ← opaque TypeScript (validated in Block 02)
  ├── logic: String             ← section before ----
  └── template: TemplateNode    ← section after ----

LayoutDef
  ├── name: String
  ├── logic: Option<String>     ← optional
  └── template: TemplateNode    ← must contain exactly one <slot />
```

`PageDef` is structurally identical to `ComponentDef` — separated for explicit analyzer validation.
`LayoutDef` has no signature: layouts have no data binding.

---

## Analyzer — new validations

On `PageDef`:
- CLT205: `page` keyword used outside `pages/` directory → error

On `LayoutDef`:
- CLT203: `_layout.ori` has more than one `<slot />`
- CLT204: `_layout.ori` has no `<slot />`
- CLT206: `layout` keyword used outside a `_layout.ori` file

---

## origami-router crate

New crate. Input: `WorkspaceManifest` (directory structure) + analyzed ASTs. Output: `RouteTable`.

### Input types

```rust
WorkspaceManifest {
    apps: Vec<App {
        pages_dir: PathBuf   // e.g. "apps/web/pages/"
    }>
}

AnalyzedFile {
    source_path: PathBuf,
    definitions: Vec<AnalyzedDefinition>,
}
```

### File → route algorithm

```
apps/web/pages/index.ori              →  /
apps/web/pages/books/index.ori        →  /books
apps/web/pages/books/[id].ori         →  /books/:id
apps/web/pages/books/[id]/reviews.ori →  /books/:id/reviews
apps/web/pages/404.ori                →  (not-found fallback)
apps/web/pages/_layout.ori            →  (not a route)
```

Rules:
- `index.ori` → empty segment (matches the directory)
- `[segment]` → `:segment` (dynamic param)
- `_layout.ori` → not a route
- `404.ori` → not-found fallback, not a normal route

### Layout chain algorithm

For each page file, walk up from its directory to `pages/` root:

```
for each page file:
  layout_chain = []
  current_dir = directory of the file

  while current_dir is inside pages/:
    if _layout.ori exists in current_dir:
      prepend layout to chain
    move up one level

  route.layout_chain = layout_chain  // [outermost, ..., innermost]
```

Example for `apps/web/pages/books/[id].ori`:
- `pages/_layout.ori` → RootLayout (outermost)
- `pages/books/_layout.ori` → BooksLayout (innermost)
- `layout_chain = [RootLayout, BooksLayout]`

### RouteTable

```rust
RouteTable {
    routes: Vec<Route {
        url_pattern: String,          // "/books/:id"
        source_file: PathBuf,         // "apps/web/pages/books/[id].ori"
        component_name: String,       // "BooksId"
        layout_chain: Vec<String>,    // ["RootLayout", "BooksLayout"]
        params: Vec<String>,          // ["id"]
    }>
}
```

### Component naming convention

PascalCase from file path, drop trailing `index`, normalise dynamic segments:

| File path | Component name |
|-----------|---------------|
| `pages/index.ori` | `Index` |
| `pages/books/index.ori` | `BooksIndex` |
| `pages/books/[id].ori` | `BooksId` |
| `pages/books/[id]/reviews.ori` | `BooksIdReviews` |
| `pages/404.ori` | `NotFound` |

### Conflict detection (CLT201)

```
if two Routes share the same url_pattern:
  emit CLT201 listing both source_files
  abort compilation
```

No implicit precedence. Ambiguity is always a fatal error.

---

## origami-codegen extension

Receives `RouteTable`, emits:

```
__generated__/
  ├── pages/
  │   ├── index.vue
  │   ├── books/
  │   │   ├── index.vue
  │   │   └── [id].vue
  │   └── 404.vue
  └── layouts/
      ├── default.vue     ← from pages/_layout.ori (root layout)
      └── books.vue       ← from pages/books/_layout.ori
```

File names in `__generated__/pages/` mirror `.ori` source names exactly, including `[id]` segments — Nuxt understands this natively. No `router.ts` is generated — Nuxt's file-based router handles resolution.

### Layout assignment

The innermost layout in the chain becomes the active Nuxt layout, declared via `definePageMeta`:

```typescript
// generated for pages/books/[id].ori with layout_chain = [RootLayout, BooksLayout]
definePageMeta({ layout: 'books', middleware: [] })
```

Nuxt's nested layout system handles the rest. The `middleware: []` slot is reserved for Block future middleware — no structural change needed when implemented.

---

## CLI contribution

`origami check`: detects route conflicts, slot errors, misplaced keywords. No output on success, exits non-zero on errors.

`origami build`: extended to invoke `origami-router` and write `__generated__/pages/` and `__generated__/layouts/`.

---

## Tests

### Unit tests — `origami-router/src/tests.rs`

- File path → URL pattern for all cases: index, dynamic, nested, 404, `_layout`
- Layout chain for: no layout, root only, intermediate only, both root and intermediate
- Conflict detection: two files producing the same route → CLT201
- Component naming: all cases in the naming convention table

### Integration fixtures — `fixtures/routing/`

```
fixtures/routing/
  ├── simple/           ← index + one static page, no layouts
  ├── dynamic/          ← dynamic segment [id]
  ├── nested-layouts/   ← root layout + intermediate layout
  └── conflict/         ← two files triggering CLT201 (must fail compilation)
```

Integration tests compile each fixture and verify the contents of `__generated__/pages/` and `__generated__/layouts/`.
