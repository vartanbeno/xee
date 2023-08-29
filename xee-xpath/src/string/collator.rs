use ahash::{HashMap, HashMapExt};
use icu::collator::{self, Collator};
use icu::locid::Locale;
use icu_provider_blob::BlobDataProvider;
use serde::Deserialize;
use serde_querystring::{from_str, ParseMode};
use std::collections::hash_map::Entry;
use std::str::FromStr;

// annoying re-implementations of various icu4x types because we need
// hashing

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Strength {
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
    Identical,
}

impl From<Strength> for collator::Strength {
    fn from(strength: Strength) -> Self {
        match strength {
            Strength::Primary => Self::Primary,
            Strength::Secondary => Self::Secondary,
            Strength::Tertiary => Self::Tertiary,
            Strength::Quaternary => Self::Quaternary,
            Strength::Identical => Self::Identical,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum MaxVariable {
    Punctuation,
    Symbol,
    Currency,
    Space,
}

impl From<MaxVariable> for collator::MaxVariable {
    fn from(max_variable: MaxVariable) -> Self {
        match max_variable {
            MaxVariable::Punctuation => Self::Punctuation,
            MaxVariable::Symbol => Self::Symbol,
            MaxVariable::Currency => Self::Currency,
            MaxVariable::Space => Self::Space,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum AlternateHandling {
    NonIgnorable,
    Shifted,
}

impl From<AlternateHandling> for collator::AlternateHandling {
    fn from(alternate: AlternateHandling) -> Self {
        match alternate {
            AlternateHandling::NonIgnorable => Self::NonIgnorable,
            AlternateHandling::Shifted => Self::Shifted,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CaseFirst {
    Off,
    Lower,
    Upper,
}

impl From<CaseFirst> for collator::CaseFirst {
    fn from(case_first: CaseFirst) -> Self {
        match case_first {
            CaseFirst::Off => Self::Off,
            CaseFirst::Lower => Self::LowerFirst,
            CaseFirst::Upper => Self::UpperFirst,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum YesNo {
    Yes,
    No,
}

impl From<YesNo> for collator::CaseLevel {
    fn from(yes_no: YesNo) -> Self {
        match yes_no {
            YesNo::Yes => collator::CaseLevel::On,
            YesNo::No => collator::CaseLevel::Off,
        }
    }
}

impl From<YesNo> for collator::Numeric {
    fn from(yes_no: YesNo) -> Self {
        match yes_no {
            YesNo::Yes => collator::Numeric::On,
            YesNo::No => collator::Numeric::Off,
        }
    }
}

impl From<YesNo> for collator::BackwardSecondLevel {
    fn from(yes_no: YesNo) -> Self {
        match yes_no {
            YesNo::Yes => collator::BackwardSecondLevel::On,
            YesNo::No => collator::BackwardSecondLevel::Off,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(default)]
pub(crate) struct CollatorQuery {
    fallback: YesNo,
    lang: Option<String>,
    strength: Strength,
    max_variable: MaxVariable,
    alternate: AlternateHandling,
    backwards: YesNo,
    normalization: YesNo,
    case_level: YesNo,
    case_first: CaseFirst,
    numeric: YesNo,
    // both version and reorder are not supported at this point, as
    // they don't seem to have equivalents in icu4x
}

impl Default for CollatorQuery {
    fn default() -> Self {
        Self {
            fallback: YesNo::Yes,
            lang: None, // implementation-defined
            strength: Strength::Tertiary,
            max_variable: MaxVariable::Punctuation,
            alternate: AlternateHandling::NonIgnorable,
            backwards: YesNo::No,
            normalization: YesNo::No,
            case_level: YesNo::No,
            case_first: CaseFirst::Off, // implementation-defined
            numeric: YesNo::No,
        }
    }
}

impl FromStr for CollatorQuery {
    type Err = serde_querystring::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        from_str(s, ParseMode::UrlEncoded)
    }
}

impl From<&CollatorQuery> for collator::CollatorOptions {
    fn from(query: &CollatorQuery) -> Self {
        let mut options = collator::CollatorOptions::new();
        options.strength = Some(query.strength.into());
        options.alternate_handling = Some(query.alternate.into());
        options.case_first = Some(query.case_first.into());
        options.max_variable = Some(query.max_variable.into());
        options.case_level = Some(query.case_level.into());
        options.numeric = Some(query.numeric.into());
        options.backward_second_level = Some(query.backwards.into());
        options
    }
}

pub(crate) struct Collators {
    collators: HashMap<CollatorQuery, Option<Collator>>,
}

impl Collators {
    pub(crate) fn new() -> Self {
        Self {
            collators: HashMap::new(),
        }
    }

    pub(crate) fn load(
        &mut self,
        provider: &BlobDataProvider,
        query: &CollatorQuery,
    ) -> Option<&Collator> {
        // try to find cached collator.
        match self.collators.entry(query.clone()) {
            Entry::Occupied(entry) => entry.into_mut().as_ref(),
            Entry::Vacant(entry) => {
                let locale = if let Some(lang) = &query.lang {
                    Locale::try_from_bytes(lang.as_bytes()).ok()
                } else {
                    Some(Locale::UND)
                };
                let collator = if let Some(locale) = locale {
                    let locale = locale.into();
                    let options = query.into();
                    Collator::try_new_with_buffer_provider(provider, &locale, options).ok()
                } else {
                    None
                };
                entry.insert(collator).as_ref()
            }
        }
    }
}

fn provider() -> BlobDataProvider {
    let blob = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/buffer_data.postcard",))
        .expect("Pre-computed postcard buffer should exist");

    BlobDataProvider::try_new_from_blob(blob.into_boxed_slice())
        .expect("Deserialization should succeed")
}

// fn collabor(provider: &BlobDataProvider, locale: &Locale) -> Collator {
//     Collator::try_new_with_buffer_provider(provider, &locale.into(), options).unwrap()
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_query_string() {
        let qs = "fallback=yes&lang=en&strength=primary&max_variable=punctuation&alternate=non-ignorable&backwards=no&normalization=no&case_level=no&case_first=upper&numeric=no";
        let query: CollatorQuery = qs.parse().unwrap();
        assert_eq!(
            query,
            CollatorQuery {
                fallback: YesNo::Yes,
                lang: Some("en".to_string()),
                strength: Strength::Primary,
                max_variable: MaxVariable::Punctuation,
                alternate: AlternateHandling::NonIgnorable,
                backwards: YesNo::No,
                normalization: YesNo::No,
                case_level: YesNo::No,
                case_first: CaseFirst::Upper,
                numeric: YesNo::No,
            }
        )
    }

    #[test]
    fn test_deserialize_query_string_default() {
        let qs = "lang=en";
        let query: CollatorQuery = qs.parse().unwrap();
        assert_eq!(
            query,
            CollatorQuery {
                fallback: YesNo::Yes,
                lang: Some("en".to_string()),
                strength: Strength::Tertiary,
                max_variable: MaxVariable::Punctuation,
                alternate: AlternateHandling::NonIgnorable,
                backwards: YesNo::No,
                normalization: YesNo::No,
                case_level: YesNo::No,
                case_first: CaseFirst::Off,
                numeric: YesNo::No,
            }
        )
    }
}
