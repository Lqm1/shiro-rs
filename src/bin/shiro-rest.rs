//! Re-estimate a model from embedded or isolated rawfloat observations.
mod common;
use clap::Parser;
use shiro_rs::{
    dataset,
    labels::SegmentationDocument,
    training::{self, DurationMode, Options},
};
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "shiro-rest",
    about = "Re-estimate HMM or HSMM parameters from a corpus"
)]
struct Arguments {
    #[arg(short = 'm')]
    model: PathBuf,
    #[arg(short = 's')]
    segmentation: PathBuf,
    #[arg(short = 'n', default_value_t = 1)]
    iterations: usize,
    #[arg(short = 'g')]
    geometric: bool,
    #[arg(short = 'p', default_value_t = 5.0)]
    state_radius: f32,
    #[arg(short = 'P', default_value_t = 0.3)]
    pruning_slope: f32,
    #[arg(short = 'd', default_value_t = 30)]
    duration_extra: usize,
    #[arg(short = 't', default_value_t = 1.0, allow_negative_numbers = true)]
    termination_threshold: f32,
    #[arg(short = 'l')]
    likelihood: Option<PathBuf>,
    #[arg(short = 'i')]
    isolated: bool,
    #[arg(short = 'D')]
    annealing: bool,
    #[arg(short = 'T')]
    parallel: bool,
    #[arg(short = 'M')]
    mean_frame_likelihood: bool,
}

fn output_path(path: &Path) -> io::Result<PathBuf> {
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
        .ok_or_else(|| io::Error::other("likelihood path has no parent"))?;
    let filename = absolute
        .file_name()
        .ok_or_else(|| io::Error::other("likelihood path has no filename"))?;
    Ok(parent.canonicalize()?.join(filename))
}

fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let model = common::read_model(&args.model)?;
    let document: SegmentationDocument =
        serde_json::from_reader(BufReader::new(File::open(&args.segmentation)?))?;
    if let Some(path) = &args.likelihood {
        let output = output_path(path)?;
        let mut inputs = vec![args.segmentation.canonicalize()?];
        if args.model != Path::new("-") {
            inputs.push(args.model.canonicalize()?);
        }
        for file in &document.files {
            let path = Path::new(&file.filename);
            if args.iterations != 0 || path.exists() {
                inputs.push(path.canonicalize()?);
            }
        }
        if inputs.contains(&output) {
            return Err(io::Error::other(
                "likelihood output must differ from model, segmentation and features",
            )
            .into());
        }
    }
    let mut options = Options {
        iterations: args.iterations,
        duration_mode: if args.geometric {
            DurationMode::Geometric
        } else {
            DurationMode::Normal
        },
        termination_threshold: args.termination_threshold,
        deterministic_annealing: args.annealing,
        mean_frame_likelihood: args.mean_frame_likelihood,
        ..Options::default()
    };
    options.hsmm.state_radius = args.state_radius;
    options.hsmm.duration_extra = args.duration_extra;
    options.geometric.pruning_slope = args.pruning_slope;
    if args.parallel && args.likelihood.is_some() {
        eprintln!("Warning: multi-threading disabled due to -l option.");
    } else if args.parallel {
        options.workers = std::thread::available_parallelism()?.get();
    }
    let files = if args.iterations == 0 {
        Vec::new()
    } else {
        dataset::load_training_files(&document, &model, i32::MAX as usize, args.isolated)?
    };
    let result = training::train_with_progress(&model, &files, options, |report| {
        if args.annealing {
            eprintln!(
                "Running iteration {}/{}, temperature = {:.2}...",
                report.iteration, args.iterations, report.temperature
            );
        } else {
            eprintln!(
                "Running iteration {}/{}...",
                report.iteration, args.iterations
            );
        }
        eprintln!(
            "Average log likelihood = {:.6}.",
            report.mean_log_likelihood
        );
    })?;
    if result.iterations.len() < args.iterations {
        eprintln!("Training converged.");
    }
    let mut model_bytes = Vec::new();
    result.model.write_to(&mut model_bytes)?;
    if let Some(path) = args.likelihood {
        let mut csv = Vec::new();
        for report in &result.iterations {
            for row in &report.file_likelihoods {
                for (index, value) in row.iter().enumerate() {
                    if index > 0 {
                        write!(csv, ",")?;
                    }
                    write!(csv, "{value:.6}")?;
                }
                writeln!(csv)?;
            }
        }
        fs::write(path, csv)?;
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
