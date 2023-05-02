// ensure we have just enough xpath to help us read the test suite more easily
use xee_xpath::evaluate;

const NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

const ROOT_FIXTURE: &str = include_str!("fixtures/root.xml");

#[test]
fn test_test_cases() {
    let sv = evaluate(ROOT_FIXTURE, "/test-set/test-case", Some(NS));
    assert_eq!(sv.as_sequence().unwrap().borrow().len(), 38);
}
