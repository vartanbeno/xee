use ahash::{HashMap, HashMapExt};
use xee_xpath_ast::{Namespaces, FN_NAMESPACE};
use xot::{Node, SpanInfo, SpanInfoKey, Xot};

use crate::{ast_core::Span, names::Names};

pub(crate) struct State {
    pub(crate) xot: Xot,
    pub(crate) span_info: SpanInfo,
    pub(crate) names: Names,
}

impl State {
    pub(crate) fn new(xot: Xot, span_info: SpanInfo, names: Names) -> Self {
        Self {
            xot,
            span_info,
            names,
        }
    }

    pub(crate) fn next(&self, node: Node) -> Option<Node> {
        self.xot.next_sibling(node)
    }

    pub(crate) fn span(&self, node: Node) -> Option<Span> {
        use xot::Value::*;

        match self.xot.value(node) {
            Element(_element) => self.span_info.get(SpanInfoKey::ElementStart(node)),
            Text(_text) => self.span_info.get(SpanInfoKey::Text(node)),
            Comment(_comment) => self.span_info.get(SpanInfoKey::Comment(node)),
            ProcessingInstruction(_pi) => self.span_info.get(SpanInfoKey::PiTarget(node)),
            Root => unreachable!(),
        }
        .map(|span| span.into())
    }
}

struct StackedContext<'a> {
    element: &'a xot::Element,

    next: Option<&'a StackedContext<'a>>,
}

impl<'a> StackedContext<'a> {
    pub(crate) fn new(element: &'a xot::Element) -> Self {
        Self {
            element,
            next: None,
        }
    }

    pub(crate) fn push(&'a self, element: &'a xot::Element) -> Self {
        Self {
            element,
            next: Some(self),
        }
    }

    fn prefixes(&self) -> xot::Prefixes {
        if let Some(next) = &self.next {
            let mut combined_prefixes = xot::Prefixes::new();
            let prefixes = next.prefixes();
            for (prefix, uri) in prefixes.iter() {
                combined_prefixes.insert(*prefix, *uri);
            }
            combined_prefixes
        } else {
            self.element.prefixes().clone()
        }
    }

    pub(crate) fn namespaces(&self, context: &'a State) -> Namespaces {
        let mut namespaces = HashMap::new();
        for (prefix, ns) in self.prefixes() {
            let prefix = context.xot.prefix_str(prefix);
            let uri = context.xot.namespace_str(ns);
            namespaces.insert(prefix, uri);
        }
        Namespaces::new(namespaces, None, Some(FN_NAMESPACE))
    }
}
