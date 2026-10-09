//! Convert SHIRO segmentation JSON to phoneme/state labels.
use clap::Parser;
use shiro_rs::labels::{self, SegmentationDocument};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};
#[derive(Parser)]
#[command(
    name = "shiro-seg2lab",
    about = "Convert SHIRO segmentation JSON to timed labels"
)]
struct Arguments {
    segmentation: PathBuf,
    #[arg(short = 't', default_value_t = 0.01)]
    hop: f64,
    #[arg(short = 'e', default_value = ".txt")]
    extension: String,
    #[arg(short = 's')]
    states: bool,
}
fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let document: SegmentationDocument =
        serde_json::from_reader(BufReader::new(File::open(&args.segmentation)?))?;
    let mut outputs = Vec::new();
    let segmentation_path = args.segmentation.canonicalize()?;
    for file in document.files {
        let path = labels::output_path(&file.filename, &args.extension);
        if path == args.segmentation
            || path
                .canonicalize()
                .is_ok_and(|output| output == segmentation_path)
        {
            return Err("label output must differ from segmentation input".into());
        }
        let rows = labels::from_states(&file.states, args.hop, args.states)?;
        let mut bytes = Vec::new();
        labels::write(&rows, &mut bytes)?;
        outputs.push((path, bytes));
    }
    for (path, bytes) in outputs {
        let mut output = BufWriter::new(File::create(path)?);
        output.write_all(&bytes)?;
        output.flush()?;
    }
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
