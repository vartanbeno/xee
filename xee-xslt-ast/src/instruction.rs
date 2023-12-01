use xot::{NameId, Xot};

use crate::ast_core::{self as ast};
use crate::combinator::{one, NodeParser};
use crate::element::{by_element, content_parse, instruction, sequence_constructor, Element};
use crate::error::ElementError as Error;

type Result<V> = std::result::Result<V, Error>;

pub(crate) trait InstructionParser: Sized {
    fn validate(&self, _element: &Element) -> Result<()> {
        Ok(())
    }

    fn should_be_empty() -> bool {
        false
    }

    fn parse(element: &Element) -> Result<Self>;

    fn parse_and_validate(element: &Element) -> Result<Self> {
        if Self::should_be_empty() {
            if let Some(child) = element.state.xot.first_child(element.node) {
                return Err(Error::Unexpected {
                    span: element.state.span(child).ok_or(Error::Internal)?,
                });
            }
        }
        let item = Self::parse(element)?;
        item.validate(element)?;
        let unseen_attributes = element.unseen_attributes();
        if !unseen_attributes.is_empty() {
            return Err(element
                .attribute_unexpected(unseen_attributes[0], "unexpected attribute")
                .into());
        }
        Ok(item)
    }
}

pub(crate) trait SequenceConstructorParser:
    InstructionParser + Into<ast::SequenceConstructorItem>
{
    fn parse_sequence_constructor_item(element: &Element) -> Result<ast::SequenceConstructorItem> {
        let item = Self::parse_and_validate(element)?;
        Ok(item.into())
    }
}

impl<T> SequenceConstructorParser for T where
    T: InstructionParser + Into<ast::SequenceConstructorItem>
{
}

pub(crate) trait DeclarationParser: InstructionParser + Into<ast::Declaration> {
    fn parse_declaration(element: &Element) -> Result<ast::Declaration> {
        let item = Self::parse_and_validate(element)?;
        Ok(item.into())
    }
}

impl<T> DeclarationParser for T where T: InstructionParser + Into<ast::Declaration> {}

pub(crate) trait OverrideContentParser:
    InstructionParser + Into<ast::OverrideContent>
{
    fn parse_override_content(element: &Element) -> Result<ast::OverrideContent> {
        let item = Self::parse_and_validate(element)?;
        Ok(item.into())
    }
}

impl<T> OverrideContentParser for T where T: InstructionParser + Into<ast::OverrideContent> {}

impl InstructionParser for ast::SequenceConstructorItem {
    fn parse(element: &Element) -> Result<ast::SequenceConstructorItem> {
        let context = element.state;
        let name = context
            .names
            .sequence_constructor_name(element.element.name());

        if let Some(name) = name {
            name.parse(element)
        } else {
            let ns = context.xot.namespace_for_name(element.element.name());
            if ns == context.names.xsl_ns {
                // we have an unknown xsl instruction, fail with error
                Err(Error::Unexpected { span: element.span })
            } else {
                // we parse the literal element
                ast::ElementNode::parse_sequence_constructor_item(element)
            }
        }
    }
}

impl InstructionParser for ast::Declaration {
    fn parse(element: &Element) -> Result<ast::Declaration> {
        let name = element.state.names.declaration_name(element.element.name());

        if let Some(name) = name {
            name.parse(element)
        } else {
            Err(Error::Unexpected { span: element.span })
        }
    }
}

