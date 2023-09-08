use regex::Regex;
use std::rc::Rc;
use std::sync::OnceLock;

use xee_xpath_ast::ast;

use crate::atomic;
use crate::context;
use crate::error;

use super::cast::{whitespace_collapse, whitespace_replace};
use super::StringType;

// https://www.w3.org/TR/xml11/#NT-Nmtoken
// 	NameStartChar	   ::=   	":" | [A-Z] | "_" | [a-z] | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
// We create the NCName versions without colon (:) so we can do ncnames easily later
static NCNAME_START_CHAR: &str = r"A-Z_a-z\xc0-\xd6\xd8-\xf6\xf8-\u02ff\u0370-\u037d\u037f-\u1fff\u200c\u200d\u2070-\u218f\u2c00-\u2fef\u3001-\ud7ff\uf900-\ufdcf\ufdf0-\ufffd\U00010000-\U000effff";
// 	NameChar	   ::=   	NameStartChar | "-" | "." | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]
static NCNAME_CHAR_ADDITIONS: &str = r"-\.0-9\xb7\u0300-\u036F\u203F-\u2040";
static LANGUAGE_REGEX: OnceLock<Regex> = OnceLock::new();
static NMTOKEN_REGEX: OnceLock<Regex> = OnceLock::new();
static NAME_REGEX: OnceLock<Regex> = OnceLock::new();
static NC_NAME_REGEX: OnceLock<Regex> = OnceLock::new();

impl atomic::Atomic {
    pub(crate) fn cast_to_string(self) -> atomic::Atomic {
        atomic::Atomic::String(atomic::StringType::String, Rc::new(self.into_canonical()))
    }

    pub(crate) fn cast_to_untyped_atomic(self) -> atomic::Atomic {
        atomic::Atomic::Untyped(Rc::new(self.into_canonical()))
    }

    pub(crate) fn cast_to_any_uri(self) -> error::Result<atomic::Atomic> {
        // https://www.w3.org/TR/xpath-functions-31/#casting-to-anyuri
        match self {
            atomic::Atomic::String(_, s) => Ok(atomic::Atomic::String(
                StringType::AnyURI,
                Rc::new(whitespace_collapse(&s)),
            )),
            atomic::Atomic::Untyped(s) => Ok(atomic::Atomic::String(StringType::AnyURI, s.clone())),
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_normalized_string(self) -> atomic::Atomic {
        let s = whitespace_replace(&self.into_canonical());
        atomic::Atomic::String(atomic::StringType::NormalizedString, Rc::new(s))
    }

    pub(crate) fn cast_to_token(self) -> atomic::Atomic {
        let s = whitespace_collapse(&self.into_canonical());
        atomic::Atomic::String(atomic::StringType::Token, Rc::new(s))
    }

    fn cast_to_regex<F>(
        self,
        string_type: atomic::StringType,
        regex_once_lock: &OnceLock<Regex>,
        f: F,
    ) -> error::Result<atomic::Atomic>
    where
        F: FnOnce() -> Regex,
    {
        let regex = regex_once_lock.get_or_init(f);
        let s = whitespace_collapse(&self.into_canonical());
        if regex.is_match(&s) {
            Ok(atomic::Atomic::String(string_type, Rc::new(s)))
        } else {
            Err(error::Error::FORG0001)
        }
    }

    pub(crate) fn cast_to_language(self) -> error::Result<atomic::Atomic> {
        self.cast_to_regex(atomic::StringType::Language, &LANGUAGE_REGEX, || {
            Regex::new(r"^[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*$").expect("Invalid regex")
        })
    }

    pub(crate) fn cast_to_nmtoken(self) -> error::Result<atomic::Atomic> {
        // Nmtoken	 ::= (NameChar)+
        self.cast_to_regex(atomic::StringType::NMTOKEN, &NMTOKEN_REGEX, || {
            // we have to add the colon for NAME_START_CHAR / NAME_CHAR
            Regex::new(&format!(
                "^[:{}{}]+$",
                NCNAME_START_CHAR, NCNAME_CHAR_ADDITIONS
            ))
            .expect("Invalid regex")
        })
    }

    pub(crate) fn cast_to_name(self) -> error::Result<atomic::Atomic> {
        // 	Name	   ::=   	NameStartChar (NameChar)*
        self.cast_to_regex(atomic::StringType::Name, &NAME_REGEX, || {
            // we have to add the colon for NAME_START_CHAR / NAME_CHAR
            Regex::new(&format!(
                "^[:{}][:{}{}]*$",
                NCNAME_START_CHAR, NCNAME_START_CHAR, NCNAME_CHAR_ADDITIONS
            ))
            .expect("Invalid regex")
        })
    }

    fn cast_to_ncname_helper(
        self,
        string_type: atomic::StringType,
    ) -> error::Result<atomic::Atomic> {
        // https://www.w3.org/TR/xml-names11/#NT-NCName
        // 	NCName	   ::=   	NCNameStartChar NCNameChar*
        // 	NCNameChar	   ::=   	NameChar - ':'
        //	NCNameStartChar	   ::=   	NameStartChar - ':'
        self.cast_to_regex(string_type, &NC_NAME_REGEX, || {
            Regex::new(&format!(
                "^[{}][{}{}]*$",
                NCNAME_START_CHAR, NCNAME_START_CHAR, NCNAME_CHAR_ADDITIONS
            ))
            .expect("Invalid regex")
        })
    }

    pub(crate) fn cast_to_ncname(self) -> error::Result<atomic::Atomic> {
        self.cast_to_ncname_helper(atomic::StringType::NCName)
    }

    pub(crate) fn cast_to_id(self) -> error::Result<atomic::Atomic> {
        self.cast_to_ncname_helper(atomic::StringType::ID)
    }

    pub(crate) fn cast_to_idref(self) -> error::Result<atomic::Atomic> {
        self.cast_to_ncname_helper(atomic::StringType::IDREF)
    }

    pub(crate) fn cast_to_entity(self) -> error::Result<atomic::Atomic> {
        // https://www.w3.org/TR/xpath-functions-31/#casting-to-ENTITY
        // we don't need to check whether it matches unparsed entities
        self.cast_to_ncname_helper(atomic::StringType::ENTITY)
    }

    pub(crate) fn cast_to_qname(
        self,
        dynamic_context: &context::DynamicContext,
    ) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::QName(_) => Ok(self.clone()),
            atomic::Atomic::String(_, s) | atomic::Atomic::Untyped(s) => {
                // https://www.w3.org/TR/xpath-functions-31/#constructor-qname-notation
                let namespaces = dynamic_context.static_context.namespaces;
                let name = ast::Name::parse(&s, namespaces);
                match name {
                    Ok(name) => {
                        let name = name.value;
                        if name.has_namespace_without_prefix() {
                            // we deliberately do not parse Qualified names, as they are not
                            // legal for xs:QName
                            Err(error::Error::FORG0001)
                        } else {
                            Ok(atomic::Atomic::QName(Rc::new(name.with_default_namespace(
                                namespaces.default_element_namespace(),
                            ))))
                        }
                    }
                    // TODO: We really want to distinguish between parse errors
                    // and namespace lookup errors, which should be a FONS0004 error
                    // This requires the parser to be modified so it retains that
                    // information.
                    Err(_) => Err(error::Error::FORG0001),
                }
            }
            _ => Err(error::Error::Type),
        }
    }
}
