use xee_qt::Tests;

#[test]
fn test_root() {
    Tests::all("fn/root").run()
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
    Tests::all("prod/LetClause").run()
}

#[test]
fn test_inline_function_expr() {
    Tests::all("prod/InlineFunctionExpr")
        // function-name, function-arity
        .exclude("inline-fn-021 inline-fn-022 inline-fn-025")
        // treat functions as objects
        .bug("inline-fn-028 inline-fn-029 inline-fn-030 inline-fn-031 inline-fn-036")
        .exclude("inline-fn-032 inline-fn-033")
        // sum function
        .exclude("inline-fn-004")
        // function type `function(*)`
        .exclude("inline-fn-014")
        .run()
}

#[test]
fn test_string_join() {
    Tests::all("fn/string-join").run()
}

#[test]
fn test_string_length() {
    // tolerate errors
    // really: X
    // XPDY0002 for fn-string-length-18
    // FOTY00313 for fn-string-length-21

    Tests::all("fn/string-length")
        .tolerate_wrong_error()
        // we get an unexpected type error here; apparently
        // the context item is first cast to a string,
        // whereas an explicit argument is not. I
        // need to understand the rule behind this still.
        // https://www.w3.org/TR/xpath-functions-31/#func-string-length
        .bug("fn-string-length-24")
        // K-StringLengthFunc-3 and K-StringLengthFunc-6 need "instance of"
        // support
        .exclude("K-StringLengthFunc-3 K-StringLengthFunc-6")
        .run()
}

#[test]
fn test_concat() {
    Tests::all("fn/concat")
        // fn:upper-case
        .exclude("fn-concat-9")
        .run()
}

#[test]
fn test_unary_minus() {
    Tests::all("op/numeric-unary-minus")
        // this depends on fn:floor
        .exclude("K-NumericUnaryMinus-15")
        .run()
}

#[test]
fn test_cast() {
    Tests::none("prod/CastExpr")
        .include("casthc*")
        // canonical float representation rules
        .exclude("casthc17 casthc18")
        // casting across different date/time types not supported yet
        .exclude("casthc30 casthc31 casthc33")
        .run()
}

#[test]
fn test_castable() {
    Tests::none("prod/CastableExpr")
        .include("CastableAs01? CastableAs02?")
        .exclude("CastableAs027 CastableAs028 CastableAs029")
        .run()
}

#[test]
fn test_instance_of() {
    Tests::all("prod/InstanceofExpr")
        .exclude("instanceof? instanceof1? instanceof2? instanceof3? instanceof6? instanceof7? instanceof111 instanceof121 instanceof12? instanceof13? K*-SeqExprInstanceOf*")
        .run()
}

#[test]
fn test_xs_base64_binary() {
    Tests::all("xs/base64Binary")
        // needs implementation of codepoints-to-string
        .exclude("base64-908 base64-909")
        .run()
}

#[test]
fn test_xs_hex_binary() {
    Tests::all("xs/hexBinary").run()
}

#[test]
fn test_xs_double() {
    Tests::all("xs/double").run()
}

#[test]
fn test_xs_float() {
    Tests::all("xs/float").run()
}

#[test]
fn test_xs_normalized_string() {
    Tests::all("xs/normalizedString").run()
}

#[test]
fn test_xs_token() {
    Tests::all("xs/token").run()
}

#[test]
fn test_xs_any_uri() {
    Tests::all("xs/anyURI").run()
}

#[test]
fn test_op_numeric_add() {
    Tests::all("op/numeric-add")
        // a bunch of tests use the remove() function or the subsequence function
        .exclude("K-NumericAdd-5? K-NumericAdd-61 K-NumericAdd-62 K-NumericAdd-63 K-NumericAdd-64")
        .run()
}

#[test]
fn test_instance_of_expr() {
    Tests::all("prod/InstanceofExpr")
        // xs:NMTOKENS
        .exclude("instanceof111")
        // function types, higher order functions
        .exclude(
            "instanceof12? instanceof130 instanceof131 instanceof132 instanceof133 instanceof134",
        )
        // various non-types which should error out
        .exclude("K-SeqExprInstanceOf-49 K-SeqExprInstanceOf-5?")
        .run()
}

#[test]
fn test_appendix_a4() {
    Tests::all("misc/AppendixA4").run()
}

#[test]
fn test_literal() {
    Tests::all("prod/Literal")
        // float representation uses different rules, though it's formally
        // correct. Check canonical float rules?
        .exclude("Literals016 Literals017 Literals025 Literals027 Literals028")
        // this enormously long decimal literal fails with a parse error
        // apparently if it's an error it wants aa FOAR0002.
        // It shouldn't be a compilation error in that case, but accepted
        // in the AST but then rejected during compilation to IR
        .exclude("K2-Literals-6")
        // apparently 'import' needs to be accepted as a keyword,
        // but why this should raise XPDY0002 is unclear; I guess because
        // `import` is not defined?
        .exclude("K2-Literals-37")
        // the same story, but for `schema`
        .exclude("K2-Literals-38")
        .run()
}

#[test]
fn test_eqname() {
    // getting these to work at all trips up over namespace axis
    // support somehow. eqname-023 uses the namespace access, we could
    // exclude namespace-axis  feature from dependency
    Tests::none("prod/EQName")
        // whitespace may not be properly collapsed inside Q{} strings
        .bug("eqname-024")
        .run();
}

// This requires union type support
// #[test]
// fn test_xs_numeric() {
//     Tests::all("xs/numeric").run()
// }

// #[test]
// fn test_boolean() {
//     Tests::all("fn/boolean")
//         // these depend on constructor functions
//         .exclude("fn-booleannnpi1args-*")
//         .exclude("fn-boolean")
//         .run()
// }
