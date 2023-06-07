use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::ItemFn;

use crate::convert::convert_code;
use crate::parse::XPathFnOptions;

pub(crate) fn xpath_fn_wrapper(
    ast: &mut ItemFn,
    options: &XPathFnOptions,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut context_ident = None;
    if !ast.sig.inputs.is_empty() {
        let maybe_context_arg = &ast.sig.inputs[0];
        match &maybe_context_arg {
            syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                syn::Pat::Ident(ident) => {
                    if ident.ident == "context" {
                        context_ident = Some(ident.ident.clone());
                    }
                }
                _ => {
                    err_spanned!(pat_type.span() => "XPath functions can only take identifiers as arguments");
                }
            },
            syn::FnArg::Receiver(r) => {
                err_spanned!(r.span() => "XPath functions cannot take `self` as an argument");
            }
        }
    };
    let name = &ast.sig.ident;
    let wrapper_name = format_ident!("wrapper_{}", name);

    let signature = &options.signature;
    let mut conversions = Vec::new();
    let mut conversion_names = Vec::new();
    if let Some(context_ident) = context_ident {
        conversion_names.push(context_ident);
    }
    for (i, param) in signature.params.iter().enumerate() {
        let name = Ident::new(param.name.as_str(), Span::call_site());
        conversion_names.push(name.clone());
        let arg = quote!(&arguments[#i]);
        let converted = convert_code(&param.type_, arg)?;
        let prepare = converted.prepare;
        let assign = converted.assign;
        conversions.push(quote! {
            #prepare
            let #name = #assign?;
        });
    }
    Ok(quote! {
        fn #wrapper_name(context: &crate::DynamicContext, arguments: &[crate::Value]) -> Result<crate::Value, crate::ValueError> {
            #(#conversions)*;
            let value = #name(#(#conversion_names),*);
            Ok(value.into())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use syn::parse_str;

    #[test]
    fn test_wrapper() {
        let options =
            parse_str::<XPathFnOptions>(r#""fn:foo($x as xs:int) as xs:string""#).unwrap();
        let mut ast = parse_str::<ItemFn>(
            r#"
            fn foo(x: &i64) -> String {
                format!("{}", x)
            }"#,
        )
        .unwrap();
        assert_debug_snapshot!(xpath_fn_wrapper(&mut ast, &options).unwrap().to_string());
    }
}
