use clap::{command, Parser};
mod yodaodict;
use std::env;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(long, default_value_t = false)]
    debug: bool,

    pub word: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.word.is_none() {
        println!("Usage: dict <单词>");
        return Ok(());
    }

    yodaodict::search_word(args);
    Ok(())
}
