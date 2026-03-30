# Porting Guide — Clutter → Origami

Porting the compiler pipeline from `../clutter/` into the empty Origami stubs.
Do this yourself: it requires simultaneous multi-repo context and judgment calls
the small model cannot reliably make. Follow the steps in order.

---

## Overview

| Clutter | Origami |
|---------|---------|
| `clutter-*` crates | `origami-*` crates |
| `.clutter` files | `.ori` files |
| `clutter_*` identifiers | `origami_*` identifiers |
| `Diagnostic` trait + plain structs | `thiserror` derive macros |
| `FileNode.components: Vec<ComponentDef>` | `FileNode.definitions: Vec<Definition>` |
| `component Name(...)` only | `component`, `page`, `layout` keywords |

The pipeline is identical: Lexer → Parser → Analyzer → Codegen.
The biggest adaptation is the error system. Everything else is mechanical rename + one
structural extension for `page`/`layout`.

---

## Step 0 — Before you start

Verify the workspace compiles clean as stubs:

```
cargo build
cargo clippy -- -D warnings
```

Both must pass before you touch anything. If they don't, fix the stub issues first.

---

## Step 1 — origami-runtime

**What goes here:** Position, Token, AST nodes, error types, DesignTokens.

### 1a. Position

Copy verbatim from `clutter-runtime/src/position.rs`. No changes needed.

```rust
// position.rs
pub struct Position { pub line: usize, pub col: usize }
```

### 1b. Token

Copy `clutter-runtime/src/tokens.rs`. Then add two variants for Origami keywords:

```rust
// After ComponentOpen/ComponentClose:
PageKeyword,     // "page"
LayoutKeyword,   // "layout"
```

All other `TokenKind` variants are identical.
The `Token` struct (kind, value: String, pos: Position) copies verbatim.

### 1c. Error types — the main adaptation

Clutter defines `LexError`, `ParseError`, `AnalyzerError`, `AnalyzerWarning` as plain
structs with a `Diagnostic` trait. Origami uses `thiserror`.

**Pattern to follow for each error category:**

```rust
// origami-runtime/src/errors.rs

use thiserror::Error;
use crate::Position;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("[{code}] {message} at {pos}")]
    UnexpectedChar { code: &'static str, message: String, pos: Position },

    #[error("[{code}] {message} at {pos}")]
    UnterminatedString { code: &'static str, message: String, pos: Position },
}

#[derive(Debug, Error)]
pub enum ParseError { /* same pattern */ }

#[derive(Debug, Error)]
pub enum AnalyzerError { /* same pattern */ }

#[derive(Debug, Error)]
pub enum AnalyzerWarning { /* same pattern */ }
```

Map each code from `clutter-runtime/src/codes.rs`:

| Clutter code | Variant name | Error type |
|---|---|---|
| L001 | `UnexpectedChar` | LexError |
| L002 | `UnterminatedString` | LexError |
| P001 | `UnexpectedToken` | ParseError |
| P002 | `MissingClosingTag` | ParseError |
| P003 | `InvalidPropValue` | ParseError |
| CLT101 | `UnknownComponent` | AnalyzerError |
| CLT102 | `UnknownProp` | AnalyzerError |
| CLT103 | `InvalidTokenValue` | AnalyzerError |
| CLT104 | `InvalidEnumValue` | AnalyzerError |
| CLT105 | `UndeclaredIdentifier` | AnalyzerError |
| CLT106 | `DuplicateComponent` | AnalyzerError |
| CLT107 | `MissingRequiredProp` | AnalyzerError |
| W001 | `UnusedProp` | AnalyzerWarning |
| W002 | `ExpressionInTokenProp` | AnalyzerWarning |

Add Origami-specific codes later (CLT201+ for router, CLT301+ for data layer, etc.)
when those blocks are implemented. Do NOT add them during this porting step.

**Note on Display:** The `#[error("...")]` attribute replaces Clutter's `message()`
method. The `miette` integration in `origami-cli` will consume these via `std::error::Error`.

### 1d. AST nodes

Copy `clutter-runtime/src/ast.rs`. Two changes required:

**Change 1 — FileNode:**

