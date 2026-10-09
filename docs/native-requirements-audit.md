# Native requirements audit

This is the native-phase acceptance checkpoint for the source snapshots below.
Its local dependency descriptions record that historical execution. The current
workspace and public dependency layout are documented in
[workspace layout](workspace-layout.md) and
[workspace split verification](workspace-split-verification.md).

This audit follows the accepted plan in
`design/shiro-port-design/acceptance-plan.md`. The full scope remains all public
functionality of three independent packages. Passing counts alone do not prove
the requirements below. C ABI and browser/Node WebAssembly are a later phase.

The native requirements are satisfied for the agreed targets and numerical
corpora at the final snapshot documented below. The decision uses the following
evidence, with detailed scope and limits in the following sections.

| Accepted native item | Decision | Authoritative evidence |
| --- | --- | --- |
| 1. Libraries, executables and checks | Pass | Final five-target full matrix, Clippy/Rustdoc/fmt checks, fourteen Cargo executable targets |
| 2. Public functions and Lua tool functionality | Pass | Three symbol/tool inventories, 88-function bundled inventory, configuration/header/ownership mappings and per-family tests |
| 3. Existing pretrained model | Pass | Historical model byte roundtrip, original reader/parameter comparisons, real_audio.rs |
| 4. Rust-to-C and C-to-Rust formats | Pass | Model/data/set codec oracles, original reader checks, definition/phone/label/rawfloat comparisons |
| 5. Real speech to features/inference/labels | Pass | Licensed three-recording corpus, original C/Lua feature/state documents, real_audio.rs |
| 6. Small initialization/training/save/reload/inference | Pass | Four C initialization modes, nine corrected-C training cases, real/synthetic composed workflow and precision-training codecs |
| 7. Alternate modes and serial/parallel behavior | Pass | Isolated/embedded HMM/HSMM, ties/untying, skips, pruning, DAEM and ordered reduction regressions |
| 8. Optional external Lua | Pass | Required host extractor execution in the native matrix; bundled native presets need no Lua |
| 9. Malformed input and disclosed bug corrections | Pass | Codec/parameter/shape/resource/I/O regressions and independent corrected numerical references |
| 10. Report actual target execution separately | Pass | Five executable target results; deferred ARM64 and external SPTK installation limits explicitly reported |

## Build, execution and independent package structure

All three repositories have their own Cargo.toml, lockfile and Git history.
SHIRO's companion dependencies are local Cargo path dependencies. The completed
bundled-function revision matrix in `native-acceptance.md` runs all package
tests and all SHIRO executable targets on the five required x86 targets.
Original C implementations are reference executables, not Rust library delegates.
The later conditioning correction has its focused matrix and the final combined
execution result recorded below, satisfying this gate for the final native snapshot.
Warning-denied Clippy/Rustdoc and formatting checks accompany each changed package.

## Public functionality and original Lua tools

Ciglet's own-header inventory has 246 rows, of which two are macro formal
parameters rather than public symbols. All other rows have implemented mappings.
Its separate installed fastapprox inventory has 88 implemented mappings. Macro,
structure, precision and ownership equivalents are described in its
`public-configuration-audit.md`, `bundled-approximation-compatibility.md`,
`filterbank-configuration-audit.md`, and algorithm compatibility documents.
Portable four-lane arrays preserve source-specific arithmetic where scalar
substitution is not equivalent. Compiler packaging and include guards do not
add runtime operations.

Liblrhsmm's 103 symbol rows have implemented mappings. Its `header-coverage.md`,
`ownership-coverage.md` and `configuration-coverage.md` additionally describe
public fields, macros, mutable numerical tables, prepared caches, external
posteriors, pools, binary32/binary64 storage and optional serialization.
Safe reconstruction and borrowing replace inconsistent count/pointer mutation.
These representation changes do not establish C layout compatibility.

SHIRO's fourteen original C/Lua tools have Rust implementations in
`tool-inventory.csv`. Tests cover model definition, phone-map creation and
conversion, initial segmentation, label conversion, feature batch extraction,
waveform conversion/splitting, initialization, alignment, re-estimation and
untying. Bundled workflows need no Lua interpreter. The optional caller-supplied
Lua extractor remains supported separately.

## Legacy model and interchange formats

`crates/shiro-rs/tests/real_audio.rs` reads and byte-reencodes the historical pretrained model.
Liblrhsmm codec fixtures verify all eight original object/set schemas at both
storage precisions, historical/current GMM layouts, malformed array arities,
binary32 wire values, resource limits and original-reader interoperability.
The original buggy double jump reader needs its disclosed float-temporary
correction for meaningful double reference comparisons.

SHIRO's `crates/shiro-rs/tests/definition.rs`, `phones.rs`, `labels.rs`, `initialization.rs`,
`untying.rs` and `rawfloat.rs` compare model construction and JSON/rawfloat
interchange to original C/Lua artifacts. Original-reader comparisons check
decoded content rather than only process exit codes. Phone-map topology and
label/segmentation boundaries are discrete comparisons. Format evidence is
separate from numerical algorithm tolerance.

