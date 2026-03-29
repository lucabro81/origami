# Block 01 — Router

**Version:** 0.1 — draft

## Contents

- [Scope](#scope)
- [Expected output](#expected-output)
- [Compiler pipeline extensions](#compiler-pipeline-extensions)
  - [Lexer — new tokens](#lexer--new-tokens)
  - [Parser — new AST nodes](#parser--new-ast-nodes)
  - [Analyzer — new validation rules](#analyzer--new-validation-rules)
- [origami-router](#origami-router)
  - [Input](#input)
  - [Algorithm: file → route](#algorithm-file--route)
  - [Algorithm: layout chain](#algorithm-layout-chain)
  - [RouteTable](#routetable)
  - [Generated component naming convention](#generated-component-naming-convention)
  - [Conflict detection](#conflict-detection-clt201)
- [origami-codegen extension](#origami-codegen-extension)
  - [Generated file structure](#generated-file-structure)
  - [Layout assignment](#layout-assignment)
- [Error codes](#error-codes)
- [CLI contribution](#cli-contribution)
- [Tests](#tests)

## Scope

This block implements Origami's file-based routing system. It includes compiler pipeline extensions for the `page` and `layout` keywords, the `origami-router` crate that transforms the filesystem structure into a typed route table, and the extension of `origami-codegen` to emit Nuxt-compatible pages and layouts into `__generated__/`.

Part of M1 (Compilable SPA). This is the first block that produces real user-facing output: navigable pages.

The codegen target is **Nuxt**. Nuxt's file-based router handles route resolution automatically — there is no `router.ts` to generate. The compiler's job is to emit correctly named and structured `.vue` files in `__generated__/pages/` and `__generated__/layouts/`.

## Expected output

- `page` and `layout` keywords recognised by the lexer, parsed, validated by the analyzer
- `origami-router`: given the pages filesystem, produces a typed `RouteTable`
- `origami-codegen` extended: from the `RouteTable`, emits Nuxt page and layout files into `__generated__/`
- Error codes CLT201–CLT206 implemented with `miette` messages
- Unit tests for file → route mapping and conflict detection
- `.ori` fixtures with pages and layouts for integration tests

---

## Compiler pipeline extensions

This block adds new language constructs. The changes touch three existing crates (ported from the POC): lexer, parser, analyzer. The principle is the same as in the POC: each crate does one thing and passes typed output to the next.

### Lexer — new tokens

The lexer recognises two new file-level keywords:

```
Token::PageKeyword      ← "page"
Token::LayoutKeyword    ← "layout"
```

All other syntax for `page` and `layout` reuses existing tokens: identifiers, parentheses, braces, `----`. The lexer does not need to understand structure — it emits tokens only.

### Parser — new AST nodes

The parser distinguishes three definition types at file level:

```
FileNode
  └── definitions: Vec<Definition>

Definition
  ├── Component(ComponentDef)   ← existing from POC
  ├── Page(PageDef)             ← new
  └── Layout(LayoutDef)         ← new

PageDef
  ├── name: String
  ├── signature: String         ← opaque TypeScript, same as ComponentDef
  ├── logic: String             ← section before ----
  └── template: TemplateNode    ← section after ----

LayoutDef
  ├── name: String
  ├── logic: Option<String>     ← optional for layouts
  └── template: TemplateNode    ← must contain exactly one <slot />
```

`PageDef` is structurally identical to `ComponentDef` — the distinction is semantic, not syntactic. The parser separates them to make analyzer validation more explicit.

`LayoutDef` has no signature: layouts receive no data from the outside.

### Analyzer — new validation rules

The analyzer receives the AST and the source file path. This block adds:

**Validations on `PageDef`:**
- CLT205: `page` keyword used outside `pages/` — error
- The signature is opaque TypeScript, not validated here (`Response<T>` and `Params<P>` are validated in Block 02)

**Validations on `LayoutDef`:**
- CLT203: `_layout.ori` contains more than one `<slot />`
- CLT204: `_layout.ori` contains no `<slot />`
- CLT206: `layout` keyword used outside a `_layout.ori` file

---

## origami-router

New crate. Receives the workspace manifest (directory structure) and the already-analyzed ASTs, produces a `RouteTable`.

### Input

```
WorkspaceManifest
  └── apps: Vec<App>
        └── pages_dir: PathBuf   ← e.g. "apps/web/pages/"

AnalyzedFile
  ├── source_path: PathBuf
  └── definitions: Vec<AnalyzedDefinition>
```

### Algorithm: file → route

The transformation from file path to URL route follows fixed rules, with no configuration:

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
- `_layout.ori` → not a route, used for layout wrapping
- `404.ori` → registered as fallback, not a normal route

Pages without a `_layout.ori` anywhere in their hierarchy are emitted as top-level Nuxt pages with no layout applied. This is correct behaviour — Nuxt handles it natively with no special casing needed.

### Algorithm: layout chain

For each route, the router computes the list of layouts that wrap it, from innermost to outermost. The algorithm walks up the filesystem from the page's directory to the `pages/` root:

```
for each page file:
  layout_chain = []
  current_dir = directory of the file

  while current_dir is inside pages/:
    if _layout.ori exists in current_dir:
      prepend layout to chain
    move up one level

  route.layout_chain = layout_chain  ← [outermost, ..., innermost]
```

Example for `apps/web/pages/books/[id].ori`:

```
pages/_layout.ori   → RootLayout    (outermost)
books/_layout.ori   → BooksLayout   (innermost)

layout_chain = [RootLayout, BooksLayout]
```

Multi-app support (`apps/web/`, `apps/admin/`) is a first-class design goal. The `WorkspaceManifest` holds a `Vec<App>` specifically to support this. This block implements and tests routing for a single app; the multi-app path (`origami dev --app admin`) is exercised once the single-app structure is stable.

### RouteTable

```
RouteTable
  └── routes: Vec<Route>

Route
  ├── url_pattern: String         ← "/books/:id"
  ├── source_file: PathBuf        ← "apps/web/pages/books/[id].ori"
  ├── component_name: String      ← "BooksId" (derived from path)
  ├── layout_chain: Vec<String>   ← ["RootLayout", "BooksLayout"]
  └── params: Vec<String>         ← ["id"]
```

### Generated component naming convention

The Vue component name is derived from the file path, in PascalCase, dropping the trailing `index` and normalising dynamic segments:

```
pages/index.ori              → Index
pages/books/index.ori        → BooksIndex
pages/books/[id].ori         → BooksId
pages/books/[id]/reviews.ori → BooksIdReviews
pages/404.ori                → NotFound
```

### Conflict detection (CLT201)

Two files producing the same URL pattern is a fatal error. The check runs on the `RouteTable` before codegen:

```
if two Routes share the same url_pattern:
  emit CLT201 listing both source_files
  abort compilation
```

There are no implicit precedence rules. Ambiguity is always an error.

---

## origami-codegen extension

`origami-codegen` receives the `RouteTable` from `origami-router` and emits the Nuxt page and layout files. No `router.ts` is generated — Nuxt's file-based router handles route resolution from the file structure.

### Generated file structure

```
__generated__/
  ├── pages/
  │   ├── index.vue
  │   ├── books/
  │   │   ├── index.vue
  │   │   └── [id].vue
  │   └── 404.vue
  ├── layouts/
  │   ├── default.vue     ← from pages/_layout.ori (root layout)
  │   └── books.vue       ← from pages/books/_layout.ori
  └── components/         ← shared components (from POC, extended)
```

The file names in `__generated__/pages/` mirror the `.ori` source names exactly, including `[id]` dynamic segments — Nuxt understands this convention natively.

### Layout assignment

For each page, the innermost layout in its chain becomes the active Nuxt layout. This is declared in the generated `<script setup>` block via `definePageMeta`:

```
// generated for apps/web/pages/books/[id].ori
// with layout_chain = [RootLayout, BooksLayout]

definePageMeta({ layout: 'books' })
```

Nuxt's layout nesting handles the rest: the `books` layout wraps the page, the `default` layout wraps the `books` layout via the `<slot />` chain. This maps exactly to the layout nesting semantics defined in the spec.

Route middleware (`_middleware.ts`) is a planned future feature. The generated page files must leave a natural integration point — `definePageMeta` already supports `middleware: [...]`, so no structural changes will be needed when that feature is implemented.

All `__generated__/` output is git-ignored. The developer never touches it.

---

## Error codes

| Code | Condition | Phase |
|------|-----------|-------|
| CLT201 | Two files produce the same URL pattern | router, before codegen |
| CLT202 | Dynamic segment in `endpoints.toml` does not match route params | data layer — noted here, implemented in Block 02 |
| CLT203 | `_layout.ori` contains more than one `<slot />` | analyzer |
| CLT204 | `_layout.ori` contains no `<slot />` | analyzer |
| CLT205 | `page` keyword used outside `pages/` | analyzer |
| CLT206 | `layout` keyword used outside a `_layout.ori` file | analyzer |

All errors use `miette` with source spans and readable messages. CLT201 includes the paths of both conflicting files.

---

## CLI contribution

With this block, `origami check` runs routing validation: detects conflicts, missing/duplicate slots, misplaced keywords. No output on success — exits non-zero on errors.

`origami build` is extended to invoke `origami-router` and write `__generated__/pages/` and `__generated__/layouts/`.

---

## Tests

**Unit tests in `origami-router/src/tests.rs`:**
- File path → URL pattern mapping for all cases (index, dynamic, nested, 404)
- Layout chain computation for various directory structures
- Conflict detection: two files producing the same route
- Generated component naming convention

**Integration fixtures in `fixtures/`:**

```
fixtures/
  └── routing/
      ├── simple/           ← app with index and one static page
      ├── dynamic/          ← app with dynamic segments [id]
      ├── nested-layouts/   ← app with root layout + intermediate layout
      └── conflict/         ← two files triggering CLT201 (error case test)
```

Integration tests compile the fixture and verify the contents of the generated `__generated__/pages/` and `__generated__/layouts/` files.
