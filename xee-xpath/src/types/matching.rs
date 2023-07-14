// This module matches types for inline functions or type checks
// The convert module is used for checking and converting values for
// external functions declared with xpath_fn
use xot::Xot;

use xee_schema_type::Xs;
use xee_xpath_ast::ast;

use crate::atomic;
use crate::error;
use crate::occurrence::Occurrence;
use crate::sequence;
use crate::xml;

impl sequence::Sequence {
    pub(crate) fn sequence_type_matching(
        self,
        t: &ast::SequenceType,
        xot: &Xot,
    ) -> error::Result<Self> {
        match t {
            ast::SequenceType::Empty => {
                if self.is_empty() {
                    Ok(self)
                } else {
                    Err(error::Error::Type)
                }
            }
            ast::SequenceType::Item(occurrence_item) => {
                self.occurrence_item_matching(occurrence_item, xot)
            }
        }
    }

    fn occurrence_item_matching(
        self,
        occurrence_item: &ast::Item,
        xot: &Xot,
    ) -> error::Result<Self> {
        let sequence = if let ast::ItemType::AtomicOrUnionType(name) = &occurrence_item.item_type {
            let name = &name.value;
            let xs = Xs::by_name(name.namespace(), name.local_name())
                .ok_or(error::Error::UndefinedTypeReference)?;
            self.atomized_sequence(xot, xs)?
        } else {
            self
        };
        match occurrence_item.occurrence {
            ast::Occurrence::One => {
                let one = sequence.items().one()?;
                one.item_type_matching(&occurrence_item.item_type, xot)?;
                Ok(sequence)
            }
            ast::Occurrence::Option => {
                let option = sequence.items().option()?;
                if let Some(item) = option {
                    item.item_type_matching(&occurrence_item.item_type, xot)?;
                    Ok(sequence)
                } else {
                    Ok(sequence)
                }
            }
            ast::Occurrence::Many => {
                for item in sequence.items() {
                    item?.item_type_matching(&occurrence_item.item_type, xot)?;
                }
                Ok(sequence)
            }
            ast::Occurrence::NonEmpty => {
                todo!("not yet")
            }
        }
    }
}

impl sequence::Item {
    fn item_type_matching(&self, item_type: &ast::ItemType, xot: &Xot) -> error::Result<()> {
        match item_type {
            ast::ItemType::Item => Ok(()),
            ast::ItemType::AtomicOrUnionType(name) => {
                self.to_atomic()?.atomic_type_matching(&name.value)
            }
            ast::ItemType::KindTest(kind_test) => self.kind_test_matching(kind_test, xot),
            _ => {
                todo!("not yet")
            }
        }
    }

    fn kind_test_matching(&self, kind_test: &ast::KindTest, xot: &Xot) -> error::Result<()> {
        match self {
            sequence::Item::Node(node) => {
                if xml::kind_test(kind_test, xot, *node) {
                    Ok(())
                } else {
                    Err(error::Error::Type)
                }
            }
            sequence::Item::Atomic(_) => Err(error::Error::Type),
            sequence::Item::Function(_) => Err(error::Error::Type),
        }
    }
}

