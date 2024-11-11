use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use rustyline::error::ReadlineError;

use crate::xpath::{execute_query, make_static_context_builder};

#[derive(Debug, Parser)]
pub(crate) struct Repl {
    /// Optional input file.
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

impl Repl {
    pub(crate) fn run(&self) -> anyhow::Result<()> {
        let mut documents = xee_xpath::Documents::new();
        let doc = if let Some(infile) = &self.infile {
            let mut reader = BufReader::new(File::open(infile)?);
            let mut input_xml = String::new();
            reader.read_to_string(&mut input_xml)?;

            Some(documents.add_string_without_uri(&input_xml)?)
        } else {
            None
        };

        let static_context_builder = make_static_context_builder(
            self.default_namespace_uri.as_deref(),
            self.namespace.as_slice(),
        )?;

        let queries = xee_xpath::Queries::new(static_context_builder);

        let mut rl = rustyline::DefaultEditor::new()?;
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    execute_query(&line, &queries, &mut documents, doc)?;
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        Ok(())
    }
}
