use quote::{format_ident, quote};
use syn::ItemFn;

use crate::parse::XPathFnOptions;

pub(crate) fn xpath_fn_wrapper(
    ast: &mut ItemFn,
    options: &XPathFnOptions,
) -> proc_macro2::TokenStream {
    let name = &ast.sig.ident;
    let wrapper_name = format_ident!("wrapper_{}", name);
    // let signature = &options.signature;
    // let signature = dbg!(signature);
    quote! {
        fn #wrapper_name(context: &xee_xpath::DynamicContext, arguments: &[&xee_xpath::Value]) -> Result<xee_xpath::Value, xee_xpath::ValueError> {
            let value = #name();
            Ok(value.into())
        }
    }
}