```rust
// Clutter:
pub struct FileNode {
    pub components: Vec<ComponentDef>,
}

// Origami:
pub struct FileNode {
    pub definitions: Vec<Definition>,
}

pub enum Definition {
    Component(ComponentDef),
    Page(PageDef),
    Layout(LayoutDef),
}

// PageDef and LayoutDef have the same shape as ComponentDef for now:
pub struct PageDef {
    pub name: String,
    pub props_raw: String,
    pub logic_block: Option<String>,
    pub template: Vec<Node>,
}
pub struct LayoutDef {
    pub name: String,
    pub props_raw: String,
    pub logic_block: Option<String>,
    pub template: Vec<Node>,
}
```

**Change 2 — UnsafeNode:**
No change needed. `UnsafeNode { reason: String, children: Vec<Node> }` copies verbatim.

All other node types (PropValue, EventBinding, PropNode, ComponentNode, TextNode,
ExpressionNode, IfNode, EachNode, Node) copy verbatim.

### 1e. DesignTokens

Clutter puts `DesignTokens` in `clutter-runtime` and depends on `serde`/`serde_json` there.
In Origami, `serde` and `serde_json` are already workspace deps available everywhere.

Copy `clutter-runtime/src/design_tokens.rs` into `origami-runtime/src/design_tokens.rs`
verbatim. Add `serde = { workspace = true }` and `serde_json = { workspace = true }` to
`origami-runtime/Cargo.toml`.

### 1f. lib.rs

```rust
// origami-runtime/src/lib.rs
pub mod ast;
pub mod design_tokens;
pub mod errors;
pub mod position;
pub mod tokens;

pub use ast::*;
pub use design_tokens::*;
pub use errors::*;
pub use position::Position;
pub use tokens::{Token, TokenKind};

#[cfg(test)]
mod tests;
```

### 1g. Verify

```
cargo build -p origami-runtime
cargo test -p origami-runtime
cargo clippy -p origami-runtime -- -D warnings
```

---

## Step 2 — origami-lexer

### 2a. component_blocks.rs

Copy `clutter-lexer/src/component_blocks.rs`. Extend `find_components()` to also
recognise `page` and `layout` keywords:

The scan currently looks for `component Name(`. Extend it to match:
- `component Name(` → `Definition::Component`
- `page Name(` → `Definition::Page`
- `layout Name(` → `Definition::Layout`

Return a `BlockKind` enum alongside each `ComponentBlock`:

```rust
pub enum BlockKind { Component, Page, Layout }
pub struct FoundBlock { pub kind: BlockKind, pub block: ComponentBlock }
```

Rename `find_components()` to `find_blocks()` returning `Vec<FoundBlock>`.

Everything else in this file (brace counting, `find_section_separator`,
`parse_component_header`) copies verbatim — just update call sites.

### 2b. template_lexer.rs

Copy `clutter-lexer/src/template_lexer.rs` verbatim. No changes needed —
it operates on the template section only and has no keyword awareness.

### 2c. lib.rs

Copy `clutter-lexer/src/lib.rs`. Renames:
- `clutter_runtime` → `origami_runtime`
- `LexError` comes from `origami_runtime::errors::LexError`
- `find_components()` → `find_blocks()`, handle the three `BlockKind` variants
- Signature stays: `pub fn tokenize(input: &str) -> (Vec<Token>, Vec<LexError>)`

The returned `Vec<Token>` is the same structure — `TokenKind::PageKeyword` and
`TokenKind::LayoutKeyword` are now emitted for `page`/`layout` block headers.

### 2d. Cargo.toml

```toml
[dependencies]
origami-runtime = { workspace = true }
```

### 2e. Verify

```
cargo build -p origami-lexer
cargo test -p origami-lexer
cargo clippy -p origami-lexer -- -D warnings
```

---

## Step 3 — origami-parser

### 3a. lib.rs

Copy `clutter-parser/src/lib.rs`. Changes:

**Rename:**
- All `clutter_*` → `origami_*`
- `ParseError` variants come from `origami_runtime::errors`

**FileNode construction — the main change:**

Clutter builds `FileNode { components: vec![...] }`.
Origami must build `FileNode { definitions: vec![...] }`.

