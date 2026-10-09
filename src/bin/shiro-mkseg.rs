//! Create initial segmentation from indexed rawfloat feature files.
use clap::Parser;
use shiro_rs::{
    index,
    labels::{PhoneMap, SegmentationDocument, SegmentedFile},
    segmentation,
};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};
#[derive(Parser)]
#[command(
    name = "shiro-mkseg",
    about = "Create equally spaced SHIRO segmentation"
)]
struct Arguments {
    index: PathBuf,
    #[arg(short = 'm')]
    phonemap: PathBuf,
    #[arg(short = 'd', default_value = ".")]
    directory: PathBuf,
    #[arg(short = 'e', default_value = ".f")]
    extension: String,
    #[arg(short = 'n', default_value_t = 36)]
    dimensions: usize,
    #[arg(short = 'L')]
    left: Option<String>,
    #[arg(short = 'R')]
    right: Option<String>,
    /// Accepted for compatibility; the original option has no effect.
    #[arg(short = 't')]
    _hop: Option<f64>,
}
fn padding(text: Option<String>) -> Vec<String> {
    text.filter(|text| !text.is_empty())
        .map_or_else(Vec::new, |text| {
            text.split(',').map(str::to_owned).collect()
        })
}
fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    segmentation::feature_frame_count(0, args.dimensions)?;
    let map: PhoneMap = serde_json::from_reader(BufReader::new(File::open(args.phonemap)?))?;
    let entries = index::read(
        BufReader::new(File::open(args.index)?),
        &args.directory,
        &padding(args.left),
        &padding(args.right),
    )?;
    let mut files = Vec::new();
    for entry in entries {
        let path = index::append_suffix(&entry.stem, &args.extension);
        let bytes = File::open(&path)?.metadata()?.len();
        let frames = segmentation::feature_frame_count(bytes, args.dimensions)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        files.push(SegmentedFile {
            filename: path
                .to_str()
                .ok_or("JSON feature paths require UTF-8")?
                .to_owned(),
            states: segmentation::initial(&entry.phonemes, &map, frames)?,
            attributes: Default::default(),
        });
    }
    let mut output = BufWriter::new(io::stdout().lock());
    serde_json::to_writer_pretty(
        &mut output,
        &SegmentationDocument {
            files,
            attributes: Default::default(),
        },
    )?;
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
