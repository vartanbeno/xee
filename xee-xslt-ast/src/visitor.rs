use blanket::blanket;

use crate::ast_core as ast;

#[blanket(default = "visit")]
pub(crate) trait AstVisitor {
    fn visit_transform(&mut self, transform: &mut ast::Transform);
    fn visit_declaration(&mut self, declaration: &mut ast::Declaration);
    fn visit_accumulator(&mut self, accumulator: &mut ast::Accumulator);
    fn visit_accumulator_rule(&mut self, rule: &mut ast::AccumulatorRule);
    fn visit_character_map(&mut self, character_map: &mut ast::CharacterMap);
    fn visit_output_character(&mut self, output_character: &mut ast::OutputCharacter);
    fn visit_sequence_constructor(&mut self, sequence_constructor: &mut ast::SequenceConstructor);
    fn visit_sequence_constructor_item(&mut self, item: &mut ast::SequenceConstructorItem);
}

pub(crate) mod visit {
    use super::AstVisitor;
    use crate::ast_core as ast;

    pub(crate) fn visit_transform<V: AstVisitor + ?Sized>(
        v: &mut V,
        transform: &mut ast::Transform,
    ) {
        for declaration in transform.declarations.iter_mut() {
            v.visit_declaration(declaration)
        }
    }

    pub(crate) fn visit_declaration<V: AstVisitor + ?Sized>(
        v: &mut V,
        declaration: &mut ast::Declaration,
    ) {
        use ast::Declaration::*;
        match declaration {
            Accumulator(accumulator) => v.visit_accumulator(accumulator),
            CharacterMap(character_map) => v.visit_character_map(character_map),
            // DecimalFormat(decimal_format) => v.visit_decimal_format(decimal_format),
            // Function(function) => v.visit_function(function),
            _ => {}
        }
    }

    pub(crate) fn visit_accumulator<V: AstVisitor + ?Sized>(
        v: &mut V,
        accumulator: &mut ast::Accumulator,
    ) {
        for rule in accumulator.rules.iter_mut() {
            v.visit_accumulator_rule(rule)
        }
    }

    pub(crate) fn visit_accumulator_rule<V: AstVisitor + ?Sized>(
        v: &mut V,
        rule: &mut ast::AccumulatorRule,
    ) {
        v.visit_sequence_constructor(&mut rule.sequence_constructor)
    }

    pub(crate) fn visit_character_map<V: AstVisitor + ?Sized>(
        v: &mut V,
        character_map: &mut ast::CharacterMap,
    ) {
        for output_character in character_map.output_characters.iter_mut() {
            v.visit_output_character(output_character)
        }
    }

    pub(crate) fn visit_output_character<V: AstVisitor + ?Sized>(
        _v: &mut V,
        _output_character: &mut ast::OutputCharacter,
    ) {
        // no children
    }

    pub(crate) fn visit_sequence_constructor<V: AstVisitor + ?Sized>(
        v: &mut V,
        sequence_constructor: &mut ast::SequenceConstructor,
    ) {
        for item in sequence_constructor.iter_mut() {
            v.visit_sequence_constructor_item(item)
        }
    }

    pub(crate) fn visit_sequence_constructor_item<V: AstVisitor + ?Sized>(
        _v: &mut V,
        _sequence_constructor_item: &mut ast::SequenceConstructorItem,
    ) {
        // TODO
    }
}
