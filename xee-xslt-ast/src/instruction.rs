use xot::{NameId, Node, Xot};

use crate::ast_core::{self as ast};
use crate::attributes::Attributes;
use crate::combinator::{one, NodeParser};
use crate::element::{by_element, content_parse, instruction, sequence_constructor, Element};
use crate::error::ElementError as Error;
use crate::state::State;

type Result<V> = std::result::Result<V, Error>;

pub(crate) trait InstructionParser: Sized {
    fn validate(&self, _node: Node, _state: &State) -> Result<()> {
        Ok(())
    }

    fn should_be_empty() -> bool {
        false
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self>;

    fn parse_and_validate(element: &Element, attributes: &Attributes) -> Result<Self> {
        if Self::should_be_empty() {
            if let Some(child) = element.state.xot.first_child(element.node) {
                return Err(Error::Unexpected {
                    span: element.state.span(child).ok_or(Error::Internal)?,
                });
            }
        }

        let node = element.node;
        let state = element.state;
        let item = Self::parse(element, attributes)?;
        item.validate(node, state)?;
        let unseen_attributes = attributes.unseen_attributes();
        if !unseen_attributes.is_empty() {
            return Err(state
                .attribute_unexpected(node, unseen_attributes[0], "unexpected attribute")
                .into());
        }
        Ok(item)
    }
}

pub(crate) trait SequenceConstructorParser:
    InstructionParser + Into<ast::SequenceConstructorItem>
{
    fn parse_sequence_constructor_item(
        element: &Element,
        attributes: &Attributes,
    ) -> Result<ast::SequenceConstructorItem> {
        let item = Self::parse_and_validate(element, attributes)?;
        Ok(item.into())
    }
}

impl<T> SequenceConstructorParser for T where
    T: InstructionParser + Into<ast::SequenceConstructorItem>
{
}

pub(crate) trait DeclarationParser: InstructionParser + Into<ast::Declaration> {
    fn parse_declaration(element: &Element, attributes: &Attributes) -> Result<ast::Declaration> {
        let item = Self::parse_and_validate(element, attributes)?;
        Ok(item.into())
    }
}

impl<T> DeclarationParser for T where T: InstructionParser + Into<ast::Declaration> {}

pub(crate) trait OverrideContentParser:
    InstructionParser + Into<ast::OverrideContent>
{
    fn parse_override_content(
        element: &Element,
        attributes: &Attributes,
    ) -> Result<ast::OverrideContent> {
        let item = Self::parse_and_validate(element, attributes)?;
        Ok(item.into())
    }
}

impl<T> OverrideContentParser for T where T: InstructionParser + Into<ast::OverrideContent> {}

impl InstructionParser for ast::SequenceConstructorItem {
    fn parse(element: &Element, attributes: &Attributes) -> Result<ast::SequenceConstructorItem> {
        let context = element.state;
        let name = context
            .names
            .sequence_constructor_name(element.element.name());

        if let Some(name) = name {
            name.parse(element, attributes)
        } else {
            let ns = context.xot.namespace_for_name(element.element.name());
            if ns == context.names.xsl_ns {
                // we have an unknown xsl instruction, fail with error
                Err(Error::Unexpected { span: element.span })
            } else {
                // we parse the literal element
                ast::ElementNode::parse_sequence_constructor_item(element, attributes)
            }
        }
    }
}

impl InstructionParser for ast::Declaration {
    fn parse(element: &Element, attributes: &Attributes) -> Result<ast::Declaration> {
        let name = element.state.names.declaration_name(element.element.name());

        if let Some(name) = name {
            name.parse(element, attributes)
        } else {
            Err(Error::Unexpected { span: element.span })
        }
    }
}

impl InstructionParser for ast::ElementNode {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<ast::ElementNode> {
        Ok(ast::ElementNode {
            name: to_name(&element.state.xot, element.element.name()),

            span: element.span,
        })
    }
}

fn to_name(xot: &Xot, name: NameId) -> ast::Name {
    let (local, namespace) = xot.name_ns_str(name);
    ast::Name {
        namespace: namespace.to_string(),
        local: local.to_string(),
    }
}

impl InstructionParser for ast::Accept {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Accept {
            component: attributes.required(names.component, attributes.component())?,
            names: attributes.required(names.names, attributes.tokens())?,
            visibility: attributes
                .required(names.visibility, attributes.visibility_with_hidden())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Accumulator {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        let parse_rules = content_parse(instruction(names.xsl_accumulator_rule).many());

        Ok(ast::Accumulator {
            name: attributes.required(names.name, attributes.eqname())?,
            initial_value: attributes.required(names.initial_value, attributes.xpath())?,
            as_: attributes.optional(names.as_, attributes.sequence_type())?,
            streamable: attributes.boolean_with_default(names.streamable, false)?,

            span: element.span,

            rules: parse_rules(element)?,
        })
    }
}

impl InstructionParser for ast::AccumulatorRule {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::AccumulatorRule {
            match_: attributes.required(names.match_, attributes.pattern())?,
            phase: attributes.optional(names.phase, attributes.phase())?,
            select: attributes.optional(names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::AnalyzeString {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let span = element.span;

        let select = attributes.required(names.select, attributes.xpath())?;
        let regex =
            attributes.required(names.regex, attributes.value_template(attributes.string()))?;
        let flags =
            attributes.optional(names.flags, attributes.value_template(attributes.string()))?;

        let parse = content_parse(
            instruction(names.xsl_matching_substring)
                .option()
                .then(instruction(names.xsl_non_matching_substring).option())
                .then(instruction(names.xsl_fallback).many()),
        );

        let ((matching_substring, non_matching_substring), fallbacks) = parse(element)?;

        Ok(ast::AnalyzeString {
            select,
            regex,
            flags,

            span,

            matching_substring,
            non_matching_substring,
            fallbacks,
        })
    }
}

impl InstructionParser for ast::ApplyImports {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        let parse = content_parse(instruction(element.state.names.xsl_with_param).many());
        Ok(ast::ApplyImports {
            span: element.span,

            with_params: parse(element)?,
        })
    }
}

impl InstructionParser for ast::ApplyTemplates {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(
            (instruction(names.xsl_with_param)
                .map(ast::ApplyTemplatesContent::WithParam)
                .or(instruction(names.xsl_sort).map(ast::ApplyTemplatesContent::Sort)))
            .many(),
        );

