//! Original model-creation command, with binary stdout and typed errors.
use clap::Parser;
use shiro_rs::definition::ModelDefinition;
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "shiro-mkhsmm",
    about = "Create a model from a SHIRO JSON model definition"
)]
struct Arguments {
    #[arg(short = 'c', value_name = "modeldef-file")]
    definition: PathBuf,
}

fn run(arguments: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let definition: ModelDefinition =
        serde_json::from_reader(BufReader::new(File::open(arguments.definition)?))?;
    let model = definition.build()?;
    let mut output = BufWriter::new(io::stdout().lock());
    model.write_to(&mut output)?;
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
