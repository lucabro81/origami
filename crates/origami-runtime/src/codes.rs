//! Machine-readable error and warning codes for the Origami compiler.
//!
//! Every diagnostic type carries a `code: &'static str` field pointing to one
//! of the constants defined here. Using the constant (rather than a string
//! literal) lets tests assert `error.code == codes::ORI102` instead of
//! checking a fragile substring of the human-readable message.
//!
//! # Code ranges
//!
//! | Range        | Stage        | Description                       |
//! |--------------|--------------|-----------------------------------|
//! | `PP00X`      | Preprocessor | Preprocessing errors              |
//! | `L00X`       | Lexer        | Tokenisation errors               |
//! | `P00X`       | Parser       | Structural / grammar errors       |
//! | `ORI10X`     | Analyzer     | Semantic / design-system errors   |
//! | `W00X`       | Analyzer     | Non-blocking unsafe-usage warnings|

pub struct ErrorCode {
  pub code: &'static str,
  pub message: &'static str
}

// ---------------------------------------------------------------------------
// Preprocessor codes
// ---------------------------------------------------------------------------

pub const PP001: ErrorCode = ErrorCode {
  code: "PP001",
  message: "Missing `----` separator between the logic block and the template"
};

pub const PP002: ErrorCode = ErrorCode {
  code: "PP002",
  message: "Symbol `----` needs its own line"
};

// ---------------------------------------------------------------------------
// Lexer codes
// ---------------------------------------------------------------------------

pub const L001: ErrorCode = ErrorCode {
  code: "L001",
  message: "Unexpected/unrecognised character in the template or inside a tag."
};

// ---------------------------------------------------------------------------
// Parser codes
// ---------------------------------------------------------------------------

pub const P001: ErrorCode = ErrorCode {
  code: "P001",
  message: "Structural mismatch: expected token X, found Y (e.g. missing prop value, unexpected tag nesting)"
};

pub const P002: ErrorCode = ErrorCode {
  code: "P002",
  message: "`<else>` without a matching `<if>`."
};

pub const P003: ErrorCode = ErrorCode {
  code: "P003",
  message: "`<unsafe>` tag is missing the `reason` attribute or its value is empty."
};

// ---------------------------------------------------------------------------
// Analyzer codes
// ---------------------------------------------------------------------------

pub const ORI101: ErrorCode = ErrorCode {
  code: "ORI101",
  message: "Unknown component name — not in the closed vocabulary."
};

pub const ORI102: ErrorCode = ErrorCode {
  code: "ORI102",
  message: "Invalid prop value for a token-checked prop — value not in `tokens.json`."
};

pub const ORI103: ErrorCode = ErrorCode {
  code: "ORI103",
  message: "Unknown prop name for a known component."
};

pub const ORI104: ErrorCode = ErrorCode {
  code: "ORI104",
  message: "Expression references an identifier not declared in the logic block."
};

pub const ORI105: ErrorCode = ErrorCode {
  code: "ORI105",
  message: "`<unsafe>` block with a missing or empty `reason` attribute."
};

pub const ORI106: ErrorCode = ErrorCode {
  code: "ORI106",
  message: "`unsafe('value', 'reason')` prop value with an empty reason string."
};

pub const ORI107: ErrorCode = ErrorCode {
  code: "ORI107",
  message: "Complex `{}` expression used in the template outside an `<unsafe>` block."
};

// ---------------------------------------------------------------------------
// Analyzer warning codes
// ---------------------------------------------------------------------------


pub const W001: ErrorCode = ErrorCode {
  code: "W001",
  message: "Well-formed `<unsafe reason=\"...\">` block — compilation proceeds, but the escape hatch is flagged for visibility"
};

pub const W002: ErrorCode = ErrorCode {
  code: "W002",
  message: "Well-formed `unsafe('value', 'reason')` prop value — compilation proceeds, but the escape hatch is flagged for visibility"
};
