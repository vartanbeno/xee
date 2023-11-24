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
