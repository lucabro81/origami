# Block 06 — Accessibility

Implements the a11y module: compile-time structural checks (missing labels, contrast violations), the `origami-a11y` crate (color contrast from `tokens.json`), and axe-core wiring into the visual preview app (Block 05).

Part of M3. Opt-in: activates when `"a11y"` is listed in `modules` in `origami.toml`. When active, violations are compile **errors**, not warnings.

---

## Checklist — done when all of these are true

- [ ] Structural a11y violations caught at compile time with `miette` messages and source spans
- [ ] Color contrast computed from `tokens.json` token values, validated against WCAG 2.1 AA (4.5:1)
- [ ] axe-core scan wired into visual preview app (Block 05) — results shown per test state
- [ ] `origami test --a11y` exits non-zero on any axe-core violation
- [ ] Error codes CLT601–CLT604 implemented
- [ ] Unit tests pass: contrast computation, CSS variable resolution, CLT601/CLT602/CLT604
- [ ] Integration fixtures exist: passing, contrast-fail, button-no-label, input-no-label, unsafe-escape
- [ ] `cargo clippy -- -D warnings` clean, `rustfmt` applied

---

## Error codes

| Code | Condition | Phase |
|------|-----------|-------|
| CLT601 | `Button` missing accessible label (no text child and no `aria-label`) | analyzer |
| CLT602 | `Input` missing associated `Label` | analyzer |
| CLT603 | `Image` missing `alt` prop | analyzer (future component) |
| CLT604 | Foreground/background token combination fails 4.5:1 contrast | origami-a11y |

RULE: A11y errors do not stop compilation mid-pipeline. Run the full pipeline, collect all errors (a11y and otherwise), then exit non-zero. Developer sees every problem at once.

RULE: The only way to suppress an a11y compile error is `<unsafe reason="...">`. No configuration flags to downgrade errors to warnings.

---

## Compile-time structural checks (analyzer)

Run on the validated AST. Require no external data.

### CLT601 — Button missing label

A `Button` is considered labelled if:
- It has at least one `Text` child in its template subtree, **or**
- It has an `aria-label` prop

A `Button` with only an icon child and no `aria-label` → CLT601.

### CLT602 — Input missing Label

`Input` and `Label` association is checked by matching:
- `for` prop on `Label` = `id` prop on `Input`

If either prop is absent, or values do not match → CLT602.

Limitation: dynamic `for`/`id` values (expressions) are not statically verifiable. Treated as correct by compiler — caught by axe-core in the preview app if wrong.

---

## Color contrast check — CLT604

Applies whenever a component declares a statically resolvable foreground + background token combination.

```
<Text color="secondary" />
  inside
<Box bg="surface">
```

The compiler resolves `color.secondary` and `color.surface` from `tokens.json`, computes the contrast ratio, fails with CLT604 if < 4.5:1.

CLT604 error message must include: both token names, their resolved hex values, computed ratio, required ratio.

### Scope

- Only token values defined as hex or RGB in `tokens.json` can be statically resolved.
- CSS custom properties referencing other variables (`var(--other-token)`) require a second-pass resolution — supported if the reference chain terminates at a concrete value within `tokens.json`.
- The check applies to `Text` foreground color against the nearest ancestor `Box`/`Row`/`Column` background. Traverse the template tree upward.
- Dynamic color values (prop expressions) are skipped — caught by axe-core.

---

## origami-a11y crate

New crate. Input: `AnalyzedWorkspace` + parsed `tokens.json`. Output: `Vec<A11yError>` for CLI rendering.

### Input types

```rust
AnalyzedWorkspace {
    files: Vec<AnalyzedFile {
        template: TemplateNode,
    }>
}

TokensJson {
    colors: HashMap<String, ColorValue>,
    variables: HashMap<String, String>,   // CSS custom property definitions
}
```

### Color contrast computation

WCAG relative luminance formula:

```
luminance(R, G, B):
  for each channel c in [R, G, B]:
    c_sRGB = c / 255
    if c_sRGB <= 0.04045: c_linear = c_sRGB / 12.92
    else:                 c_linear = ((c_sRGB + 0.055) / 1.055) ^ 2.4
  L = 0.2126 * R_linear + 0.7152 * G_linear + 0.0722 * B_linear

contrast_ratio(L1, L2):
  lighter = max(L1, L2)
  darker  = min(L1, L2)
  ratio   = (lighter + 0.05) / (darker + 0.05)
```

WCAG 2.1 AA thresholds:
- Normal text: ratio ≥ 4.5:1
- Large text (≥ 18pt or ≥ 14pt bold): ratio ≥ 3:1

For this version: apply 4.5:1 uniformly. Origami does not yet distinguish text size tiers in the token system. When `size` tokens become part of the type system, selective thresholds can be added.

---

## Runtime checks — axe-core in the preview app

The visual preview app (Block 05) runs axe-core on every rendered test state after mount. Results displayed inline below the component render:

```
Preview panel:
  [component render]
  ─────────────────
  A11y
  ✓ No violations
  — or —
  ✗ 2 violations
    · Button: missing label
    · Contrast: 3.1:1
```

axe-core runs client-side. Results are advisory in the preview UI.

`origami test --a11y`: runs axe-core on all test states, exits non-zero on any violation.

axe-core catches what the compiler cannot: focus order, ARIA attribute correctness on dynamic content, contrast on dynamically set colors, missing alt on dynamic image sources.

---

## The escape hatch

```
<unsafe reason="Icon-only button, aria-label provided by parent context">
  <Button @click={onClose} />
</unsafe>
```

`origami unsafe-report` (Block 07) lists all `<unsafe>` usages, including those suppressing a11y errors. Technical debt is auditable.

---

## CLI contribution

`origami check`: runs all compile-time a11y checks (CLT601–CLT604).

`origami test --a11y`: runs axe-core on all test states in the preview app, exits non-zero on any violation.

---

## Tests

### Unit tests — `origami-a11y/src/tests.rs`

- Contrast computation: known hex pairs with known ratios (verify against WCAG reference values)
- CSS variable resolution: token that references another token resolves to the final concrete value
- CLT604: token pair below 4.5:1 → error with correct ratio in message
- CLT601: `Button` with no text child and no `aria-label`
- CLT602: `Input` with no matching `Label`

### Integration fixtures — `fixtures/a11y/`

```
fixtures/a11y/
  ├── passing/           ← all checks pass, all token pairs have sufficient contrast
  ├── contrast-fail/     ← CLT604: a color token pair below 4.5:1
  ├── button-no-label/   ← CLT601
  ├── input-no-label/    ← CLT602
  └── unsafe-escape/     ← CLT601 suppressed with <unsafe reason="...">
```
