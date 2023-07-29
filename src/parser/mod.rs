use std::fmt::{Display, Formatter};
use syn::{Field, Fields, GenericArgument, ItemEnum, ItemStruct, PathArguments, Type};

pub use crate::parser::doc_comments::extract_doc_comment;
use crate::parser::rename_rule::{get_rename_rule, RenameRule};
use crate::{EnumInfo, FieldInfo, StructInfo, TypeInfo, VariantInfo};

mod doc_comments;
mod rename_rule;

pub fn parse_struct(item: ItemStruct) -> StructInfo {
    let rename_rule = get_rename_rule(&item.attrs);

    let fields = item
        .fields
        .into_iter()
        .map(|f| parse_field(f, rename_rule))
        .collect();

    StructInfo { fields }
}

pub fn parse_enum(item: ItemEnum) -> EnumInfo {
    let rename_rule = get_rename_rule(&item.attrs);

    let variants = item
        .variants
        .into_iter()
        .map(|variant| {
            let name = rename_rule.apply_to_variant(&variant.ident.to_string());

            let description = extract_doc_comment(&variant.attrs);

            let fields = match variant.fields {
                Fields::Named(fields) => fields
                    .named
                    .into_iter()
                    .map(|f| parse_field(f, rename_rule))
                    .collect(),
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .into_iter()
                    .map(|f| parse_field(f, rename_rule))
                    .collect(),
                Fields::Unit => vec![],
            };

            VariantInfo {
                name,
                description,
                fields,
            }
        })
        .collect();

    EnumInfo { variants }
}

fn parse_field(field: Field, rename_rule: RenameRule) -> FieldInfo {
    let name = field
        .ident
        .map(|ident| ident.to_string())
        .map(|name| rename_rule.apply_to_field(&name))
        .unwrap_or_default();

    let description = extract_doc_comment(&field.attrs);
    let type_info = parse_type(Box::new(field.ty));

    FieldInfo {
        name,
        description,
        ty: type_info,
    }
}

fn parse_type(ty: Box<Type>) -> TypeInfo {
    let mut info = TypeInfo::default();

    match *ty {
        Type::Path(path) => {
            let mut name = vec![];
            for segment in path.path.segments {
                name.push(segment.ident.to_string());

                if let PathArguments::AngleBracketed(args) = segment.arguments {
                    for arg in args.args {
                        if let GenericArgument::Type(ty) = arg {
                            info.generics.push(parse_type(Box::new(ty)))
                        }
                    }
                }
            }

            info.name = name.join("::");
        }
        _ => {
            info.name = "Unknown".to_string();
        }
    };

    info
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.generics.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(
                f,
                "{}<{}>",
                self.name,
                self.generics
                    .iter()
                    .map(|generic| generic.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

impl TypeInfo {
    pub(crate) fn to_doc_string(&self, simplify: bool) -> String {
        if simplify {
            match self.name.as_str() {
                "Box" | "Arc" | "Rc" | "Cell" | "RefCell" | "RwLock" | "Mutex" => self
                    .generics
                    .iter()
                    .map(|generic| generic.to_doc_string(simplify))
                    .collect::<Vec<_>>()
                    .join(", "),
                "Option" => format!(
                    "{}?",
                    self.generics
                        .iter()
                        .map(|generic| generic.to_doc_string(simplify))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                "str" => "String".to_string(),
                _ => self.name.clone(),
            }
        } else {
            self.to_string()
        }
    }
}
