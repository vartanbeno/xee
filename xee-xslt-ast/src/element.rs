use ahash::{HashSet, HashSetExt};
use xee_xpath_ast::{ast as xpath_ast, Namespaces};
use xot::{NameId, Node, SpanInfoKey, Value};

use crate::ast_core::Span;
use crate::ast_core::{self as ast};
use crate::attributes::Attributes;
use crate::combinator::{end, one, NodeParser, OneParser};
use crate::context::Context;
use crate::error::{AttributeError, ElementError};
use crate::instruction::{DeclarationParser, InstructionParser, SequenceConstructorParser};
use crate::name::XmlName;
use crate::names::StandardNames;
use crate::state::State;
use crate::tokenize::split_whitespace_with_spans;
use crate::value_template::ValueTemplateTokenizer;

struct ElementParsers {
    sequence_constructor_sibling_parser: Box<dyn NodeParser<Vec<ast::SequenceConstructorItem>>>,
    sequence_constructor_parser: Box<dyn NodeParser<Vec<ast::SequenceConstructorItem>>>,
    declarations_parser: Box<dyn NodeParser<Vec<ast::Declaration>>>,
}

impl ElementParsers {
    fn new() -> Self {
        let sequence_constructor_sibling_parser = one(|node, state, context| {
            match state.xot.value(node) {
                Value::Text(text) => Ok(ast::SequenceConstructorItem::TextNode(
                    text.get().to_string(),
                )),
                Value::Element(element) => {
                    let element = Element::new(node, element, context, state)?;
                    ast::SequenceConstructorItem::parse_sequence_constructor_item(
                        &element,
                        &element.attributes,
                    )
                }
                _ => Err(ElementError::Unexpected {
                    // TODO: get span right
                    span: Span::new(0, 0),
                }),
            }
        })
        .many();

        let sequence_constructor_parser = sequence_constructor_sibling_parser
            .clone()
            .then_ignore(end())
            .contains();

        let declarations_parser = one(|node, state, context| match state.xot.value(node) {
            Value::Element(element) => {
                let element = Element::new(node, element, context, state)?;
                ast::Declaration::parse_declaration(&element, &element.attributes)
            }
            _ => Err(ElementError::Unexpected {
                // TODO: get span right
                span: Span::new(0, 0),
            }),
        })
        .many()
        .then_ignore(end())
        .contains();

        Self {
            sequence_constructor_sibling_parser: Box::new(sequence_constructor_sibling_parser),
            sequence_constructor_parser: Box::new(sequence_constructor_parser),
            declarations_parser: Box::new(declarations_parser),
        }
    }
}

pub(crate) struct XsltParser<'a> {
    state: &'a State,
}

impl<'a> XsltParser<'a> {
    pub(crate) fn new(state: &'a State) -> Self {
        Self { state }
    }

    pub(crate) fn parse_transform(&self, node: Node) -> Result<ast::Transform, ElementError> {
        let parser = instruction(self.state.names.xsl_transform);
        parser.parse(Some(node), self.state, &Context::empty())
    }
}

pub(crate) fn by_element<V>(
    f: impl Fn(&Element) -> Result<V, ElementError>,
) -> impl Fn(Node, &State, &Context) -> Result<V, ElementError> {
    move |node, state, context| {
        let element = state.xot.element(node).ok_or(ElementError::Unexpected {
            span: state.span(node).ok_or(ElementError::Internal)?,
        })?;
        let element = Element::new(node, element, context, state)?;
        f(&element)
    }
}

pub(crate) fn by_element_name<V>(
    name: NameId,
    f: impl Fn(&Element) -> Result<V, ElementError>,
) -> impl Fn(Node, &State, &Context) -> Result<V, ElementError> {
    by_element(move |element| {
        if element.element.name() == name {
            f(&element)
        } else {
            Err(ElementError::Unexpected { span: element.span })
        }
    })
}

pub(crate) fn by_instruction<V: InstructionParser>(
    name: NameId,
) -> impl Fn(Node, &State, &Context) -> Result<V, ElementError> {
    by_element_name(name, move |element| {
        V::parse_and_validate(element, &element.attributes)
    })
}

pub(crate) fn instruction<V: InstructionParser>(
    name: NameId,
) -> OneParser<V, impl Fn(Node, &State, &Context) -> Result<V, ElementError>> {
    one(by_instruction(name))
}

pub(crate) struct SequenceConstructorNodeParser;

