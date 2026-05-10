use std::collections::BTreeMap;

use crate::{DesignTokens, TokenCategory, design_tokens::Typography};

const CORRECT_JSON: &str = r##"{
    "spacing": ["xs", "sm", "md", "lg", "xl", "xxl"],
    "colors": ["primary", "secondary", "danger", "surface", "background"],
    "typography": {
        "sizes": ["xs", "sm", "base", "lg", "xl", "xxl"],
        "weights": ["normal", "medium", "semibold", "bold"]
    },
    "radii": ["none", "sm", "md", "lg", "full"],
    "shadows": ["sm", "md", "lg"],
    "button_variants": ["primary", "secondary", "outline", "ghost", "danger"],
    "button_sizes": ["sm", "md", "lg"],
    "variables": {
        "--spacing-md": "1rem",
        "--color-primary": "#3b82f6"
    }
}"##;

fn test_tokens() -> DesignTokens {
    DesignTokens::deserialize_json(CORRECT_JSON).unwrap()
}

#[test]
fn deserialize_correct_design_token_object() {
    let design_tokens = DesignTokens::deserialize_json(CORRECT_JSON).unwrap();
    assert_eq!(
        design_tokens,
        DesignTokens {
            spacing: vec![
                "xs".to_string(),
                "sm".to_string(),
                "md".to_string(),
                "lg".to_string(),
                "xl".to_string(),
                "xxl".to_string()
            ],
            colors: vec![
                "primary".to_string(),
                "secondary".to_string(),
                "danger".to_string(),
                "surface".to_string(),
                "background".to_string()
            ],
            typography: Typography {
                sizes: vec![
                    "xs".to_string(),
                    "sm".to_string(),
                    "base".to_string(),
                    "lg".to_string(),
                    "xl".to_string(),
                    "xxl".to_string()
                ],
                weights: vec![
                    "normal".to_string(),
                    "medium".to_string(),
                    "semibold".to_string(),
                    "bold".to_string()
                ]
            },
            radii: vec![
                "none".to_string(),
                "sm".to_string(),
                "md".to_string(),
                "lg".to_string(),
                "full".to_string()
            ],
            shadows: vec!["sm".to_string(), "md".to_string(), "lg".to_string()],
            button_variants: vec!["primary".to_string(), "secondary".to_string(), "outline".to_string(), "ghost".to_string(), "danger".to_string()],
            button_sizes: vec!["sm".to_string(), "md".to_string(), "lg".to_string()],
            variables: Some(BTreeMap::from([
                ("--spacing-md".to_string(), "1rem".to_string()),
                ("--color-primary".to_string(), "#3b82f6".to_string()),
            ])),
        }
    );
}

#[test]
fn deserialize_invalid_json_missing_required_field() {
    let result = DesignTokens::deserialize_json(
        r#"{ "spacing": ["xs", "sm"] }"#
    );
    assert!(result.is_err());
}

#[test]
fn design_tokens_parses_valid_json() {
    let t = test_tokens();
    assert_eq!(
        t.valid_values(TokenCategory::Spacing), 
        vec![
          "xs".to_string(), 
          "sm".to_string(),
          "md".to_string(),
          "lg".to_string(),
          "xl".to_string(),
          "xxl".to_string()
        ]
    );
    assert_eq!(
      t.valid_values(TokenCategory::Color), 
        vec![
            "primary".to_string(),
            "secondary".to_string(),
            "danger".to_string(),
            "surface".to_string(),
            "background".to_string()
        ]);
    assert_eq!(
      t.valid_values(TokenCategory::FontSize), 
        vec![
            "xs".to_string(),
            "sm".to_string(),
            "base".to_string(),
            "lg".to_string(),
            "xl".to_string(),
            "xxl".to_string()
        ]);
    assert_eq!(
      t.valid_values(TokenCategory::FontWeight), 
        vec!["normal".to_string(), "medium".to_string(), "semibold".to_string(), "bold".to_string()]);
    assert_eq!(
      t.valid_values(TokenCategory::Radius), 
        vec![
            "none".to_string(),
            "sm".to_string(),
            "md".to_string(),
            "lg".to_string(),
            "full".to_string()
        ]);
    assert_eq!(
      t.valid_values(TokenCategory::Shadow), 
        vec!["sm".to_string(), "md".to_string(), "lg".to_string()]);
}

#[test]
fn design_tokens_spacing_accessor() {
    assert_eq!(
        test_tokens().spacing(),
        &["xs", "sm", "md", "lg", "xl", "xxl"]
    );
}

#[test]
fn design_tokens_colors_accessor() {
    assert_eq!(
        test_tokens().colors(),
        &["primary", "secondary", "danger", "surface", "background"]
    );
}

#[test]
fn design_tokens_font_sizes_accessor() {
    assert_eq!(
        test_tokens().font_sizes(),
        &["xs", "sm", "base", "lg", "xl", "xxl"]
    );
}

#[test]
fn design_tokens_font_weights_accessor() {
    assert_eq!(
        test_tokens().font_weights(),
        &["normal", "medium", "semibold", "bold"]
    );
}

#[test]
fn design_tokens_radii_accessor() {
    assert_eq!(test_tokens().radii(), &["none", "sm", "md", "lg", "full"]);
}

#[test]
fn design_tokens_shadows_accessor() {
    assert_eq!(test_tokens().shadows(), &["sm", "md", "lg"]);
}
