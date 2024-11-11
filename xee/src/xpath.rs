use clap::Parser;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use xee_xpath::error::Error;
use xee_xpath::Query;
use xee_xpath::{Atomic, Item};
use xot::output::xml::Parameters;
use xot::Xot;

#[derive(Debug, Parser)]
pub(crate) struct XPath {
    /// xpath expression
    pub(crate) xpath: String,
    /// input xml file (default stdin)
    pub(crate) infile: Option<PathBuf>,
    /// Namespace URI to use in XPath for element names without a namespace
    /// prefix.
    ///
    /// If omitted, the default namespace is the empty string (i.e. the
    /// names are not in a namespace).
    #[arg(long)]
    pub(crate) default_namespace_uri: Option<String>,
    /// Namespace declaration to make available in XPath (can be repeated)
    /// The format is prefix=uri.
    #[arg(long)]
    pub(crate) namespace: Vec<String>,
}

impl XPath {
    pub(crate) fn run(&self) -> Result<(), anyhow::Error> {
        let mut reader: Box<dyn BufRead> = if let Some(infile) = &self.infile {
            Box::new(BufReader::new(File::open(infile)?))
        } else {
            Box::new(BufReader::new(std::io::stdin()))
        };

        let mut input_xml = String::new();
        reader.read_to_string(&mut input_xml)?;

        let mut documents = xee_xpath::Documents::new();
        let doc = documents.add_string_without_uri(&input_xml)?;

        let mut static_context_builder = xee_xpath::context::StaticContextBuilder::default();
        if let Some(default_namespace_uri) = &self.default_namespace_uri {
            static_context_builder.default_element_namespace(default_namespace_uri);
        }
        let namespaces = self
            .namespace
            .iter()
            .map(|declaration| {
                let mut parts = declaration.splitn(2, '=');
                let prefix = parts.next().ok_or(anyhow::anyhow!("missing prefix"))?;
                let uri = parts.next().ok_or(anyhow::anyhow!("missing uri"))?;
                Ok((prefix, uri))
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?;

        static_context_builder.namespaces(namespaces);

        let queries = xee_xpath::Queries::new(static_context_builder);
        let sequence_query = queries.sequence(&self.xpath);
        let sequence_query = match sequence_query {
            Ok(sequence_query) => sequence_query,
            Err(e) => {
                render_error(&self.xpath, e);
                return Ok(());
            }
        };
        let sequence = sequence_query.execute(&mut documents, doc);
        let sequence = match sequence {
            Ok(sequence) => sequence,
            Err(e) => {
                render_error(&self.xpath, e);
                return Ok(());
            }
        };
        for item in sequence.items()? {
            display_item(documents.xot(), &item).unwrap();
        }
        Ok(())
    }
}

fn display_item(xot: &Xot, item: &Item) -> Result<(), xot::Error> {
    match item {
        Item::Node(node) => {
            println!("node: \n{}", display_node(xot, *node)?);
        }
        Item::Atomic(value) => println!("atomic: {}", display_atomic(value)),
        Item::Function(function) => println!("function: {:?}", function),
    }
    Ok(())
}

fn display_atomic(atomic: &Atomic) -> String {
    format!("{}", atomic)
}

fn display_node(xot: &Xot, node: xot::Node) -> Result<String, xot::Error> {
    match xot.value(node) {
        xot::Value::Attribute(attribute) => {
            let value = attribute.value();
            let (name, namespace) = xot.name_ns_str(attribute.name());
            let name = if !namespace.is_empty() {
                format!("Q{{{}}}{}", namespace, name)
            } else {
                name.to_string()
            };
            Ok(format!("Attribute {}=\"{}\"", name, value))
        }
        xot::Value::Namespace(..) => {
            todo!()
        }
        _ => xot.serialize_xml_string(
            {
                Parameters {
                    indentation: Default::default(),
                    ..Default::default()
                }
            },
            node,
        ),
    }
}

fn render_error(src: &str, e: Error) {
    let red = ariadne::Color::Red;

    let mut report =
        ariadne::Report::build(ariadne::ReportKind::Error, "source", 0).with_code(e.error.code());

    if let Some(span) = e.span {
        report = report.with_label(
            ariadne::Label::new(("source", span.range()))
                .with_message(e.error.message())
                .with_color(red),
        )
    }
    report
        .finish()
        .print(("source", ariadne::Source::from(src)))
        .unwrap();
    println!("{}", e.error.note());
}
