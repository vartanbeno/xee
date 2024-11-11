mod indent;

use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use xee_xpath_compiler::{atomic::Atomic, error::SpannedError, evaluate_root, sequence::Item};
use xot::output::xml::Parameters;
use xot::Xot;

use crate::indent::indent;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Format an XML document with indentation to make it more readable.
    Indent { xml: PathBuf },
    /// Evaluate an xpath expression on an xml document.
    Xpath {
        xml: PathBuf,
        xpath: String,
        /// The default namespace for elements
        #[arg(long, short)]
        namespace_default: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Indent { xml } => {
            indent(&xml, &mut std::io::stdout())?;
        }
        Commands::Xpath {
            xml,
            xpath,
            namespace_default,
        } => {
            let mut xot = Xot::new();
            let xml_file = File::open(xml).unwrap();
            let mut buf_reader = BufReader::new(xml_file);
            let mut xml = String::new();
            buf_reader.read_to_string(&mut xml).unwrap();
            let root = xot.parse(&xml).unwrap();
            let result = evaluate_root(
                &mut xot,
                root,
                &xpath,
                &namespace_default.unwrap_or(String::new()),
            );
            match result {
                Ok(sequence) => {
                    for item in sequence.items()? {
                        display_item(&xot, &item).unwrap();
                    }
                }
                Err(e) => render_error(&xpath, e),
            }
        }
    }
    Ok(())
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

fn render_error(src: &str, e: SpannedError) {
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
