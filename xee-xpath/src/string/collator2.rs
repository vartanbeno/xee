use std::borrow::Cow;
use std::rc::Rc;
use std::{collections::hash_map::Entry, str::FromStr};

use ahash::{HashMap, HashMapExt};
use icu::collator::{BackwardSecondLevel, CaseLevel, Numeric};
use icu::{
    collator::{self, AlternateHandling, CaseFirst, Collator, MaxVariable, Strength},
    locid::Locale,
};
use icu_provider_adapters::{either::EitherProvider, fallback::LocaleFallbackProvider};
use icu_provider_blob::BlobDataProvider;
use url::Url;

use crate::{error, DynamicContext, Error};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CollatorQuery {
    pub(crate) fallback: bool,
    pub(crate) lang: Option<String>,
    pub(crate) strength: Strength,
    pub(crate) max_variable: MaxVariable,
    pub(crate) alternate: AlternateHandling,
    pub(crate) backwards: bool,
    pub(crate) normalization: bool,
    pub(crate) case_level: bool,
    pub(crate) case_first: CaseFirst,
    pub(crate) numeric: bool,
    // both version and reorder are not supported at this point, as
    // they don't seem to have equivalents in icu4x
}

impl From<CollatorQuery> for collator::CollatorOptions {
    fn from(query: CollatorQuery) -> Self {
        let mut options = collator::CollatorOptions::new();
        options.strength = Some(query.strength);
        options.alternate_handling = Some(query.alternate);
        options.case_first = Some(query.case_first);
        options.max_variable = Some(query.max_variable);
        options.case_level = Some(if query.case_level {
            CaseLevel::On
        } else {
            CaseLevel::Off
        });
        options.numeric = Some(if query.numeric {
            Numeric::On
        } else {
            Numeric::Off
        });
        options.backward_second_level = Some({
            if query.backwards {
                BackwardSecondLevel::On
            } else {
                BackwardSecondLevel::Off
            }
        });
        options
    }
}

impl CollatorQuery {
    fn from_url(url: &Url) -> error::Result<Self> {
        // this should let the last query parameter win in case of duplicates
        let query = url.query_pairs().collect::<HashMap<_, _>>();
        Self::from_query_hashmap(query)
    }

    fn from_query_hashmap(mut query: HashMap<Cow<str>, Cow<str>>) -> error::Result<Self> {
        let fallback = yes_no_query_parameter(query.remove("fallback"), true).unwrap_or(true);
        let lang = query.remove("lang");
        // version: ignore
        let strength = strength_query_parameter(query.remove("strength"));
        let max_variable = max_variable_query_parameter(query.remove("maxVariable"));
        let alternate = alternate_query_parameter(query.remove("alternate"));
        let backwards = yes_no_query_parameter(query.remove("backwards"), false);
        let normalization = yes_no_query_parameter(query.remove("normalization"), false);
        let case_level = yes_no_query_parameter(query.remove("caseLevel"), false);
        let case_first = case_first_query_parameter(query.remove("caseFirst"));
        let numeric = yes_no_query_parameter(query.remove("numeric"), false);
        // reorder: ignore

        if fallback {
            Ok(CollatorQuery {
                fallback,
                lang: lang.map(|s| s.to_string()),
                strength: strength.unwrap_or(Strength::Tertiary),
                max_variable: max_variable.unwrap_or(MaxVariable::Punctuation),
                alternate: alternate.unwrap_or(AlternateHandling::NonIgnorable),
                backwards: backwards.unwrap_or(false),
                normalization: normalization.unwrap_or(false),
                case_level: case_level.unwrap_or(false),
                case_first: case_first.unwrap_or(CaseFirst::Off),
                numeric: numeric.unwrap_or(false),
            })
        } else {
            // if any parameters are left, fail with an error
            if !query.is_empty() {
                return Err(error::Error::FOCH0002);
            }
            Ok(CollatorQuery {
                fallback,
                lang: lang.map(|s| s.to_string()),
                strength: strength.ok_or(Error::FOCH0002)?,
                max_variable: max_variable.ok_or(Error::FOCH0002)?,
                alternate: alternate.ok_or(Error::FOCH0002)?,
                backwards: backwards.ok_or(Error::FOCH0002)?,
                normalization: normalization.ok_or(Error::FOCH0002)?,
                case_level: case_level.ok_or(Error::FOCH0002)?,
                case_first: case_first.ok_or(Error::FOCH0002)?,
                numeric: numeric.ok_or(Error::FOCH0002)?,
            })
        }
    }
}

