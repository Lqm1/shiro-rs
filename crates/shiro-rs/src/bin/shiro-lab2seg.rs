//! Convert indexed timed labels to SHIRO segmentation JSON.
use clap::Parser;
use shiro_rs::{
    index,
    labels::{self, PhoneMap, SegmentationDocument, SegmentedFile},
};
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};
#[derive(Parser)]
#[command(
    name = "shiro-lab2seg",
    about = "Convert timed labels to SHIRO segmentation JSON"
)]
struct Arguments {
    index: PathBuf,
    #[arg(short = 'm')]
    phonemap: PathBuf,
    #[arg(short = 'd', default_value = ".")]
    directory: PathBuf,
    #[arg(short = 't', default_value_t = 0.01)]
    hop: f64,
    #[arg(short = 'e', default_value = ".txt")]
    extension: String,
    #[arg(short = 'E', default_value = ".f")]
    feature_extension: String,
}
fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let map: PhoneMap = serde_json::from_reader(BufReader::new(File::open(args.phonemap)?))?;
    let entries = index::read(
        BufReader::new(File::open(args.index)?),
        &args.directory,
        &[],
        &[],
    )?;
    let mut files = Vec::new();
    for entry in entries {
        let path = index::append_suffix(&entry.stem, &args.extension);
        let labels = labels::parse(&fs::read_to_string(path)?)?;
        let feature = index::append_suffix(&entry.stem, &args.feature_extension);
        files.push(SegmentedFile {
            filename: feature
                .to_str()
                .ok_or("JSON feature paths require UTF-8")?
                .to_owned(),
            states: labels::to_states(&labels, &map, args.hop)?,
            attributes: Default::default(),
        });
    }
    let document = SegmentationDocument {
        files,
        attributes: Default::default(),
    };
    let mut output = BufWriter::new(io::stdout().lock());
    serde_json::to_writer_pretty(&mut output, &document)?;
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
