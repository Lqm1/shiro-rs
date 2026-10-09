//! Expand a phone set into a SHIRO phone map.
use clap::Parser;
use shiro_rs::phonemap::{self, Options};
use std::{
    fs,
    io::{self, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};
#[derive(Parser)]
#[command(
    name = "shiro-mkpm",
    about = "Create a SHIRO phone map from a phone set"
)]
struct Arguments {
    phoneset: PathBuf,
    #[arg(short = 's', default_value_t = 3)]
    states: usize,
    #[arg(short = 'S', default_value_t = 3)]
    streams: usize,
    #[arg(short = 't')]
    topology: Option<String>,
    #[arg(short = 'w')]
    weak_skips: bool,
}
fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let map = phonemap::create(
        &fs::read_to_string(args.phoneset)?,
        &Options {
            states_per_phone: args.states,
            streams: args.streams,
            topology: args.topology,
            weak_skips: args.weak_skips,
        },
    )?;
    let mut output = BufWriter::new(io::stdout().lock());
    serde_json::to_writer_pretty(&mut output, &map)?;
    writeln!(output)?;
    output.flush()?;
    Ok(())
}
fn main() -> ExitCode {
    match run(Arguments::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