The top-level loop that calls `parse_component()` must also call `parse_page()` and
`parse_layout()` based on which keyword was seen. The three parsers have identical
structure — only the Definition variant wrapping the result differs:

```rust
fn parse_definition(&mut self, kind: BlockKind) -> Result<Definition, ParseError> {
    let def = self.parse_component_body()?;  // reuse same body parser
    Ok(match kind {
        BlockKind::Component => Definition::Component(ComponentDef { .. }),
        BlockKind::Page      => Definition::Page(PageDef { .. }),
        BlockKind::Layout    => Definition::Layout(LayoutDef { .. }),
    })
}
```

All other methods (`parse_template`, `parse_node`, `parse_if`, `parse_each`,
`parse_unsafe_call`, `parse_props`) copy verbatim — they operate on tokens and
return Node variants which are unchanged.

**typed-arena:** `origami-parser` already has `typed-arena` as a workspace dep.
Clutter did not use it. You have two options:
- **Option A (recommended):** Ignore it for now. Port the parser using the same
  `Vec`-based allocation as Clutter. The arena is there for future optimisation.
- **Option B:** Use `Arena<Node>` for template node storage. This is a larger
  adaptation — defer to M2 if you want it.

### 3b. Signature

```rust
pub fn parse_file(tokens: &[Token]) -> (FileNode, Vec<ParseError>)
```

Identical to Clutter. The `(result, errors)` pair pattern is the same.

### 3c. Verify

```
cargo build -p origami-parser
cargo test -p origami-parser
cargo clippy -p origami-parser -- -D warnings
```

---

## Step 4 — origami-analyzer

### 4a. vocabulary.rs

Copy `clutter-analyzer/src/vocabulary.rs`. Vocabulary (Column, Row, Box, Text,
Button, Input, Select) is identical. No changes needed.

The `VocabularyMap`, `ComponentSchema`, `PropValidation`, and `TokenCategory` types
copy verbatim.

### 4b. lib.rs

Copy `clutter-analyzer/src/lib.rs`. Changes:

**Rename:**
- All `clutter_*` → `origami_*`
- `AnalyzerError`/`AnalyzerWarning` come from `origami_runtime::errors`

**FileNode change:**

Clutter iterates `file.components`. Origami must iterate `file.definitions`:

```rust
for definition in &file.definitions {
    let (name, template) = match definition {
        Definition::Component(c) => (&c.name, &c.template),
        Definition::Page(p)      => (&p.name, &p.template),
        Definition::Layout(l)    => (&l.name, &l.template),
    };
    // existing analysis logic unchanged
}
```

All other functions (`extract_identifiers`, `is_simple_identifier`, `is_member_access`,
the template walker) copy verbatim.

**Signature:**

```rust
pub fn analyze_file(
    file: &FileNode,
    design_tokens: &DesignTokens,
) -> (Vec<AnalyzerError>, Vec<AnalyzerWarning>)
```

Identical to Clutter.

### 4c. Verify

```
cargo build -p origami-analyzer
cargo test -p origami-analyzer
cargo clippy -p origami-analyzer -- -D warnings
```

---

## Step 5 — origami-codegen

### 5a. css.rs

Copy `clutter-codegen/src/css.rs` verbatim. The CSS generation is token-based
and has no keyword dependency.

### 5b. vue.rs

Copy `clutter-codegen/src/vue.rs`. Changes:

**Rename:**
- All `clutter_*` → `origami_*`

**FileNode change:**

Clutter iterates `file.components`. Origami iterates `file.definitions`:

```rust
for definition in &file.definitions {
    let (name, logic_block, template) = match definition {
        Definition::Component(c) => (&c.name, &c.logic_block, &c.template),
        Definition::Page(p)      => (&p.name, &p.logic_block, &p.template),
        Definition::Layout(l)    => (&l.name, &l.logic_block, &l.template),
    };
    files.push(generate_sfc(name, logic_block, template));
}
```

All SFC generation logic (BUILTIN map, generate_template, generate_props,
generate_events, generate_if, generate_each, generate_unsafe, generate_select)
copies verbatim.

