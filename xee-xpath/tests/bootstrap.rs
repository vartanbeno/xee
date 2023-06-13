// ensure we have just enough xpath to help us read the test suite more easily
use xee_xpath::evaluate;

const NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

const ROOT_FIXTURE: &str = include_str!("fixtures/root.xml");

#[test]
fn test_test_cases() {
    let items = evaluate(ROOT_FIXTURE, "/test-set/test-case", Some(NS)).unwrap();
    assert_eq!(items.len(), 38);
}

#[test]
fn test_specific_attribute() {
    let items = evaluate(
        ROOT_FIXTURE,
        "/test-set/test-case[@name eq 'fn-root-1']",
        Some(NS),
    )
    .unwrap();
    assert_eq!(items.len(), 1);
}
