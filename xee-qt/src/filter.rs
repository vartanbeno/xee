use std::str::FromStr;

use fxhash::{FxHashMap, FxHashSet};

use crate::{outcome::TestSetOutcomes, qt};

trait TestFilter {
    fn is_included(&self, test_set: &qt::TestSet, test_case: &qt::TestCase) -> bool;
}

pub(crate) struct IncludeAllFilter {}

impl TestFilter for IncludeAllFilter {
    fn is_included(&self, _test_set: &qt::TestSet, _test_case: &qt::TestCase) -> bool {
        true
    }
}

struct ExcludedName {
    name: String,
    comment: Option<String>,
}

pub(crate) struct ExcludedNamesFilter {
    names: FxHashMap<String, FxHashSet<String>>,
    comments: FxHashMap<String, FxHashMap<String, String>>,
}

impl TestFilter for ExcludedNamesFilter {
    fn is_included(&self, test_set: &qt::TestSet, test_case: &qt::TestCase) -> bool {
        let test_set_name = &test_set.name;
        let test_case_name = &test_case.name;
        if let Some(excluded_names) = self.names.get(test_set_name) {
            !excluded_names.contains(test_case_name)
        } else {
            true
        }
    }
}

// The format to load exclude names filters is
// = test_set_name
// # any comment
// excluded_test_case_name
// another # with a comment
impl ExcludedNamesFilter {
    pub(crate) fn new() -> Self {
        Self {
            names: FxHashMap::default(),
            comments: FxHashMap::default(),
        }
    }

    pub(crate) fn update_with_test_set_outcomes(&mut self, test_set_outcomes: &TestSetOutcomes) {
        // remove the previous entry
        self.names.remove(&test_set_outcomes.test_set_name);
        let failing_names: FxHashSet<String> =
            test_set_outcomes.failing_names().into_iter().collect();
        self.names
            .insert(test_set_outcomes.test_set_name.clone(), failing_names);
        // there may be extra comments left for names that aren't relevant anymore,
        // but that's okay: we won't serialize them later
    }

    pub(crate) fn update_with_catalog_outcomes(&mut self, catalog_outcomes: &[TestSetOutcomes]) {
        for test_set_outcomes in catalog_outcomes {
            self.update_with_test_set_outcomes(test_set_outcomes);
        }
    }
}

impl FromStr for ExcludedNamesFilter {
    type Err = String;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut names = FxHashMap::default();
        let mut comments = FxHashMap::default();
        let mut test_set_name: Option<String> = None;
        let mut test_case_names = FxHashSet::default();
        let mut test_case_comments = FxHashMap::default();
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with('=') {
                if let Some(test_set_name) = test_set_name {
                    names.insert(test_set_name.clone(), test_case_names);
                    comments.insert(test_set_name, test_case_comments);
                }
                test_set_name = Some(line.strip_prefix('=').unwrap().trim().to_string());
                test_case_names = FxHashSet::default();
                test_case_comments = FxHashMap::default();
            } else if !line.is_empty() {
                let mut parts = line.split('#');
                let test_case_name = parts.next().unwrap();
                let test_case_name = test_case_name.trim();
                let comment = parts.next().map(|s| s.trim().to_string());
                test_case_names.insert(test_case_name.to_string());
                if let Some(comment) = comment {
                    test_case_comments.insert(test_case_name.to_string(), comment);
                }
            }
        }
        if let Some(test_set_name) = test_set_name {
            names.insert(test_set_name.clone(), test_case_names);
            comments.insert(test_set_name, test_case_comments);
        }
        Ok(Self { names, comments })
    }
}

impl ToString for ExcludedNamesFilter {
    fn to_string(&self) -> String {
        let mut result = String::new();
        let mut sorted_names = self.names.keys().collect::<Vec<_>>();
        sorted_names.sort();
        for test_set_name in sorted_names {
            let excluded_names = self.names.get(test_set_name).unwrap();
            let comments = self.comments.get(test_set_name);
            result.push_str(&format!("= {}\n", test_set_name));
            let mut sorted_excluded_names = excluded_names.iter().collect::<Vec<_>>();
            sorted_excluded_names.sort();
            for excluded_name in sorted_excluded_names {
                result.push_str(&excluded_name.to_string());
                if let Some(comments) = comments {
                    let comment = comments.get(excluded_name);
                    if let Some(comment) = comment {
                        let comment = comment.trim();
                        if !comment.is_empty() {
                            result.push_str(&format!(" # {}", comment));
                        }
                    }
                }
                result.push('\n');
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let source = r#"
= test_set_1
test_case_1
test_case_2 # with a comment
= test_set_2
test_case_3
"#;
        let filter: ExcludedNamesFilter = source.parse().unwrap();
        assert_eq!(filter.names.len(), 2);
        assert!(filter.names.contains_key("test_set_1"));
        assert!(filter.names.contains_key("test_set_2"));
        assert!(filter
            .names
            .get("test_set_1")
            .unwrap()
            .contains("test_case_1"));
        assert!(filter
            .names
            .get("test_set_1")
            .unwrap()
            .contains("test_case_2"));
        assert!(filter
            .names
            .get("test_set_2")
            .unwrap()
            .contains("test_case_3"));
        assert_eq!(filter.comments.len(), 2);
        assert!(filter.comments.contains_key("test_set_1"));
        assert!(filter.comments.contains_key("test_set_2"));
        assert_eq!(
            filter
                .comments
                .get("test_set_1")
                .unwrap()
                .get("test_case_2")
                .unwrap(),
            "with a comment"
        );
    }

    #[test]
    fn test_parse_serialize() {
        let source = r#"
= test_set_1
test_case_1
test_case_2 # with a comment
= test_set_2
test_case_3
"#
        .trim_start();
        let filter: ExcludedNamesFilter = source.parse().unwrap();
        let serialized = filter.to_string();
        assert_eq!(source, serialized);
    }
}
