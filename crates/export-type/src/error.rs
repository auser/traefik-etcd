use proc_macro2::TokenStream;
use std::fmt;

pub type TSTypeResult<T> = Result<T, TSTypeError>;

#[derive(Debug)]
pub enum TSTypeError {
    ParsingError(syn::Error),
    UnsupportedLanguage(String),
    UnsupportedType(String),
    IOError(std::io::Error),
}

impl fmt::Display for TSTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParsingError(e) => write!(f, "Parsing error: {}", e),
            Self::UnsupportedLanguage(lang) => write!(f, "Unsupported language: {}", lang),
            Self::UnsupportedType(ty) => write!(f, "Unsupported type: {}", ty),
            Self::IOError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<std::io::Error> for TSTypeError {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<syn::Error> for TSTypeError {
    fn from(e: syn::Error) -> Self {
        Self::ParsingError(e)
    }
}

pub trait ToCompileError {
    fn to_compile_error(&self) -> TokenStream;
}

impl ToCompileError for TSTypeError {
    fn to_compile_error(&self) -> TokenStream {
        let message = self.to_string();
        quote::quote! {
            compile_error!(#message);
        }
    }
}
