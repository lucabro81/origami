//! Design token types shared across the compiler pipeline.
//!
//! [`DesignTokens`] is deserialised from `tokens.json` once at the start of
//! compilation and passed read-only to the analyzer (for prop validation) and
//! to the codegen (for CSS class generation).

use std::collections::BTreeMap;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// TokenCategory
// ---------------------------------------------------------------------------

/// Design token category that a prop value may belong to.
///
/// Used by the analyzer's prop validation and by the codegen's CSS generator
/// to look up valid values in [`DesignTokens`].
/// 
#[derive(Debug, Clone, Copy)]
pub enum TokenCategory {
    Spacing,
    Color,
    FontSize,
    FontWeight,
    Radius,
    Shadow,
    ButtonVariant,
    ButtonSize
}

// ---------------------------------------------------------------------------
// DesignTokens
// ---------------------------------------------------------------------------

#[derive(PartialEq, Debug, Deserialize)]
pub struct Typography {
    pub sizes: Vec<String>,
    pub weights: Vec<String>,
}

/// Design system deserialised from `tokens.json`.
///
/// Holds the valid values for every token category. Built once at the start of
/// compilation and passed read-only to the analyzer and the codegen.
///
/// # Expected JSON format
///
/// ```json
/// {
///   "spacing":    ["xs", "sm", "md", "lg", "xl", "xxl"],
///   "colors":     ["primary", "secondary", "danger", "surface", "background"],
///   "typography": { "sizes": [...], "weights": [...] },
///   "radii":      ["none", "sm", "md", "lg", "full"],
///   "shadows":    ["sm", "md", "lg"],
///   "variables": {
///     "--spacing-md": "1rem",
///     "--color-primary": "#3b82f6"
///   }
/// }
/// ```
///
/// The `variables` key is optional. When present, the codegen emits a `:root { }`
/// block at the top of `clutter.css` so that the generated utility classes resolve
/// correctly. Variable names should follow the convention `--{category}-{value}`
/// (e.g. `--spacing-md`) to match what the utility classes reference.
/// 
#[derive(PartialEq, Debug, Deserialize)]
pub struct DesignTokens {
    pub spacing: Vec<String>,
    pub colors: Vec<String>,
    pub typography: Typography,
    pub radii: Vec<String>,
    pub shadows: Vec<String>,
    pub button_variants: Vec<String>,
    pub button_sizes: Vec<String>,
    #[serde(default)]
    pub variables: Option<BTreeMap<String, String>>,
}

impl DesignTokens {
    /// Deserialises a [`DesignTokens`] from a JSON string.
    pub fn deserialize_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Returns the valid values for the given token category.
    ///
    /// Used by the analyzer for prop validation.
    pub fn valid_values(&self, category: TokenCategory) -> &[String] {
        match category {
            TokenCategory::Spacing        => &self.spacing,
            TokenCategory::Color          => &self.colors,
            TokenCategory::FontSize       => &self.typography.sizes,
            TokenCategory::FontWeight     => &self.typography.weights,
            TokenCategory::Radius         => &self.radii,
            TokenCategory::Shadow         => &self.shadows,
            TokenCategory::ButtonVariant  => &self.button_variants,
            TokenCategory::ButtonSize     => &self.button_sizes,
        }
    }

    pub fn spacing(&self) -> &[String] { &self.spacing }
    pub fn colors(&self) -> &[String] { &self.colors }
    pub fn font_sizes(&self) -> &[String] { &self.typography.sizes }
    pub fn font_weights(&self) -> &[String] { &self.typography.weights }
    pub fn radii(&self) -> &[String] { &self.radii }
    pub fn shadows(&self) -> &[String] { &self.shadows }
    pub fn button_variants(&self) -> &[String] { &self.button_variants }
    pub fn button_sizes(&self) -> &[String] { &self.button_sizes }

    /// Returns the CSS variable definitions, if any.
    ///
    /// When `Some`, the codegen emits a `:root { }` block with these declarations
    /// at the top of `clutter.css`.
    pub fn variables(&self) -> Option<&BTreeMap<String, String>> {
        self.variables.as_ref()
    }
}