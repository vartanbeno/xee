// parse XPath via pest

// XXX extra-grammatical constraints still need to be handled:
// https://www.w3.org/TR/xpath-31/#extra-grammatical-constraints
// XXX reserved function names still need to be handled:
// https://www.w3.org/TR/xpath-31/#id-reserved-fn-names

#[derive(Parser)]
#[grammar = "parser/xpath-31.pest"]
pub(crate) struct XPathParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::iterators::Pairs;
    use pest::Parser;

    // parse function signature as described in
    // https://www.w3.org/TR/xpath-functions-31/#func-signatures
    // This is almost the same as an inline function definition except
    // for the name part.
    #[allow(clippy::result_large_err)]
    pub(crate) fn parse_signature(
        signature: &str,
    ) -> Result<Pairs<Rule>, pest::error::Error<Rule>> {
        XPathParser::parse(Rule::Signature, signature)
    }

    #[test]
    fn test_char() {
        let successful_parse = XPathParser::parse(Rule::Char, "abc");
        assert!(successful_parse.is_ok());
        let unsuccessful_parse = XPathParser::parse(Rule::Char, "\u{10}");
        assert!(unsuccessful_parse.is_err());
    }

    #[test]
    fn test_not_xpath() {
        let unsuccessful_parse = XPathParser::parse(Rule::Xpath, ")(");
        assert!(unsuccessful_parse.is_err());
    }

    #[test]
    fn test_not_xpath_incomplete() {
        let unsuccessful_parse = XPathParser::parse(Rule::Xpath, "foo(");
        assert!(unsuccessful_parse.is_err());
    }

    #[test]
    fn test_xpath_simple_name() {
        let successful_parse = XPathParser::parse(Rule::Xpath, "foo");
        assert!(successful_parse.is_ok());
    }

    #[test]
    fn test_xpath_relative_path() {
        let successful_parse = XPathParser::parse(Rule::Xpath, "foo/bar");
        assert!(successful_parse.is_ok());
    }

    #[test]
    fn test_xpath_absolute_path() {
        let successful_parse = XPathParser::parse(Rule::Xpath, "/foo/bar");
        assert!(successful_parse.is_ok());
    }

    #[test]
    fn test_signature() {
        let successful_parse = parse_signature("math:exp($arg as xs:double?) as xs:double?");
        assert!(successful_parse.is_ok());
    }

    #[test]
    fn test_signature_parameter_type_required() {
        let successful_parse = parse_signature("math:exp($arg) as xs:double?");
        assert!(successful_parse.is_err());
    }

    #[test]
    fn test_signature_return_type_required() {
        let successful_parse = parse_signature("math:exp($arg as xs:double?)");
        assert!(successful_parse.is_err());
    }
}
