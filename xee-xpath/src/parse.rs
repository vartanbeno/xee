// parse XPath via pest

// XXX extra-grammatical constraints still need to be handled:
// https://www.w3.org/TR/xpath-31/#extra-grammatical-constraints
// XXX reserved function names still need to be handled:
// https://www.w3.org/TR/xpath-31/#id-reserved-fn-names

#[derive(Parser)]
#[grammar = "xpath-31.pest"]
pub struct XPathParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

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
}