#[derive(Debug)]
pub(crate) enum Collation {
    // 5.3.2
    CodePoint,
    // 5.3.3
    Uca(Box<Collator>),
    // 5.3.4
    HtmlAscii,
}

impl Collation {
    fn new(provider: BlobDataProvider, base_uri: Option<&Url>, uri: &str) -> error::Result<Self> {
        let url = if let Some(base_uri) = base_uri {
            base_uri.join(uri).map_err(|_| error::Error::FOCH0002)?
        } else {
            Url::parse(uri).map_err(|_| error::Error::FOCH0002)?
        };
        if url.scheme() != "http" || url.host_str() != Some("www.w3.org") {
            return Err(error::Error::FOCH0002);
        }
        let path = url.path();
        Ok(match path {
            "/xpath-functions/collation/codepoint" => Collation::CodePoint,
            "/2013/collation/UCA" => {
                let collator_query = CollatorQuery::from_url(&url)?;
                Collation::Uca(Box::new(Self::uca_collator(provider, collator_query)?))
            }
            "/xpath-functions/collation/html-ascii-case-insensitive" => Collation::HtmlAscii,
            _ => return Err(error::Error::FOCH0002),
        })
    }

    fn uca_collator(
        provider: BlobDataProvider,
        collator_query: CollatorQuery,
    ) -> error::Result<Collator> {
        let provider = if collator_query.fallback {
            EitherProvider::A(
                LocaleFallbackProvider::try_new_with_buffer_provider(provider).unwrap(),
            )
        } else {
            EitherProvider::B(provider)
        };
        let locale = if let Some(lang) = &collator_query.lang {
            match Locale::try_from_bytes(lang.as_bytes()) {
                Ok(locale) => locale,
                Err(_) => {
                    if collator_query.fallback {
                        // in case of fallback, get a locale anyway
                        Locale::UND
                    } else {
                        return Err(error::Error::FOCH0002);
                    }
                }
            }
        } else {
            // this is implementation defined according to the XPath spec
            // we choose to use the undefined locale
            Locale::UND
        };

        let locale = locale.into();
        let options = collator_query.into();

        Collator::try_new_with_buffer_provider(&provider, &locale, options)
            .map_err(|_| error::Error::FOCH0002)
    }
}

#[derive(Debug)]
pub(crate) struct Collations {
    collations: HashMap<String, Rc<Collation>>,
}

impl Collations {
    pub(crate) fn new() -> Self {
        Self {
            collations: HashMap::new(),
        }
    }

    pub(crate) fn load(
        &mut self,
        provider: BlobDataProvider,
        base_uri: Option<&Url>,
        uri: &str,
    ) -> error::Result<Rc<Collation>> {
        // try to find cached collator. we cache by uri
        match self.collations.entry(uri.to_string()) {
            Entry::Occupied(entry) => Ok(entry.into_mut().clone()),
            Entry::Vacant(entry) => {
                let collation = Collation::new(provider, base_uri, uri)?;
                Ok(entry.insert(Rc::new(collation)).clone())
            }
        }
    }
}

pub(crate) fn provider() -> BlobDataProvider {
    let blob = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/buffer_data.postcard",))
        .expect("Pre-computed postcard buffer should exist");

    BlobDataProvider::try_new_from_blob(blob.into_boxed_slice())
        .expect("Deserialization should succeed")
}

fn yes_no_query_parameter(value: Option<Cow<str>>, default: bool) -> Option<bool> {
    match value {
        Some(value) => match value.as_ref() {
            "yes" => Some(true),
            "no" => Some(false),
            _ => None,
        },
        None => Some(default),
    }
}

