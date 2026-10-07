//! Expand tied model distributions into independent corpus states.
mod common;
use clap::Parser;
use shiro_rs::{labels::SegmentationDocument, untying};
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "shiro-untie",
    about = "Create independent model states for each corpus occurrence"
)]
struct Arguments {
    #[arg(short = 'm')]
    model: PathBuf,
    #[arg(short = 's')]
    segmentation: PathBuf,
    #[arg(short = 'o')]
    output_segmentation: Option<PathBuf>,
    #[arg(short = 'O')]
    output_summary: Option<PathBuf>,
}

fn absolute_output(path: &Path) -> io::Result<PathBuf> {
    if path.exists() {
        return path.canonicalize();
    }
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let parent = absolute
        .parent()
        .ok_or_else(|| io::Error::other("output path has no parent"))?;
    let name = absolute
        .file_name()
        .ok_or_else(|| io::Error::other("output path has no filename"))?;
    Ok(parent.canonicalize()?.join(name))
}

fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let model = common::read_model(&args.model)?;
    let document: SegmentationDocument =
        serde_json::from_reader(BufReader::new(File::open(&args.segmentation)?))?;
    let untied = untying::untie(&model, &document)?;
    let inputs = [
        Some(args.segmentation.canonicalize()?),
        if args.model == Path::new("-") {
            None
        } else {
            Some(args.model.canonicalize()?)
        },
    ];
    let mut outputs = Vec::new();
    for path in [&args.output_segmentation, &args.output_summary]
        .into_iter()
        .flatten()
    {
        let resolved = absolute_output(path)?;
        if inputs.iter().flatten().any(|input| input == &resolved) || outputs.contains(&resolved) {
            return Err(
                io::Error::other("output files must differ from inputs and each other").into(),
            );
        }
        outputs.push(resolved);
    }
    // Prepare all formats before creating either optional output file.
    let segmentation = if args.output_segmentation.is_some() {
        let mut bytes = serde_json::to_vec_pretty(&untied.segmentation)?;
        bytes.push(b'\n');
        Some(bytes)
    } else {
        None
    };
    let summary = if args.output_summary.is_some() {
        let mut bytes = Vec::new();
        untied.write_summary(&mut bytes)?;
        Some(bytes)
    } else {
        None
    };
    let mut model_bytes = Vec::new();
    untied.model.write_to(&mut model_bytes)?;
    if let (Some(path), Some(bytes)) = (args.output_segmentation, segmentation) {
        fs::write(path, bytes)?;
    }
    if let (Some(path), Some(bytes)) = (args.output_summary, summary) {
        fs::write(path, bytes)?;
    }
    let mut output = BufWriter::new(io::stdout().lock());
    output.write_all(&model_bytes)?;
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
