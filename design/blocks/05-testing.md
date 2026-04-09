# Block 05 — Testing

Implements the testing system: `test`/`e2e` block compiler extensions, `origami-test` crate (fixture validation, visual preview app, Playwright codegen, snapshots).

Part of M3. Opt-in: activates when `"test"` is listed in `modules` in `origami.toml`.

---

## Checklist — done when all of these are true

- [ ] `test` and `e2e` blocks in `.ori` files parsed and validated by the compiler
- [ ] Page fixture shapes validated against `Response<T>` types at compile time (CLT703, CLT708)
- [ ] Playwright TypeScript files generated from `e2e` blocks into `__generated__/tests/`
- [ ] Visual preview app available via `origami test --preview` (plain Vue, not Nuxt)
- [ ] Snapshot tests work via `origami test --snapshot` / `--update-snapshots`
- [ ] Error codes CLT701–CLT708 implemented with `miette` messages
- [ ] Unit tests pass: fixture validation, testId conflicts, steps vocabulary rules, Playwright codegen
- [ ] Integration fixtures exist for all test tier variants
- [ ] `cargo clippy -- -D warnings` clean, `rustfmt` applied

---

## Error codes

| Code | Condition | Phase |
|------|-----------|-------|
| CLT701 | Duplicate test name in a component or page | analyzer |
| CLT702 | Test fixture contains non-JSON-compatible values | analyzer |
| CLT703 | Page test fixture does not match `Response<T>` shape | analyzer + DataManifest |
| CLT704 | `navigate` step in a `test` block (only valid in `e2e`) | analyzer |
| CLT705 | `api_mock` in a `test` block (only valid in `e2e`) | analyzer |
| CLT706 | Duplicate `testId` value in a static template | analyzer |
| CLT707 | `e2e` block declared on a `component` (only valid on pages) | analyzer |
| CLT708 | Fixture parameter name does not match page signature parameter name | analyzer |

CLT703 and CLT708 require the `DataManifest` from Block 02.

---

## Test tiers

| Keyword | Target | Steps | Visual preview | Playwright |
|---------|--------|-------|----------------|------------|
| `test` | `component` or `page` | no | ✓ static render | — |
| `test` | `component` or `page` | yes | ✓ render + step results | ✓ component test |
| `e2e` | `page` only | always | — | ✓ full-browser |

- `test` always appears in the preview app.
- `test` + `steps` additionally generates a Playwright component test (isolated render, no router, no real network).
- `e2e` generates only a Playwright full-browser test. No preview — requires real router and mocked network.
- `e2e` on a `component` is CLT707.

---

## Compiler pipeline extensions

### Lexer — new tokens

```
Token::TestKeyword    ← "test"
Token::E2eKeyword     ← "e2e"
```

Block body syntax reuses existing tokens (braces, string literals, colon, brackets).

### Parser — new AST nodes

`test` and `e2e` blocks are trailing definitions on `ComponentDef`, `PageDef`, `LayoutDef`. Siblings to the template, not nested inside it.

```
TestBlock
  ├── name: String
  ├── fixture: TestFixture
  └── steps: Option<Vec<TestStep>>   ← absent → visual only; present → also Playwright

E2eBlock
  ├── name: String
  ├── api_mock: HashMap<String, Value>   ← "GET /api/books" → mock response body
  └── steps: Vec<TestStep>              ← always required

TestFixture
  ├── ComponentFixture(HashMap<String, Value>)  ← for component test blocks
  └── PageFixture(HashMap<String, Value>)       ← keyed by page signature param name

TestStep
  ├── Trigger  { test_id: String, event: TriggerEvent, value: Option<String> }
  ├── Assert   { test_id: String, assertion: Assertion }
  └── Navigate { path: String }    ← e2e only (CLT704 if in test)
```

### Analyzer — new validations

- CLT701: duplicate test name within same component or page
- CLT702: fixture contains non-JSON-compatible values
- CLT703: page fixture does not match `Response<T>` shape (requires DataManifest)
- CLT704: `navigate` in a `test` block
- CLT705: `api_mock` in a `test` block
- CLT706: duplicate `testId` in a static template
- CLT707: `e2e` block on a `component`
- CLT708: fixture parameter name does not match page signature parameter name

---

## steps vocabulary

Unified interaction and assertion language across all test tiers. Intentionally minimal.

```
Actions:
  { trigger: "<testId>", event: "click" }
  { trigger: "<testId>", event: "input", value: "some text" }
  { trigger: "<testId>", event: "change", value: "option-value" }
  { navigate: "<path>" }                     ← e2e only

Assertions:
  { assert: "<testId>", visible: true|false }
  { assert: "<testId>", disabled: true|false }
  { assert: "<testId>", text: "expected text" }
  { assert: "<testId>", count: 3 }
  { assert: "<testId>", checked: true }
```

Every `<testId>` must correspond to a `testId` prop on a built-in component in the same template.

---

## testId prop

