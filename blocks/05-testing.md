# Block 05 — Testing

**Version:** 0.1 — draft

## Contents

- [Scope](#scope)
- [Expected output](#expected-output)
- [Design principles](#design-principles)
- [Test tiers](#test-tiers)
- [Compiler pipeline extensions](#compiler-pipeline-extensions)
  - [Lexer — new tokens](#lexer--new-tokens)
  - [Parser — new AST nodes](#parser--new-ast-nodes)
  - [Analyzer — new validation rules](#analyzer--new-validation-rules)
- [origami-test](#origami-test)
  - [Input](#input)
  - [Fixture validation](#fixture-validation)
  - [Visual preview app](#visual-preview-app)
  - [Playwright codegen](#playwright-codegen)
  - [Snapshot tests](#snapshot-tests)
- [origami-codegen extension](#origami-codegen-extension)
- [The steps vocabulary](#the-steps-vocabulary)
- [testId prop](#testid-prop)
- [Error codes](#error-codes)
- [CLI contribution](#cli-contribution)
- [Tests](#tests)

---

## Scope

This block implements Origami's testing system. It includes compiler pipeline extensions for `test` and `e2e` blocks, the `origami-test` crate that validates fixtures, generates Playwright tests, and drives the visual preview app, and the codegen that wires test states into the generated Nuxt application.

Part of M3 (Feature-complete framework). The testing module is opt-in: it activates when `"test"` is listed in `modules` in `origami.toml`.

---

## Expected output

- `test` and `e2e` blocks in `.ori` files parsed and validated by the compiler
- Fixture shapes validated against `Response<T>` types at compile time
- Playwright TypeScript test files generated from `e2e` blocks
- Visual preview app available via `origami test --preview`
- Snapshot tests via `origami test --snapshot`
- axe-core a11y scan results per test state in the preview app (wired to Block 06)
- Error codes CLT701–CLT708 implemented with `miette` messages

---

## Design principles

**Testing is structural, not optional.** A component without test states is a component with unknown behaviour. The visual preview app makes this visible by inspection — coverage gaps are obvious, not hidden behind a tool report.

**One vocabulary, three contexts.** The `steps` array is the same in `test` blocks (component/page), and `e2e` blocks. What changes is the execution context and what gets generated. Developers do not need to learn different assertion APIs.

**The compiler executes tests, not the developer.** A `test` block with `steps` generates a Playwright component test automatically. The developer writes fixture data and interaction steps; the framework decides how to run them. There is no test runner configuration.

**E2E blocks are network-level.** `e2e` blocks intercept HTTP at the network level via `api_mock`. They do not share mocking infrastructure with `test` blocks. `navigate` and `api_mock` are only valid in `e2e` blocks.

---

## Test tiers

| Keyword | Target | Steps | Visual preview | Playwright |
|---------|--------|-------|----------------|------------|
| `test` | `component` or `page` | no | ✓ static render | — |
| `test` | `component` or `page` | yes | ✓ render + step results | ✓ component test |
| `e2e` | `page` only | always required | — | ✓ full-browser test |

Rules:
- A `test` block always appears in the visual preview app, regardless of whether it has `steps`.
- A `test` block with `steps` additionally generates a Playwright component test (isolated render, no router, no real network).
- An `e2e` block generates only a Playwright full-browser test. It does not appear in the visual preview because it requires a real router and real (mocked) network — it cannot be rendered in isolation.
- `e2e` blocks are only valid on `page` definitions, not on `component` definitions (CLT707).

---

## Compiler pipeline extensions

### Lexer — new tokens

```
Token::TestKeyword    ← "test"
Token::E2eKeyword     ← "e2e"
```

The block body syntax (braces, string literals, colon, brackets) reuses existing tokens.

### Parser — new AST nodes

`test` and `e2e` blocks are parsed as trailing definitions on `ComponentDef`, `PageDef`, and `LayoutDef`. They are siblings to the template, not nested inside it.

```
TestBlock
  ├── name: String                   ← e.g. "default state"
  ├── fixture: TestFixture            ← prop values for components, Response<T> for pages
  └── steps: Option<Vec<TestStep>>   ← absent → visual only; present → also Playwright

E2eBlock
  ├── name: String
  ├── api_mock: HashMap<String, Value>  ← "GET /api/books" → mock response
  └── steps: Vec<TestStep>             ← always required

TestFixture
  ├── ComponentFixture(HashMap<String, Value>)   ← for component test blocks
  └── PageFixture(HashMap<String, Value>)        ← for page test blocks, keyed by param name

TestStep
  ├── Trigger { test_id: String, event: TriggerEvent, value: Option<String> }
  ├── Assert  { test_id: String, assertion: Assertion }
  └── Navigate { path: String }   ← e2e only
```

### Analyzer — new validation rules

- CLT701: duplicate test name within a component or page
- CLT702: test fixture contains non-JSON-compatible values
- CLT703: page test fixture does not match the `Response<T>` shape for that page
- CLT704: `navigate` step used in a `test` block (only valid in `e2e`)
- CLT705: `api_mock` used in a `test` block (only valid in `e2e`)
- CLT706: duplicate `testId` value in a static template
- CLT707: `e2e` block declared on a `component` (only valid on pages)
- CLT708: page test fixture parameter name does not match the page signature parameter name

CLT703 and CLT708 require the `DataManifest` from Block 02 — the analyzer must have access to it to validate fixture shapes against the generated `Response<T>` types.

---

## origami-test

New crate. Receives the full `AnalyzedWorkspace` (all files with their ASTs and test blocks) and the `DataManifest`, produces:

- Validated `TestManifest` (all test blocks, fixture-checked, ready for codegen)
- Playwright test files (from `e2e` blocks)
- Visual preview app (on demand, via `origami test --preview`)
- Snapshot comparisons (on demand, via `origami test --snapshot`)

### Input

```
AnalyzedWorkspace
  └── files: Vec<AnalyzedFile>
        └── test_blocks: Vec<TestBlock>
        └── e2e_blocks: Vec<E2eBlock>

DataManifest                ← from origami-data (Block 02)
  └── keys: HashMap<RouteKey, ResponseType>
```

### Fixture validation

For `test` blocks on **components**: the `fixture` object is validated as JSON-compatible. No type checking against the component's props — props are opaque TypeScript in the compiler. The developer is responsible for shape correctness; the compiler ensures serializability.

For `test` blocks on **pages**: the `fixture` object is validated against the `Response<T>` shape from the `DataManifest`. Key presence, nesting, and basic type compatibility (null vs. object) are checked. This catches the most common mistake — stale fixtures after an API type change.

For `e2e` blocks: `api_mock` keys are validated against `endpoints.toml`. A mock for an endpoint not declared in `[queries]` or `[mutations]` is a warning — not an error, since E2E tests sometimes mock endpoints outside the declared data layer.

### Visual preview app

`origami test --preview` builds and serves a standalone **plain Vue** app (not Nuxt) rendering every test state of every component and page.

Plain Vue is the right choice here: components and pages in test state are isolated renders with mock data — no Nuxt routing, no SSR, no full framework initialization required. The preview app is also more portable this way: when Nuxt is eventually replaced as the codegen target, the preview app is unaffected.

No iframes. Storybook uses iframes to isolate arbitrary CSS and JavaScript between stories. Origami has neither problem: CSS is globally defined by `tokens.json` via `origami.css` — it is global by design, not by accident — and Vue's `QueryClient` is scoped per component instance via provide/inject. The preview app is a plain Vue app with a sidebar and a `<component :is="currentState" />` mount in the main panel. Simple, no accidental complexity.

```
Layout:
  Sidebar
    └── file list (all files with test blocks)
          └── per file: one entry per test state (name)
  Main panel
    └── isolated render of the selected test state
          └── a11y results (from axe-core, wired in Block 06)
```

**Component test states** are rendered as plain Vue components with mock props injected directly.

**Page test states** are rendered as Vue components with mock `Response<T>` data injected via a pre-populated TanStack `QueryClient`. No network calls are made — the fixture data bypasses the real data layer entirely. This allows rendering "loading state", "empty state", "error state", and "populated state" side by side without any server.

**Layout preview** is deferred — a layout without real page content is low value. It can be added when a concrete use case emerges.

The preview app is generated into `__generated__/preview/` and is separate from the main application — it shares no routing or data layer with it.

`origami test --build-preview` outputs a static site suitable for deployment as a visual review environment (e.g. on a PR preview URL).

### Playwright codegen

From every `test` block with `steps` and every `e2e` block, `origami-test` generates a Playwright TypeScript test file into `__generated__/tests/`:

```
__generated__/
  └── tests/
      ├── components/
      │   └── BookCard.spec.ts   ← from test blocks on BookCard component
      └── pages/
          ├── BookList.spec.ts   ← from test + e2e blocks on BookList page
          └── BookDetail.spec.ts
```

The developer never writes Playwright directly. The generated spec files are git-ignored.

**Component test structure (from `test` block with `steps`):**

```typescript
// generated — BookCard.spec.ts
import { mount } from '@playwright/experimental-ct-vue'
import BookCard from '../../components/BookCard.vue'

test('interaction: title is visible', async ({ mount }) => {
  const component = await mount(BookCard, {
    props: { book: { title: 'Dune', author: 'Frank Herbert' } }
  })
  await expect(component.getByTestId('bookTitle')).toBeVisible()
  await expect(component.getByTestId('bookTitle')).toHaveText('Dune')
})
```

**E2E test structure (from `e2e` block):**

```typescript
// generated — BookList.spec.ts
import { test, expect } from '@playwright/test'

test('user sees the book list', async ({ page, route }) => {
  await page.route('/api/books', route =>
    route.fulfill({ json: { books: [{ id: '1', title: 'Dune', author: 'Herbert' }] } })
  )
  await page.goto('/books')
  await expect(page.getByTestId('bookList')).toBeVisible()
  await expect(page.getByTestId('bookCard')).toHaveCount(1)
})
```

### Snapshot tests

`origami test --snapshot` renders each test state to HTML and saves the output as a snapshot file. `origami test --snapshot` on subsequent runs diffs against the saved snapshot. `--update-snapshots` re-records all snapshots deliberately.

Snapshots are stored in `snapshots/` at the workspace root and committed to version control. They are a regression safety net — unexpected visual changes are caught before code review.

---

## origami-codegen extension

`origami-codegen` receives the `TestManifest` and emits:

- The visual preview app pages (one `.vue` per test state, under `__generated__/preview/`)
- The Playwright spec files (under `__generated__/tests/`)
- A `playwright.config.ts` at the workspace root (if not already present)

The `playwright.config.ts` is only generated once — subsequent runs do not overwrite it if it already exists, allowing the developer to customise Playwright configuration.

---

## The steps vocabulary

`steps` is the unified interaction and assertion language across all test tiers. It is intentionally minimal — new step types are added when required by real use cases, never speculatively.

```
Actions:
  { trigger: "<testId>", event: "click" }
  { trigger: "<testId>", event: "input", value: "some text" }
  { trigger: "<testId>", event: "change", value: "option-value" }
  { navigate: "<path>" }                    ← e2e only (CLT704 if used in test)

Assertions:
  { assert: "<testId>", visible: true }
  { assert: "<testId>", visible: false }
  { assert: "<testId>", disabled: true }
  { assert: "<testId>", disabled: false }
  { assert: "<testId>", text: "expected text" }
  { assert: "<testId>", count: 3 }
  { assert: "<testId>", checked: true }
```

Every `<testId>` reference must correspond to a `testId` prop on a built-in component in the same template (see testId prop section). References to unknown `testId` values are a compile error.

---

## testId prop

`testId` is a valid prop on all built-in layout and leaf components: `Box`, `Row`, `Column`, `Text`, `Button`, `Input`, `Select`. It emits `data-testid="..."` on the generated HTML element and is not validated against design tokens.

```
<Column gap="md" testId="bookList">
  <BookCard testId="bookCard" book={book} />
</Column>
```

Rules:
- `testId` values must be unique within a page's static template (CLT706).
- Duplicate values inside `<each>` (dynamically generated) are a warning, not an error — they are expected and handled by Playwright's `getByTestId().nth()` API.

---

## Error codes

| Code | Condition | Phase |
|------|-----------|-------|
| CLT701 | Duplicate test name within a component or page | analyzer |
| CLT702 | Test fixture contains non-JSON-compatible values | analyzer |
| CLT703 | Page test fixture does not match `Response<T>` shape | analyzer + data manifest |
| CLT704 | `navigate` step used in a `test` block | analyzer |
| CLT705 | `api_mock` used in a `test` block | analyzer |
| CLT706 | Duplicate `testId` value in a static template | analyzer |
| CLT707 | `e2e` block declared on a `component` | analyzer |
| CLT708 | Page test fixture parameter name does not match page signature | analyzer |

---

## CLI contribution

This block fully implements the `origami test` subcommand:

```
origami test [--preview] [--build-preview] [--snapshot] [--update-snapshots]
             [--e2e] [--a11y] [--filter <pattern>] [--env <name>] [--watch]
```

- `--preview`: serve the visual preview app (does not exit)
- `--build-preview`: build the preview app as a static site
- `--snapshot`: run HTML snapshot comparison
- `--update-snapshots`: re-record all snapshots
- `--e2e`: run `e2e` blocks via Playwright full-browser
- `--a11y`: run axe-core on all test states (wired to Block 06)
- `--filter <pattern>`: run only tests whose name matches the pattern
- `--env <name>`: use a specific env section (default: `[env.test]` if present)
- `--watch`: re-run on file changes

---

## Tests

**Unit tests in `origami-test/src/tests.rs`:**
- Fixture validation: JSON-compatible check, `Response<T>` shape matching
- `testId` conflict detection within static templates
- `navigate` / `api_mock` in wrong block type (CLT704, CLT705)
- Playwright codegen: verify generated spec file structure for a known test block

**Integration fixtures in `fixtures/`:**

```
fixtures/
  └── testing/
      ├── component-visual/     ← test block, no steps (visual only)
      ├── component-steps/      ← test block with steps → Playwright component test
      ├── page-fixture/         ← page test block with Response<T> fixture
      ├── e2e/                  ← e2e block with api_mock and navigate
      ├── fixture-mismatch/     ← CLT703: fixture shape doesn't match Response<T>
      └── duplicate-testid/     ← CLT706: two elements with same testId
```
