use crate::case::RenameRule;
use crate::{Field, Output, OutputKind, TSTypeError, TSTypeResult, Type, Variant};
use syn::{Data, DeriveInput, Fields, Type as SynType};

pub(crate) fn handle_export_type_parsing(input: &DeriveInput, _lang: &str) -> TSTypeResult<Output> {
    let name = input.ident.to_string();
    let mut lang = "typescript".to_string();
    let mut rename_all = None;
    for attr in &input.attrs {
        if attr.path().is_ident("export_type") {
            if let Ok(nested) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) {
                for meta in nested {
                    if let syn::Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("lang") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = nv.value
                            {
                                lang = lit_str.value();
                            }
                        } else if nv.path.is_ident("rename_all") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = nv.value
                            {
                                rename_all = RenameRule::from_str(&lit_str.value());
                            }
                        }
                    }
                }
            }
        }
    }

    let kind = match &input.data {
        Data::Struct(data) => {
            let fields = parse_fields(&data.fields, rename_all.as_ref())?;
            OutputKind::Struct(fields)
        }
        Data::Enum(data) => {
            let variants = data
                .variants
                .iter()
                .map(|v| parse_variant(v))
                .collect::<TSTypeResult<Vec<_>>>()?;
            OutputKind::Enum(variants)
        }
        Data::Union(_) => {
            return Err(TSTypeError::UnsupportedType(
                "Union types are not supported".to_string(),
            ))
        }
    };

    let generics = input
        .generics
        .type_params()
        .map(|param| param.ident.to_string())
        .collect();

    Ok(Output {
        name,
        kind,
        generics,
        lang,
        rename_all,
        export_path: None,
    })
}

fn parse_fields(fields: &Fields, rename_all: Option<&RenameRule>) -> TSTypeResult<Vec<Field>> {
    let mut result = Vec::new();

    match fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                let field = parse_field(field, rename_all)?;
                result.push(field);
            }
        }
        _ => {
            return Err(TSTypeError::UnsupportedType(
                "Only named fields are supported".to_string(),
            ))
        }
    }

    Ok(result)
}

fn parse_field(field: &syn::Field, rename_all: Option<&RenameRule>) -> TSTypeResult<Field> {
    let name = field
        .ident
        .as_ref()
        .map(|ident| ident.to_string())
        .unwrap_or_else(|| "".to_string());

    let mut renamed = name.clone();
    let mut has_explicit_rename = false;

    for attr in &field.attrs {
        if attr.path().is_ident("export_type") {
            if let Ok(nested) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) {
                for meta in nested {
                    if let syn::Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("rename") {
                            has_explicit_rename = true;
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = nv.value
                            {
                                renamed = lit_str.value();
                            }
                        }
                    }
                }
            }
        }
    }

    if !has_explicit_rename {
        if let Some(rule) = rename_all {
            renamed = rule.apply(&renamed);
        }
    }

    let ty = parse_type(&field.ty)?;
    let optional = is_optional(&field.ty);

    Ok(Field {
        name: renamed,
        ty,
        optional,
    })
}

fn parse_variant(variant: &syn::Variant) -> TSTypeResult<Variant> {
    let name = variant.ident.to_string();
    let fields = match &variant.fields {
        Fields::Named(named) => Some(parse_fields(&Fields::Named(named.clone()), None)?),
        Fields::Unit => None,
        Fields::Unnamed(unnamed) => {
            let fields = unnamed
                .unnamed
                .iter()
                .map(|field| {
                    let ty = parse_type(&field.ty)?;
                    let optional = is_optional(&field.ty);
                    Ok(Field {
                        name: "".to_string(),
                        ty,
                        optional,
                    })
                })
                .collect::<TSTypeResult<Vec<_>>>()?;
            Some(fields)
        }
    };

    Ok(Variant { name, fields })
}

