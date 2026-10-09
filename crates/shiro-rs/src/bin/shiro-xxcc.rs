//! Original feature-extraction command with seek-free binary stream input.
use clap::Parser;
use shiro_rs::{
    features::{Energy, FeatureKind, FeatureOptions, extract},
    rawfloat,
};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

fn feature_kind(value: &str) -> Result<FeatureKind, String> {
    match value {
        "mfcc" => Ok(FeatureKind::Mfcc),
        "mfbe" => Ok(FeatureKind::Mfbe),
        "plpcc" => Ok(FeatureKind::Plpcc),
        _ => Err("feature type must be mfcc, mfbe, or plpcc".into()),
    }
}
#[derive(Parser)]
#[command(
    name = "shiro-xxcc",
    about = "Extract MFCC, MFBE or PLPCC features from rawfloat audio"
)]
struct Arguments {
    #[arg(default_value = "-", value_name = "RAW_FILE")]
    input: PathBuf,
    #[arg(short='f',default_value="mfcc",value_parser=feature_kind)]
    kind: FeatureKind,
    #[arg(short = 'm', default_value_t = 12)]
    order: usize,
    #[arg(short = 'c', default_value_t = 36)]
    channels: usize,
    #[arg(short = 'l', default_value_t = 1024)]
    frame_length: usize,
    #[arg(short = 'p', default_value_t = 256.0)]
    hop: f32,
    /// Sample rate in kHz, as in the original command.
    #[arg(short = 's', default_value_t = 32.0)]
    sample_rate_khz: f32,
    #[arg(short = 'w', default_value_t = 400.0)]
    minimum_bandwidth_hz: f32,
    #[arg(short = 'W', default_value_t = 1.0)]
    warp: f32,
    #[arg(short = 'd')]
    delta: bool,
    #[arg(short = 'a')]
    acceleration: bool,
    #[arg(short = '0')]
    include_dc: bool,
    #[arg(short = 'e')]
    include_energy: bool,
    /// Zero selects RMS; any nonzero value selects decibels.
    #[arg(short = 'E', default_value_t = 0, allow_hyphen_values = true)]
    energy_type: i32,
}
fn run(arguments: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let options = FeatureOptions {
        kind: arguments.kind,
        order: arguments.order,
        channels: arguments.channels,
        frame_length: arguments.frame_length,
        hop: arguments.hop,
        sample_rate_hz: arguments.sample_rate_khz * 1000.0,
        minimum_bandwidth_hz: arguments.minimum_bandwidth_hz,
        warp: arguments.warp,
        include_dc: arguments.include_dc,
        energy: arguments
            .include_energy
            .then_some(if arguments.energy_type == 0 {
                Energy::Rms
            } else {
                Energy::Decibels
            }),
        delta: arguments.delta,
        acceleration: arguments.acceleration,
    };
    extract(&[], options)?;
    let signal = if arguments.input == Path::new("-") {
        rawfloat::read(BufReader::new(io::stdin().lock()), i32::MAX as usize)?
    } else {
        rawfloat::read(
            BufReader::new(File::open(arguments.input)?),
            i32::MAX as usize,
        )?
    };
    let features = extract(&signal, options)?;
    let mut output = BufWriter::new(io::stdout().lock());
    rawfloat::write(&mut output, &features.values)?;
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
