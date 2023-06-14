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

#[test]
fn test_for_clause() {
    // TODO: investigate why these tests run slowly
    // fsx is a big file but it's not THAT big, is there a
    // nested loop in there or something?
    Tests::all("prod/ForClause")
        .bug("ForExpr012")
        .bug("ForExpr015")
        .bug("ForExpr016")
        .exclude("ForExpr013")
        .exclude("K-ForExprWithout-5 K-ForExprWithout-10 K-ForExprWithout-11 K-ForExprWithout-18 K-ForExprWithout-20 K-ForExprWithout-22 K-ForExprWithout-23 K-ForExprWithout-24 K-ForExprWithout-25")
        .exclude("K-ForExprWithout-27 K-ForExprWithout-30 K-ForExprWithout-33 K-ForExprWithout-33 K-ForExprWithout-47 K-ForExprWithout-49 K-ForExprWithout-55 K-ForExprWithout-56")
        .exclude("K2-ForExprWithout-7 K2-ForExprWithout-40 K2-ForExprWithout-41 K2-ForExprWithout-45")
        .run()
}

#[test]
fn test_let_clause() {
    Tests::all("prod/LetClause")
        // The 2.0 is represented as 2.0 instead of 2
        .bug("LetExpr015")
        .exclude("LetExpr004 LetExpr005 LetExpr006 LetExpr013")
        .run()
}