impl NodeParser<ast::SequenceConstructor> for SequenceConstructorNodeParser {
    fn parse_next(
        &self,
        node: Option<Node>,
        state: &State,
        context: &Context,
    ) -> Result<(ast::SequenceConstructor, Option<Node>), ElementError> {
        let element_parsers = ElementParsers::new();
        element_parsers
            .sequence_constructor_sibling_parser
            .parse_next(node, state, context)
    }
}

pub(crate) fn sequence_constructor() -> SequenceConstructorNodeParser {
    SequenceConstructorNodeParser
}

pub(crate) fn content_parse<V, P>(parser: P) -> impl Fn(&Element) -> Result<V, ElementError>
where
    P: NodeParser<V>,
{
    move |element| {
        let (item, next) = parser.parse_next(
            element.state.xot.first_child(element.node),
            element.state,
            &element.context,
        )?;
        // handle end of content check here
        if let Some(next) = next {
            Err(ElementError::Unexpected {
                span: element.state.span(next).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(item)
        }
    }
}

pub(crate) struct Element<'a> {
    pub(crate) node: Node,
    pub(crate) element: &'a xot::Element,
    pub(crate) span: Span,
    pub(crate) context: Context<'a>,
    pub(crate) state: &'a State,
    pub(crate) attributes: Attributes<'a>,
}

impl<'a> Element<'a> {
    pub(crate) fn new(
        node: Node,
        element: &'a xot::Element,
        context: &'a Context<'a>,
        state: &'a State,
    ) -> Result<Self, ElementError> {
        let span = state.span(node).ok_or(ElementError::Internal)?;
        let attributes = Attributes::new(node, element, state, context.clone())?;
        let context = context.sub(element, attributes.standard()?);

        Ok(Self {
            node,
            element,
            span,
            context,
            state,
            attributes,
        })
    }

    pub(crate) fn sequence_constructor(&self) -> Result<ast::SequenceConstructor, ElementError> {
        let element_parsers = ElementParsers::new();
        element_parsers.sequence_constructor_parser.parse(
            Some(self.node),
            self.state,
            &self.context,
        )
    }

    pub(crate) fn declarations(&self) -> Result<ast::Declarations, ElementError> {
        let element_parsers = ElementParsers::new();
        element_parsers
            .declarations_parser
            .parse(Some(self.node), self.state, &self.context)
    }

    pub(crate) fn optional<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str, Span) -> Result<T, AttributeError>,
    ) -> Result<Option<T>, AttributeError> {
        self.attributes.optional(name, parse_value)
    }

    pub(crate) fn required<T>(
        &self,
        name: NameId,
        parse_value: impl Fn(&'a str, Span) -> Result<T, AttributeError>,
    ) -> Result<T, AttributeError> {
        self.attributes.required(name, parse_value)
    }

    pub(crate) fn boolean_with_default(
        &self,
        name: NameId,
        default: bool,
    ) -> Result<bool, AttributeError> {
        self.attributes.boolean_with_default(name, default)
    }

    pub(crate) fn unseen_attributes(&self) -> Vec<NameId> {
        self.attributes.unseen_attributes()
    }

    fn namespaces(&'a self) -> Namespaces<'a> {
        self.context.namespaces(self.state)
    }

    // fn value_span(&self, name: NameId) -> Result<Span, AttributeError> {
    //     let span = self
    //         .state
    //         .span_info
    //         .get(SpanInfoKey::AttributeValue(self.node, name))
    //         .ok_or(AttributeError::Internal)?;
    //     Ok(span.into())
    // }

    pub(crate) fn value_template<T>(
        &self,
        parse_value: impl Fn(&'a str, Span) -> Result<T, AttributeError> + 'a,
    ) -> impl Fn(&'a str, Span) -> Result<ast::ValueTemplate<T>, AttributeError> + '_
    where
        T: Clone + PartialEq + Eq,
    {
        self.attributes.value_template(parse_value)
    }

    pub(crate) fn eqname(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<xpath_ast::Name, AttributeError> + '_ {
        self.attributes.eqname()
    }

    fn _qname(s: &str, _span: Span) -> Result<ast::QName, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn qname(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::QName, AttributeError> + '_ {
        Self::_qname
    }

    fn _ncname(s: &str, _span: Span) -> Result<ast::NcName, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn ncname(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::NcName, AttributeError> + '_ {
        Self::_ncname
    }

    fn _id(s: &str, _span: Span) -> Result<ast::Id, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn id(&self) -> impl Fn(&'a str, Span) -> Result<ast::Id, AttributeError> + '_ {
        Self::_id
    }

    fn _string(s: &str, _span: Span) -> Result<String, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn string(&self) -> impl Fn(&'a str, Span) -> Result<String, AttributeError> + '_ {
        Self::_string
    }

    fn _input_type_annotations(
        s: &str,
        span: Span,
    ) -> Result<ast::InputTypeAnnotations, AttributeError> {
        match s {
            "strip" => Ok(ast::InputTypeAnnotations::Strip),
            "preserve" => Ok(ast::InputTypeAnnotations::Preserve),
            "unspecified" => Ok(ast::InputTypeAnnotations::Unspecified),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn input_type_annotations(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::InputTypeAnnotations, AttributeError> + '_ {
        Self::_input_type_annotations
    }

    fn _token(s: &str, _span: Span) -> Result<ast::Token, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn token(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Token, AttributeError> + '_ {
        Self::_token
    }

    fn _nmtoken(s: &str, _span: Span) -> Result<ast::NmToken, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn nmtoken(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::NmToken, AttributeError> + '_ {
        Self::_nmtoken
    }

    fn _tokens(s: &str, span: Span) -> Result<Vec<ast::Token>, AttributeError> {
        let mut result = Vec::new();
        for s in s.split_whitespace() {
            result.push(Self::_token(s, span)?);
        }
        Ok(result)
    }

    pub(crate) fn tokens(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Token>, AttributeError> + '_ {
        Self::_tokens
    }

    pub(crate) fn uri(&self) -> impl Fn(&'a str, Span) -> Result<ast::Uri, AttributeError> + '_ {
        self.attributes.uri()
    }

    pub(crate) fn uris(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Uri>, AttributeError> + '_ {
        self.attributes.uris()
    }

    pub(crate) fn xpath(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Expression, AttributeError> + '_ {
        self.attributes.xpath()
    }

    fn _pattern(s: &str, _span: Span) -> Result<ast::Pattern, AttributeError> {
        Ok(s.to_string())
    }

    pub(crate) fn pattern(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Pattern, AttributeError> + '_ {
        Self::_pattern
    }

    fn _phase(s: &str, span: Span) -> Result<ast::AccumulatorPhase, AttributeError> {
        match s {
            "start" => Ok(ast::AccumulatorPhase::Start),
            "end" => Ok(ast::AccumulatorPhase::End),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn phase(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::AccumulatorPhase, AttributeError> + '_ {
        Self::_phase
    }

    // fn _eqnames(&self, s: &str, span: Span) -> Result<Vec<xpath_ast::Name>, AttributeError> {
    //     let mut result = Vec::new();
    //     for (s, span) in split_whitespace_with_spans(s, span) {
    //         result.push(self._eqname(s, span)?);
    //     }
    //     Ok(result)
    // }

    pub(crate) fn eqnames(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<xpath_ast::Name>, AttributeError> + '_ {
        self.attributes.eqnames()
    }

    fn _sequence_type(
        &self,
        s: &str,
        _span: Span,
    ) -> Result<xpath_ast::SequenceType, AttributeError> {
        Ok(xpath_ast::SequenceType::parse(s, &self.namespaces())?)
    }

    pub(crate) fn sequence_type(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<xpath_ast::SequenceType, AttributeError> + '_ {
        |s, span| self._sequence_type(s, span)
    }

    fn _item_type(&self, s: &str, _span: Span) -> Result<xpath_ast::ItemType, AttributeError> {
        Ok(xpath_ast::ItemType::parse(s, &self.namespaces())?)
    }

    pub(crate) fn item_type(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<xpath_ast::ItemType, AttributeError> + '_ {
        |s, span| self._item_type(s, span)
    }

    // fn _boolean(s: &str, span: Span) -> Result<bool, AttributeError> {
    //     match s {
    //         "yes" | "true" | "1" => Ok(true),
    //         "no" | "false" | "0" => Ok(false),
    //         _ => Err(AttributeError::Invalid {
    //             value: s.to_string(),
    //             span,
    //         }),
    //     }
    // }

    pub(crate) fn boolean(&self) -> impl Fn(&'a str, Span) -> Result<bool, AttributeError> + '_ {
        self.attributes.boolean()
    }

    fn _integer(s: &str, span: Span) -> Result<usize, AttributeError> {
        match s.parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn integer(&self) -> impl Fn(&'a str, Span) -> Result<usize, AttributeError> + '_ {
        Self::_integer
    }

    pub(crate) fn default_mode(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::DefaultMode, AttributeError> + '_ {
        self.attributes.default_mode()
    }

    pub(crate) fn default_validation(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::DefaultValidation, AttributeError> + '_ {
        self.attributes.default_validation()
    }

    pub(crate) fn prefix(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Prefix, AttributeError> + '_ {
        self.attributes.prefix()
    }

    pub(crate) fn prefix_or_default(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::PrefixOrDefault, AttributeError> + '_ {
        self.attributes.prefix_or_default()
    }

    pub(crate) fn prefixes(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<Vec<ast::Prefix>, AttributeError> + '_ {
        self.attributes.prefixes()
    }

    pub(crate) fn decimal(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Decimal, AttributeError> + '_ {
        self.attributes.decimal()
    }

    pub(crate) fn exclude_result_prefixes(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::ExcludeResultPrefixes, AttributeError> {
        self.attributes.exclude_result_prefixes()
    }

    fn _component(s: &str, span: Span) -> Result<ast::Component, AttributeError> {
        use ast::Component::*;

        match s {
            "template" => Ok(Template),
            "function" => Ok(Function),
            "attribute-set" => Ok(AttributeSet),
            "variable" => Ok(Variable),
            "mode" => Ok(Mode),
            "*" => Ok(Star),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn component(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Component, AttributeError> + '_ {
        Self::_component
    }

    fn _visibility(s: &str, span: Span) -> Result<ast::Visibility, AttributeError> {
        use ast::Visibility::*;

        match s {
            "public" => Ok(Public),
            "private" => Ok(Private),
            "final" => Ok(Final),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn visibility(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Visibility, AttributeError> + '_ {
        Self::_visibility
    }

    fn _visibility_with_abstract(
        s: &str,
        span: Span,
    ) -> Result<ast::VisibilityWithAbstract, AttributeError> {
        use ast::VisibilityWithAbstract::*;

        match s {
            "public" => Ok(Public),
            "private" => Ok(Private),
            "final" => Ok(Final),
            "abstract" => Ok(Abstract),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn visibility_with_abstract(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::VisibilityWithAbstract, AttributeError> {
        Self::_visibility_with_abstract
    }

    fn _visibility_with_hidden(
        s: &str,
        span: Span,
    ) -> Result<ast::VisibilityWithHidden, AttributeError> {
        use ast::VisibilityWithHidden::*;

        match s {
            "public" => Ok(Public),
            "private" => Ok(Private),
            "final" => Ok(Final),
            "hidden" => Ok(Hidden),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn visibility_with_hidden(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::VisibilityWithHidden, AttributeError> {
        Self::_visibility_with_hidden
    }

    fn _validation(s: &str, span: Span) -> Result<ast::Validation, AttributeError> {
        use ast::Validation::*;

        match s {
            "strict" => Ok(Strict),
            "lax" => Ok(Lax),
            "preserve" => Ok(Preserve),
            "strip" => Ok(Strip),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn validation(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Validation, AttributeError> + '_ {
        Self::_validation
    }

    fn _on_no_match(s: &str, span: Span) -> Result<ast::OnNoMatch, AttributeError> {
        use ast::OnNoMatch::*;

        match s {
            "deep-copy" => Ok(DeepCopy),
            "shallow-copy" => Ok(ShallowCopy),
            "deep-skip" => Ok(DeepSkip),
            "shallow-skip" => Ok(ShallowSkip),
            "text-only-copy" => Ok(TextOnlyCopy),
            "fail" => Ok(Fail),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn on_no_match(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::OnNoMatch, AttributeError> + '_ {
        Self::_on_no_match
    }

    fn _on_multiple_match(s: &str, span: Span) -> Result<ast::OnMultipleMatch, AttributeError> {
        use ast::OnMultipleMatch::*;

        match s {
            "use-last" => Ok(UseLast),
            "fail" => Ok(Fail),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn on_multiple_match(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::OnMultipleMatch, AttributeError> + '_ {
        Self::_on_multiple_match
    }

    fn _typed(s: &str, span: Span) -> Result<ast::Typed, AttributeError> {
        use ast::Typed::*;

        match s {
            "boolean" => Ok(Boolean),
            "strict" => Ok(Strict),
            "lax" => Ok(Lax),
            "unspecified" => Ok(Unspecified),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn typed(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Typed, AttributeError> + '_ {
        Self::_typed
    }

    fn _level(s: &str, span: Span) -> Result<ast::NumberLevel, AttributeError> {
        use ast::NumberLevel::*;

        match s {
            "single" => Ok(Single),
            "multiple" => Ok(Multiple),
            "any" => Ok(Any),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn level(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::NumberLevel, AttributeError> + '_ {
        Self::_level
    }

    fn _letter_value(s: &str, span: Span) -> Result<ast::LetterValue, AttributeError> {
        use ast::LetterValue::*;

        match s {
            "alphabetic" => Ok(Alphabetic),
            "traditional" => Ok(Traditional),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn letter_value(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::LetterValue, AttributeError> + '_ {
        Self::_letter_value
    }

    pub(crate) fn method(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::OutputMethod, AttributeError> + '_ {
        self.attributes.method()
    }

    pub(crate) fn json_node_output_method(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::JsonNodeOutputMethod, AttributeError> + '_ {
        self.attributes.json_node_output_method()
    }

    fn _normalization_form(
        &self,
        s: &str,
        span: Span,
    ) -> Result<ast::NormalizationForm, AttributeError> {
        use ast::NormalizationForm::*;

        match s {
            "NFC" => Ok(Nfc),
            "NFD" => Ok(Nfd),
            "NFKC" => Ok(Nfkc),
            "NFKD" => Ok(Nfkd),
            _ => Ok(NmToken(Self::_nmtoken(s, span)?)),
        }
    }

    pub(crate) fn normalization_form(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::NormalizationForm, AttributeError> + '_ {
        |s, span| self._normalization_form(s, span)
    }

    fn _standalone(s: &str, _span: Span) -> Result<ast::Standalone, AttributeError> {
        match s {
            "yes" | "1" | "true" => Ok(ast::Standalone::Bool(true)),
            "no" | "0" | "false" => Ok(ast::Standalone::Bool(false)),
            "omit" => Ok(ast::Standalone::Omit),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span: _span,
            }),
        }
    }

    pub(crate) fn standalone(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Standalone, AttributeError> + '_ {
        Self::_standalone
    }

    fn _language(s: &str, _span: Span) -> Result<ast::Language, AttributeError> {
        // TODO
        Ok(s.to_string())
    }

    pub(crate) fn language(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Language, AttributeError> + '_ {
        Self::_language
    }

    fn _order(s: &str, span: Span) -> Result<ast::Order, AttributeError> {
        use ast::Order::*;

        match s {
            "ascending" => Ok(Ascending),
            "descending" => Ok(Descending),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn order(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Order, AttributeError> + '_ {
        Self::_order
    }

    fn _case_order(s: &str, span: Span) -> Result<ast::CaseOrder, AttributeError> {
        use ast::CaseOrder::*;

        match s {
            "upper-first" => Ok(UpperFirst),
            "lower-first" => Ok(LowerFirst),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn case_order(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::CaseOrder, AttributeError> + '_ {
        Self::_case_order
    }

    pub(crate) fn data_type(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::DataType, AttributeError> + '_ {
        self.attributes.data_type()
    }

    fn _char(s: &str, span: Span) -> Result<char, AttributeError> {
        let mut chars = s.chars();
        if let Some(char) = chars.next() {
            if chars.next().is_none() {
                return Ok(char);
            }
        }
        Err(AttributeError::Invalid {
            value: s.to_string(),
            span,
        })
    }

    pub(crate) fn char(&self) -> impl Fn(&'a str, Span) -> Result<char, AttributeError> {
        Self::_char
    }

    fn _use(s: &str, span: Span) -> Result<ast::Use, AttributeError> {
        use ast::Use::*;

        match s {
            "optional" => Ok(Optional),
            "required" => Ok(Required),
            "absent" => Ok(Absent),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn use_(&self) -> impl Fn(&'a str, Span) -> Result<ast::Use, AttributeError> {
        Self::_use
    }

    pub(crate) fn streamability(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::Streamability, AttributeError> + '_ {
        self.attributes.streamability()
    }

    fn _new_each_time(s: &str, span: Span) -> Result<ast::NewEachTime, AttributeError> {
        use ast::NewEachTime::*;

        match s {
            "yes" | "1" | "true" => Ok(Yes),
            "no" | "0" | "false" => Ok(No),
            "maybe" => Ok(Maybe),
            _ => Err(AttributeError::Invalid {
                value: s.to_string(),
                span,
            }),
        }
    }

    pub(crate) fn new_each_time(
        &self,
    ) -> impl Fn(&'a str, Span) -> Result<ast::NewEachTime, AttributeError> + '_ {
        Self::_new_each_time
    }
}
