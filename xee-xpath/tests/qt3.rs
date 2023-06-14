use xee_qt::Tests;

#[test]
fn test_root() {
    Tests::all("fn/root").tolerate_wrong_error().run()
    // test_all("fn/root")
}

#[test]
fn test_something() {
    Tests::new("prod/IfExpr")
        .include("CondExpr008 CondExpr009 CondExpr010 CondExpr014 CondExpr015")
        .run()
}
