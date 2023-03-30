use pest::iterators::Pairs;

use crate::ast;
use crate::parse::{parse, Rule};

pub struct Error {}

fn pairs_to_ast(pairs: Pairs<Rule>) -> Result<ast::XPath, Error> {
    Ok(ast::XPath { exprs: vec![] })
}

pub(crate) fn parse_ast(xpath: &str) -> Result<ast::XPath, Error> {
    match parse(xpath) {
        Ok(pairs) => {
            let ast = pairs_to_ast(pairs);
            match ast {
                Ok(ast) => Ok(ast),
                Err(e) => Err(Error {}),
            }
        }
        Err(e) => Err(Error {}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let pairs = parse("/foo").unwrap();
    }
}