impl InstructionParser for ast::ElementNode {
    fn parse(element: &Element) -> Result<ast::ElementNode> {
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

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Accept {
            component: element.required(names.component, element.component())?,
            names: element.required(names.names, element.tokens())?,
            visibility: element.required(names.visibility, element.visibility_with_hidden())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Accumulator {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let parse_rules = content_parse(instruction(names.xsl_accumulator_rule).many());

        Ok(ast::Accumulator {
            name: element.required(names.name, element.eqname())?,
            initial_value: element.required(names.initial_value, element.xpath())?,
            as_: element.optional(names.as_, element.sequence_type())?,
            streamable: element.boolean_with_default(names.streamable, false)?,

            span: element.span,

            rules: parse_rules(element)?,
        })
    }
}

impl InstructionParser for ast::AccumulatorRule {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::AccumulatorRule {
            match_: element.required(names.match_, element.pattern())?,
            phase: element.optional(names.phase, element.phase())?,
            select: element.optional(names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::AnalyzeString {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let select = element.required(names.select, element.xpath())?;
        let regex = element.required(names.regex, element.value_template(element.string()))?;
        let flags = element.optional(names.flags, element.value_template(element.string()))?;

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

            span: element.span,

            matching_substring,
            non_matching_substring,
            fallbacks,
        })
    }
}

impl InstructionParser for ast::ApplyImports {
    fn parse(element: &Element) -> Result<Self> {
        let parse = content_parse(instruction(element.state.names.xsl_with_param).many());
        Ok(ast::ApplyImports {
            span: element.span,

            with_params: parse(element)?,
        })
    }
}

impl InstructionParser for ast::ApplyTemplates {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(
            (instruction(names.xsl_with_param)
                .map(ast::ApplyTemplatesContent::WithParam)
                .or(instruction(names.xsl_sort).map(ast::ApplyTemplatesContent::Sort)))
            .many(),
        );

        Ok(ast::ApplyTemplates {
            select: element.optional(names.select, element.xpath())?,
            mode: element.optional(names.mode, element.token())?,

            span: element.span,

            content: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Assert {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Assert {
            test: element.required(names.test, element.xpath())?,
            select: element.optional(names.select, element.xpath())?,
            error_code: element
                .optional(names.error_code, element.value_template(element.eqname()))?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Attribute {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Attribute {
            name: element.required(names.name, element.value_template(element.qname()))?,
            namespace: element.optional(names.namespace, element.value_template(element.uri()))?,
            select: element.optional(names.select, element.xpath())?,
            separator: element
                .optional(names.separator, element.value_template(element.string()))?,
            type_: element.optional(names.type_, element.eqname())?,
            validation: element.optional(names.validation, element.validation())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::AttributeSet {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let parse = content_parse(instruction(names.xsl_attribute).many());

        Ok(ast::AttributeSet {
            name: element.required(names.name, element.eqname())?,
            use_attribute_sets: element.optional(names.use_attribute_sets, element.eqnames())?,
            visibility: element.optional(names.visibility, element.visibility_with_abstract())?,
            streamable: element.boolean_with_default(names.streamable, false)?,

            span: element.span,

            attributes: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Break {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::Break {
            select: element.optional(element.state.names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::CallTemplate {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(instruction(element.state.names.xsl_with_param).many());

        Ok(ast::CallTemplate {
            name: element.required(names.name, element.eqname())?,

            span: element.span,

            with_params: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Catch {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Catch {
            errors: element.optional(names.errors, element.tokens())?,
            select: element.optional(names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::CharacterMap {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let parse = content_parse(instruction(names.xsl_output_character).many());
        Ok(ast::CharacterMap {
            name: element.required(names.name, element.eqname())?,
            use_character_maps: element.optional(names.use_character_maps, element.eqnames())?,

            span: element.span,

            output_characters: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Choose {
    fn parse(element: &Element) -> Result<Self> {
        let parse = content_parse(
            instruction(element.state.names.xsl_when)
                .one_or_more()
                .then(instruction(element.state.names.xsl_otherwise).option()),
        );

        let (when, otherwise) = parse(element)?;
        Ok(ast::Choose {
            span: element.span,

            when,
            otherwise,
        })
    }
}

impl InstructionParser for ast::Comment {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::Comment {
            select: element.optional(element.state.names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::ContextItem {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::ContextItem {
            as_: element.optional(names.as_, element.item_type())?,
            use_: element.optional(names.use_, element.use_())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Copy {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Copy {
            select: element.optional(names.select, element.xpath())?,
            copy_namespaces: element.boolean_with_default(names.copy_namespaces, true)?,
            inherit_namespaces: element.boolean_with_default(names.inherit_namespaces, true)?,
            use_attribute_sets: element.optional(names.use_attribute_sets, element.eqnames())?,
            type_: element.optional(names.type_, element.eqname())?,
            validation: element
                .optional(names.validation, element.validation())?
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

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::CopyOf {
            select: element.required(names.select, element.xpath())?,
            copy_accumulators: element.boolean_with_default(names.copy_accumulators, false)?,
            copy_namespaces: element.boolean_with_default(names.copy_namespaces, true)?,
            type_: element.optional(names.type_, element.eqname())?,
            validation: element.optional(names.validation, element.validation())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::DecimalFormat {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::DecimalFormat {
            name: element.optional(names.name, element.eqname())?,
            decimal_separator: element.optional(names.decimal_separator, element.char())?,
            grouping_separator: element.optional(names.grouping_separator, element.char())?,
            infinity: element.optional(names.infinity, element.string())?,
            minus_sign: element.optional(names.minus_sign, element.char())?,
            exponent_separator: element.optional(names.exponent_separator, element.char())?,
            nan: element.optional(names.nan, element.string())?,
            percent: element.optional(names.percent, element.char())?,
            per_mille: element.optional(names.per_mille, element.char())?,
            zero_digit: element.optional(names.zero_digit, element.char())?,
            digit: element.optional(names.digit, element.char())?,
            pattern_separator: element.optional(names.pattern_separator, element.char())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Document {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Document {
            validation: element.optional(names.validation, element.validation())?,
            type_: element.optional(names.type_, element.eqname())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Element {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Element {
            name: element.required(names.name, element.value_template(element.qname()))?,
            namespace: element.optional(names.namespace, element.value_template(element.uri()))?,
            inherit_namespaces: element.boolean_with_default(names.inherit_namespaces, false)?,
            use_attribute_sets: element.optional(names.use_attribute_sets, element.eqnames())?,
            type_: element.optional(names.type_, element.eqname())?,
            validation: element.optional(names.validation, element.validation())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Evaluate {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(
            instruction(names.xsl_with_param)
                .map(ast::EvaluateContent::WithParam)
                .or(instruction(names.xsl_fallback).map(ast::EvaluateContent::Fallback))
                .many(),
        );

        Ok(ast::Evaluate {
            xpath: element.required(names.xpath, element.xpath())?,
            as_: element.optional(names.as_, element.sequence_type())?,
            base_uri: element.optional(names.base_uri, element.value_template(element.uri()))?,
            with_params: element.optional(names.with_params, element.xpath())?,
            context_item: element.optional(names.context_item, element.xpath())?,
            namespace_context: element.optional(names.namespace_context, element.xpath())?,
            schema_aware: element.optional(
                names.schema_aware,
                element.value_template(element.boolean()),
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

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Expose {
            component: element.required(names.component, element.component())?,
            names: element.required(names.names, element.tokens())?,
            visibility: element.required(names.visibility, element.visibility_with_abstract())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Fallback {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::Fallback {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::ForEach {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let select = element.required(names.select, element.xpath())?;
        let parse = content_parse(
            instruction(names.xsl_sort)
                .many()
                .then(sequence_constructor()),
        );
        let (sort, sequence_constructor) = parse(element)?;

        Ok(ast::ForEach {
            select,

            span: element.span,

            sort,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::ForEachGroup {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let select = element.required(names.select, element.xpath())?;
        let group_by = element.optional(names.group_by, element.xpath())?;
        let group_adjacent = element.optional(names.group_adjacent, element.xpath())?;
        let group_starting_with = element.optional(names.group_starting_with, element.pattern())?;
        let group_ending_with = element.optional(names.group_ending_with, element.pattern())?;
        let composite = element.boolean_with_default(names.composite, false)?;
        let collation = element.optional(names.collation, element.value_template(element.uri()))?;

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

            span: element.span,

            sort,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::Fork {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

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
            span: element.span,

            fallbacks,
            content,
        })
    }
}

impl InstructionParser for ast::Function {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let name = element.required(names.name, element.eqname())?;
        let as_ = element.optional(names.as_, element.sequence_type())?;
        let visibility = element.optional(names.visibility, element.visibility_with_abstract())?;
        let streamability = element.optional(names.streamability, element.streamability())?;
        let override_extension_function =
            element.boolean_with_default(names.override_extension_function, false)?;
        // deprecated
        let override_ = element.boolean_with_default(names.override_, false)?;
        let new_each_time = element.optional(names.new_each_time, element.new_each_time())?;
        let cache = element.boolean_with_default(names.cache, false)?;

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

            span: element.span,

            params,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::GlobalContextItem {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::GlobalContextItem {
            as_: element.optional(names.as_, element.item_type())?,
            use_: element.optional(names.use_, element.use_())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::If {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::If {
            test: element.required(names.test, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Import {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Import {
            href: element.required(names.href, element.uri())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::ImportSchema {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::ImportSchema {
            namespace: element.optional(names.namespace, element.uri())?,
            schema_location: element.optional(names.schema_location, element.uri())?,

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

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Include {
            href: element.required(names.href, element.uri())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Iterate {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let select = element.required(names.select, element.xpath())?;

        let parse = content_parse(
            instruction(names.xsl_param)
                .many()
                .then(instruction(names.xsl_on_completion).option())
                .then(sequence_constructor()),
        );
        let ((params, on_completion), sequence_constructor) = parse(element)?;

        Ok(ast::Iterate {
            select,

            span: element.span,

            params,
            on_completion,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::Key {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::Key {
            name: element.required(names.name, element.eqname())?,
            match_: element.required(names.match_, element.pattern())?,
            use_: element.optional(names.use_, element.xpath())?,
            composite: element.boolean_with_default(names.composite, false)?,
            collation: element.optional(names.collation, element.uri())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Map {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::Map {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MapEntry {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::MapEntry {
            key: element.required(names.key, element.xpath())?,
            select: element.optional(names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MatchingSubstring {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::MatchingSubstring {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Merge {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(instruction(names.xsl_merge_source).one_or_more().then(
            instruction(names.xsl_merge_action).then(instruction(names.xsl_fallback).many()),
        ));

        let (merge_sources, (merge_action, fallbacks)) = parse(element)?;

        Ok(ast::Merge {
            span: element.span,

            merge_sources,
            merge_action,
            fallbacks,
        })
    }
}

impl InstructionParser for ast::MergeAction {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::MergeAction {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MergeKey {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::MergeKey {
            select: element.optional(names.select, element.xpath())?,
            lang: element.optional(names.lang, element.value_template(element.language()))?,
            order: element.optional(names.order, element.value_template(element.order()))?,
            collation: element.optional(names.collation, element.value_template(element.uri()))?,
            case_order: element.optional(
                names.case_order,
                element.value_template(element.case_order()),
            )?,
            data_type: element
                .optional(names.data_type, element.value_template(element.data_type()))?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MergeSource {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let parse = content_parse(instruction(names.xsl_merge_key).one_or_more());

        Ok(ast::MergeSource {
            name: element.optional(names.name, element.ncname())?,
            for_each_item: element.optional(names.for_each_item, element.xpath())?,
            for_each_source: element.optional(names.for_each_source, element.xpath())?,
            select: element.required(names.select, element.xpath())?,
            streamable: element.boolean_with_default(names.streamable, false)?,
            use_accumulators: element.optional(names.use_accumulators, element.tokens())?,
            sort_before_merge: element.boolean_with_default(names.sort_before_merge, false)?,
            validation: element.optional(names.validation, element.validation())?,
            type_: element.optional(names.type_, element.eqname())?,

            span: element.span,

            merge_keys: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Message {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Message {
            select: element.optional(names.select, element.xpath())?,
            terminate: element
                .optional(names.terminate, element.value_template(element.boolean()))?,
            error_code: element
                .optional(names.error_code, element.value_template(element.eqname()))?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Mode {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Mode {
            name: element.optional(names.name, element.eqname())?,
            streamable: element.boolean_with_default(names.streamable, false)?,
            use_accumulators: element.optional(names.use_accumulators, element.tokens())?,
            on_no_match: element.optional(names.on_no_match, element.on_no_match())?,
            on_multiple_match: element
                .optional(names.on_multiple_match, element.on_multiple_match())?,
            warning_on_no_match: element.boolean_with_default(names.warning_on_no_match, false)?,
            warning_on_multiple_match: element
                .boolean_with_default(names.warning_on_multiple_match, false)?,
            typed: element.optional(names.typed, element.typed())?,
            visibility: element.optional(names.visibility, element.visibility())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Namespace {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Namespace {
            name: element.required(names.name, element.value_template(element.ncname()))?,
            select: element.optional(names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::NamespaceAlias {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::NamespaceAlias {
            stylesheet_prefix: element
                .required(names.stylesheet_prefix, element.prefix_or_default())?,
            result_prefix: element.required(names.result_prefix, element.prefix_or_default())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::NextIteration {
    fn parse(element: &Element) -> Result<Self> {
        let parse = content_parse(instruction(element.state.names.xsl_with_param).many());
        Ok(ast::NextIteration {
            span: element.span,

            with_params: parse(element)?,
        })
    }
}

impl InstructionParser for ast::NextMatch {
    fn parse(element: &Element) -> Result<Self> {
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
    fn parse(element: &Element) -> Result<Self> {
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

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::Number {
            value: element.optional(names.value, element.xpath())?,
            select: element.optional(names.select, element.xpath())?,
            level: element.optional(names.level, element.level())?,
            count: element.optional(names.count, element.pattern())?,
            from: element.optional(names.from, element.pattern())?,
            format: element.optional(names.format, element.value_template(element.string()))?,
            lang: element.optional(names.lang, element.value_template(element.language()))?,
            letter_value: element.optional(
                names.letter_value,
                element.value_template(element.letter_value()),
            )?,
            ordinal: element.optional(names.ordinal, element.value_template(element.string()))?,
            start_at: element.optional(names.start_at, element.value_template(element.string()))?,
            grouping_separator: element.optional(
                names.grouping_separator,
                element.value_template(element.char()),
            )?,
            grouping_size: element.optional(
                names.grouping_size,
                element.value_template(element.integer()),
            )?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::OnCompletion {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::OnCompletion {
            select: element.optional(element.state.names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::OnEmpty {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::OnEmpty {
            select: element.optional(element.state.names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::OnNonEmpty {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::OnNonEmpty {
            select: element.optional(element.state.names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Otherwise {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::Otherwise {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Output {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Output {
            name: element.optional(names.name, element.eqname())?,
            method: element.optional(names.method, element.method())?,
            allow_duplicate_names: element
                .boolean_with_default(names.allow_duplicate_names, false)?,
            build_tree: element.boolean_with_default(names.build_tree, false)?,
            byte_order_mark: element.boolean_with_default(names.byte_order_mark, false)?,
            cdata_section_elements: element
                .optional(names.cdata_section_elements, element.eqnames())?,
            doctype_public: element.optional(names.doctype_public, element.string())?,
            doctype_system: element.optional(names.doctype_system, element.string())?,
            encoding: element.optional(names.encoding, element.string())?,
            escape_uri_attributes: element
                .boolean_with_default(names.escape_uri_attributes, true)?,
            html_version: element.optional(names.html_version, element.decimal())?,
            include_content_type: element.boolean_with_default(names.include_content_type, true)?,
            // TODO default value is informed by the method
            indent: element.boolean_with_default(names.indent, false)?,
            item_separator: element.optional(names.item_separator, element.string())?,
            json_node_output_method: element.optional(
                names.json_node_output_method,
                element.json_node_output_method(),
            )?,
            media_type: element.optional(names.media_type, element.string())?,
            normalization_form: element
                .optional(names.normalization_form, element.normalization_form())?,
            omit_xml_declaration: element
                .boolean_with_default(names.omit_xml_declaration, false)?,
            parameter_document: element.optional(names.parameter_document, element.uri())?,
            standalone: element.optional(names.standalone, element.standalone())?,
            suppress_indentation: element
                .optional(names.suppress_indentation, element.eqnames())?,
            undeclare_prefixes: element.boolean_with_default(names.undeclare_prefixes, false)?,
            use_character_maps: element.optional(names.use_character_maps, element.eqnames())?,
            version: element.optional(names.version, element.nmtoken())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::OutputCharacter {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::OutputCharacter {
            character: element.required(names.character, element.char())?,
            string: element.required(names.string, element.string())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::OverrideContent {
    fn parse(element: &Element) -> Result<ast::OverrideContent> {
        let name = element
            .state
            .names
            .override_content_name(element.element.name());

        if let Some(name) = name {
            name.parse(element)
        } else {
            Err(Error::Unexpected { span: element.span })
        }
    }
}

impl InstructionParser for ast::Override {
    fn parse(element: &Element) -> Result<Self> {
        let parse = content_parse(
            one(by_element(|element| {
                ast::OverrideContent::parse_override_content(&element)
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
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Param {
            name: element.required(names.name, element.eqname())?,
            select: element.optional(names.select, element.xpath())?,
            as_: element.optional(names.as_, element.sequence_type())?,
            required: element.boolean_with_default(names.required, false)?,
            tunnel: element.boolean_with_default(names.tunnel, false)?,
            static_: element.boolean_with_default(names.static_, false)?,

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

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::PreserveSpace {
            elements: element.required(names.elements, element.tokens())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::ProcessingInstruction {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::ProcessingInstruction {
            name: element.required(names.name, element.value_template(element.ncname()))?,
            select: element.optional(names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

// TODO: xsl:result-document

impl InstructionParser for ast::Sequence {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Sequence {
            select: element.optional(names.select, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Sort {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Sort {
            select: element.optional(names.select, element.xpath())?,
            lang: element.optional(names.lang, element.value_template(element.language()))?,
            order: element.optional(names.order, element.value_template(element.order()))?,
            collation: element.optional(names.collation, element.value_template(element.uri()))?,
            stable: element.optional(names.stable, element.value_template(element.boolean()))?,
            case_order: element.optional(
                names.case_order,
                element.value_template(element.case_order()),
            )?,
            data_type: element
                .optional(names.data_type, element.value_template(element.data_type()))?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::SourceDocument {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::SourceDocument {
            href: element.required(names.href, element.value_template(element.uri()))?,
            streamable: element.boolean_with_default(names.streamable, false)?,
            use_accumulators: element.optional(names.use_accumulators, element.tokens())?,
            validation: element.optional(names.validation, element.validation())?,
            type_: element.optional(names.type_, element.eqname())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::StripSpace {
    fn should_be_empty() -> bool {
        true
    }
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::StripSpace {
            elements: element.required(names.elements, element.tokens())?,

            span: element.span,
        })
    }
}

impl InstructionParser for ast::Template {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let match_ = element.optional(names.match_, element.pattern())?;
        let name = element.optional(names.name, element.eqname())?;
        let priority = element.optional(names.priority, element.decimal())?;
        let mode = element.optional(names.mode, element.tokens())?;
        let as_ = element.optional(names.as_, element.sequence_type())?;
        let visibility = element.optional(names.visibility, element.visibility_with_abstract())?;

        let parse = content_parse(
            instruction(names.context_item)
                .option()
                .then(instruction(names.xsl_param).many())
                .then(sequence_constructor()),
        );
        let ((context_item, params), sequence_constructor) = parse(element)?;

        Ok(ast::Template {
            match_,
            name,
            priority,
            mode,
            as_,
            visibility,

            span: element.span,

            context_item,
            params,
            sequence_constructor,
        })
    }
}

impl InstructionParser for ast::Text {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Text {
            disable_output_escaping: element
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
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Transform {
            id: element.optional(names.id, element.id())?,
            input_type_annotations: element.optional(
                names.input_type_annotations,
                element.input_type_annotations(),
            )?,
            extension_element_prefixes: element
                .optional(names.extension_element_prefixes, element.prefixes())?,

            span: element.span,

            declarations: element.declarations()?,
        })
    }
}

// TODO: xsl:try

// TODO: xsl:use-package

impl InstructionParser for ast::ValueOf {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::ValueOf {
            select: element.optional(names.select, element.xpath())?,
            separator: element
                .optional(names.separator, element.value_template(element.string()))?,
            disable_output_escaping: element
                .boolean_with_default(names.disable_output_escaping, false)?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Variable {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        // This is a rule somewhere, but not sure whether it's correct;
        // can visibility be absent or is there a default visibility?
        // let visibility = visibility.unwrap_or(if static_ {
        //     ast::VisibilityWithAbstract::Private
        // } else {
        //     ast::VisibilityWithAbstract::Public
        // });

        Ok(ast::Variable {
            name: element.required(names.name, element.eqname())?,
            select: element.optional(names.select, element.xpath())?,
            as_: element.optional(names.as_, element.sequence_type())?,
            static_: element.boolean_with_default(names.static_, false)?,
            visibility: element.optional(names.visibility, element.visibility_with_abstract())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }

    fn validate(&self, element: &Element) -> Result<()> {
        if self.visibility == Some(ast::VisibilityWithAbstract::Abstract) && self.select.is_some() {
            return Err(element
                .attribute_unexpected(
                    element.state.names.select,
                    "select attribute is not allowed when visibility is abstract",
                )
                .into());
        }
        Ok(())
    }
}

impl InstructionParser for ast::When {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::When {
            test: element.required(names.test, element.xpath())?,

            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::WherePopulated {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::WherePopulated {
            span: element.span,

            sequence_constructor: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::WithParam {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::WithParam {
            name: element.required(names.name, element.eqname())?,
            select: element.optional(names.select, element.xpath())?,
            as_: element.optional(names.as_, element.sequence_type())?,
            tunnel: element.boolean_with_default(names.tunnel, false)?,

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
            let context = Context::new(element);
            let element = Element::new(node, element, context, &state)?;
            ast::SequenceConstructorItem::parse_sequence_constructor_item(&element)
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
    #[ignore]
    fn test_no_expand_text_should_not_expand_text() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()">Hello {world}!</xsl:if>"#
        ));
    }

    #[test]
    #[ignore]
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