fn strength_query_parameter(value: Option<Cow<str>>) -> Option<Strength> {
    match value {
        Some(value) => match value.as_ref() {
            "primary" | "1" => Some(Strength::Primary),
            "secondary" | "2" => Some(Strength::Secondary),
            "tertiary" | "3" => Some(Strength::Tertiary),
            "quaternary" | "4" => Some(Strength::Quaternary),
            "identical" | "5" => Some(Strength::Identical),
            _ => None,
        },
        None => Some(Strength::Tertiary),
    }
}

fn max_variable_query_parameter(value: Option<Cow<str>>) -> Option<MaxVariable> {
    match value {
        Some(value) => match value.as_ref() {
            "space" => Some(MaxVariable::Space),
            "punct" => Some(MaxVariable::Punctuation),
            "symbol" => Some(MaxVariable::Symbol),
            "currency" => Some(MaxVariable::Currency),
            _ => None,
        },
        None => Some(MaxVariable::Punctuation),
    }
}

fn alternate_query_parameter(value: Option<Cow<str>>) -> Option<AlternateHandling> {
    match value {
        Some(value) => match value.as_ref() {
            "non-ignorable" => Some(AlternateHandling::NonIgnorable),
            "shifted" => Some(AlternateHandling::Shifted),
            // blanked not supported by icu4x
            _ => None,
        },
        None => Some(AlternateHandling::NonIgnorable),
    }
}

