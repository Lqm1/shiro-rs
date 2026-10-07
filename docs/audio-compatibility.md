# WAV conversion compatibility

`shiro-wav2raw` implements the original input path and -e/-r/-d/-N/-h options.
Output rate uses Hz. A literal replacement extension includes its leading dot,
as in C; default output is .raw. Successful conversion writes a file and leaves
stdout empty. WAV decoding retains the left channel for stereo inputs.
Processing order is normalization, uniform additive dither, then resampling.
`audio::prepare` exposes the same processing with caller-owned uniform draws.

Normalization divides by the largest absolute sample. Silent and empty inputs
remain finite instead of dividing by zero. Paths without extensions retain
their full basename, correcting C's backward character scan for short names.
The command rejects an identical input/output path before writing. Input samples, parameters and dither arithmetic are validated before creating the output. The writer
propagates I/O failures and explicitly flushes its buffer.

Default resampling includes sample zero, correcting the original boundary
omission. `--legacy-resample` retains the original boundary values. FIR and
interpolation details are in ciglet-rs/docs/filter-resample-compatibility.md.
Rawfloat output is headerless little-endian binary32.

AudioOptions::kernel independently selects the Lanczos numerical policy.
The default Stable policy corrects the reproduced near-zero approximate-sine
division defect; the original kernel remains selectable as Legacy.
Both native extraction commands select Legacy with `--legacy-resample`, alongside
the original sample-zero omission. See ciglet's dsp-optimization-audit.md.
The related ciglet and SHIRO suites pass on all five targets in Debug and Release.

## Dither

The original command never calls srand, so its implicit seed-one sequence
depends on the C runtime. `audio::DitherSequence` independently implements the
Windows 15-bit recurrence and Linux GNU degree-31 additive-feedback generator
with fixed-size state and explicit unsigned wrapping. Both sequences are
tested against C executables using unmodified ciglet.h randu. The existing CLI
uses the Windows sequence on Windows and the Linux GNU sequence on Linux GNU.
Other Tier 1 runtimes have not yet been validated and must be audited before
claiming their default dither compatibility.

Uniform integer division retains f32 rounding. Signed conversion retains C's
double subtraction/multiplication before conversion back to f32. Addition and
dither scaling then use f32. Nonpositive finite levels disable dither, as in C.
Invalid uniform draws and output overflow return errors.

The additional `--seed` option explicitly selects rand 0.10.3 StdRng rather
than the original runtime sequence. It permits repeatability within the tested
dependency build. StdRng does not promise portable output or stability across
dependency versions. It does not change default command behavior. Rand default
features are disabled; only std_rng is enabled. No OS entropy backend is used.
MIT notices for rand, rand_core, chacha20, cpufeatures, cfg-if and libc are kept
under LICENSES. Libc is a transitive CPU-feature dependency on supported Unix
targets; no C SHIRO or ciglet implementation is linked.

Primary references:
https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/rand,
https://github.com/bminor/glibc/blob/master/stdlib/rand.c,
https://github.com/bminor/glibc/blob/master/stdlib/random_r.c,
https://docs.rs/rand/0.10.3/rand/rngs/struct.StdRng.html,
https://rust-random.github.io/book/crate-reprod.html and
https://doc.rust-lang.org/std/primitive.u32.html#method.wrapping_add.

## Original-C reference

Pinned SHIRO: 203ef7b71bf382c8b5ce3f86b8116f63265e2711.
Pinned ciglet: 895ba9b1c0eabee83d3544208bbc82420efa3206.
Stage shiro-wav2raw.c, changing only its include path to ciglet.h. Build with
gcc -O2 -DFP_TYPE=float -ffunction-sections -fdata-sections -I<upstream>
<staged-source> <ciglet>/ciglet.c <ciglet>/external/wavfile.c
-Wl,--gc-sections -lm. Retain the original unchecked-fread warnings.
No fast-math is used; original -Ofast comparisons remain pending.

`tests/fixtures/c-audio-input.wav` is a synthetic 257-frame, 16000-Hz mono PCM16
file. Sample i is (i modulo 17 minus 8) times 1024. Its canonical 44-byte RIFF
header uses format one, block alignment two, byte rate 32000 and data length
514. Invoke the staged C command on it with these extension/option pairs:

```text
-e .plain.raw
-e .normalized.raw -N
-e .up.raw -r 32000
-e .down.raw -r 8000
-e .normalized-down.raw -N -r 8000
-e .dither-linux.raw -d 0.125
```

The first five files are compared to Rust CLI output using explicit legacy
resampling where needed. The Linux dither file is compared byte for byte on
Linux. Initial Windows MSVC x86_64 differences are zero for all five ordinary
conversion files; the scoped normalized bound is 2e-7.

`tests/dither_oracle.c` is compiled separately with Linux gcc and Windows
x86_64-w64-mingw32-gcc, including pinned ciglet.h with FP_TYPE=float and -O2.
Run each executable on its actual OS with a destination fixture path. DTH1
stores uint32 count 64 and RAND_MAX, then 64 pairs of uint32 rand draw/f32
normalized draw, followed by 64 f32 values from unmodified randu after a fresh
srand(1). Both OS fixtures are checked on every Rust target. All normalized
draws and signed dither values match exactly on initial Windows x86_64.

Independent tests cover silent normalization, signed zero, controlled draws,
processing order, corrected/legacy first-sample behavior, errors before output,
paths containing spaces, a filename without an extension, malformed WAV,
seed repeatability and a synthetic WAV -> rawfloat -> MFCC command workflow.
The feature workflow produces eight frames of twelve finite coefficients.
This does not establish real-audio/model/training acceptance.

All 14 current integration tests pass on Windows MSVC x86_64/i686, Windows GNU x86_64 and Linux GNU x86_64/i686. Each of the five ordinary conversion outputs has zero normalized difference on every target. Both C-runtime dither corpora match exact binary32 bits on every target; Linux default CLI dither output matches C bytes on both Linux targets. Formatting and Clippy with warnings denied pass. Eleven other SHIRO tools and all bindings remain incomplete.


A separate original-C reproduction invoked the command on the bare relative filename a with extension .relative.raw. Original C wrote .relative.raw, losing the input basename. The Rust CLI regression invokes the same relative input and writes a.relative.raw. It verifies that the basename-free output is absent. The two CLI integration tests, including the added regression, pass after a targeted rerun on all five native targets.
