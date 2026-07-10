//! Command-line interface for PreFigure, shadowing prefig/cli.py.
//!
//! The subcommands match the Python CLI. Most are placeholders until the
//! drawing pipeline is ported; `eval` works today and is useful for trying
//! out expressions.

use clap::{Parser, Subcommand};
use prefig_core::evaluator::ExpressionContext;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "prefig", version, about = "PreFigure: an authoring system for mathematical diagrams")]
struct Cli {
    /// -v for information and -vv for debugging
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build a PreFigure diagram from source
    Build {
        /// Output format
        #[arg(short, long, default_value = "svg")]
        format: String,
        filename: String,
    },
    /// Convert the PreFigure SVG into a PDF
    Pdf { filename: String },
    /// Convert the PreFigure SVG into a PNG
    Png { filename: String },
    /// Set up a new PreFigure project
    New,
    /// Initialize the local installation of PreFigure
    Init,
    /// Install PreFigure examples in the current directory
    Examples,
    /// Check that a source file is valid
    Validate { filename: String },
    /// Evaluate a PreFigure expression and print the result
    Eval { expression: String },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Eval { expression } => {
            let mut ctx = ExpressionContext::new();
            match ctx.valid_eval(&expression) {
                Ok(value) => {
                    println!("{value:?}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Command::Build { .. }
        | Command::Pdf { .. }
        | Command::Png { .. }
        | Command::New
        | Command::Init
        | Command::Examples
        | Command::Validate { .. } => {
            eprintln!(
                "not implemented yet: the drawing pipeline is still being ported \
                 (see rust/PORTING.md; use the Python `prefig` command meanwhile)"
            );
            ExitCode::FAILURE
        }
    }
}
