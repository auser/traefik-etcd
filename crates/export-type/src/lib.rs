use std::path::PathBuf;

use lang::TSExporter;
use parsers::handle_export_type_parsing;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, DeriveInput};

mod case;
mod error;
mod exporter;
mod lang;
mod parsers;

use error::ToCompileError;

use case::*;
use error::*;
use exporter::*;

pub(crate) static DEFAULT_EXPORT_PATH: &str = "exports";

/// Derives the ExportType trait for a struct or enum, generating TypeScript type definitions.
///
/// # Examples
///
/// ```ignore
/// #[derive(ExportType)]
/// #[export_type(lang = "typescript", path = "types/generated")]
/// struct User {
///     id: i32,
///     name: String,
///     #[export_type(rename = "emailAddress")]
///     email: Option<String>,
/// }
///
/// #[derive(ExportType)]
/// enum Status {
///     Active,
///     Inactive,
///     Pending { reason: String },
/// }
/// ```
#[proc_macro_derive(ExportType, attributes(export_type))]
pub fn export_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match handle_export_type(input.clone()) {
        Ok(_output) => {
            if let Ok(export_path) = get_export_path_from_attrs(&input.attrs) {
                let _ = create_exporter_files(export_path);
            }
            quote::quote! {}.into()
        }
        Err(e) => e.to_compile_error().into(),
    }
}

fn handle_export_type(input: DeriveInput) -> TSTypeResult<proc_macro2::TokenStream> {
    let lang = get_lang_from_attrs(&input.attrs)?;
    let mut output = handle_export_type_parsing(&input, &lang)?;
    let name = input.ident.to_string();
    let generics = get_generics_from_attrs(&input.attrs)?;
    output.generics = generics;
    output.lang = lang;
    add_struct_or_enum(name.clone(), output)?;
    Ok(quote::quote! {})
}

// fn get_exporter_from_attrs(
//     attrs: &[Attribute],
//     output: Output,
//     generics: Vec<String>,
// ) -> TSTypeResult<Box<dyn ToOutput>> {
//     match get_lang_from_attrs(attrs)?.as_str() {
//         "typescript" | "ts" => Ok(Box::new(TSExporter::new(output, None, generics))),
//         lang => Err(TSTypeError::UnsupportedLanguage(lang.to_string())),
//     }
// }

fn get_exporter_from_lang(
    lang: &str,
    output: Output,
    generics: Vec<String>,
) -> TSTypeResult<Box<dyn ToOutput>> {
    match lang {
        "typescript" | "ts" => Ok(Box::new(TSExporter::new(output, None, generics))),
        lang => Err(TSTypeError::UnsupportedLanguage(lang.to_string())),
    }
}

fn get_generics_from_attrs(attrs: &[Attribute]) -> TSTypeResult<Vec<String>> {
    let mut generics = vec![];

    for attr in attrs {
        if attr.path().is_ident("export_type") {
            if let Ok(nested) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) {
                for meta in nested {
                    if let syn::Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("generics") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = nv.value
                            {
                                generics = lit_str
                                    .value()
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect();
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(generics)
}

fn get_lang_from_attrs(attrs: &[Attribute]) -> TSTypeResult<String> {
    let mut lang = String::from("typescript");

    for attr in attrs {
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
                        }
                    }
                }
            }
        }
    }

    Ok(lang)
}

fn get_export_path_from_attrs(attrs: &[Attribute]) -> TSTypeResult<PathBuf> {
    let mut export_path = PathBuf::from(DEFAULT_EXPORT_PATH);

    for attr in attrs {
        if attr.path().is_ident("export_type") {
            if let Ok(nested) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) {
                for meta in nested {
                    if let syn::Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("path") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = nv.value
                            {
                                export_path = PathBuf::from(lit_str.value());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(export_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_get_lang_from_attrs() {
        let attrs = vec![];
        assert_eq!(
            get_lang_from_attrs(&attrs).unwrap(),
            "typescript".to_string()
        );

        let attrs = vec![parse_quote! { #[export_type(lang = "typescript")] }];
        assert_eq!(
            get_lang_from_attrs(&attrs).unwrap(),
            "typescript".to_string()
        );

        let attrs = vec![parse_quote! { #[export_type(lang = "unsupported")] }];
        assert_eq!(
            get_lang_from_attrs(&attrs).unwrap(),
            "unsupported".to_string()
        );
    }

    #[test]
    fn test_get_export_path_from_attrs() {
        let attrs = vec![];
        assert_eq!(
            get_export_path_from_attrs(&attrs).unwrap(),
            PathBuf::from("exports")
        );

        let attrs = vec![parse_quote! { #[export_type(path = "test")] }];
        assert_eq!(
            get_export_path_from_attrs(&attrs).unwrap(),
            PathBuf::from("test")
        );
    }
}
