// Mostly adapted from `serde_with_macros`.
//! Proc macro for the `rs3cache` crate.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::Parser, spanned::Spanned, Attribute, Error, Field, Fields, ItemEnum, ItemStruct, Meta, NestedMeta, Path, Type};

/// Applies `#[pyo3(get)]` to all fields of a struct.
#[proc_macro_attribute]
pub fn pyo3_get_all(_args: TokenStream, input: TokenStream) -> TokenStream {
    let res = match apply_function_to_struct_fields(input, add_pyo3_get_to_field) {
        Ok(res) => res,
        Err(err) => err.to_compile_error(),
    };
    TokenStream::from(res)
}

fn is_std_option(path: &Path) -> bool {
    path.leading_colon.is_none() && path.segments.len() == 1
}

fn field_has_attribute(field: &Field, namespace: &str, name: &str) -> bool {
    for attr in &field.attrs {
        if attr.path.is_ident(namespace) {
            // Ignore non parsable attributes, as these are not important for us
            if let Ok(Meta::List(expr)) = attr.parse_meta() {
                for expr in expr.nested {
                    if let NestedMeta::Meta(Meta::NameValue(expr)) = expr {
                        if let Some(ident) = expr.path.get_ident() {
                            if *ident == name {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn add_pyo3_get_to_field(field: &mut Field) -> Result<(), String> {
    if let Type::Path(path) = &field.ty {
        if is_std_option(&path.path) {
            let has_skip_serializing_if = field_has_attribute(field, "serde", "skip_serializing_if");

            // Remove the `serialize_always` attribute
            let mut has_always_attr = false;
            field.attrs.retain(|attr| {
                let has_attr = attr.path.is_ident("serialize_always");
                has_always_attr |= has_attr;
                !has_attr
            });

            // Error on conflicting attributes
            if has_always_attr && has_skip_serializing_if {
                let mut msg =
                    r#"The attributes `serialize_always` and `serde(skip_serializing_if = "...")` cannot be used on the same field"#.to_string();
                if let Some(ident) = &field.ident {
                    msg += ": `";
                    msg += &ident.to_string();
                    msg += "`";
                }
                msg += ".";
                return Err(msg);
            }

            // Do nothing if `skip_serializing_if` or `serialize_always` is already present
            if has_skip_serializing_if || has_always_attr {
                return Ok(());
            }

            // Add the `skip_serializing_if` attribute
            let attr_tokens = quote!(
                #[pyo3(get)]
            );
            let attrs = Attribute::parse_outer.parse2(attr_tokens).expect("Static attr tokens should not panic");
            field.attrs.extend(attrs);
        }
    }
    Ok(())
}

fn apply_function_to_struct_fields<F>(input: TokenStream, function: F) -> Result<TokenStream2, Error>
where
    F: Copy,
    F: Fn(&mut Field) -> Result<(), String>,
{
    /// Handle a single struct or a single enum variant
    fn apply_on_fields<F>(fields: &mut Fields, function: F) -> Result<(), Error>
    where
        F: Fn(&mut Field) -> Result<(), String>,
    {
        match fields {
            // simple, no fields, do nothing
            Fields::Unit => Ok(()),
            Fields::Named(ref mut fields) => fields
                .named
                .iter_mut()
                .map(|field| function(field).map_err(|err| Error::new(field.span(), err)))
                .collect_error(),
            Fields::Unnamed(ref mut fields) => fields
                .unnamed
                .iter_mut()
                .map(|field| function(field).map_err(|err| Error::new(field.span(), err)))
                .collect_error(),
        }
    }

    // For each field in the struct given by `input`, add the `skip_serializing_if` attribute,
    // if and only if, it is of type `Option`
    if let Ok(mut input) = syn::parse::<ItemStruct>(input.clone()) {
        apply_on_fields(&mut input.fields, function)?;
        Ok(quote!(#input))
    } else if let Ok(mut input) = syn::parse::<ItemEnum>(input) {
        input
            .variants
            .iter_mut()
            .map(|variant| apply_on_fields(&mut variant.fields, function))
            .collect_error()?;
        Ok(quote!(#input))
    } else {
        Err(Error::new(Span::call_site(), "The attribute can only be applied to struct definitions."))
    }
}

/// Merge multiple [`syn::Error`] into one.
trait IteratorExt {
    fn collect_error(self) -> Result<(), Error>
    where
        Self: Iterator<Item = Result<(), Error>> + Sized,
    {
        let accu = Ok(());
        self.fold(accu, |accu, error| match (accu, error) {
            (Ok(()), error) => error,
            (accu, Ok(())) => accu,
            (Err(mut err), Err(error)) => {
                err.combine(error);
                Err(err)
            }
        })
    }
}
impl<I> IteratorExt for I where I: Iterator<Item = Result<(), Error>> + Sized {}
