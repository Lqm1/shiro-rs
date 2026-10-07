//! Original batch feature extraction with native presets and host adapters.
use ciglet_rs::resampling::BoundaryPolicy;
use clap::Parser;
use rand::{RngExt, SeedableRng, rngs::StdRng};
use shiro_rs::{
    audio::{AudioOptions, DitherSequence},
    batch::{self, Extractor, Options, Preset, SptkPrograms},
    index,
};
use std::{fs::File, io::BufReader, path::PathBuf, process::ExitCode};
#[derive(Parser)]
#[command(
    name = "shiro-fextr",
    about = "Extract indexed WAV files into rawfloat feature files"
)]
struct Arguments {
    index: PathBuf,
    #[arg(short = 'd', default_value = ".")]
    directory: PathBuf,
    #[arg(short = 'e', default_value = ".wav")]
    extension: String,
    #[arg(short = 'x')]
    extractor: Option<PathBuf>,
    #[arg(short = 'r', default_value_t = 0)]
    sample_rate: u32,
    #[arg(short = 'n')]
    normalize: bool,
    #[arg(short = 'D', default_value_t = 0.0, allow_hyphen_values = true)]
    dither_level: f32,
    /// External Lua interpreter, used only for a custom script.
    #[arg(long, default_value = "lua")]
    lua: PathBuf,
    #[arg(long)]
    sptk_directory: Option<PathBuf>,
    #[arg(long)]
    legacy_resample: bool,
    #[arg(long)]
    seed: Option<u64>,
}
fn select(arguments: &Arguments) -> Result<Extractor, Box<dyn std::error::Error>> {
    let Some(selector) = &arguments.extractor else {
        return Ok(Extractor::Native(Preset::default()));
    };
    let script = if selector.extension().is_some_and(|ext| ext == "lua") {
        selector.clone()
    } else {
        index::append_suffix(selector, ".lua")
    };
    let native = if !script.exists() {
        match selector.to_str() {
            Some("extractor-xxcc-mfcc12-da-16k" | "extractors/extractor-xxcc-mfcc12-da-16k") => {
                Some(Preset::Mfcc12Da16k)
            }
            Some("extractor-xxcc-mfcc12-dae-16k" | "extractors/extractor-xxcc-mfcc12-dae-16k") => {
                Some(Preset::Mfcc12Dae16k)
            }
            Some("extractor-xxcc-plpcc12-da-16k" | "extractors/extractor-xxcc-plpcc12-da-16k") => {
                Some(Preset::Plpcc12Da16k)
            }
            _ => None,
        }
    } else {
        None
    };
    if let Some(preset) = native {
        return Ok(Extractor::Native(preset));
    }
    if !script.exists()
        && (selector.to_str() == Some("extractor-sptk-mfcc12-da-16k")
            || selector.to_str() == Some("extractors/extractor-sptk-mfcc12-da-16k"))
    {
        let programs = if let Some(directory) = &arguments.sptk_directory {
            SptkPrograms {
                frame: index::append_suffix(&directory.join("frame"), std::env::consts::EXE_SUFFIX),
                mfcc: index::append_suffix(&directory.join("mfcc"), std::env::consts::EXE_SUFFIX),
                delta: index::append_suffix(&directory.join("delta"), std::env::consts::EXE_SUFFIX),
            }
        } else {
            SptkPrograms::default()
        };
        return Ok(Extractor::Sptk(programs));
    }
    let executable_directory = std::env::current_exe()?
        .parent()
        .ok_or("executable has no parent directory")?
        .to_owned();
    Ok(Extractor::Lua {
        interpreter: arguments.lua.clone(),
        script,
        executable_directory,
    })
}
fn run(arguments: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let extractor = select(&arguments)?;
    let entries = index::read(
        BufReader::new(File::open(&arguments.index)?),
        &arguments.directory,
        &[],
        &[],
    )?;
    let options = Options {
        audio: AudioOptions {
            normalize: arguments.normalize,
            dither_level: arguments.dither_level,
            output_sample_rate: (arguments.sample_rate != 0).then_some(arguments.sample_rate),
            boundary: if arguments.legacy_resample {
                BoundaryPolicy::LegacySkipFirst
            } else {
                BoundaryPolicy::IncludeFirst
            },
        },
        input_extension: arguments.extension,
    };
    for entry in entries {
        println!(
            "Processing {}",
            index::append_suffix(&entry.stem, &options.input_extension).display()
        );
        let mut legacy = if cfg!(windows) {
            DitherSequence::windows()
        } else {
            DitherSequence::linux_gnu()
        };
        let mut rng = arguments.seed.map(StdRng::seed_from_u64);
        batch::extract_file(&entry.stem, &options, &extractor, || match &mut rng {
            Some(rng) => rng.random::<f32>(),
            None => legacy.next_uniform(),
        })?;
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
