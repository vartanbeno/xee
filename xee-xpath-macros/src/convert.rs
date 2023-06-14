use proc_macro2::TokenStream;
use quote::quote;

use xee_xpath_ast::ast::{ItemType, KindTest, Occurrence, SequenceType};

pub(crate) struct ConvertedCode {
    pub(crate) prepare: TokenStream,
    pub(crate) assign: TokenStream,
}

impl ConvertedCode {
    fn new(assign: TokenStream) -> Self {
        ConvertedCode {
            prepare: quote!(),
            assign,
        }
    }

    fn with_prepare(self, prepare: TokenStream) -> Self {
        ConvertedCode {
            prepare,
            assign: self.assign,
        }
    }
}
trait ConvertCode {
    fn code(&self, arg: TokenStream, occurrence: Occurrence) -> syn::Result<ConvertedCode>;
}

pub(crate) fn convert_code(
    sequence_type: &SequenceType,
    arg: TokenStream,
) -> syn::Result<ConvertedCode> {
    match sequence_type {
        SequenceType::Empty => {
            panic!("Empty sequence not yet supported");
        }
        SequenceType::Item(item) => item.item_type.code(arg, item.occurrence),
        _ => {
            panic!("Unsupported");
        }
    }
}

impl ConvertCode for ItemType {
    fn code(&self, arg: TokenStream, occurrence: Occurrence) -> syn::Result<ConvertedCode> {
        match self {
            ItemType::Item => Ok(match occurrence {
                Occurrence::One => ConvertedCode::new(quote!(#arg.to_one())),
                Occurrence::Option => ConvertedCode::new(quote!(#arg.to_option())),
                // XXX what happens if we have multiple tmp?
                Occurrence::Many => {
                    let converted = ConvertedCode::new(quote!(Ok(tmp2.as_slice())));
                    converted.with_prepare(quote!(
                        let tmp = #arg.to_many();
                        let tmp2 = tmp.borrow();
                    ))
                }
                Occurrence::NonEmpty => panic!("NonEmpty not yet supported"),
            }),
            ItemType::AtomicOrUnionType(_name) => Ok(match occurrence {
                // XXX no type checking for one version option takes place
                Occurrence::One | Occurrence::Option => ConvertedCode::new(quote!(
                    crate::context::ContextTryInto::context_try_into(#arg, context)
                )),
                Occurrence::Many => {
                    let converted = ConvertedCode::new(quote!(Ok(tmp3.as_slice())));
                    converted.with_prepare(quote!(
                      let tmp = #arg.to_many();
                      let tmp2 = tmp.borrow();
                      let tmp3 =  tmp2.as_slice().iter().map(|v| v.try_into()?).collect::<Vec<_>>();
                    ;))
                }
                _ => panic!("Unsupported occurrence for atomic or union"),
            }),
            ItemType::KindTest(kind_test) => kind_test.code(arg, occurrence),
            _ => {
                panic!("Unsupported ItemType");
            }
        }
    }
}

impl ConvertCode for KindTest {
    fn code(&self, arg: TokenStream, occurrence: Occurrence) -> syn::Result<ConvertedCode> {
        match self {
            KindTest::Any => match occurrence {
                // XXX no type checking for option for one
                Occurrence::One | Occurrence::Option => Ok(ConvertedCode::new(
                    quote!(crate::context::ContextTryInto::context_try_into(#arg, context)),
                )),
                _ => {
                    panic!("Unsupported occurrence for Any");
                }
            },
            _ => {
                panic!("Unsupported KindTest");
            }
        }
    }
}
