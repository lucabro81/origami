# origami-lexer

Turns `.ori` source text into a flat `Vec<Token>` ready for the parser.

The pipeline is two stages:

1. **Preprocessor** (`preprocess`) — scans the raw source and replaces opaque zones with fixed-width placeholders before lexing:
   - Logic block content (between `{` and `----`) → `__LOGIC__`, saved verbatim in `PreprocessResult::logic_blocks`
   - Unsafe block content (between `<unsafe …>` and `</unsafe>`) → `__UNSAFE__`

   An `offset_map` records where each substitution happened and by how many bytes, so that any span produced by the lexer can be mapped back to the original source for accurate diagnostics.

   Returns `Result<PreprocessResult, PreprocessorError>`. Errors:
   - `PP001 SymbolNotFound` — `{` opened but no `----` separator found
   - `PP002 DisplacedToken` — `----` found but not on its own line

2. **Lexer** (`lex`) — runs [Logos](https://github.com/maciejhirsz/logos) over the sanitized string, then splices the saved logic block content back into each `LogicBlock` token. Returns `Result<Vec<Token>, LexError>`. Errors:
   - `L001 UnexpectedChar` — unrecognised character; span is remapped to the original source via `offset_map`

The two stages are intentionally separate so that the CLI can report preprocessor and lexer errors with distinct codes and diagnostics.

## Usage

```rust
use origami_lexer::{preprocess, lex};

let preprocessed = preprocess(source, "my_component.ori")?;
let tokens = lex(preprocessed)?;
```