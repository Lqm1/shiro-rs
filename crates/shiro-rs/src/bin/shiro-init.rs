//! Initialize a SHIRO model from rawfloat features and segmentation JSON.
mod common;
use clap::Parser;
use shiro_rs::{
    dataset,
    initialization::{self, Options},
    labels::SegmentationDocument,
};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};
#[derive(Parser)]
#[command(
    name = "shiro-init",
    about = "Initialize a model from aligned or flat-start observations"
)]
struct Arguments {
    #[arg(short = 'm')]
    model: PathBuf,
    #[arg(short = 's')]
    segmentation: PathBuf,
    #[arg(short = 'v', default_value_t = 0.1)]
    variance_floor: f32,
    #[arg(short = 'F')]
    flat: bool,
    #[arg(short = 'T')]
    tied: bool,
}
fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let model = common::read_model(&args.model)?;
    let document: SegmentationDocument =
        serde_json::from_reader(BufReader::new(File::open(args.segmentation)?))?;
    let data = dataset::load(&document, &model, i32::MAX as usize)?;
    let initialized = initialization::initialize(
        &model,
        &data,
        Options {
            flat_start: args.flat,
            globally_tied: args.tied,
            variance_floor_ratio: args.variance_floor,
        },
    )?;
    let mut output = BufWriter::new(io::stdout().lock());
    initialized.write_to(&mut output)?;
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
