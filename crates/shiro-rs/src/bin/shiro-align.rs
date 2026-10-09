//! Align phoneme states against rawfloat observations.
mod common;
use clap::Parser;
use shiro_rs::{
    alignment::{self, DurationMode, Options},
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
    name = "shiro-align",
    about = "Align phoneme states with HMM or HSMM inference"
)]
struct Arguments {
    #[arg(short = 'm')]
    model: PathBuf,
    #[arg(short = 's')]
    segmentation: PathBuf,
    #[arg(short = 'g')]
    geometric: bool,
    #[arg(short = 'p', default_value_t = 5.0)]
    state_radius: f32,
    #[arg(short = 'P', default_value_t = 0.3)]
    pruning_slope: f32,
    #[arg(short = 'd', default_value_t = 30)]
    duration_extra: usize,
    #[arg(short = 'i')]
    isolated: bool,
}
fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let model = common::read_model(&args.model)?;
    let document: SegmentationDocument =
        serde_json::from_reader(BufReader::new(File::open(args.segmentation)?))?;
    let mut options = Options {
        isolated: args.isolated,
        duration_mode: if args.geometric {
            DurationMode::Geometric
        } else {
            DurationMode::Explicit
        },
        ..Options::default()
    };
    options.hsmm.state_radius = args.state_radius;
    options.hsmm.duration_extra = args.duration_extra;
    options.geometric.pruning_slope = args.pruning_slope;
    let aligned = alignment::align_document(&model, &document, options)?;
    let mut output = BufWriter::new(io::stdout().lock());
    serde_json::to_writer_pretty(&mut output, &aligned)?;
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
