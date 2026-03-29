# Block 06 — Accessibility

**Version:** 0.1 — draft

## Contents

- [Scope](#scope)
- [Expected output](#expected-output)
- [Design principles](#design-principles)
- [Compile-time checks](#compile-time-checks)
  - [Structural checks](#structural-checks)
  - [Color contrast checks](#color-contrast-checks)
- [origami-a11y](#origami-a11y)
  - [Input](#input)
  - [Color contrast computation](#color-contrast-computation)
- [Runtime checks — axe-core in the preview app](#runtime-checks--axe-core-in-the-preview-app)
- [The escape hatch](#the-escape-hatch)
- [Error codes](#error-codes)
- [CLI contribution](#cli-contribution)
- [Tests](#tests)

---

## Scope

This block implements Origami's accessibility module. It includes compile-time structural checks (missing labels, missing slots, contrast violations), the `origami-a11y` crate that computes color contrast from `tokens.json`, and the wiring of axe-core into the visual preview app (built in Block 05).

Part of M3 (Feature-complete framework). The a11y module is opt-in: it activates when `"a11y"` is listed in `modules` in `origami.toml`. When active, violations are compile errors — not warnings.

---

## Expected output

- Structural a11y violations in `.ori` templates caught at compile time with `miette` messages
- Color contrast ratios computed from `tokens.json` values and validated against WCAG 2.1 AA (4.5:1)
- axe-core scan wired into the visual preview app — results shown per test state
- `origami test --a11y` exits non-zero if any axe-core violation is found
- Error codes CLT601–CLT604 implemented

---

## Design principles

**Errors, not warnings.** When the a11y module is enabled, violations are compile errors. There is no "a11y warning level" configuration. The escape hatch is `<unsafe reason="...">`, consistent with the rest of the system. The "warning that nobody fixes" failure mode is eliminated by design.

**Closed vocabulary makes static analysis tractable.** Because the component vocabulary is fixed, accessibility rules can be checked at compile time with certainty. An arbitrary HTML tree cannot be statically audited for a11y — a closed set of components can. This is a structural advantage of Origami's design.

**Compile-time and runtime checks are complementary.** The compiler catches structural violations (missing labels, wrong nesting) and contrast violations (known at build time from token values). axe-core in the preview app catches everything else (focus management, ARIA attribute correctness, dynamic content) that requires a real rendered DOM.

**Full pipeline before failing.** A11y errors do not stop compilation mid-pipeline. The compiler runs the full pipeline, collects all errors (a11y and otherwise), then exits non-zero. The developer sees every problem at once.

---

## Compile-time checks

### Structural checks

These checks run in the analyzer on the validated AST. They require no external data — only the template structure.

| Code | Rule |
|------|------|
| CLT601 | `Button` has no accessible label: no text child and no `aria-label` prop |
| CLT602 | `Input` has no associated `Label` component in the same template scope |
| CLT603 | `Image` (future component) missing `alt` prop |

**CLT601 detail:** a `Button` is considered labelled if it has at least one `Text` child in its template subtree, or if it has an `aria-label` prop. A `Button` with only an icon child and no `aria-label` is CLT601.

**CLT602 detail:** `Input` and `Label` association is checked by matching a `for` prop on `Label` with the `id` prop on `Input`. If either prop is absent, or if the values do not match, it is CLT602. This check is limited to static prop values — dynamic `for`/`id` values (expressions) are not statically verifiable and are treated as correct by the compiler. A dynamic value that is actually wrong will be caught by axe-core in the preview app.

### Color contrast checks

**CLT604:** a foreground/background color token combination fails the WCAG 4.5:1 contrast ratio for normal text.

This check is unique: it requires knowing the actual CSS color values behind the token names. It runs in `origami-a11y` (not the core analyzer) after `tokens.json` is parsed.

The check applies whenever a component declares both a foreground and a background token that can be statically resolved. The most common case:

```
<Text color="secondary" />           ← foreground token
  inside
<Box bg="surface">                   ← background token
```

The compiler resolves `color.secondary` and `color.surface` from `tokens.json`, computes the contrast ratio, and fails with CLT604 if it falls below 4.5:1.

**Scope of the check:**
- Only token values defined as hex or RGB in `tokens.json` can be statically resolved. CSS custom properties that reference other variables (`var(--other-token)`) require a second-pass resolution — supported if the reference chain terminates at a concrete value within `tokens.json`.
- The check applies to `Text` foreground color against any ancestor `Box`/`Row`/`Column` background. The compiler traverses the template tree to find the nearest background ancestor.
- Dynamic color values (prop expressions) are not statically verifiable and are skipped. They are candidates for axe-core in the preview app.

---

## origami-a11y

New crate. Receives the analyzed AST and the parsed `tokens.json`, runs structural and contrast checks, and returns typed `A11yError` values to the CLI for `miette` rendering.

### Input

```
AnalyzedWorkspace
  └── files: Vec<AnalyzedFile>
        └── template: TemplateNode   ← validated AST

TokensJson
  └── colors: HashMap<String, ColorValue>
  └── variables: HashMap<String, String>   ← CSS custom property definitions
```

### Color contrast computation

The contrast ratio between two colors is computed using the WCAG relative luminance formula:

```
luminance(R, G, B):
  for each channel c in [R, G, B]:
    c_sRGB = c / 255
    if c_sRGB <= 0.04045:
      c_linear = c_sRGB / 12.92
    else:
      c_linear = ((c_sRGB + 0.055) / 1.055) ^ 2.4
  L = 0.2126 * R_linear + 0.7152 * G_linear + 0.0722 * B_linear

contrast_ratio(L1, L2):
  lighter = max(L1, L2)
  darker  = min(L1, L2)
  ratio   = (lighter + 0.05) / (darker + 0.05)
```

WCAG 2.1 AA requires:
- Normal text: ratio ≥ 4.5:1
- Large text (≥ 18pt or ≥ 14pt bold): ratio ≥ 3:1

For this version, the 4.5:1 threshold is applied uniformly — Origami does not yet distinguish text size tiers in the token system. When `size` tokens become part of the type system, the large text threshold can be applied selectively.

CLT604 error message includes: the two token names, their resolved hex values, the computed ratio, and the required ratio.

---

## Runtime checks — axe-core in the preview app

The visual preview app (Block 05) runs axe-core on every rendered test state after mount. Results are displayed inline in the preview panel, below the component render.

```
Preview panel:
  ┌─────────────────────────────┐
  │  [component render]         │
  ├─────────────────────────────┤
  │  A11y                       │
  │  ✓ No violations            │
  │  — or —                     │
  │  ✗ 2 violations             │
  │    · Button: missing label  │
  │    · Contrast: 3.1:1        │
  └─────────────────────────────┘
```

axe-core runs client-side in the preview app — no server involvement. The results are advisory in the preview UI. They become blocking only when `origami test --a11y` is run: that command exits non-zero if any axe-core violation is found across all test states.

axe-core catches what the compiler cannot:
- Focus order and keyboard trap detection
- ARIA attribute correctness on dynamic content
- Contrast violations on dynamically set colors
- Missing alt text on dynamically set image sources

---

## The escape hatch

`<unsafe reason="...">` is the only way to suppress an a11y compile error. It is consistent with the rest of the system and deliberately visible:

```
<unsafe reason="Icon-only button, aria-label provided by parent context">
  <Button @click={onClose} />
</unsafe>
```

`origami unsafe-report` (Block 07) lists all `<unsafe>` usages, including those suppressing a11y errors. This makes the technical debt auditable.

---

## Error codes

| Code | Condition | Phase |
|------|-----------|-------|
| CLT601 | `Button` missing accessible label (no text child, no `aria-label`) | analyzer |
| CLT602 | `Input` missing associated `Label` | analyzer |
| CLT603 | `Image` missing `alt` prop | analyzer (future component) |
| CLT604 | Foreground/background token combination fails 4.5:1 contrast | origami-a11y |

---

## CLI contribution

With this block, `origami check` additionally runs all compile-time a11y checks (CLT601–CLT604).

`origami test --a11y` runs axe-core on all test states in the preview app and exits non-zero on any violation.

---

## Tests

**Unit tests in `origami-a11y/src/tests.rs`:**
- Contrast computation: known hex pairs with known ratios (verified against WCAG reference values)
- CSS variable resolution: token that references another token resolves to the final concrete value
- CLT604: token pair below 4.5:1 triggers error with correct ratio in message
- CLT601: `Button` with no text child and no `aria-label`
- CLT602: `Input` with no matching `Label`

**Integration fixtures in `fixtures/`:**

```
fixtures/
  └── a11y/
      ├── passing/           ← all checks pass, all tokens have sufficient contrast
      ├── contrast-fail/     ← CLT604: a color token pair below 4.5:1
      ├── button-no-label/   ← CLT601
      ├── input-no-label/    ← CLT602
      └── unsafe-escape/     ← CLT601 suppressed with <unsafe reason="...">
```