The generated `.vue` file names follow the same convention — use `definition.name()`.

### 5c. lib.rs

```rust
pub mod css;
pub mod vue;

pub use css::generate_css;
pub use vue::{GeneratedFile, generate_vue};

#[cfg(test)]
mod tests;
```

### 5d. Cargo.toml

```toml
[dependencies]
origami-runtime  = { workspace = true }
origami-analyzer = { workspace = true }
serde_json = { workspace = true }
```

### 5e. Verify

```
cargo build -p origami-codegen
cargo test -p origami-codegen
cargo clippy -p origami-codegen -- -D warnings
```

---

## Step 6 — origami-cli

### 6a. lib.rs

Copy `clutter-cli/src/lib.rs`. Changes:

**Rename:**
- `find_clutter_files()` → `find_ori_files()`
- Extension filter: `.clutter` → `.ori`
- All `clutter_*` → `origami_*`

**Error rendering:**

Clutter renders errors by iterating `Vec<impl Diagnostic>` and calling `.message()`,
`.code()`, `.pos()`. Origami errors are `thiserror` enums — render them via their
`Display` impl (the `#[error("...")]` string) and `miette` for CLI output:

```rust
// Instead of:
eprintln!("[{}] {} at {}:{}", e.code(), e.message(), e.pos().line, e.pos().col);

// Use:
eprintln!("{}", e);   // thiserror Display
// Or wrap in miette::Report for fancy output — see existing CLI skeleton
```

Check `origami-cli/src/main.rs` (from M0 skeleton) — it may already have a
`miette`-based render function to wire into.

**compile() function shape:**

```rust
pub fn compile(path: &Path) -> Result<Vec<GeneratedFile>, Vec<Box<dyn std::error::Error>>>
```

Or keep the `(result, errors)` tuple pattern from Clutter — either works as long as
the CLI renders all errors before exiting non-zero.

### 6b. Cargo.toml

```toml
[dependencies]
origami-runtime  = { workspace = true }
origami-lexer    = { workspace = true }
origami-parser   = { workspace = true }
origami-analyzer = { workspace = true }
origami-codegen  = { workspace = true }
clap    = { workspace = true }
miette  = { workspace = true }
```

### 6c. Verify

```
cargo build -p origami-cli
cargo clippy -p origami-cli -- -D warnings
```

---

## Step 7 — Full pipeline smoke test

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
rustfmt --check $(find crates -name "*.rs")
```

Then run the CLI on a minimal `.ori` fixture:

```
echo 'component Hello() { --- <Text value="hello" /> }' > /tmp/test.ori
cargo run -p origami-cli -- build /tmp/test.ori
```

Expected: one `.vue` file written, exit zero.

---

## What is NOT ported in this step

The following are Origami additions that do not exist in Clutter.
Do NOT attempt to port or implement them during this step:

| Feature | Block | When |
|---------|-------|------|
| Route table (page/layout validation) | Block 01 | M1 |
| `endpoints.toml` + DataManifest | Block 02 | M1 |
| File watcher + Nuxt process | Block 03 | M2 |
| `t()` / i18n | Block 04 | M3 |
| `test`/`e2e` blocks | Block 05 | M3 |
| A11y checks | Block 06 | M3 |
| `origami init`, release pipeline | Block 07 | M4 |

After this porting step, the pipeline handles `component`, `page`, and `layout`
definitions and emits `.vue` files. That is sufficient to begin Block 01.

---

## Common mistakes to avoid

1. **Do not add CLT2xx–CLT7xx error variants yet.** Add them only when implementing
   the block that defines them.

2. **Do not use `unwrap()` or `expect()` outside `#[cfg(test)]`.** Every `.unwrap()`
   in Clutter that handles an error case must become an explicit `?` propagation or
   a match arm.

3. **Clippy is errors.** Run `cargo clippy -- -D warnings` after each step.
   Do not accumulate warnings.

4. **rustfmt before commit.** Run `rustfmt crates/origami-*/src/**/*.rs` or
   `cargo fmt --all` after each step.

5. **Keep commits atomic.** One commit per crate ported. Message format:
   `port origami-runtime from clutter`
