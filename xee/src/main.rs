use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use xee_xpath::evaluate;
// use xot::Xot;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate an xpath expression on an xml document.
    Xpath { xml: PathBuf, xpath: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            Commands::Xpath { xml, xpath } => {
                let xml_file = File::open(xml)
                    .into_diagnostic()
                    .wrap_err("Cannot open XML file")?;
                let mut buf_reader = BufReader::new(xml_file);
                let mut xml = String::new();
                buf_reader
                    .read_to_string(&mut xml)
                    .into_diagnostic()
                    .wrap_err("Cannot read XML file")?;
                let result = evaluate(&xml, &xpath, None)?;
                dbg!(result);
            }
        }
    }
    Ok(())
}
