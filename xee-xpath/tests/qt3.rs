use xee_qt::Tests;

#[test]
fn test_root() {
    Tests::all("fn/root").tolerate_wrong_error().run()
    // test_all("fn/root")
}

#[test]
fn test_if_expr() {
    Tests::all("prod/IfExpr")
        .exclude("CondExpr017 CondExpr018 CondExpr022 K-CondExpr-3 K-CondExpr-4 K-CondExpr-5 K-CondExpr-6 K-CondExpr-7 K-CondExpr-8 K-CondExpr-9 K-CondExpr-10 K-CondExpr-11 K-CondExpr-12 cbcl-*")
        .run()
}
