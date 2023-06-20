// In this module, we generate quoted code that converts an incoming
// Sequence into the required Rust type, using the SequenceType as a guide.

use proc_macro2::TokenStream;
use quote::quote;

use xee_xpath_ast::ast;
use xee_xpath_ast::XS_NAMESPACE;

pub(crate) fn convert_sequence_type(
    sequence_type: &ast::SequenceType,
    arg: TokenStream,
) -> syn::Result<TokenStream> {
    match sequence_type {
        ast::SequenceType::Empty => Ok(quote!(
            let #arg = #arg.ensure_empty()?;
        )),
        ast::SequenceType::Item(item) => convert_item(item, arg),
        _ => {
            panic!("Unsupported");
        }
    }
}

fn convert_item(item: &ast::Item, arg: TokenStream) -> syn::Result<TokenStream> {
    let (iterator, want_result_occurrence) = convert_item_type(&item.item_type, arg.clone())?;
    let occurrence = if want_result_occurrence {
        quote!(crate::ResultOccurrence)
    } else {
        quote!(crate::Occurrence)
    };
    Ok(match &item.occurrence {
        ast::Occurrence::One => quote!(
            let #arg = #occurrence::one(#iterator)?;
        ),
        ast::Occurrence::Option => quote!(
            let #arg = #occurrence::option(#iterator)?;
        ),
        ast::Occurrence::Many => {
            let arg_temp = syn::Ident::new(&format!("tmp_{}", arg), proc_macro2::Span::call_site());
            quote!(
                let #arg_temp = #occurrence::many(#iterator)?;
                let #arg = #arg_temp.as_slice();
            )
        }
        ast::Occurrence::NonEmpty => todo!("NonEmpty not yet supported"),
    })
}

fn convert_item_type(item: &ast::ItemType, arg: TokenStream) -> syn::Result<(TokenStream, bool)> {
    match item {
        ast::ItemType::Item => Ok((quote!(#arg.items()), false)),
        ast::ItemType::AtomicOrUnionType(name) => {
            Ok((convert_atomic_or_union_type(name, arg)?, true))
        }
        _ => {
            todo!("Not yet")
        }
    }
}

fn convert_atomic_or_union_type(name: &ast::Name, arg: TokenStream) -> syn::Result<TokenStream> {
    // TODO: we don't handle anything but xs: yes
    assert_eq!(name.namespace(), Some(XS_NAMESPACE));

    let local_name = name.as_str();
    if local_name == "xs:anyAtomicType" {
        return Ok(quote!(#arg.atomized()));
    }

    let convert = match local_name {
        "boolean" => quote!(atomic.to_boolean()),
        "integer" => quote!(atomic.to_integer()),
        "float" => quote!(atomic.to_float()),
        "double" => quote!(atomic.to_double()),
        "decimal" => quote!(atomic.to_decimal()),
        "string" => quote!(atomic.to_str()),
        _ => {
            todo!("Not yet")
        }
    };

    Ok(quote!(#arg.unboxed_atomized(|atomic| #convert)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use xee_xpath_ast::Namespaces;

    fn convert(s: &str) -> String {
        let namespaces = Namespaces::default();
        let sequence_type = ast::parse_sequence_type(s, &namespaces).unwrap();
        let arg = quote!(a);
        convert_sequence_type(&sequence_type, arg)
            .unwrap()
            .to_string()
    }

    #[test]
    fn test_convert() {
        assert_debug_snapshot!(convert("xs:integer"));
    }

    #[test]
    fn test_convert_option() {
        assert_debug_snapshot!(convert("xs:integer?"));
    }

    #[test]
    fn test_convert_many() {
        assert_debug_snapshot!(convert("xs:integer*"));
    }

    #[test]
    fn test_convert_empty_sequence() {
        assert_debug_snapshot!(convert("empty-sequence()"));
    }

    #[test]
    fn test_convert_item() {
        assert_debug_snapshot!(convert("item()"));
    }

    #[test]
    fn test_convert_any_atomic_type() {
        assert_debug_snapshot!(convert("xs:anyAtomicType"));
    }
}
