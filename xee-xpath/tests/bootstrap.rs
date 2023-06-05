// ensure we have just enough xpath to help us read the test suite more easily
use xee_xpath::{evaluate, Sequence};

const NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

const ROOT_FIXTURE: &str = include_str!("fixtures/root.xml");

#[test]
fn test_test_cases() {
    let sv = evaluate(ROOT_FIXTURE, "/test-set/test-case", Some(NS)).unwrap();
    let seq: Sequence = sv.try_into().unwrap();
    assert_eq!(seq.borrow().len(), 38);
}

#[test]
fn test_specific_attribute() {
    let sv = evaluate(
        ROOT_FIXTURE,
        "/test-set/test-case[@name eq 'fn-root-1']",
        Some(NS),
    )
    .unwrap();
    let seq: Sequence = sv.try_into().unwrap();
    assert_eq!(seq.borrow().len(), 1);
}
