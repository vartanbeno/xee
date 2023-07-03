// This module matches types for inline functions or type checks
// The convert module is used for checking and converting values for
// external functions declared with xpath_fn
use std::borrow::Cow;
use xee_xpath_ast::{ast, XS_NAMESPACE};
use xot::Xot;

use crate::atomic;
use crate::error;
use crate::occurrence::Occurrence;
use crate::sequence;

impl sequence::Sequence {
    fn sequence_type_matching(
        &self,
        t: &ast::SequenceType,
        xot: &Xot,
    ) -> error::Result<Cow<sequence::Sequence>> {
        match t {
            ast::SequenceType::Empty => {
                if self.is_empty() {
                    Ok(Cow::Borrowed(self))
                } else {
                    Err(error::Error::Type)
                }
            }
            ast::SequenceType::Item(occurrence_item) => {
                self.occurrence_item_matching(occurrence_item, xot)
            }
            _ => todo!("Not yet implemented"),
        }
    }

    fn occurrence_item_matching(
        &self,
        occurrence_item: &ast::Item,
        xot: &Xot,
    ) -> error::Result<Cow<sequence::Sequence>> {
        let sequence = if occurrence_item.item_type.is_generalized_atomic_type() {
            Cow::Owned(self.atomized_sequence(xot)?)
        } else {
            Cow::Borrowed(self)
        };
        match occurrence_item.occurrence {
            ast::Occurrence::One => {
                let one = sequence.items().one()?;
                one.item_type_matching(&occurrence_item.item_type)?;
                Ok(sequence)
            }
            ast::Occurrence::Option => {
                let option = sequence.items().option()?;
                if let Some(item) = option {
                    item.item_type_matching(&occurrence_item.item_type)?;
                    Ok(sequence)
                } else {
                    Ok(sequence)
                }
            }
            ast::Occurrence::Many => {
                for item in sequence.items() {
                    item?.item_type_matching(&occurrence_item.item_type)?;
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
    fn item_type_matching(&self, item_type: &ast::ItemType) -> error::Result<()> {
        match item_type {
            ast::ItemType::Item => Ok(()),
            ast::ItemType::AtomicOrUnionType(name) => self.to_atomic()?.atomic_type_matching(name),
            _ => {
                todo!("not yet")
            }
        }
    }
}

impl atomic::Atomic {
    fn atomic_type_matching(&self, name: &ast::Name) -> error::Result<()> {
        if let Some(namespace) = name.namespace() {
            // TODO union and derived types need something fancier
            if namespace != XS_NAMESPACE {
                return Ok(());
            }
        }

        if self.match_type_name(name.local_name()) {
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

    use xee_xpath_ast::ast::parse_sequence_type;
    use xee_xpath_ast::Namespaces;

    use crate::xml;

    fn is_owned(sequence: error::Result<Cow<sequence::Sequence>>) -> bool {
        if let Ok(sequence) = sequence {
            match sequence {
                Cow::Borrowed(_) => false,
                Cow::Owned(_) => true,
            }
        } else {
            false
        }
    }

    fn is_borrowed(sequence: error::Result<Cow<sequence::Sequence>>) -> bool {
        !is_owned(sequence)
    }

    #[test]
    fn test_one_integer() {
        let namespaces = Namespaces::default();
        let sequence_type = parse_sequence_type("xs:integer", &namespaces).unwrap();

        let right_sequence = sequence::Sequence::from(vec![sequence::Item::from(ibig!(1))]);
        let wrong_amount_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(ibig!(1)),
            sequence::Item::from(ibig!(2)),
        ]);
        let wrong_type_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(false))]);
        let xot = Xot::new();

        let right_result = right_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(Cow::Borrowed(&right_sequence)));
        assert!(is_owned(right_result));
        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let wrong_type_result = wrong_type_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_type_result, Err(error::Error::Type));
    }

    #[test]
    fn test_one_any_atomic() {
        let namespaces = Namespaces::default();
        let sequence_type = parse_sequence_type("xs:anyAtomicType", &namespaces).unwrap();

        let right_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(1i64))]);
        let wrong_amount_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(ibig!(1)),
            sequence::Item::from(ibig!(2)),
        ]);
        let right_type_sequence2 =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(false))]);
        let xot = Xot::new();

        let right_result = right_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(Cow::Borrowed(&right_sequence)));
        assert!(is_owned(right_result));
        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let right_type_result2 = right_type_sequence2.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_type_result2, Ok(Cow::Borrowed(&right_type_sequence2)));
        assert!(is_owned(right_type_result2));
    }

    #[test]
    fn test_one_item() {
        let namespaces = Namespaces::default();
        let sequence_type = parse_sequence_type("item()", &namespaces).unwrap();
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

        let right_result = right_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(Cow::Borrowed(&right_sequence)));
        assert!(is_borrowed(right_result));

        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let right_type_result2 = right_type_sequence2.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_type_result2, Ok(Cow::Borrowed(&right_type_sequence2)));
        assert!(is_borrowed(right_type_result2));
    }

    #[test]
    fn test_option_integer() {
        let namespaces = Namespaces::default();
        let sequence_type = parse_sequence_type("xs:integer?", &namespaces).unwrap();

        let right_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(ibig!(1)))]);
        let wrong_amount_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(ibig!(1)),
            sequence::Item::from(ibig!(2)),
        ]);
        let right_empty_sequence = sequence::Sequence::empty();
        let xot = Xot::new();

        let right_result = right_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(Cow::Borrowed(&right_sequence)));
        assert!(is_owned(right_result));
        let wrong_amount_result =
            wrong_amount_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(wrong_amount_result, Err(error::Error::Type));
        let right_empty_result = right_empty_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_empty_result, Ok(Cow::Borrowed(&right_empty_sequence)));
        assert!(is_owned(right_empty_result));
    }

    #[test]
    fn test_many_integer() {
        let namespaces = Namespaces::default();
        let sequence_type = parse_sequence_type("xs:integer*", &namespaces).unwrap();

        let right_sequence =
            sequence::Sequence::from(vec![sequence::Item::from(atomic::Atomic::from(ibig!(1)))]);
        let right_multi_sequence = sequence::Sequence::from(vec![
            sequence::Item::from(ibig!(1)),
            sequence::Item::from(ibig!(2)),
        ]);
        let right_empty_sequence = sequence::Sequence::empty();
        let xot = Xot::new();

        let right_result = right_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_result, Ok(Cow::Borrowed(&right_sequence)));
        assert!(is_owned(right_result));
        let right_multi_result = right_multi_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_multi_result, Ok(Cow::Borrowed(&right_multi_sequence)));
        assert!(is_owned(right_multi_result));
        let right_empty_result = right_empty_sequence.sequence_type_matching(&sequence_type, &xot);
        assert_eq!(right_empty_result, Ok(Cow::Borrowed(&right_empty_sequence)));
        assert!(is_owned(right_empty_result));
    }
}