impl atomic::Atomic {
    fn atomic_type_matching(&self, name: &ast::Name) -> error::Result<()> {
        // XXX error should be detectable statically, earlier
        let xs = Xs::by_name(name.namespace(), name.local_name())
            .ok_or(error::Error::UndefinedTypeReference)?;
        if self.schema_type().derives_from(xs) {
            Ok(())
        } else {
            Err(error::Error::Type)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ibig::ibig;

    use xee_xpath_ast::ast;
    use xee_xpath_ast::Namespaces;

    use crate::xml;

    #[test]
    fn test_one_integer() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("xs:integer", &namespaces).unwrap();

        let right_sequence = sequence::Sequence::from(vec![sequence::Item::from(ibig!(1))]);
        let wrong_amount_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(ibig!(1)),
            sequence::Item::from(ibig!(2)),
        ]);
        let wrong_type_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(false))]);
        let xot = Xot::new();

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(&right_result.unwrap(), &right_sequence);

        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let wrong_type_result = wrong_type_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_type_result, Err(error::Error::Type));
    }

    #[test]
    fn test_one_long_matches_integer() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("xs:integer", &namespaces).unwrap();

        let right_sequence = sequence::Sequence::from(vec![sequence::Item::from(1i64)]);
        let wrong_amount_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(1i64), sequence::Item::from(1i64)]);
        let wrong_type_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(false))]);
        let xot = Xot::new();

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(right_sequence));
        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let wrong_type_result = wrong_type_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_type_result, Err(error::Error::Type));
    }

    #[test]
    fn test_one_any_atomic() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("xs:anyAtomicType", &namespaces).unwrap();

        let right_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(1i64))]);
        let wrong_amount_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(ibig!(1)),
            sequence::Item::from(ibig!(2)),
        ]);
        let right_type_sequence2 =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(false))]);
        let xot = Xot::new();

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(right_sequence));
        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let right_type_result2 = right_type_sequence2
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_type_result2, Ok(right_type_sequence2));
    }

    #[test]
    fn test_one_item() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("item()", &namespaces).unwrap();
        let mut xot = Xot::new();
        let root = xot.parse("<doc/>").unwrap();
        let node = xot.document_element(root).unwrap();
        let node = xml::Node::Xot(node);
        let right_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(1i64))]);
        let wrong_amount_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(atomic::Atomic::from(1i64)),
            sequence::Item::from(atomic::Atomic::from(2i64)),
        ]);
        let right_type_sequence2 = sequence::Sequence::from(vec![sequence::Item::from(node)]);

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(right_sequence));

        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let right_type_result2 = right_type_sequence2
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_type_result2, Ok(right_type_sequence2));
    }

    #[test]
    fn test_option_integer() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("xs:integer?", &namespaces).unwrap();

        let right_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(ibig!(1)))]);
        let wrong_amount_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(ibig!(1)),
            sequence::Item::from(ibig!(2)),
        ]);
        let right_empty_sequence = sequence::Sequence::empty();
        let xot = Xot::new();

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(right_sequence));
        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let right_empty_result = right_empty_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_empty_result, Ok(right_empty_sequence));
    }

    #[test]
    fn test_many_integer() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("xs:integer*", &namespaces).unwrap();

        let right_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(ibig!(1)))]);
        let right_multi_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(ibig!(1)),
            sequence::Item::from(ibig!(2)),
        ]);
        let right_empty_sequence = sequence::Sequence::empty();
        let xot = Xot::new();

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(right_sequence));

        let right_multi_result = right_multi_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_multi_result, Ok(right_multi_sequence));

        let right_empty_result = right_empty_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_empty_result, Ok(right_empty_sequence));
    }

    #[test]
    fn test_many_node() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("node()*", &namespaces).unwrap();

        let mut xot = Xot::new();
        let doc = xot.parse(r#"<doc><a attr="Attr">A</a><b/></doc>"#).unwrap();
        let doc = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc).unwrap();
        let b = xot.next_sibling(a).unwrap();
        let text = xot.first_child(a).unwrap();

        let doc = xml::Node::Xot(doc);
        let attr = xml::Node::Attribute(a, xot.name("attr").unwrap());
        let a = xml::Node::Xot(a);
        let b = xml::Node::Xot(b);
        let text = xml::Node::Xot(text);

        let right_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(doc),
            sequence::Item::from(a),
            sequence::Item::from(b),
            sequence::Item::from(text),
            sequence::Item::from(attr),
        ]);

        let wrong_sequence = sequence::Sequence::from(vec![sequence::Item::from(ibig!(1))]);

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(right_sequence));

        let wrong_result = wrong_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_result, Err(error::Error::Type));
    }

    #[test]
    fn test_many_element() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("element()*", &namespaces).unwrap();

        let mut xot = Xot::new();
        let doc = xot.parse(r#"<doc><a attr="Attr">A</a><b/></doc>"#).unwrap();
        let doc = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc).unwrap();
        let b = xot.next_sibling(a).unwrap();
        let text = xot.first_child(a).unwrap();

        let doc = xml::Node::Xot(doc);
        let attr = xml::Node::Attribute(a, xot.name("attr").unwrap());
        let a = xml::Node::Xot(a);
        let b = xml::Node::Xot(b);
        let text = xml::Node::Xot(text);

        let right_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(doc),
            sequence::Item::from(a),
            sequence::Item::from(b),
        ]);

        let wrong_sequence_text = sequence::Sequence::from(vec![sequence::Item::from(text)]);
        let wrong_sequence_attr = sequence::Sequence::from(vec![sequence::Item::from(attr)]);

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(right_sequence));

        let wrong_result = wrong_sequence_text.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_result, Err(error::Error::Type));
        let wrong_result = wrong_sequence_attr.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_result, Err(error::Error::Type));
    }

    #[test]
    fn test_many_atomized() {
        let namespaces = Namespaces::default();
        let sequence_type = ast::SequenceType::parse("xs:integer*", &namespaces).unwrap();

        let mut xot = Xot::new();
        let doc = xot.parse(r#"<doc><a>1</a><b>2</b></doc>"#).unwrap();
        let doc = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc).unwrap();
        let b = xot.next_sibling(a).unwrap();

        let a = xml::Node::Xot(a);
        let b = xml::Node::Xot(b);

        let right_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(a), sequence::Item::from(b)]);

        let right_result = right_sequence
            .clone()
            .sequence_type_matching(&sequence_type, &xot);
        // atomization has changed the result sequence
        assert_ne!(right_result, Ok(right_sequence));
        assert_eq!(
            right_result,
            Ok(sequence::Sequence::from(vec![
                sequence::Item::from(atomic::Atomic::from(ibig!(1))),
                sequence::Item::from(atomic::Atomic::from(ibig!(2))),
            ]))
        );
    }
}
