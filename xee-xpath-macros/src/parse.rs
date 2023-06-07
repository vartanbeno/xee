use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitStr, Result,
};

use xee_xpath_ast::ast::{parse_signature, Signature};
use xee_xpath_ast::Namespaces;

#[derive(Debug)]
pub(crate) struct XPathFnOptions {
    signature: Signature,
}

mod kw {
    syn::custom_keyword!(signature);
}

impl Parse for XPathFnOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut options = Vec::new();
        while !input.is_empty() {
            options.push(input.parse()?);
        }

        let mut signature: Option<String> = None;
        for option in options {
            match option {
                XPathFnOption::Signature(signature_option) => {
                    signature = Some(signature_option);
                }
            }
        }
        let signature = signature.unwrap();
        let namespaces = Namespaces::default();
        let signature = parse_signature(&signature, &namespaces)
            .map_err(|e| syn::Error::new(input.span(), e))?;
        Ok(Self { signature })
    }
}

enum XPathFnOption {
    Signature(String),
}

impl Parse for XPathFnOption {
    fn parse(input: ParseStream) -> Result<Self> {
        let string_literal: LitStr = input.parse()?;
        let signature = string_literal.value();
        Ok(XPathFnOption::Signature(signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_parse_signature() {
        assert_debug_snapshot!(syn::parse_str::<XPathFnOptions>(
            r#""fn:foo() as xs:string""#
        ));
    }

    #[test]
    fn test_parse_signature_parse_error() {
        assert_debug_snapshot!(syn::parse_str::<XPathFnOptions>(r#""wrong wrong""#));
    }

    #[test]
    fn test_parse_not_a_signature_string() {
        assert_debug_snapshot!(syn::parse_str::<XPathFnOptions>(r#"wrong"#));
    }
}
