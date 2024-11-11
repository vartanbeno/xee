mod format;
mod indent;
mod xpath;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Format an XML document with various options.
    Format(format::Format),
    /// Format an XML document with indentation to make it more readable.
    ///
    /// This is a shortcut for `format --indent`.
    Indent(indent::Indent),
    /// Evaluate an xpath expression on an xml document.
    Xpath(xpath::XPath),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Indent(indent) => {
            indent.run()?;
        }
        Commands::Format(format) => {
            format.run()?;
        }
        Commands::Xpath(xpath) => {
            xpath.run()?;
        }
    }
    Ok(())
}
