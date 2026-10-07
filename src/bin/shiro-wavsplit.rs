//! Utterance segmentation with native feature extraction and model estimation.
mod common;
use clap::Parser;
use shiro_rs::{
    audio::DitherSequence,
    features::FeatureKind,
    labels, rawfloat,
    utterances::{self, ModelSource, Options},
};
use std::{fs, path::PathBuf, process::ExitCode};

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
    name = "shiro-wavsplit",
    about = "Train and align alternating silence and utterance states"
)]
struct Arguments {
    input: PathBuf,
    #[arg(short = 'n')]
    utterances: usize,
    #[arg(short = 't', default_value_t = 0.1)]
    hop_seconds: f64,
    #[arg(short = 'd', default_value_t = 13)]
    dimensions: usize,
    #[arg(short = 'f', default_value = "mfcc", value_parser = feature_kind)]
    kind: FeatureKind,
    #[arg(short = 'N', default_value_t = 15)]
    iterations: usize,
    #[arg(short = 's', default_value_t = 0.3, allow_hyphen_values = true)]
    minimum_silence_seconds: f64,
    #[arg(short = 'v', default_value_t = 0.3, allow_hyphen_values = true)]
    minimum_voicing_seconds: f64,
    /// Load a trained model and skip estimation. Takes precedence over -i.
    #[arg(short = 'l')]
    trained: Option<PathBuf>,
    /// Load an initialized model and perform estimation.
    #[arg(short = 'i')]
    initialized: Option<PathBuf>,
}

fn run(args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let input = args.input.canonicalize()?;
    let input_text = args
        .input
        .to_str()
        .ok_or("input path must be UTF-8 for segmentation JSON")?;
    let param = labels::output_path(input_text, ".param");
    let filename = param.to_str().ok_or("feature path must be UTF-8")?;
    let loaded_path = args.trained.as_ref().or(args.initialized.as_ref());
    let loaded = loaded_path
        .map(|path| common::read_model(path))
        .transpose()?;
    let source = match &loaded {
        Some(model) if args.trained.is_some() => ModelSource::Trained(model),
        Some(model) => ModelSource::Initialized(model),
        None => ModelSource::Fresh,
    };
    let mut random = if cfg!(windows) {
        DitherSequence::windows()
    } else {
        DitherSequence::linux_gnu()
    };
    let result = utterances::split_wave(
        ciglet_rs::wave::read_file(&args.input, i32::MAX as usize)?,
        filename,
        args.dimensions,
        args.kind,
        Options {
            utterances: args.utterances,
            hop_seconds: args.hop_seconds,
            minimum_silence_seconds: args.minimum_silence_seconds,
            minimum_voicing_seconds: args.minimum_voicing_seconds,
            iterations: args.iterations,
        },
        source,
        || random.next_uniform(),
    )?;
    let stages = result.utterances;
    let mut outputs = Vec::new();
    let mut raw = Vec::new();
    rawfloat::write(&mut raw, &result.audio.samples)?;
    outputs.push((labels::output_path(input_text, ".raw"), raw));
    let mut features = Vec::new();
    rawfloat::write(&mut features, &result.features.values)?;
    outputs.push((param, features));
    let index_path = labels::output_path(input_text, ".index");
    let stem_path = labels::output_path(input_text, "");
    let stem = stem_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("input has no stem")?;
    outputs.push((
        index_path,
        format!("{stem},{}", stages.phones.join(" ")).into_bytes(),
    ));
    outputs.push((
        labels::output_path(input_text, ".phonemap"),
        serde_json::to_vec_pretty(&stages.phonemap)?,
    ));
    outputs.push((
        labels::output_path(input_text, ".modeldef"),
        serde_json::to_vec_pretty(&stages.definition)?,
    ));
    outputs.push((
        labels::output_path(input_text, ".init.segm"),
        serde_json::to_vec_pretty(&stages.initial_segmentation)?,
    ));
    outputs.push((
        labels::output_path(input_text, ".aligned.segm"),
        serde_json::to_vec_pretty(&stages.alignment)?,
    ));
    for (suffix, model) in [
        (".uninit.hsmm", stages.uninitialized_model.as_ref()),
        (".flat.hsmm", stages.initialized_model.as_ref()),
        (
            ".trained.hsmm",
            args.trained.is_none().then_some(&stages.model),
        ),
    ] {
        if let Some(model) = model {
            let mut bytes = Vec::new();
            model.write_to(&mut bytes)?;
            outputs.push((labels::output_path(input_text, suffix), bytes));
        }
    }
    let mut labels = Vec::new();
    labels::write(&stages.labels, &mut labels)?;
    outputs.push((labels::output_path(input_text, ".txt"), labels));
    let loaded_path = loaded_path.map(|path| path.canonicalize()).transpose()?;
    // Check all destinations before opening any output. An input with an
    // intermediate suffix, or an aliased loaded model, must not be overwritten.
    for (path, _) in &outputs {
        if path
            .canonicalize()
            .is_ok_and(|path| path == input || loaded_path.as_ref() == Some(&path))
        {
            return Err("output path aliases the input WAV or loaded model".into());
        }
    }
    for (path, bytes) in outputs {
        fs::write(path, bytes)?;
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
