//! Convert a phone map to a SHIRO model definition.
use clap::Parser;
use shiro_rs::{labels::PhoneMap, phonemap};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};
#[derive(Parser)]
#[command(
    name = "shiro-pm2md",
    about = "Convert a SHIRO phone map to a model definition"
)]
struct Arguments {
    phonemap: PathBuf,
    #[arg(short = 'd', default_value_t = 12)]
    dimensions: usize,
    #[arg(short = 't', default_value_t = 0.01)]
    hop: f64,
}
fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let map: PhoneMap = serde_json::from_reader(BufReader::new(File::open(args.phonemap)?))?;
    let definition = phonemap::to_definition(&map, args.dimensions, args.hop)?;
    let mut output = BufWriter::new(io::stdout().lock());
    serde_json::to_writer_pretty(&mut output, &definition)?;
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
