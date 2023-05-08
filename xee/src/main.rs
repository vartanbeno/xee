use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use xee_xpath::Atomic;
use xee_xpath::Item;
use xee_xpath::{evaluate_root, Node, Sequence, StackValue};
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
            match result {
                StackValue::Atomic(value) => println!("atomic: {}", display_atomic(&value)),
                StackValue::Sequence(sequence) => {
                    println!(
                        "sequence: \n{}",
                        display_sequence(&xot, &sequence.borrow())
                            .into_diagnostic()
                            .wrap_err("Could not display sequence")?
                    )
                }
                StackValue::Closure(closure) => println!("{:?}", closure),
                StackValue::Step(step) => println!("{:?}", step),
                StackValue::Node(node) => println!(
                    "node: \n{}",
                    display_node(&xot, node)
                        .into_diagnostic()
                        .wrap_err("Could not display node")?
                ),
            }
        }
    }
    Ok(())
}

fn display_atomic(atomic: &Atomic) -> String {
    format!("{}", atomic)
}

fn display_sequence(xot: &Xot, sequence: &Sequence) -> Result<String, xot::Error> {
    let mut v = Vec::new();
    for item in sequence.as_slice() {
        match item {
            Item::Node(node) => {
                v.push(format!("node: \n{}", display_node(xot, *node)?));
            }
            Item::Atomic(value) => v.push(format!("atomic: {}", display_atomic(value))),
            Item::Function(function) => v.push(format!("{:?}", function)),
        }
    }
    Ok(v.join("\n"))
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