## Real speech and existing-model inference

`crates/shiro-rs/tests/real_audio.rs` covers the first three licensed CMU US SLT ARCTIC WAV files.
`real-audio-compatibility.md` records original URLs, hashes and retained notices.
Wave conversion, three 36-column MFCC/delta/acceleration feature files and both
complete HMM/HSMM alignment documents are independently generated by C/Lua.
Feature comparisons retain the measured corpus-specific 5e-5 normalized bound.
Complete state documents compare directly, followed by Rust label output.
This verifies compatibility on these recordings, not recognition accuracy or
a universal error bound.

## Initialization, learning and model reuse

`crates/shiro-rs/tests/initialization.rs` compares four initialization modes and flat-start
cases to original C model bytes. `crates/shiro-rs/tests/training.rs` and `rest_cli.rs` compare
nine corrected-C learning/CLI cases, including one/two HSMM updates, HMM,
DAEM, isolated variants, reporting, convergence and zero iterations.
`training-compatibility.md` records source patches, likelihood bounds and exact
model-byte comparisons. `crates/shiro-rs/tests/utterances.rs` independently checks the composed
waveform/initialization/training/alignment workflow against corrected C/Lua
intermediate artifacts.

The real-speech training regression separately executes fresh initialization,
two HMM and two HSMM DAEM iterations, saving, reloading and subsequent inference.
It checks finite reports, actual parameter changes and unchanged inference
across model reload. It does not compare real-speech trained parameters to C.
The required numerical training comparison is supplied by the reproducible
small synthetic C corpus, not inferred from the real-data regression.

## Alternate inference and learning modes

Tests in the dependency and SHIRO packages cover weak/skip transitions,
fractional pruning, HMM and HSMM forward/backward/Viterbi, isolated and embedded
training, globally tied initialization, untying, DAEM and selected precision.
SHIRO parallel statistics use per-file accumulators reduced in file order.
Repeated parallel results are bit-identical on the tested corpus. Comparison
to sequential reduction retains the documented 2e-6 parameter bound. The source
likelihood race is corrected and worker failures propagate after joining.

## Optional external extraction

`crates/shiro-rs/tests/host_extractors.rs` and `fextr_cli.rs` run representative external Lua
extractors and subprocess byte/array exchange in the native matrix. Native
MFCC/PLP presets run independently of external tools. Actual SPTK 3.9-3
comparison is verified on Linux x86_64; other targets use explicit subprocess
fixtures and do not thereby verify an actual SPTK installation. Caller-installed
external tools and arbitrary user scripts are not bundled Rust dependencies.
This limitation remains visible in `tool-inventory.csv`.

## Invalid input and disclosed corrections

Malformed/truncated models, JSON, rawfloat streams, invalid dimensions,
nonfinite parameters, negative bounds, allocation limits and writer/process
failures have explicit regression cases. Compatibility documents identify
source corrections instead of treating changed buggy output as rounding.
The subsequent near-zero Lanczos correction in ciglet's
`dsp-optimization-audit.md` has an independent sinc-limit regression and keeps
Legacy kernel selection. Its focused five-target Debug/Release validation passes.

## Numerical and platform limits

Both precisions are compared separately with original source arithmetic;
posterior probabilities and model wire scalars intentionally remain binary32.
Effective fast-math comparisons record compiler flags/macros and measured
differences. Liblrhsmm's `optimized-c-numerics.md` covers helpers, inference
decisions and expectation statistics. Ciglet's DSP report additionally exposes
the severe resampling conditioning defect. SHIRO's feature/audio optimized-C
workflow diagnostics are now recorded in frontend-optimization-audit.md. The
six silent-delta category changes are reproduced by effective fast-math and
restored by disabling finite-math-only. Rust keeps strict source categories.
These findings do not imply universal
equivalence to every compiler's `-Ofast` output is claimed.

Five-target execution evidence is specific to Windows MSVC x86_64/i686,
Windows GNU x86_64 and Linux GNU x86_64/i686. Tier 1 ARM64 execution is deferred.
The refreshed combined default-feature Debug matrix completes at ciglet 70ac0fc,
liblrhsmm f9bf303 and SHIRO 842e479. All five targets pass 198/120/58 tests
respectively, including required host fixtures, with zero failed or ignored
tests. All ignored tests are explicitly included. This snapshot includes both
the conditioning correction and ciglet's generic, fallibly allocated f32/f64
frame extraction. The two liblrhsmm compile-fail ownership doctests pass with
their expected E0502 diagnostics. The earlier focused five-target Debug/Release
matrix passes for the conditioning correction; it does not prove Release
execution of the later frame change or subsequently added bindings.

The native gate is satisfied for the agreed five-target execution scope and
recorded numerical corpora. This is an intermediate milestone. Deferred Tier 1
ARM64 execution, caller-installed SPTK on additional targets, universal algorithm
accuracy and compiler-dependent nonfinite fast-math outputs are not inferred
from it. The complete public-functionality scope remains in place. All three
packages' C ABI and browser/Node binding acceptance remains required.
