use proc_macro2::TokenTree;
use syn::{Attribute, LitStr, Token};

pub fn get_rename_rule(attributes: &[Attribute]) -> RenameRule {
    attributes
        .iter()
        .find(|attr| attr.path().is_ident("serde"))
        .map(|attr| {
            let mut value = RenameRule::None;

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename_all") {
                    meta.input.parse::<Token![=]>()?;
                    let str = meta.input.parse::<LitStr>()?;

                    value = RenameRule::from_str(&str.value());
                } else {
                    let mut find_value = false;

                    meta.input
                        .step(|cursor| {
                            let mut rest = *cursor;
                            while let Some((tt, next)) = rest.token_tree() {
                                match &tt {
                                    TokenTree::Ident(ident) if ident == "rename_all" => {
                                        find_value = true;
                                        rest = next
                                    }
                                    TokenTree::Literal(lit) if find_value => {
                                        let lit = lit.to_string();
                                        let str = lit.trim_matches('"');
                                        value = RenameRule::from_str(str);
                                        return Ok(((), next));
                                    }
                                    _ => rest = next,
                                }
                            }

                            Err(cursor.error("nothing found"))
                        })
                        .ok();
                }

                Ok(())
            })
            .ok();

            value
        })
        .unwrap_or_default()
}

/*
Below taken from serde_derive:
<https://github.com/serde-rs/serde/blob/48aa054f5395d2570f51b9d0c85e486f1b3b46ef/serde_derive/src/internals/case.rs#L21>
*/

/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum RenameRule {
    /// Don't apply a default rename rule.
    #[default]
    None,
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    UpperCase,
    /// Rename direct children to "PascalCase" style, as typically used for
    /// enum variants.
    PascalCase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnakeCase,
    /// Rename direct children to "kebab-case" style.
    KebabCase,
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebabCase,
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", RenameRule::LowerCase),
    ("UPPERCASE", RenameRule::UpperCase),
    ("PascalCase", RenameRule::PascalCase),
    ("camelCase", RenameRule::CamelCase),
    ("snake_case", RenameRule::SnakeCase),
    ("SCREAMING_SNAKE_CASE", RenameRule::ScreamingSnakeCase),
    ("kebab-case", RenameRule::KebabCase),
    ("SCREAMING-KEBAB-CASE", RenameRule::ScreamingKebabCase),
];

impl RenameRule {
    pub fn from_str(rename_all_str: &str) -> Self {
        for (name, rule) in RENAME_RULES {
            if rename_all_str == *name {
                return *rule;
            }
        }

        RenameRule::None
    }

    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub fn apply_to_variant(self, variant: &str) -> String {
        match self {
            RenameRule::None | RenameRule::PascalCase => variant.to_owned(),
            RenameRule::LowerCase => variant.to_ascii_lowercase(),
            RenameRule::UpperCase => variant.to_ascii_uppercase(),
            RenameRule::CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            RenameRule::SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            RenameRule::ScreamingSnakeCase => RenameRule::SnakeCase
                .apply_to_variant(variant)
                .to_ascii_uppercase(),
            RenameRule::KebabCase => RenameRule::SnakeCase
                .apply_to_variant(variant)
                .replace('_', "-"),
            RenameRule::ScreamingKebabCase => RenameRule::ScreamingSnakeCase
                .apply_to_variant(variant)
                .replace('_', "-"),
        }
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    pub fn apply_to_field(self, field: &str) -> String {
        match self {
            RenameRule::None | RenameRule::LowerCase | RenameRule::SnakeCase => field.to_owned(),
            RenameRule::UpperCase => field.to_ascii_uppercase(),
            RenameRule::PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            RenameRule::CamelCase => {
                let pascal = RenameRule::PascalCase.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            RenameRule::ScreamingSnakeCase => field.to_ascii_uppercase(),
            RenameRule::KebabCase => field.replace('_', "-"),
            RenameRule::ScreamingKebabCase => RenameRule::ScreamingSnakeCase
                .apply_to_field(field)
                .replace('_', "-"),
        }
    }
}
