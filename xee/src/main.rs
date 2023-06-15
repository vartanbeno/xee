use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use xee_xpath::{evaluate_root, Node};
use xee_xpath::{Atomic, Item, ItemValue};
use xot::Xot;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate an xpath expression on an xml document.
    Xpath {
        xml: PathBuf,
        xpath: String,
        /// The default namespace for elements
        #[arg(long, short)]
        namespace_default: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Xpath {
            xml,
            xpath,
            namespace_default,
        } => {
            let mut xot = Xot::new();
            let xml_file = File::open(xml)
                .into_diagnostic()
                .wrap_err("Cannot open XML file")?;
            let mut buf_reader = BufReader::new(xml_file);
            let mut xml = String::new();
            buf_reader
                .read_to_string(&mut xml)
                .into_diagnostic()
                .wrap_err("Cannot read XML file")?;
            let root = xot
                .parse(&xml)
                .into_diagnostic()
                .wrap_err("Cannot parse XML")?;
            let result = evaluate_root(&xot, root, &xpath, namespace_default.as_deref())?;
            for item in result.iter() {
                display_item(&xot, &item)
                    .into_diagnostic()
                    .wrap_err("Could not display item")?;
            }
        }
    }
    Ok(())
}

fn display_item(xot: &Xot, item: &Item) -> Result<(), xot::Error> {
    match item.value() {
        ItemValue::Node(node) => {
            println!("node: \n{}", display_node(xot, node)?);
        }
        ItemValue::Atomic(value) => println!("atomic: {}", display_atomic(&value)),
        ItemValue::Function(function) => println!("{:?}", function),
    }
    Ok(())
}

fn display_atomic(atomic: &Atomic) -> String {
    format!("{}", atomic)
}

fn display_node(xot: &Xot, node: Node) -> Result<String, xot::Error> {
    match node {
        Node::Xot(node) => xot
            .with_serialize_options(xot::SerializeOptions { pretty: true })
            .to_string(node),
        Node::Attribute(node, name) => {
            let value = xot.element(node).unwrap().get_attribute(name).unwrap();
            let (name, namespace) = xot.name_ns_str(name);
            let name = if !namespace.is_empty() {
                format!("Q{{{}}}{}", namespace, name)
            } else {
                name.to_string()
            };
            Ok(format!("Attribute {}=\"{}\"", name, value))
        }
        Node::Namespace(..) => {
            todo!()
        }
    }
}
