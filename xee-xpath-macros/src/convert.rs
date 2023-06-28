// In this module, we generate quoted code that converts an incoming
// Sequence into the required Rust type, using the SequenceType as a guide.

use proc_macro2::TokenStream;
use quote::quote;

use xee_xpath_ast::ast;
use xee_xpath_ast::XS_NAMESPACE;

pub(crate) fn convert_sequence_type(
    sequence_type: &ast::SequenceType,
    fn_arg: &syn::FnArg,
    name: TokenStream,
    arg: TokenStream,
) -> syn::Result<TokenStream> {
    match sequence_type {
        ast::SequenceType::Empty => Ok(quote!(
            let #name = #arg.ensure_empty()?;
        )),
        ast::SequenceType::Item(item) => convert_item(item, fn_arg, name, arg),
        _ => {
            panic!("Unsupported");
        }
    }
}

fn convert_item(
    item: &ast::Item,
    fn_arg: &syn::FnArg,
    name: TokenStream,
    arg: TokenStream,
) -> syn::Result<TokenStream> {
    let (iterator, borrow) = convert_item_type(&item.item_type, arg.clone())?;
    let occurrence = quote!(crate::Occurrence);

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
                quote!(let #name = #name.as_deref())
            } else {
                quote!()
            };
            quote!(
                let #name = #occurrence::option(&mut #iterator)?;
                #as_ref
            )
        }
        ast::Occurrence::Many => {
            if is_sequence_arg(fn_arg) {
                // we already have a reference argument, so
                // we don't need to do anything to it
                return Ok(quote!(let #name = &(#arg);));
            }
            let name_temp =
                syn::Ident::new(&format!("tmp_{}", name), proc_macro2::Span::call_site());
            let as_ref = if borrow {
                quote!(let #name_temp = #name_temp.iter().map(|s| s.as_ref()).collect::<Vec<_>>();)
            } else {
                quote!()
            };
            let many = quote!(#occurrence::many(&mut #iterator)?);
            quote!(
                let #name_temp = #many;
                #as_ref
                let #name = #name_temp.as_slice();
            )
        }
        ast::Occurrence::NonEmpty => todo!("NonEmpty not yet supported"),
    })
}

fn convert_item_type(item: &ast::ItemType, arg: TokenStream) -> syn::Result<(TokenStream, bool)> {
    match item {
        ast::ItemType::Item => Ok((quote!(#arg.items()), false)),
        ast::ItemType::AtomicOrUnionType(name) => {
            let (token_stream, borrow) = convert_atomic_or_union_type(name, arg)?;
            Ok((token_stream, borrow))
        }
        ast::ItemType::KindTest(kind_test) => Ok((convert_kind_test(kind_test, arg)?, false)),
        _ => {
            todo!("Not yet")
        }
    }
}

fn convert_atomic_or_union_type(
    name: &ast::Name,
    arg: TokenStream,
) -> syn::Result<(TokenStream, bool)> {
    // TODO: we don't handle anything but xs: yet
    assert_eq!(name.namespace(), Some(XS_NAMESPACE));

    let local_name = name.as_str();
    if local_name == "anyAtomicType" {
        return Ok((quote!(#arg.atomized(context.xot)), false));
    }

    let type_name = syn::parse_str::<syn::Type>(&rust_type_name(local_name))?;
    let convert = quote!(std::convert::TryInto::<#type_name>::try_into(atomic));

    let borrow = local_name == "string";
    Ok((
        quote!(#arg.unboxed_atomized(context.xot, |atomic| #convert)),
        borrow,
    ))
}

fn rust_type_name(local_name: &str) -> String {
    match local_name {
        "string" => "String".to_string(),
        "boolean" => "bool".to_string(),
        "decimal" => "rust_decimal::Decimal".to_string(),
        "integer" => "i64".to_string(),
        "int" => "i32".to_string(),
        "short" => "i16".to_string(),
        "byte" => "i8".to_string(),
        "unsignedLong" => "u64".to_string(),
        "unsignedInt" => "u32".to_string(),
        "unsignedShort" => "u16".to_string(),
        "unsignedByte" => "u8".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        _ => {
            panic!("Cannot get type name for {}", local_name);
        }
    }
}

fn convert_kind_test(kind_test: &ast::KindTest, arg: TokenStream) -> syn::Result<TokenStream> {
    match kind_test {
        ast::KindTest::Any => Ok(quote!(#arg.nodes())),
        _ => {
            todo!("Not yet")
        }
    }
}

fn is_sequence_arg(fn_arg: &syn::FnArg) -> bool {
    match fn_arg {
        syn::FnArg::Receiver(_) => false,
        syn::FnArg::Typed(type_) => match type_.ty.as_ref() {
            syn::Type::Reference(type_) => match type_.elem.as_ref() {
                syn::Type::Path(type_) => {
                    let segment = type_.path.segments.iter().last();
                    match segment {
                        Some(syn::PathSegment {
                            ident,
                            arguments: _arguments,
                        }) => ident == "Sequence",
                        _ => false,
                    }
                }
                _ => false,
            },
            _ => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use xee_xpath_ast::Namespaces;

    fn convert(s: &str) -> String {
        // dummy fixed fn arg here
        convert_fn_arg(s, &syn::parse_str("a: &str").unwrap())
    }

    fn convert_fn_arg(s: &str, fn_arg: &syn::FnArg) -> String {
        let namespaces = Namespaces::default();
        let sequence_type = ast::parse_sequence_type(s, &namespaces).unwrap();
        let name = quote!(a);
        let arg = quote!(arguments[0]);

        convert_sequence_type(&sequence_type, fn_arg, name, arg)
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

    #[test]
    fn test_convert_sequence_arg() {
        assert_debug_snapshot!(convert_fn_arg(
            "item()*",
            &syn::parse_str("a: &crate::Sequence").unwrap()
        ));
    }
}
