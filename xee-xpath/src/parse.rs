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
    fn test_xpath() {
        let successful_parse = XPathParser::parse(Rule::Xpath, "/a/c");
        assert!(successful_parse.is_ok());
        let unsuccessful_parse = XPathParser::parse(Rule::Xpath, ")(");
        assert!(unsuccessful_parse.is_err());
    }
}
