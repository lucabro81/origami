# origami-parser

Turns a `Vec<Token>` (from `origami-lexer`) into an `OriFile` AST.

## AST root

`OriFile { declarations: Vec<Declaration> }` — one declaration per component/page/layout defined in the file. The analyzer enforces the one-per-file constraint; the parser accepts multiples.

## Declarations

| Variant | Syntax |
|---------|--------|
| `Component { name, props, body }` | `component Name(prop: Type) { … }` |
| `Page { name, props, body }` | `page Name(prop: Type) { … }` |
| `Layout { name, body }` | `layout Name { … }` |

`Body { logic_block: String, template: Vec<Node> }` — logic block is an opaque JS/TS string (already extracted by the preprocessor); template is a parsed node tree.

## Template nodes

| Variant | Source |
|---------|--------|
| `Component(ComponentNode)` | `<Name attrs… />` or `<Name attrs…>children</Name>` |
| `Expr(ExpressionNode)` | `{{expr}}` standalone in template |
| `Literal(LiteralNode)` | raw string or number literal as child content |
| `If(IfNode)` | `<if condition={{…}}>…</if><elseIf …>…</elseIf><else>…</else>` |
| `Each(EachNode)` | `<each collection={{…}} as=x indexAs=i>…</each>` |
| `Unsafe(UnsafeNode)` | `<unsafe reason="…">…</unsafe>` |
| `Slot(SlotNode)` | `<slot />` |

## Attribute values

| Variant | Example |
|---------|---------|
| `Literal(Static::String)` | `color="red"` |
| `Literal(Static::NumberInt)` | `size=12` |
| `Literal(Static::NumberFloat)` | `ratio=1.5` |
| `Dynamic(SimpleExpression)` | `value={{book.title}}` |
| `UnsafeValue { value, reason }` | `size={{unsafe(42, "legacy API")}}` |

## Public parsers

Lower-level parsers are exported for use in tests or future crates:

- `node_parser()` — full recursive template node parser
- `node_if_block_parser(node)` / `node_each_block_parser(node)` — control flow, accept a node parser for children
- `node_expr_parser()`, `node_slot_parser()`, `node_unsafe_block_parser()`, `node_literal_static_parser()`
- `declaration_parser()`, `ori_file_parser()`
- `props::prop_parser()`, `props::props_parser()`
- `attrs::attr_parser()`, `attrs::attr_value_parser()`, expression sub-parsers

## Usage

```rust
use origami_parser::ori_file_parser;
use chumsky::Parser;

let ast = ori_file_parser().parse(&tokens).into_result()?;
```