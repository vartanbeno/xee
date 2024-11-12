use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use ahash::HashMap;
use clap::Parser;
use rustyline::error::ReadlineError;
use xee_xpath::{error::Error, DocumentHandle, Documents, Itemable, Query};

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

struct RunContext {
    documents: Documents,
    document_handle: Option<DocumentHandle>,
    default_namespace_uri: Option<String>,
    namespaces: HashMap<String, String>,
}

impl RunContext {
    fn new() -> Self {
        Self {
            documents: Documents::new(),
            document_handle: None,
            default_namespace_uri: None,
            namespaces: HashMap::default(),
        }
    }

    fn set_default_namespace_uri(&mut self, default_namespace_uri: String) -> anyhow::Result<()> {
        self.default_namespace_uri = Some(default_namespace_uri);
        Ok(())
    }

    fn add_namespace_declaration(&mut self, prefix: String, uri: String) -> anyhow::Result<()> {
        self.namespaces.insert(prefix, uri);
        Ok(())
    }

    fn set_context_document(&mut self, path: &Path) -> anyhow::Result<()> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut input_xml = String::new();
        reader.read_to_string(&mut input_xml)?;

        self.document_handle = Some(self.documents.add_string_without_uri(&input_xml)?);
        Ok(())
    }

    fn queries(&self) -> xee_xpath::Queries {
        let mut static_context_builder = xee_xpath::context::StaticContextBuilder::default();
        if let Some(default_namespace_uri) = &self.default_namespace_uri {
            static_context_builder.default_element_namespace(default_namespace_uri);
        }
        for (prefix, uri) in &self.namespaces {
            static_context_builder.add_namespace(prefix, uri);
        }
        xee_xpath::Queries::new(static_context_builder)
    }

    pub(crate) fn execute(&mut self, xpath: &str) -> Result<(), anyhow::Error> {
        let queries = self.queries();
        let sequence_query = queries.sequence(xpath);
        let sequence_query = match sequence_query {
            Ok(sequence_query) => sequence_query,
            Err(e) => {
                render_error(xpath, e);
                return Ok(());
            }
        };
        let mut context_builder = sequence_query.dynamic_context_builder(&self.documents);
        if let Some(doc) = self.document_handle {
            context_builder.context_item(doc.to_item(&self.documents)?);
        }
        let context = context_builder.build();

        let sequence = sequence_query.execute_with_context(&mut self.documents, &context);
        let sequence = match sequence {
            Ok(sequence) => sequence,
            Err(e) => {
                render_error(xpath, e);
                return Ok(());
            }
        };
        println!(
            "{}",
            sequence.display_representation(self.documents.xot(), &context)
        );
        Ok(())
    }
}

impl Repl {
    pub(crate) fn run(self) -> anyhow::Result<()> {
        let mut run_context = RunContext::new();
        if let Some(infile) = &self.infile {
            run_context.set_context_document(infile)?;
        }
        if let Some(default_namespace_uri) = self.default_namespace_uri {
            run_context.set_default_namespace_uri(default_namespace_uri)?;
        }
        for namespace in self.namespace {
            let parts = namespace.split('=').collect::<Vec<_>>();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid namespace declaration: {}",
                    namespace
                ));
            }
            run_context.add_namespace_declaration(parts[0].to_string(), parts[1].to_string())?;
        }

        let command_definitions = CommandDefinitions::new(vec![
            CommandDefinition::new(
                "load",
                vec![ArgumentDefinition::default()],
                Box::new(|args, run_context| {
                    let path: PathBuf = args[0].into();
                    run_context.set_context_document(&path)?;
                    Ok(())
                }),
            ),
            CommandDefinition::new(
                "default_namespace",
                vec![ArgumentDefinition::default()],
                Box::new(|args, run_context| {
                    run_context.set_default_namespace_uri(args[0].to_string())?;
                    Ok(())
                }),
            ),
            CommandDefinition::new(
                "namespace",
                vec![ArgumentDefinition::default(), ArgumentDefinition::default()],
                Box::new(|args, run_context| {
                    run_context
                        .add_namespace_declaration(args[0].to_string(), args[1].to_string())?;
                    Ok(())
                }),
            ),
        ]);

        let mut rl = rustyline::DefaultEditor::new()?;
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    rl.add_history_entry(line)?;
                    if !line.starts_with("!") {
                        run_context.execute(line)?;
                    } else {
                        let command = line[1..].trim();
                        if command == "quit" {
                            break;
                        }
                        command_definitions.execute(command, &mut run_context)?;
                    }
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

type Execute = Box<dyn Fn(&[&str], &mut RunContext) -> anyhow::Result<()>>;

struct CommandDefinition {
    name: &'static str,
    args: Vec<ArgumentDefinition>,
    execute: Execute,
}

#[derive(Default)]
struct CommandDefinitions {
    definitions: Vec<CommandDefinition>,
    by_name: HashMap<&'static str, usize>,
}

#[derive(Default)]
struct ArgumentDefinition {
    default: Option<&'static str>,
}

impl CommandDefinitions {
    fn new(defitions: Vec<CommandDefinition>) -> Self {
        let mut definitions = Self {
            definitions: Vec::new(),
            by_name: HashMap::default(),
        };
        for definition in defitions {
            definitions.add(definition);
        }
        definitions
    }

    fn add(&mut self, definition: CommandDefinition) {
        let index = self.definitions.len();
        self.by_name.insert(definition.name, index);
        self.definitions.push(definition);
    }

    fn execute(&self, command: &str, run_context: &mut RunContext) -> anyhow::Result<()> {
        let parts = command.split_whitespace().collect::<Vec<_>>();
        let command_s = parts[0];
        let args = &parts[1..];
        let command = self.get(command_s);
        if let Some(command) = command {
            if args.len() > command.args.len() {
                println!("Too many arguments for command: {}", command_s);
                return Ok(());
            }
            let args = command.preprocess_arguments(args);
            if args.len() < command.args.len() {
                println!("Too few arguments for command: {}", command_s);
                return Ok(());
            }
            (command.execute)(&args, run_context)
        } else {
            println!("Unknown command: {}", command_s);
            Ok(())
        }
    }

    fn get(&self, command: &str) -> Option<&CommandDefinition> {
        self.by_name.get(command).map(|&i| &self.definitions[i])
    }
}

impl CommandDefinition {
    fn new(name: &'static str, args: Vec<ArgumentDefinition>, execute: Execute) -> Self {
        Self {
            name,
            args,
            execute,
        }
    }

    fn preprocess_arguments<'a>(&self, args: &[&'a str]) -> Vec<&'a str> {
        let mut result = Vec::new();
        let mut i = 0;
        for arg in &self.args {
            if i < args.len() {
                result.push(args[i]);
                i += 1;
            } else if let Some(default) = arg.default {
                result.push(default);
            }
        }
        result
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