fn case_first_query_parameter(value: Option<Cow<str>>) -> Option<CaseFirst> {
    match value {
        Some(value) => match value.as_ref() {
            "upper" => Some(CaseFirst::UpperFirst),
            "lower" => Some(CaseFirst::LowerFirst),
            _ => None,
        },
        None => Some(CaseFirst::Off),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // these tests verify the behavior to the url crate

    #[test]
    fn test_base_url() {
        let base = Url::parse("http://www.w3.org/").unwrap();
        let path = "/xpath-functions/collation/codepoint";
        let url = base.join(path).unwrap();
        assert_eq!(
            url.as_str(),
            "http://www.w3.org/xpath-functions/collation/codepoint"
        );
    }

    #[test]
    fn test_base_url_with_full_url() {
        let base = Url::parse("http://www.another.org/").unwrap();
        let path = "http://www.w3.org/xpath-functions/collation/codepoint";
        let url = base.join(path).unwrap();
        assert_eq!(
            url.as_str(),
            "http://www.w3.org/xpath-functions/collation/codepoint"
        );
    }

    #[test]
    fn test_base_url_with_just_qs() {
        let base = Url::parse("http://www.w3.org/2013/collation/UCA").unwrap();
        let path = "?lang=foo";
        let url = base.join(path).unwrap();
        assert_eq!(
            url.as_str(),
            "http://www.w3.org/2013/collation/UCA?lang=foo"
        );
    }

    #[test]
    fn test_deserialize_query_string() {
        let url = "http://www.w3.org/2013/collation/UCA?fallback=yes&lang=en&strength=primary&max_variable=punctuation&alternate=non-ignorable&backwards=no&normalization=no&caseLevel=no&caseFirst=upper&numeric=no";
        let query = CollatorQuery::from_url(&Url::parse(url).unwrap()).unwrap();
        assert_eq!(
            query,
            CollatorQuery {
                fallback: true,
                lang: Some("en".to_string()),
                strength: Strength::Primary,
                max_variable: MaxVariable::Punctuation,
                alternate: AlternateHandling::NonIgnorable,
                backwards: false,
                normalization: false,
                case_level: false,
                case_first: CaseFirst::UpperFirst,
                numeric: false,
            }
        )
    }

    #[test]
    fn test_deserialize_query_string_default() {
        let url = "http://www.w3.org/2013/collation/UCA?lang=en";
        let query = CollatorQuery::from_url(&Url::parse(url).unwrap()).unwrap();
        assert_eq!(
            query,
            CollatorQuery {
                fallback: true,
                lang: Some("en".to_string()),
                strength: Strength::Tertiary,
                max_variable: MaxVariable::Punctuation,
                alternate: AlternateHandling::NonIgnorable,
                backwards: false,
                normalization: false,
                case_level: false,
                case_first: CaseFirst::Off,
                numeric: false,
            }
        )
    }

    #[test]
    fn test_deserialize_query_no_fallback_reject_wrong_value() {
        let url = "http://www.w3.org/2013/collation/UCA?lang=en&fallback=no&strength=nonsense";
        assert!(CollatorQuery::from_url(&Url::parse(url).unwrap()).is_err());
    }

    #[test]
    fn test_deserialize_query_no_fallback_reject_extra_param() {
        let url = "http://www.w3.org/2013/collation/UCA?lang=en&fallback=no&extra=nonsense";
        assert!(CollatorQuery::from_url(&Url::parse(url).unwrap()).is_err());
    }

    #[test]
    fn test_deserialize_query_yes_fallback_default_for_wrong_value() {
        let url = "http://www.w3.org/2013/collation/UCA?lang=en&fallback=yes&strength=nonsense";
        let query = CollatorQuery::from_url(&Url::parse(url).unwrap()).unwrap();
        assert_eq!(
            query,
            CollatorQuery {
                fallback: true,
                lang: Some("en".to_string()),
                strength: Strength::Tertiary,
                max_variable: MaxVariable::Punctuation,
                alternate: AlternateHandling::NonIgnorable,
                backwards: false,
                normalization: false,
                case_level: false,
                case_first: CaseFirst::Off,
                numeric: false,
            }
        )
    }

    #[test]
    fn test_deserialize_query_yes_fallback_ignore_extra_parameter() {
        let url = "http://www.w3.org/2013/collation/UCA?lang=en&fallback=yes&extra=nonsense";
        let query = CollatorQuery::from_url(&Url::parse(url).unwrap()).unwrap();
        assert_eq!(
            query,
            CollatorQuery {
                fallback: true,
                lang: Some("en".to_string()),
                strength: Strength::Tertiary,
                max_variable: MaxVariable::Punctuation,
                alternate: AlternateHandling::NonIgnorable,
                backwards: false,
                normalization: false,
                case_level: false,
                case_first: CaseFirst::Off,
                numeric: false,
            }
        )
    }

    #[test]
    fn test_load_uri_collation() {
        let provider = provider();
        let mut collations = Collations::new();
        let collation = collations.load(
            provider,
            None,
            "http://www.w3.org/2013/collation/UCA?lang=se&fallback=no",
        );
        assert!(collation.is_ok());
    }

    #[test]
    fn test_load_codepoint_collation() {
        let provider = provider();
        let mut collations = Collations::new();
        let collation = collations.load(
            provider,
            None,
            "http://www.w3.org/xpath-functions/collation/codepoint",
        );
        assert!(collation.is_ok());
    }

    #[test]
    fn test_load_html_ascii_collation() {
        let provider = provider();
        let mut collations = Collations::new();
        let collation = collations.load(
            provider,
            None,
            "http://www.w3.org/xpath-functions/collation/html-ascii-case-insensitive",
        );
        assert!(collation.is_ok());
    }

    // #[test]
    // fn test_load_collator_with_fallback() {
    //     let provider = provider();
    //     let mut collators = Collators::new();
    //     // fallback is the default, but make it explicit
    //     let query: CollatorQuery = "lang=en-US&fallback=yes".parse().unwrap();
    //     let collator = collators.load(provider, &query);
    //     assert!(collator.is_some());
    // }

    // #[test]
    // fn test_load_collator_without_fails() {
    //     let provider = provider();
    //     let mut collators = Collators::new();
    //     // without fallback we can't find en-US
    //     let query: CollatorQuery = "lang=en-US&fallback=no".parse().unwrap();
    //     let collator = collators.load(provider, &query);
    //     assert!(collator.is_none());
    // }
}