fn parse_type(ty: &SynType) -> TSTypeResult<Type> {
    match ty {
        SynType::Path(path) => {
            let segment = path.path.segments.last().ok_or_else(|| {
                TSTypeError::ParsingError(syn::Error::new_spanned(path, "Invalid type"))
            })?;

            match segment.ident.to_string().as_str() {
                "String" => Ok(Type::String),
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64"
                | "usize" => Ok(Type::Number),
                "bool" => Ok(Type::Boolean),
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(arg) = args.args.first() {
                            if let syn::GenericArgument::Type(inner_ty) = arg {
                                return Ok(Type::Array(Box::new(parse_type(inner_ty)?)));
                            }
                        }
                    }
                    Err(TSTypeError::UnsupportedType("Invalid Vec type".to_string()))
                }
                "HashMap" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if args.args.len() == 2 {
                            if let (
                                syn::GenericArgument::Type(key_ty),
                                syn::GenericArgument::Type(value_ty),
                            ) = (&args.args[0], &args.args[1])
                            {
                                return Ok(Type::HashMap(
                                    Box::new(parse_type(key_ty)?),
                                    Box::new(parse_type(value_ty)?),
                                ));
                            }
                        }
                    }
                    Err(TSTypeError::UnsupportedType(
                        "Invalid HashMap type".to_string(),
                    ))
                }
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(arg) = args.args.first() {
                            if let syn::GenericArgument::Type(inner_ty) = arg {
                                return Ok(Type::Optional(Box::new(parse_type(inner_ty)?)));
                            }
                        }
                    }
                    Err(TSTypeError::UnsupportedType(
                        "Invalid Option type".to_string(),
                    ))
                }
                other => Ok(Type::Custom(other.to_string())),
            }
        }
        _ => Err(TSTypeError::UnsupportedType("Unsupported type".to_string())),
    }
}

fn is_optional(ty: &SynType) -> bool {
    if let SynType::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_basic_struct() {
        let input: DeriveInput = parse_quote! {
            struct Test {
                id: i32,
                name: String,
            }
        };

        let output = handle_export_type_parsing(&input, "typescript").unwrap();
        assert_eq!(output.name, "Test");

        if let OutputKind::Struct(fields) = output.kind {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "id");
            assert!(matches!(fields[0].ty, Type::Number));
            assert_eq!(fields[1].name, "name");
            assert!(matches!(fields[1].ty, Type::String));
        } else {
            panic!("Expected Struct");
        }
    }

    #[test]
    fn test_parse_enum() {
        let input: DeriveInput = parse_quote! {
            enum Status {
                Active,
                Pending { reason: String }
            }
        };

        let output = handle_export_type_parsing(&input, "typescript").unwrap();
        assert_eq!(output.name, "Status");

        if let OutputKind::Enum(variants) = output.kind {
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Active");
            assert!(variants[0].fields.is_none());
            assert_eq!(variants[1].name, "Pending");
            assert!(variants[1].fields.is_some());
        } else {
            panic!("Expected Enum");
        }
    }

    #[test]
    fn test_parse_generic_struct() {
        let input: DeriveInput = parse_quote! {
            struct Container<T, U> {
                data: T,
                metadata: Option<U>,
            }
        };

        let output = handle_export_type_parsing(&input, "typescript").unwrap();
        assert_eq!(output.name, "Container");
        assert_eq!(output.generics, vec!["T", "U"]);

        if let OutputKind::Struct(fields) = output.kind {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "data");
            assert!(matches!(fields[0].ty, Type::Custom(ref s) if s == "T"));
            assert_eq!(fields[1].name, "metadata");
            assert!(
                matches!(fields[1].ty, Type::Optional(ref inner) if matches!(**inner, Type::Custom(ref s) if s == "U"))
            );
        } else {
            panic!("Expected Struct");
        }
    }

    #[test]
    fn test_parse_generic_enum() {
        let input: DeriveInput = parse_quote! {
            enum Result<T, E> {
                Ok(T),
                Err { error: E }
            }
        };

        let output = handle_export_type_parsing(&input, "typescript").unwrap();
        assert_eq!(output.name, "Result");
        assert_eq!(output.generics, vec!["T", "E"]);
    }

    #[test]
    fn test_parse_renamed_field() {
        let field: syn::Field = parse_quote! {
            #[export_type(rename = "emailAddress")]
            email: String
        };

        let parsed = parse_field(&field, None).unwrap();
        assert_eq!(parsed.name, "emailAddress");
        assert!(matches!(parsed.ty, Type::String));
        assert!(!parsed.optional);
    }

    #[test]
    fn test_parse_optional_renamed_field() {
        let field: syn::Field = parse_quote! {
            #[export_type(rename = "phoneNumber")]
            phone: Option<String>
        };

        let parsed = parse_field(&field, None).unwrap();
        assert_eq!(parsed.name, "phoneNumber");
        assert!(matches!(parsed.ty, Type::Optional(ref inner) if matches!(**inner, Type::String)));
        assert!(parsed.optional);
    }
}
