extern crate proc_macro;

#[macro_use]
mod error;
mod parse;

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use parse::XPathFnOptions;

#[proc_macro_attribute]
pub fn xpath_fn(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // let options = parse_macro_input!(attr as XPathFnOptions);
    let ast = parse_macro_input!(input as syn::ItemFn);

    // let options = dbg!(options);
    quote!(
        // options
        #ast
    )
    .into()
}
