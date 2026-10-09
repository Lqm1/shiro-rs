//! Native file extraction, original presets and optional host extractors.
use crate::{
    audio::{self, AudioOptions},
    features::{self, Energy, FeatureKind, FeatureOptions},
    index, rawfloat,
};
use ciglet_rs::wave;
use std::{
    fmt,
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Signal(ciglet_rs::Error),
    Spawn {
        program: PathBuf,
        source: io::Error,
    },
    Process {
        program: PathBuf,
        status: ExitStatus,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::Signal(e) => e.fmt(f),
            Self::Spawn { program, source } => {
                write!(f, "cannot launch {}: {source}", program.display())
            }
            Self::Process { program, status } => {
                write!(f, "{} failed with {status}", program.display())
            }
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Signal(e) => Some(e),
            Self::Spawn { source, .. } => Some(source),
            Self::Process { .. } => None,
        }
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<ciglet_rs::Error> for Error {
    fn from(e: ciglet_rs::Error) -> Self {
        Self::Signal(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preset {
    #[default]
    Mfcc12Da16k,
    Mfcc12Dae16k,
    Plpcc12Da16k,
}
impl Preset {
    pub fn feature_options(self) -> FeatureOptions {
        FeatureOptions {
            kind: if self == Self::Plpcc12Da16k {
                FeatureKind::Plpcc
            } else {
                FeatureKind::Mfcc
            },
            frame_length: 512,
            hop: 80.0,
            order: 12,
            sample_rate_hz: 16000.0,
            delta: true,
            acceleration: true,
            energy: if self == Self::Mfcc12Dae16k {
                Some(Energy::Rms)
            } else {
                None
            },
            ..Default::default()
        }
    }
}
#[derive(Debug, Clone)]
pub struct SptkPrograms {
    pub frame: PathBuf,
    pub mfcc: PathBuf,
    pub delta: PathBuf,
}
impl Default for SptkPrograms {
    fn default() -> Self {
        Self {
            frame: "frame".into(),
            mfcc: "mfcc".into(),
            delta: "delta".into(),
        }
    }
}
#[derive(Debug, Clone)]
pub enum Extractor {
    Native(Preset),
    Sptk(SptkPrograms),
    /// The script returns function(try_execute, stem, rawfile, executable_prefix).
    Lua {
        interpreter: PathBuf,
        script: PathBuf,
        executable_directory: PathBuf,
    },
}
#[derive(Debug, Clone)]
pub struct Options {
    pub audio: AudioOptions,
    pub input_extension: String,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            audio: AudioOptions::default(),
            input_extension: ".wav".into(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Outputs {
    pub raw: PathBuf,
    pub parameters: PathBuf,
    pub mfcc: Option<PathBuf>,
}

/// Write .raw and .param, retaining .mfcc for the SPTK preset. Each caller
/// supplies its random source; the CLI resets the original sequence per file.
pub fn extract_file(
    stem: &Path,
    options: &Options,
    extractor: &Extractor,
    uniform: impl FnMut() -> f32,
) -> Result<Outputs, Error> {
    let input = index::append_suffix(stem, &options.input_extension);
    let raw = index::append_suffix(stem, ".raw");
    let parameters = index::append_suffix(stem, ".param");
    if input == raw
        || input == parameters
        || matches!(extractor, Extractor::Sptk(_)) && input == index::append_suffix(stem, ".mfcc")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "audio input must differ from extraction outputs",
        )
        .into());
    }
    let wave = wave::read_file(input, i32::MAX as usize)?;
    let audio = audio::prepare(wave, options.audio, uniform)?;
    let native = match extractor {
        Extractor::Native(preset) => {
            Some(features::extract(&audio.samples, preset.feature_options())?)
        }
        _ => None,
    };
    write(&raw, &audio.samples)?;
    let mfcc = match extractor {
        Extractor::Native(_) => {
            write(
                &parameters,
                &native.expect("native features computed").values,
            )?;
            None
        }
        Extractor::Sptk(programs) => {
            let mfcc = index::append_suffix(stem, ".mfcc");
            sptk(programs, &raw, &mfcc, &parameters)?;
            Some(mfcc)
        }
        Extractor::Lua {
            interpreter,
            script,
            executable_directory,
        } => {
            lua(interpreter, script, stem, &raw, executable_directory)?;
            None
        }
    };
    Ok(Outputs {
        raw,
        parameters,
        mfcc,
    })
}
fn write(path: &Path, values: &[f32]) -> io::Result<()> {
    let mut output = BufWriter::new(File::create(path)?);
    rawfloat::write(&mut output, values)?;
    output.flush()
}

/// Reap a child on every exit path, including failures while spawning a pipeline.
struct Running {
    child: Child,
    finished: bool,
}
impl Drop for Running {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
impl Running {
    fn start(command: &mut Command) -> Result<Self, Error> {
        command
            .spawn()
            .map(|child| Self {
                child,
                finished: false,
            })
            .map_err(|source| Error::Spawn {
                program: PathBuf::from(command.get_program()),
                source,
            })
    }
    fn wait(&mut self, program: &Path) -> Result<(), Error> {
        let status = self.child.wait()?;
        self.finished = true;
        if status.success() {
            Ok(())
        } else {
            Err(Error::Process {
                program: program.to_owned(),
                status,
            })
        }
    }
}
fn sptk(programs: &SptkPrograms, raw: &Path, mfcc: &Path, parameters: &Path) -> Result<(), Error> {
    let mut frame = Running::start(
        Command::new(&programs.frame)
            .args(["-l", "512", "-p", "80"])
            .arg(raw)
            .stdout(Stdio::piped()),
    )?;
    let stream = frame.child.stdout.take().expect("frame stdout piped");
    let mut cepstra = Running::start(
        Command::new(&programs.mfcc)
            .args(["-l", "512", "-m", "12", "-s", "16"])
            .stdin(stream)
            .stdout(File::create(mfcc)?),
    )?;
    // The consumer must finish before waiting on the producer's successful path.
    cepstra.wait(&programs.mfcc)?;
    frame.wait(&programs.frame)?;
    let mut delta = Running::start(
        Command::new(&programs.delta)
            .args([
                "-l", "12", "-d", "-0.5", "0", "0.5", "-d", "0.25", "0", "-0.5", "0", "0.25",
            ])
            .arg(mfcc)
            .stdout(File::create(parameters)?),
    )?;
    delta.wait(&programs.delta)
}
const LUA_BRIDGE: &str = r#"
local script, stem, rawfile, prefix = arg[1], arg[2], arg[3], arg[4]
local directory = script:match("^(.*[/\\])") or "./"
local searchers = package.searchers or package.loaders
table.insert(searchers, 3, function(name)
    local relative = name:gsub("%.", "/")
    local messages = {}
    for _, suffix in ipairs({".lua", "/init.lua"}) do
        local filename = directory .. relative .. suffix
        local loader, message = loadfile(filename)
        if loader then return loader, filename end
        messages[#messages + 1] = "\n\t" .. tostring(message)
    end
    return table.concat(messages)
end)
local function try_execute(command)
    local ok, kind, code = os.execute(command)
    if ok ~= true and ok ~= 0 then
        error("extractor command failed: " .. command .. " (" .. tostring(kind) .. ", " .. tostring(code) .. ")")
    end
end
_G.try_execute = try_execute
local extractor = assert(loadfile(script))()
assert(type(extractor) == "function", "extractor script must return a function")
extractor(try_execute, stem, rawfile, prefix)
"#;
fn lua(
    interpreter: &Path,
    script: &Path,
    stem: &Path,
    raw: &Path,
    executable_directory: &Path,
) -> Result<(), Error> {
    let mut prefix = executable_directory.as_os_str().to_os_string();
    prefix.push(std::path::MAIN_SEPARATOR.to_string());
    let mut command = Command::new(interpreter);
    command
        .arg("-")
        .arg(script)
        .arg(stem)
        .arg(raw)
        .arg(prefix)
        .stdin(Stdio::piped());
    let mut child = Running::start(&mut command)?;
    {
        let mut stdin = child.child.stdin.take().expect("Lua stdin piped");
        stdin.write_all(LUA_BRIDGE.as_bytes())?;
    }
    child.wait(interpreter)
}
