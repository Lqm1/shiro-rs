# SHIRO tool and IO binding audit

This audit records source inspection at the native-preset batch checkpoint
7f54551, plus the likelihood CSV correction. It does not establish final native
platform execution or combined acceptance. The original fourteen tools and full
dependency functionality remain required. No tool inventory status is promoted
solely from this source inspection.

## Computational paths

All fourteen src/bin/shiro-* implementations were inspected against the WASM
modules, including their option conversion and output preparation. The browser
contract uses in-memory bytes and copied owners under ADR 0007. File opening,
stdout/stderr, process invocation, filesystem alias/canonicalization and CLI
argument parsing retain native behavior. They are not browser operations.

| Tool | Computation and configuration | WASM path and existing verifier |
| --- | --- | --- |
| shiro-align | Explicit/geometric inference, isolated mode, radius, pruning and duration search | Model.align_states/align_document, AlignmentOptions and FeatureFiles; wasm_alignment_checks |
| shiro-fextr | Index iteration, literal suffixes, native presets and all audio settings | IndexEntries, batch_feature_options and BatchExtraction; wasm_index_checks/wasm_batch_checks; external Lua/SPTK remain native |
| shiro-init | Paired observation loading, flat/global initialization and variance ratio | Dataset.load and Model.initialize, InitializationOptions; wasm_loading_checks/wasm_initialization_checks |
| shiro-lab2seg | Timed label parsing, hop and state expansion, input/output suffixes | Labels.parse/to_states, IndexEntries/index_append_suffix and full document replacement; wasm_interchange_checks/wasm_index_checks |
| shiro-mkhsmm | Model definition and complete binary model output | ModelDefinition.build and Model.write; wasm_models_checks |
| shiro-mkpm | State/stream counts, optional topology and weak skips | PhoneMap.create and PhoneMapOptions; wasm_phones_checks |
| shiro-mkseg | Indexed/padded phone expansion, feature-size frame counting and suffix | IndexEntries, feature_frame_count, PhoneMap.initial and full document replacement; wasm_phones_checks includes indexed two-file generation and label output; full fourteen-tool workflow remains pending |
| shiro-pm2md | Dimensions, hop and tied duration constraints | PhoneMap.to_definition and ModelDefinition; wasm_phones_checks |
| shiro-rest | Iterations, both duration modes, radius/pruning/search, convergence, annealing, isolated groups, ordered workers and likelihood rows | Datasets.load_training_files, Model.train/train_with_progress, TrainingOptions and TrainingResult.write_likelihood_csv; wasm_training_checks |
| shiro-seg2lab | Hop, state/phone labels and output suffix | Labels.from_states/write and label_output_path; wasm_interchange_checks |
| shiro-untie | Full untied model, segmentation, assignments and summary | Model.untie, UntiedModel and write_summary; wasm_untying_checks |
| shiro-wav2raw | Bounded WAV decode, all audio settings, platform legacy draws and rawfloat bytes | Wave.read, Audio.prepare/prepare_with_sequence and rawfloat_write; wasm_audio_checks |
| shiro-wavsplit | All five timing/training options, dimensions, all feature kinds and fresh/initialized/trained model paths; intermediate artifacts | SegmentedWave/SegmentedUtterances, UtteranceOptions and UtteranceModelSource; wasm_utterances_checks |
| shiro-xxcc | All twelve feature settings, rawfloat decode and feature bytes | FeatureOptions, Features.extract and rawfloat_read/write; wasm_features_checks |

The audit found that full report fields did not by themselves provide the CLI's
likelihood CSV formatting. The native writer is now shared by the CLI, WASM and
C byte codec. Its six-decimal binary32 output is checked independently, including
signed zero, nonfinite values, empty file rows, partial/interrupted IO and errors.
The byte codec and direct C stream callback now invoke the same native writer.
Windows x86_64 C/Python callers check all nine training cases, partial writes,
interruption, late failure, invalid callback counts, missing writers and empty
reports without implicit flush. Other targets still require current-source runs.

## Generic IO and host behavior

Native rawfloat readers/writers, observation readers, label writers, index
buffered readers and untying summary writers have direct C callback adapters in
src/c_api/streams.rs. Their partial reads/writes, interruption, consumed input,
explicit flush and non-retained context contracts remain native capabilities.
WASM bytes replace the host transport under the agreed contract; they do not
provide OS streams or imply that arbitrary host callbacks run in the browser.
The likelihood CSV writer also has a direct C stream adapter in that module.

All JSON document owners retain native fields, unknown attributes and optional
metadata through complete JSON replacement. Computational owners retain their
full configuration/result fields through copied getters and complete setters.
This inspection does not replace a full public-field audit or runtime tests.

## Remaining acceptance work

The C TrainingResult owner now supports arbitrary construction and full model
and report replacement. Owned iteration reports also support complete replacement
using the existing arbitrary report constructor's row contract. Windows x86_64
Rust/C/Python tests cover complete fields, arbitrary IEEE values, nested empty and
repeated inputs, invalid pointer/count failures, atomic updates and independent
source/result/clone lifetimes. This closes the identified result-field gap on
that target; it does not complete the remaining public-field or platform audit.
The native mkseg CLI and C/WASM bindings share feature_frame_count, including
positive i32 dimensions, complete-frame byte lengths and target usize limits.
The WASM verifier composes padded index reading, literal suffixes, two virtual
feature files, original Lua state comparison, complete segmentation documents
and distinct label byte outputs. This is one part of the combined tool audit.
Run a combined virtual-file workflow exercising all fourteen tool computations. Audit
the remaining public-field coverage in all three packages, especially ciglet.
Refresh native execution for Windows MSVC x86_64/i686, Windows GNU x86_64 and
Linux GNU x86_64/i686 against the final commits. Preserve the existing original
C/model/real-audio/learning acceptance gates. Finish combined three-package
C/Python and browser/Node checks, and distinguish actual SPTK execution from
protocol fixtures on each target. No Release, static-library or Python32
acceptance is inferred from debug-library checks.
