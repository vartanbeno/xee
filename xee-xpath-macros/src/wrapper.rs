use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{ItemFn, LitStr};

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
    let vis = &ast.vis;
    let signature_string = LitStr::new(&options.signature_string, Span::call_site());

    Ok(quote! {
        // #[doc(hidden)]
        // create a module with the same name as the function - this way `use
        // <the function> will bring both the function and module into scope.
        // This module contains information about the wrapper function
        // we access with the wrap_xpath_fn! macro.
        // #vis mod #name {
        //     pub(crate) struct MakeWrapper;
        //     pub const WRAPPER: fn(&crate::DynamicContext, &[crate::Value]) -> Result<crate::Value, crate::ValueError> = MakeWrapper::WRAPPER;
        //     // We store the signature as a string; this means we need to
        //     // reparse it again later during registration, but it's a lot
        //     // easier than trying to serialize a data structure, so it will
        //     // do for now.
        //     pub const SIGNATURE: String = #signature_string.to_string();
        // }

        // Generate the function inside of the same scope at the original
        // function (but in an isolated block), so that it can easily call the
        // original function. Using `super` isn't useful for that, as the
        // original function may be inside of a function body.
        // const _: () = {
        //     // This is a trick to ensure we can get it into the module defined
        //     // above
        //     impl #name::MakeWrapper {
        //         const WRAPPER: fn(&crate::DynamicContext, &[crate::Value]) -> Result<crate::Value, crate::ValueError> = #wrapper_name;
        //     }
            fn #wrapper_name(context: &crate::DynamicContext, arguments: &[crate::Value]) -> Result<crate::Value, crate::ValueError> {
                #(#conversions)*;
                let value = #name(#(#conversion_names),*);
                Ok(value.into())
            }
        // }
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
