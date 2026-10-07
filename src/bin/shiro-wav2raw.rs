//! Original WAV-to-raw command with checked output and caller-seeded dither.
use ciglet_rs::{resampling::BoundaryPolicy, wave};
use clap::Parser;
use rand::{RngExt, SeedableRng, rngs::StdRng};
use shiro_rs::{
    audio::{self, AudioOptions, DitherSequence},
    rawfloat,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Parser)]
#[command(name = "shiro-wav2raw", about = "Convert WAV audio to a rawfloat file")]
struct Arguments {
    #[arg(value_name = "WAV_FILE")]
    input: PathBuf,
    /// Literal replacement extension, including any leading period.
    #[arg(short = 'e', default_value = ".raw")]
    extension: String,
    /// Output sample rate in Hz. Omit to retain the input rate.
    #[arg(short = 'r')]
    sample_rate: Option<u32>,
    #[arg(short = 'd', default_value_t = 0.0, allow_hyphen_values = true)]
    dither_level: f32,
    #[arg(short = 'N')]
    normalize: bool,
    /// Opt into seeded Rust dither. Omit to retain the original C-runtime sequence.
    #[arg(long)]
    seed: Option<u64>,
    /// Retain the original resampler's omission of source sample zero.
    #[arg(long)]
    legacy_resample: bool,
}
fn run(arguments: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let stem = arguments
        .input
        .file_stem()
        .ok_or("input path has no file name")?;
    let mut output_name = stem.to_os_string();
    output_name.push(&arguments.extension);
    let output_path = arguments.input.with_file_name(output_name);
    if output_path == arguments.input {
        return Err("output path must differ from the input WAV path".into());
    }
    let wave = wave::read_file(&arguments.input, i32::MAX as usize)?;
    let mut rng = arguments.seed.map(StdRng::seed_from_u64);
    let mut legacy = if cfg!(windows) {
        DitherSequence::windows()
    } else {
        DitherSequence::linux_gnu()
    };
    let output = audio::prepare(
        wave,
        AudioOptions {
            normalize: arguments.normalize,
            dither_level: arguments.dither_level,
            output_sample_rate: arguments.sample_rate,
            boundary: if arguments.legacy_resample {
                BoundaryPolicy::LegacySkipFirst
            } else {
                BoundaryPolicy::IncludeFirst
            },
        },
        || match &mut rng {
            Some(rng) => rng.random::<f32>(),
            None => legacy.next_uniform(),
        },
    )?;
    let mut writer = BufWriter::new(File::create(output_path)?);
    rawfloat::write(&mut writer, &output.samples)?;
    writer.flush()?;
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