Valid on all built-in layout and leaf components: `Box`, `Row`, `Column`, `Text`, `Button`, `Input`, `Select`. Emits `data-testid="..."` on the generated element.

```
<Column gap="md" testId="bookList">
  <BookCard testId="bookCard" book={book} />
</Column>
```

- `testId` values must be unique in static templates (CLT706).
- Duplicates inside `<each>` are a warning, not an error — expected and handled by `getByTestId().nth()`.

---

## origami-test crate

New crate. Input: `AnalyzedWorkspace` + `DataManifest`. Output: `TestManifest` + Playwright files + preview app (on demand).

### Fixture validation

**Component fixtures:** validated as JSON-compatible only. Props are opaque TypeScript — shape checking is the developer's responsibility; the compiler ensures serializability.

**Page fixtures:** validated against the `Response<T>` shape from the `DataManifest`. Key presence, nesting, and null vs. object compatibility are checked. Catches stale fixtures after API type changes.

**E2E `api_mock` keys:** validated against `endpoints.toml`. A mock for an undeclared endpoint is a warning — E2E tests may legitimately mock endpoints outside the data layer.

### Visual preview app

`origami test --preview` builds and serves a standalone plain Vue app (not Nuxt) rendering every test state.

RULE: Use plain Vue, not Nuxt. Test states are isolated renders with mock data — no routing, no SSR, no full framework initialization needed.
RULE: No iframes. CSS is global by design (`origami.css` from `tokens.json`). Vue `QueryClient` is scoped per component via provide/inject. A plain `<component :is="currentState" />` mount is sufficient.

```
Layout:
  Sidebar: all files with test blocks → per file: one entry per test state
  Main panel: isolated render of the selected test state
               + a11y results from axe-core (Block 06)
```

**Component states:** plain Vue component with mock props injected directly.

**Page states:** Vue component with mock `Response<T>` data injected via a pre-populated TanStack `QueryClient`. No network calls.

`origami test --build-preview`: output a static site for deployment as a visual review environment.

Layout preview is deferred — a layout without real page content has low value.

### Playwright codegen

Generated into `__generated__/tests/`:

```
__generated__/tests/
  ├── components/BookCard.spec.ts    ← from test blocks with steps
  └── pages/BookList.spec.ts         ← from test + e2e blocks
```

Developer never writes Playwright directly. Generated files are git-ignored.

**Component test (from `test` block with `steps`):**

```typescript
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

**E2E test (from `e2e` block):**

```typescript
import { test, expect } from '@playwright/test'

test('user sees the book list', async ({ page }) => {
  await page.route('/api/books', route =>
    route.fulfill({ json: { books: [{ id: '1', title: 'Dune', author: 'Herbert' }] } })
  )
  await page.goto('/books')
  await expect(page.getByTestId('bookList')).toBeVisible()
  await expect(page.getByTestId('bookCard')).toHaveCount(1)
})
```

### Snapshot tests

`origami test --snapshot`: renders each test state to HTML, saves as snapshot. Subsequent runs diff against saved snapshot. `--update-snapshots` re-records deliberately.

Snapshots stored in `snapshots/` at workspace root, committed to version control.

---

## origami-codegen extension

Receives `TestManifest`, emits:

- Preview app pages into `__generated__/preview/` (one `.vue` per test state)
- Playwright spec files into `__generated__/tests/`
- `playwright.config.ts` at workspace root (generated once — not overwritten if already exists)

---

## CLI contribution

```
origami test [--preview] [--build-preview] [--snapshot] [--update-snapshots]
             [--e2e] [--a11y] [--filter <pattern>] [--env <name>] [--watch]
```

- `--preview`: serve visual preview app (does not exit)
- `--build-preview`: build preview as static site
- `--snapshot`: run snapshot comparison
- `--update-snapshots`: re-record all snapshots
- `--e2e`: run `e2e` blocks via Playwright full-browser
- `--a11y`: run axe-core on all test states (Block 06), exits non-zero on violation
- `--filter <pattern>`: run only tests whose name matches
- `--env <name>`: env section to use (default: `[env.test]` if present)
- `--watch`: re-run on file changes

---

## Tests

### Unit tests — `origami-test/src/tests.rs`

- Fixture validation: JSON-compatible check, `Response<T>` shape matching (CLT703)
- `testId` conflict detection in static templates (CLT706)
- `navigate` / `api_mock` in wrong block type (CLT704, CLT705)
- Playwright codegen: verify generated spec structure for a known `test` block and `e2e` block

### Integration fixtures — `fixtures/testing/`

```
fixtures/testing/
  ├── component-visual/     ← test block, no steps (visual only)
  ├── component-steps/      ← test block with steps → Playwright component test
  ├── page-fixture/         ← page test block with Response<T> fixture
  ├── e2e/                  ← e2e block with api_mock and navigate
  ├── fixture-mismatch/     ← CLT703: fixture shape doesn't match Response<T>
  └── duplicate-testid/     ← CLT706: two elements with same testId
```