        Ok(ast::ApplyTemplates {
            select: attributes.optional(names.select, attributes.xpath())?,
            mode: attributes.optional(names.mode, attributes.token())?,

            span: element.span,

            content: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Assert {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Assert {
            test: attributes.required(names.test, attributes.xpath())?,
            select: attributes.optional(names.select, attributes.xpath())?,
            error_code: attributes.optional(
                names.error_code,
                attributes.value_template(attributes.eqname()),
            )?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Attribute {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Attribute {
            name: attributes.required(names.name, attributes.value_template(attributes.qname()))?,
            namespace: attributes
                .optional(names.namespace, attributes.value_template(attributes.uri()))?,
            select: attributes.optional(names.select, attributes.xpath())?,
            separator: attributes.optional(
                names.separator,
                attributes.value_template(attributes.string()),
            )?,
            type_: attributes.optional(names.type_, attributes.eqname())?,
            validation: attributes.optional(names.validation, attributes.validation())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::AttributeSet {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        let parse = content_parse(instruction(names.xsl_attribute).many());

        Ok(ast::AttributeSet {
            name: attributes.required(names.name, attributes.eqname())?,
            use_attribute_sets: attributes
                .optional(names.use_attribute_sets, attributes.eqnames())?,
            visibility: attributes
                .optional(names.visibility, attributes.visibility_with_abstract())?,
            streamable: attributes.boolean_with_default(names.streamable, false)?,

            span: element.span,

            attributes: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Break {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        Ok(ast::Break {
            select: attributes.optional(element.state.names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::CallTemplate {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(instruction(element.state.names.xsl_with_param).many());

        Ok(ast::CallTemplate {
            name: attributes.required(names.name, attributes.eqname())?,

            span: element.span,

            with_params: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Catch {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Catch {
            errors: attributes.optional(names.errors, attributes.tokens())?,
            select: attributes.optional(names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::CharacterMap {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        let parse = content_parse(instruction(names.xsl_output_character).many());
        Ok(ast::CharacterMap {
            name: attributes.required(names.name, attributes.eqname())?,
            use_character_maps: attributes
                .optional(names.use_character_maps, attributes.eqnames())?,

            span: element.span,

            output_characters: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Choose {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        let span = element.span;

        let parse = content_parse(
            instruction(element.state.names.xsl_when)
                .one_or_more()
                .then(instruction(element.state.names.xsl_otherwise).option()),
        );

        let (when, otherwise) = parse(element)?;
        Ok(ast::Choose {
            span,

            when,
            otherwise,
        })
    }
}

impl InstructionParser for ast::Comment {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        Ok(ast::Comment {
            select: attributes.optional(element.state.names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::ContextItem {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::ContextItem {
            as_: attributes.optional(names.as_, attributes.item_type())?,
            use_: attributes.optional(names.use_, attributes.use_())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Copy {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Copy {
            select: attributes.optional(names.select, attributes.xpath())?,
            copy_namespaces: attributes.boolean_with_default(names.copy_namespaces, true)?,
            inherit_namespaces: attributes.boolean_with_default(names.inherit_namespaces, true)?,
            use_attribute_sets: attributes
                .optional(names.use_attribute_sets, attributes.eqnames())?,
            type_: attributes.optional(names.type_, attributes.eqname())?,
            validation: attributes
                .optional(names.validation, attributes.validation())?
                // TODO: should depend on global validation attribute
                .unwrap_or(ast::Validation::Strip),

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::CopyOf {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::CopyOf {
            select: attributes.required(names.select, attributes.xpath())?,
            copy_accumulators: attributes.boolean_with_default(names.copy_accumulators, false)?,
            copy_namespaces: attributes.boolean_with_default(names.copy_namespaces, true)?,
            type_: attributes.optional(names.type_, attributes.eqname())?,
            validation: attributes.optional(names.validation, attributes.validation())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::DecimalFormat {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::DecimalFormat {
            name: attributes.optional(names.name, attributes.eqname())?,
            decimal_separator: attributes.optional(names.decimal_separator, attributes.char())?,
            grouping_separator: attributes.optional(names.grouping_separator, attributes.char())?,
            infinity: attributes.optional(names.infinity, attributes.string())?,
            minus_sign: attributes.optional(names.minus_sign, attributes.char())?,
            exponent_separator: attributes.optional(names.exponent_separator, attributes.char())?,
            nan: attributes.optional(names.nan, attributes.string())?,
            percent: attributes.optional(names.percent, attributes.char())?,
            per_mille: attributes.optional(names.per_mille, attributes.char())?,
            zero_digit: attributes.optional(names.zero_digit, attributes.char())?,
            digit: attributes.optional(names.digit, attributes.char())?,
            pattern_separator: attributes.optional(names.pattern_separator, attributes.char())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Document {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Document {
            validation: attributes.optional(names.validation, attributes.validation())?,
            type_: attributes.optional(names.type_, attributes.eqname())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Element {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Element {
            name: attributes.required(names.name, attributes.value_template(attributes.qname()))?,
            namespace: attributes
                .optional(names.namespace, attributes.value_template(attributes.uri()))?,
            inherit_namespaces: attributes.boolean_with_default(names.inherit_namespaces, false)?,
            use_attribute_sets: attributes
                .optional(names.use_attribute_sets, attributes.eqnames())?,
            type_: attributes.optional(names.type_, attributes.eqname())?,
            validation: attributes.optional(names.validation, attributes.validation())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Evaluate {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(
            instruction(names.xsl_with_param)
                .map(ast::EvaluateContent::WithParam)
                .or(instruction(names.xsl_fallback).map(ast::EvaluateContent::Fallback))
                .many(),
        );

        Ok(ast::Evaluate {
            xpath: attributes.required(names.xpath, attributes.xpath())?,
            as_: attributes.optional(names.as_, attributes.sequence_type())?,
            base_uri: attributes
                .optional(names.base_uri, attributes.value_template(attributes.uri()))?,
            with_params: attributes.optional(names.with_params, attributes.xpath())?,
            context_item: attributes.optional(names.context_item, attributes.xpath())?,
            namespace_context: attributes.optional(names.namespace_context, attributes.xpath())?,
            schema_aware: attributes.optional(
                names.schema_aware,
                attributes.value_template(attributes.boolean()),
            )?,

            span: element.span,

            content: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Expose {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Expose {
            component: attributes.required(names.component, attributes.component())?,
            names: attributes.required(names.names, attributes.tokens())?,
            visibility: attributes
                .required(names.visibility, attributes.visibility_with_abstract())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Fallback {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        Ok(ast::Fallback {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::ForEach {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let select = attributes.required(names.select, attributes.xpath())?;
        let span = element.span;

        let parse = content_parse(
            instruction(names.xsl_sort)
                .many()
                .then(sequence_constructor()),
        );
        let (sort, sequence_constructor) = parse(element)?;

        Ok(ast::ForEach {
            select,

            span,

            sort,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::ForEachGroup {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let select = attributes.required(names.select, attributes.xpath())?;
        let group_by = attributes.optional(names.group_by, attributes.xpath())?;
        let group_adjacent = attributes.optional(names.group_adjacent, attributes.xpath())?;
        let group_starting_with =
            attributes.optional(names.group_starting_with, attributes.pattern())?;
        let group_ending_with =
            attributes.optional(names.group_ending_with, attributes.pattern())?;
        let composite = attributes.boolean_with_default(names.composite, false)?;
        let collation =
            attributes.optional(names.collation, attributes.value_template(attributes.uri()))?;
        let span = element.span;

        let parse = content_parse(
            instruction(names.xsl_sort)
                .many()
                .then(sequence_constructor()),
        );
        let (sort, sequence_constructor) = parse(element)?;

        Ok(ast::ForEachGroup {
            select,
            group_by,
            group_adjacent,
            group_starting_with,
            group_ending_with,
            composite,
            collation,

            span,

            sort,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::Fork {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let span = element.span;

        let sequence_fallbacks =
            (instruction(names.xsl_sequence).then(instruction(names.xsl_fallback).many())).many();
        let for_each_group_fallbacks =
            instruction(names.xsl_for_each_group).then(instruction(names.xsl_fallback).many());

        // look for for-each-group first, and only if that fails,
        // look for sequence fallbacks (which can be the empty list and thus
        // would greedily conclude the parse if it was done first)
        let parse = content_parse(
            instruction(names.xsl_fallback).many().then(
                for_each_group_fallbacks
                    .map(ast::ForkContent::ForEachGroup)
                    .or(sequence_fallbacks.map(ast::ForkContent::SequenceFallbacks)),
            ),
        );
        let (fallbacks, content) = parse(element)?;

        Ok(ast::Fork {
            span,

            fallbacks,
            content,
        })
    }
}

impl InstructionParser for ast::Function {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let name = attributes.required(names.name, attributes.eqname())?;
        let as_ = attributes.optional(names.as_, attributes.sequence_type())?;
        let visibility =
            attributes.optional(names.visibility, attributes.visibility_with_abstract())?;
        let streamability = attributes.optional(names.streamability, attributes.streamability())?;
        let override_extension_function =
            attributes.boolean_with_default(names.override_extension_function, false)?;
        // deprecated
        let override_ = attributes.boolean_with_default(names.override_, false)?;
        let new_each_time = attributes.optional(names.new_each_time, attributes.new_each_time())?;
        let cache = attributes.boolean_with_default(names.cache, false)?;

        let span = element.span;

        let parse = content_parse(
            instruction(names.xsl_param)
                .many()
                .then(sequence_constructor()),
        );
        let (params, sequence_constructor) = parse(element)?;

        Ok(ast::Function {
            name,
            as_,
            visibility,
            streamability,
            override_extension_function,
            override_,
            new_each_time,
            cache,

            span,

            params,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::GlobalContextItem {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::GlobalContextItem {
            as_: attributes.optional(names.as_, attributes.item_type())?,
            use_: attributes.optional(names.use_, attributes.use_())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::If {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::If {
            test: attributes.required(names.test, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Import {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Import {
            href: attributes.required(names.href, attributes.uri())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::ImportSchema {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::ImportSchema {
            namespace: attributes.optional(names.namespace, attributes.uri())?,
            schema_location: attributes.optional(names.schema_location, attributes.uri())?,

            span: element.span,

            // TODO
            schema: None,
        })
    }
}

impl InstructionParser for ast::Include {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Include {
            href: attributes.required(names.href, attributes.uri())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Iterate {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let select = attributes.required(names.select, attributes.xpath())?;

        let span = element.span;

        let parse = content_parse(
            instruction(names.xsl_param)
                .many()
                .then(instruction(names.xsl_on_completion).option())
                .then(sequence_constructor()),
        );
        let ((params, on_completion), sequence_constructor) = parse(element)?;

        Ok(ast::Iterate {
            select,

            span,

            params,
            on_completion,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::Key {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::Key {
            name: attributes.required(names.name, attributes.eqname())?,
            match_: attributes.required(names.match_, attributes.pattern())?,
            use_: attributes.optional(names.use_, attributes.xpath())?,
            composite: attributes.boolean_with_default(names.composite, false)?,
            collation: attributes.optional(names.collation, attributes.uri())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Map {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        Ok(ast::Map {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MapEntry {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::MapEntry {
            key: attributes.required(names.key, attributes.xpath())?,
            select: attributes.optional(names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MatchingSubstring {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        Ok(ast::MatchingSubstring {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Merge {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(instruction(names.xsl_merge_source).one_or_more().then(
            instruction(names.xsl_merge_action).then(instruction(names.xsl_fallback).many()),
        ));
        let span = element.span;
        let (merge_sources, (merge_action, fallbacks)) = parse(element)?;

        Ok(ast::Merge {
            span,

            merge_sources,
            merge_action,
            fallbacks,
        })
    }
}

impl InstructionParser for ast::MergeAction {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        Ok(ast::MergeAction {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MergeKey {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::MergeKey {
            select: attributes.optional(names.select, attributes.xpath())?,
            lang: attributes
                .optional(names.lang, attributes.value_template(attributes.language()))?,
            order: attributes
                .optional(names.order, attributes.value_template(attributes.order()))?,
            collation: attributes
                .optional(names.collation, attributes.value_template(attributes.uri()))?,
            case_order: attributes.optional(
                names.case_order,
                attributes.value_template(attributes.case_order()),
            )?,
            data_type: attributes.optional(
                names.data_type,
                attributes.value_template(attributes.data_type()),
            )?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MergeSource {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        let parse = content_parse(instruction(names.xsl_merge_key).one_or_more());

        Ok(ast::MergeSource {
            name: attributes.optional(names.name, attributes.ncname())?,
            for_each_item: attributes.optional(names.for_each_item, attributes.xpath())?,
            for_each_source: attributes.optional(names.for_each_source, attributes.xpath())?,
            select: attributes.required(names.select, attributes.xpath())?,
            streamable: attributes.boolean_with_default(names.streamable, false)?,
            use_accumulators: attributes.optional(names.use_accumulators, attributes.tokens())?,
            sort_before_merge: attributes.boolean_with_default(names.sort_before_merge, false)?,
            validation: attributes.optional(names.validation, attributes.validation())?,
            type_: attributes.optional(names.type_, attributes.eqname())?,

            span: element.span,

            merge_keys: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Message {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Message {
            select: attributes.optional(names.select, attributes.xpath())?,
            terminate: attributes.optional(
                names.terminate,
                attributes.value_template(attributes.boolean()),
            )?,
            error_code: attributes.optional(
                names.error_code,
                attributes.value_template(attributes.eqname()),
            )?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Mode {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Mode {
            name: attributes.optional(names.name, attributes.eqname())?,
            streamable: attributes.boolean_with_default(names.streamable, false)?,
            use_accumulators: attributes.optional(names.use_accumulators, attributes.tokens())?,
            on_no_match: attributes.optional(names.on_no_match, attributes.on_no_match())?,
            on_multiple_match: attributes
                .optional(names.on_multiple_match, attributes.on_multiple_match())?,
            warning_on_no_match: attributes
                .boolean_with_default(names.warning_on_no_match, false)?,
            warning_on_multiple_match: attributes
                .boolean_with_default(names.warning_on_multiple_match, false)?,
            typed: attributes.optional(names.typed, attributes.typed())?,
            visibility: attributes.optional(names.visibility, attributes.visibility())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Namespace {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Namespace {
            name: attributes
                .required(names.name, attributes.value_template(attributes.ncname()))?,
            select: attributes.optional(names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::NamespaceAlias {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::NamespaceAlias {
            stylesheet_prefix: attributes
                .required(names.stylesheet_prefix, attributes.prefix_or_default())?,
            result_prefix: attributes
                .required(names.result_prefix, attributes.prefix_or_default())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::NextIteration {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        let parse = content_parse(instruction(element.state.names.xsl_with_param).many());
        Ok(ast::NextIteration {
            span: element.span,

            with_params: parse(element)?,
        })
    }
}

impl InstructionParser for ast::NextMatch {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        let parse = content_parse(
            (instruction(names.xsl_with_param)
                .map(ast::NextMatchContent::WithParam)
                .or(instruction(names.xsl_fallback).map(ast::NextMatchContent::Fallback)))
            .many(),
        );

        Ok(ast::NextMatch {
            span: element.span,

            content: parse(element)?,
        })
    }
}

impl InstructionParser for ast::NonMatchingSubstring {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        Ok(ast::NonMatchingSubstring {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Number {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::Number {
            value: attributes.optional(names.value, attributes.xpath())?,
            select: attributes.optional(names.select, attributes.xpath())?,
            level: attributes.optional(names.level, attributes.level())?,
            count: attributes.optional(names.count, attributes.pattern())?,
            from: attributes.optional(names.from, attributes.pattern())?,
            format: attributes
                .optional(names.format, attributes.value_template(attributes.string()))?,
            lang: attributes
                .optional(names.lang, attributes.value_template(attributes.language()))?,
            letter_value: attributes.optional(
                names.letter_value,
                attributes.value_template(attributes.letter_value()),
            )?,
            ordinal: attributes.optional(
                names.ordinal,
                attributes.value_template(attributes.string()),
            )?,
            start_at: attributes.optional(
                names.start_at,
                attributes.value_template(attributes.string()),
            )?,
            grouping_separator: attributes.optional(
                names.grouping_separator,
                attributes.value_template(attributes.char()),
            )?,
            grouping_size: attributes.optional(
                names.grouping_size,
                attributes.value_template(attributes.integer()),
            )?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::OnCompletion {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        Ok(ast::OnCompletion {
            select: attributes.optional(element.state.names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::OnEmpty {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        Ok(ast::OnEmpty {
            select: attributes.optional(element.state.names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::OnNonEmpty {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        Ok(ast::OnNonEmpty {
            select: attributes.optional(element.state.names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Otherwise {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        Ok(ast::Otherwise {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Output {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Output {
            name: attributes.optional(names.name, attributes.eqname())?,
            method: attributes.optional(names.method, attributes.method())?,
            allow_duplicate_names: attributes
                .boolean_with_default(names.allow_duplicate_names, false)?,
            build_tree: attributes.boolean_with_default(names.build_tree, false)?,
            byte_order_mark: attributes.boolean_with_default(names.byte_order_mark, false)?,
            cdata_section_elements: attributes
                .optional(names.cdata_section_elements, attributes.eqnames())?,
            doctype_public: attributes.optional(names.doctype_public, attributes.string())?,
            doctype_system: attributes.optional(names.doctype_system, attributes.string())?,
            encoding: attributes.optional(names.encoding, attributes.string())?,
            escape_uri_attributes: attributes
                .boolean_with_default(names.escape_uri_attributes, true)?,
            html_version: attributes.optional(names.html_version, attributes.decimal())?,
            include_content_type: attributes
                .boolean_with_default(names.include_content_type, true)?,
            // TODO default value is informed by the method
            indent: attributes.boolean_with_default(names.indent, false)?,
            item_separator: attributes.optional(names.item_separator, attributes.string())?,
            json_node_output_method: attributes.optional(
                names.json_node_output_method,
                attributes.json_node_output_method(),
            )?,
            media_type: attributes.optional(names.media_type, attributes.string())?,
            normalization_form: attributes
                .optional(names.normalization_form, attributes.normalization_form())?,
            omit_xml_declaration: attributes
                .boolean_with_default(names.omit_xml_declaration, false)?,
            parameter_document: attributes.optional(names.parameter_document, attributes.uri())?,
            standalone: attributes.optional(names.standalone, attributes.standalone())?,
            suppress_indentation: attributes
                .optional(names.suppress_indentation, attributes.eqnames())?,
            undeclare_prefixes: attributes.boolean_with_default(names.undeclare_prefixes, false)?,
            use_character_maps: attributes
                .optional(names.use_character_maps, attributes.eqnames())?,
            version: attributes.optional(names.version, attributes.nmtoken())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::OutputCharacter {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::OutputCharacter {
            character: attributes.required(names.character, attributes.char())?,
            string: attributes.required(names.string, attributes.string())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::OverrideContent {
    fn parse(element: &Element, attributes: &Attributes) -> Result<ast::OverrideContent> {
        let name = element
            .state
            .names
            .override_content_name(element.element.name());

        if let Some(name) = name {
            name.parse(element, attributes)
        } else {
            Err(Error::Unexpected { span: element.span })
        }
    }
}

impl InstructionParser for ast::Override {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        let parse = content_parse(
            one(by_element(|element, attributes| {
                ast::OverrideContent::parse_override_content(element, attributes)
            }))
            .many(),
        );

        Ok(ast::Override {
            span: element.span,

            content: parse(element)?,
        })
    }
}

// TODO: xsl:package

impl InstructionParser for ast::Param {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Param {
            name: attributes.required(names.name, attributes.eqname())?,
            select: attributes.optional(names.select, attributes.xpath())?,
            as_: attributes.optional(names.as_, attributes.sequence_type())?,
            required: attributes.boolean_with_default(names.required, false)?,
            tunnel: attributes.boolean_with_default(names.tunnel, false)?,
            static_: attributes.boolean_with_default(names.static_, false)?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

// TODO: xsl:perform-sort

// TODO: xsl:preserve-space

impl InstructionParser for ast::PreserveSpace {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::PreserveSpace {
            elements: attributes.required(names.elements, attributes.tokens())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::ProcessingInstruction {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::ProcessingInstruction {
            name: attributes
                .required(names.name, attributes.value_template(attributes.ncname()))?,
            select: attributes.optional(names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

// TODO: xsl:result-document

impl InstructionParser for ast::Sequence {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Sequence {
            select: attributes.optional(names.select, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Sort {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Sort {
            select: attributes.optional(names.select, attributes.xpath())?,
            lang: attributes
                .optional(names.lang, attributes.value_template(attributes.language()))?,
            order: attributes
                .optional(names.order, attributes.value_template(attributes.order()))?,
            collation: attributes
                .optional(names.collation, attributes.value_template(attributes.uri()))?,
            stable: attributes.optional(
                names.stable,
                attributes.value_template(attributes.boolean()),
            )?,
            case_order: attributes.optional(
                names.case_order,
                attributes.value_template(attributes.case_order()),
            )?,
            data_type: attributes.optional(
                names.data_type,
                attributes.value_template(attributes.data_type()),
            )?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::SourceDocument {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::SourceDocument {
            href: attributes.required(names.href, attributes.value_template(attributes.uri()))?,
            streamable: attributes.boolean_with_default(names.streamable, false)?,
            use_accumulators: attributes.optional(names.use_accumulators, attributes.tokens())?,
            validation: attributes.optional(names.validation, attributes.validation())?,
            type_: attributes.optional(names.type_, attributes.eqname())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::StripSpace {
    fn should_be_empty() -> bool {
        true
    }
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::StripSpace {
            elements: attributes.required(names.elements, attributes.tokens())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Template {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        let match_ = attributes.optional(names.match_, attributes.pattern())?;
        let name = attributes.optional(names.name, attributes.eqname())?;
        let priority = attributes.optional(names.priority, attributes.decimal())?;
        let mode = attributes.optional(names.mode, attributes.tokens())?;
        let as_ = attributes.optional(names.as_, attributes.sequence_type())?;
        let visibility =
            attributes.optional(names.visibility, attributes.visibility_with_abstract())?;

        let parse = content_parse(
            instruction(names.context_item)
                .option()
                .then(instruction(names.xsl_param).many())
                .then(sequence_constructor()),
        );
        let span = element.span;
        let ((context_item, params), sequence_constructor) = parse(element)?;

        Ok(ast::Template {
            match_,
            name,
            priority,
            mode,
            as_,
            visibility,

            span,

            context_item,
            params,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::Text {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Text {
            disable_output_escaping: attributes
                .boolean_with_default(names.disable_output_escaping, false)?,

            span: element.span,

            content: element
                .state
                .xot
                .text_str(element.node)
                .unwrap_or("")
                .to_string(),
        })
    }
}

impl InstructionParser for ast::Transform {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Transform {
            id: attributes.optional(names.id, attributes.id())?,
            input_type_annotations: attributes.optional(
                names.input_type_annotations,
                attributes.input_type_annotations(),
            )?,
            extension_element_prefixes: attributes
                .optional(names.extension_element_prefixes, attributes.prefixes())?,

            span: element.span,

            declarations: element.declarations()?,
        })
    }
}

// TODO: xsl:try

// TODO: xsl:use-package

impl InstructionParser for ast::ValueOf {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::ValueOf {
            select: attributes.optional(names.select, attributes.xpath())?,
            separator: attributes.optional(
                names.separator,
                attributes.value_template(attributes.string()),
            )?,
            disable_output_escaping: attributes
                .boolean_with_default(names.disable_output_escaping, false)?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Variable {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;

        // This is a rule somewhere, but not sure whether it's correct;
        // can visibility be absent or is there a default visibility?
        // let visibility = visibility.unwrap_or(if static_ {
        //     ast::VisibilityWithAbstract::Private
        // } else {
        //     ast::VisibilityWithAbstract::Public
        // });

        Ok(ast::Variable {
            name: attributes.required(names.name, attributes.eqname())?,
            select: attributes.optional(names.select, attributes.xpath())?,
            as_: attributes.optional(names.as_, attributes.sequence_type())?,
            static_: attributes.boolean_with_default(names.static_, false)?,
            visibility: attributes
                .optional(names.visibility, attributes.visibility_with_abstract())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }

    fn validate(&self, node: Node, state: &State) -> Result<()> {
        if self.visibility == Some(ast::VisibilityWithAbstract::Abstract) && self.select.is_some() {
            return Err(state
                .attribute_unexpected(
                    node,
                    state.names.select,
                    "select attribute is not allowed when visibility is abstract",
                )
                .into());
        }
        Ok(())
    }
}

impl InstructionParser for ast::When {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::When {
            test: attributes.required(names.test, attributes.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::WherePopulated {
    fn parse(element: &Element, _attributes: &Attributes) -> Result<Self> {
        Ok(ast::WherePopulated {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::WithParam {
    fn parse(element: &Element, attributes: &Attributes) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::WithParam {
            name: attributes.required(names.name, attributes.eqname())?,
            select: attributes.optional(names.select, attributes.xpath())?,
            as_: attributes.optional(names.as_, attributes.sequence_type())?,
            tunnel: attributes.boolean_with_default(names.tunnel, false)?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::context::Context;
    use crate::element::Element;
    use crate::{element::XsltParser, names::Names, state::State};

    use super::*;
    use insta::assert_ron_snapshot;

    fn parse_sequence_constructor_item(s: &str) -> Result<ast::SequenceConstructorItem> {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let (node, span_info) = xot.parse_with_span_info(s).unwrap();
        let state = State::new(xot, span_info, names);
        let node = state.xot.document_element(node).unwrap();

        if let Some(element) = state.xot.element(node) {
            let context = Context::new(element.prefixes().clone());
            let attributes = Attributes::new(node, element, &state, context.clone())?;
            let context = context.sub(element.prefixes(), attributes.standard()?);
            let element = Element::new(node, element, context, &state)?;
            ast::SequenceConstructorItem::parse_sequence_constructor_item(&element, &attributes)
        } else {
            Err(Error::Internal)
        }
    }

    fn parse_transform(s: &str) -> Result<ast::Transform> {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);

        let (node, span_info) = xot.parse_with_span_info(s).unwrap();
        let node = xot.document_element(node).unwrap();
        let context = State::new(xot, span_info, names);
        let parser = XsltParser::new(&context);
        parser.parse_transform(node)
    }

    #[test]
    fn test_if() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_variable() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_missing_required() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_broken_xpath() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" select="let $x := 1">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_sequence_type() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:xs="http://www.w3.org/2001/XMLSchema" name="foo" as="xs:string" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_boolean_default_no_with_explicit_yes() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" static="yes" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" visibility="public">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility_abstract_with_select_is_error() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" visibility="abstract" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_copy() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" copy-namespaces="no" inherit-namespaces="no" validation="strict">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_eqnames() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" use-attribute-sets="foo bar baz">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_eqnames_error() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" use-attribute-sets="foo br!ken bar">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_nested_if() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()"><xsl:if test="true()">Hello</xsl:if></xsl:if>"#
        ));
    }

    #[test]
    fn test_if_with_standard_attribute() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" expand-text="yes">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_literal_result_element() {
        assert_ron_snapshot!(parse_sequence_constructor_item(r#"<foo/>"#));
    }

    #[test]
    fn test_literal_result_element_with_standard_attribute() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<foo xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xsl:expand-text="yes"/>"#
        ));
    }

    #[test]
    fn test_no_fn_namespace_by_default() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="fn:true()">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_attribute_value_template_just_string() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:assert xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" error-code="foo">Hello</xsl:assert>"#
        ));
    }

    #[test]
    fn test_analyze_string() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:matching-substring>Matching</xsl:matching-substring><xsl:non-matching-substring>Nonmatching</xsl:non-matching-substring><xsl:fallback>Fallback 1</xsl:fallback><xsl:fallback>Fallback 2</xsl:fallback></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_absent_matching_substring() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:non-matching-substring>Nonmatching</xsl:non-matching-substring><xsl:fallback>Fallback 1</xsl:fallback><xsl:fallback>Fallback 2</xsl:fallback></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_absent_non_matching_substring() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:matching-substring>Matching</xsl:matching-substring><xsl:fallback>Fallback 1</xsl:fallback><xsl:fallback>Fallback 2</xsl:fallback></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_absent_fallbacks() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:matching-substring>Matching</xsl:matching-substring><xsl:non-matching-substring>Nonmatching</xsl:non-matching-substring></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_absent_all() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_matching_non_matching_wrong_order() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:non-matching-substring>Nonmatching</xsl:non-matching-substring><xsl:matching-substring>Matching</xsl:matching-substring></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_accumulator() {
        assert_ron_snapshot!(parse_transform(
            r#"<xsl:transform version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:accumulator name="foo" initial-value="1"><xsl:accumulator-rule match="foo"/></xsl:accumulator></xsl:transform>"#
        ));
    }

    #[test]
    fn test_should_be_empty_not_empty() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:copy-of xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()">Illegal content</xsl:copy-of>"#
        ))
    }

    #[test]
    fn test_apply_templates_with_mixed_content() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:apply-templates xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:sort>Sort</xsl:sort><xsl:with-param name="a">With param</xsl:with-param></xsl:apply-templates>"#
        ))
    }

    #[test]
    fn test_for_each() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:for-each xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()"><xsl:sort>Sort 1</xsl:sort><xsl:sort>Sort 2</xsl:sort>Sequence constructor</xsl:for-each>"#
        ))
    }

    #[test]
    fn test_fork1() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:fork xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:sequence>Sequence 1</xsl:sequence><xsl:sequence>Sequence 2</xsl:sequence></xsl:fork>"#
        ))
    }

    #[test]
    fn test_fork2() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:fork xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:for-each-group select="true()">Content</xsl:for-each-group></xsl:fork>"#
        ))
    }

    #[test]
    fn test_unsupported_attribute() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" unsupported="Unsupported">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_no_expand_text_should_not_expand_text() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()">Hello {world}!</xsl:if>"#
        ));
    }

    #[test]
    fn test_expand_text_should_expand_text() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" expand-text="yes">Hello {world}!</xsl:if>"#
        ));
    }

    #[test]
    #[ignore]
    fn test_xsl_expand_text_should_expand_text() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()"><p xsl:expand-text="yes">Hello {world}!</p></xsl:if>"#
        ));
    }

    #[test]
    #[ignore]
    fn test_expand_text_disabled_should_not_expand_text() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" expand-text="yes"><p xsl:expand-text="no">Hello {world}!</p></xsl:if>"#
        ));
    }
}
