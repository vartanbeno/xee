// In this module, we generate quoted code that converts an incoming
// Sequence into the required Rust type, using the SequenceType as a guide.

use proc_macro2::TokenStream;
use quote::quote;

use xee_xpath_ast::ast;
use xee_xpath_ast::XS_NAMESPACE;

pub(crate) fn convert_sequence_type(
    sequence_type: &ast::SequenceType,
    name: TokenStream,
    arg: TokenStream,
) -> syn::Result<TokenStream> {
    match sequence_type {
        ast::SequenceType::Empty => Ok(quote!(
            let #name = #arg.ensure_empty()?;
        )),
        ast::SequenceType::Item(item) => convert_item(item, name, arg),
        _ => {
            panic!("Unsupported");
        }
    }
}

fn convert_item(item: &ast::Item, name: TokenStream, arg: TokenStream) -> syn::Result<TokenStream> {
    let (iterator, want_result_occurrence, borrow) = convert_item_type(&item.item_type, arg)?;
    let occurrence = if want_result_occurrence {
        quote!(crate::ResultOccurrence)
    } else {
        quote!(crate::Occurrence)
    };

    Ok(match &item.occurrence {
        ast::Occurrence::One => {
            let as_ref = if borrow {
                quote!(let #name = #name.as_ref())
            } else {
                quote!()
            };
            quote!(
                let #name = #occurrence::one(&mut #iterator)?;
                #as_ref
            )
        }
        ast::Occurrence::Option => {
            let as_ref = if borrow {
                quote!(let #name = #name.as_ref())
            } else {
                quote!()
            };
            quote!(
                let #name = #occurrence::option(&mut #iterator)?;
                #as_ref
            )
        }
        ast::Occurrence::Many => {
            let name_temp =
                syn::Ident::new(&format!("tmp_{}", name), proc_macro2::Span::call_site());
            let as_ref = if borrow {
                quote!(let #name_temp = #name_temp.iter().map(|s| s.as_ref()).collect::<Vec<_>>();)
            } else {
                quote!()
            };
            let many = if want_result_occurrence {
                quote!(#occurrence::many(&mut #iterator)?)
            } else {
                quote!(#occurrence::many(&mut #iterator))
            };
            quote!(
                let #name_temp = #many;
                #as_ref
                let #name = #name_temp.as_slice();
            )
        }
        ast::Occurrence::NonEmpty => todo!("NonEmpty not yet supported"),
    })
}

fn convert_item_type(
    item: &ast::ItemType,
    arg: TokenStream,
) -> syn::Result<(TokenStream, bool, bool)> {
    match item {
        ast::ItemType::Item => Ok((quote!(#arg.items()), false, false)),
        ast::ItemType::AtomicOrUnionType(name) => {
            let (token_stream, borrow) = convert_atomic_or_union_type(name, arg)?;
            Ok((token_stream, true, borrow))
        }
        ast::ItemType::KindTest(kind_test) => Ok((convert_kind_test(kind_test, arg)?, true, false)),
        _ => {
            todo!("Not yet")
        }
    }
}

fn convert_atomic_or_union_type(
    name: &ast::Name,
    arg: TokenStream,
) -> syn::Result<(TokenStream, bool)> {
    // TODO: we don't handle anything but xs: yes
    assert_eq!(name.namespace(), Some(XS_NAMESPACE));

    let local_name = name.as_str();
    if local_name == "anyAtomicType" {
        return Ok((quote!(#arg.atomized(context.xot)), false));
    }

    let (convert, borrow) = match local_name {
        "boolean" => (quote!(atomic.to_boolean()), false),
        "integer" => (quote!(atomic.to_integer()), false),
        "int" => (quote!(atomic.to_integer()), false),
        "float" => (quote!(atomic.to_float()), false),
        "double" => (quote!(atomic.to_double()), false),
        "decimal" => (quote!(atomic.to_decimal()), false),
        "string" => (quote!(atomic.to_string()), true),
        _ => {
            todo!("Not yet {}", local_name)
        }
    };

    Ok((
        quote!(#arg.unboxed_atomized(context.xot, |atomic| #convert)),
        borrow,
    ))
}

fn convert_kind_test(kind_test: &ast::KindTest, arg: TokenStream) -> syn::Result<TokenStream> {
    match kind_test {
        ast::KindTest::Any => Ok(quote!(#arg.nodes())),
        _ => {
            todo!("Not yet")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use xee_xpath_ast::Namespaces;

    fn convert(s: &str) -> String {
        let namespaces = Namespaces::default();
        let sequence_type = ast::parse_sequence_type(s, &namespaces).unwrap();
        let name = quote!(a);
        let arg = quote!(arguments[0]);
        convert_sequence_type(&sequence_type, name, arg)
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

    #[test]
    fn test_convert_node() {
        assert_debug_snapshot!(convert("node()"));
    }

    #[test]
    fn test_convert_string() {
        assert_debug_snapshot!(convert("xs:string"));
    }

    #[test]
    fn test_convert_string_option() {
        assert_debug_snapshot!(convert("xs:string?"));
    }

    #[test]
    fn test_convert_string_many() {
        assert_debug_snapshot!(convert("xs:string*"));
    }
}
