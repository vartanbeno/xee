use xmlparser::{
    ElementEnd, EntityDefinition, ExternalId, StrSpan, Token as ParserToken, Tokenizer,
};

struct Namespaces {
    namespaces: Vec<Vec<(String, String)>>,
}

enum Token<'a> {
    Declaration {
        version: StrSpan<'a>,
        encoding: Option<StrSpan<'a>>,
        standalone: Option<bool>,
        span: StrSpan<'a>,
    },
    ProcessingInstruction {
        target: StrSpan<'a>,
        content: Option<StrSpan<'a>>,
        span: StrSpan<'a>,
    },
    Comment {
        text: StrSpan<'a>,
        span: StrSpan<'a>,
    },
    DtdStart {
        name: StrSpan<'a>,
        external_id: Option<ExternalId<'a>>,
        span: StrSpan<'a>,
    },
    EmptyDtd {
        name: StrSpan<'a>,
        external_id: Option<ExternalId<'a>>,
        span: StrSpan<'a>,
    },
    EntityDeclaration {
        name: StrSpan<'a>,
        definition: EntityDefinition<'a>,
        span: StrSpan<'a>,
    },
    DtdEnd {
        span: StrSpan<'a>,
    },
    ElementStart {
        prefix: StrSpan<'a>,
        local: StrSpan<'a>,
        namespace: &'a str,
        span: StrSpan<'a>,
    },
    Attribute {
        prefix: StrSpan<'a>,
        local: StrSpan<'a>,
        namespace: &'a str,
        value: StrSpan<'a>,
        span: StrSpan<'a>,
    },
    ElementEnd {
        end: ElementEnd<'a>,
        namespace: &'a str,
        span: StrSpan<'a>,
    },
    Text {
        text: StrSpan<'a>,
    },
    Cdata {
        text: StrSpan<'a>,
        span: StrSpan<'a>,
    },
}

struct NamespacedTokenizer<'a> {
    namespaces: Namespaces,
    tokenizer: Tokenizer<'a>,
}

// impl<'a> Iterator for NamespacedTokenizer<'a> {
//     type Item = Result<Token<'a>, xmlparser::Error>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.tokenizer.next()
//     }
// }
