# Browser and Node.js WebAssembly API

The optional `wasm` feature exposes native computations through independent
owners and copied byte/typed arrays. Full SHIRO bindings and final combined
acceptance are still in progress. This checkpoint implements rawfloat, all
native label and phone-map operations, model construction/IO/parameter access,
observation and segmentation import with complete owned field access,
isolated grouping,
dataset owners, document loading, model initialization, training, alignment, untying,
feature extraction, audio conversion, utterance segmentation, index parsing
and native-preset batch extraction,
and complete JSON document owners used by
the remaining workflows. CLI invocation, OS files and external Lua/SPTK
processes remain native as specified by ADR 0007.

## Likelihood CSV

`TrainingResult.write_likelihood_csv()` returns the native CLI likelihood CSV
as owned bytes. It preserves iteration/file/group order, six fractional digits,
signed zero, nonfinite spelling and empty rows. Empty report sets return empty
bytes. The CLI and WASM share the native `write_likelihood_csv` method; reports
and model parameters are unchanged. This avoids relying on JavaScript number
formatting to reproduce Rust's binary32 formatting.

Training checks verify CSV from all nine original C training cases with the
existing 1e-5 likelihood tolerance, empty reports and the exact arbitrary row
`-0.000000,inf,NaN`. Native writer checks cover exact formatting, empty rows,
partial writes, interrupted-write retry, failure propagation and no implicit
flush. Actual C/Python callers validate nine training CSV outputs and unchanged
owners on failed encoding. These checks do not close the final platform or
combined-workflow acceptance gates.

## Batch extraction

`batch_feature_options(preset)` returns all twelve native feature settings for
0 MFCC12-DA-16k, 1 MFCC12-DAE-16k or 2 PLPCC12-DA-16k. Other codes fail.
`BatchOptions` preserves audio options by copied getter/setter and editable
input extension. Defaults and cloning match the native configuration.

`BatchExtraction.extract(wave, stem, options, preset, uniform)` and
`extract_with_sequence(..., sequence)` run the same native preparation and
extraction functions as the file tool. Decode bytes with `Wave.read` first.
The result retains full Audio, Features and BatchOutputs owners, each with
copied access, complete replacement and independent cloning. It can also be
constructed from arbitrary owners. Write raw or parameter bytes with
`rawfloat_write` from the returned sample/value arrays.

BatchOutputs exposes mutable raw/parameter paths and optional MFCC path,
construction and cloning. For native presets the MFCC path is absent. Empty
optional paths remain distinct from absence. Output names append `.raw` and
`.param` to the original stem, including when the input extension is changed.
Input/output aliases fail before drawing noise or publishing results. Callback
exceptions retain their original identity and stop immediately.

The native file tool shares output-path validation and the preparation/extraction
helper with WASM; its filesystem side effects, external Lua scripts and SPTK
process pipelines remain native under ADR 0007. These host extractors are still
supported through native Rust and C ABI interfaces. WASM performs the three
native-preset computations without opening files or spawning processes.

`wasm_batch_node.cjs` and `wasm_batch_browser.html` run all fifteen current
families twice in opposite orders. All three original C preset outputs compare
333 feature values within the existing 2e-5 normalized-error threshold; observed
maximum is 7.748603820800781e-7. Raw audio matches C bit for bit, including
normalization/downsampling and Linux dither. Tests cover all twelve preset fields,
both options fields, all three output/result fields, copied nested settings,
optional paths, custom suffixes, Unicode stems, source release, callback
consumption/errors, alias checks, invalid inputs and arbitrary result owners.
Final combined acceptance and current-source platform verification remain open.

String field access follows the official wasm-bindgen
[getter_with_clone guide](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/getter_with_clone.html).

## Index parsing

`IndexEntries.read(bytes, directory, left_json, right_json)` invokes the native
index parser over an in-memory byte buffer. Padding is supplied as JSON string
arrays, retaining empty and arbitrary strings. Parsing preserves literal-space
tokenization, empty phoneme fields, skipped blank rows, CRLF handling and physical
line numbers in format errors. Invalid UTF-8 and malformed rows fail without
publishing a partial result. Paths use the WebAssembly target's native path
joining rules; strings represent virtual UTF-8 paths, with OS filesystem access
remaining native under ADR 0007.

`IndexEntry` retains both fields: mutable stem and complete atomic replacement
of the phoneme JSON string array. Construction and cloning retain arbitrary
values. `IndexEntries` supports length, copied indexed get/push/replacement,
clear and independent cloning. `index_append_suffix` appends the suffix literally
through the native helper, including empty strings, Unicode and embedded NULs;
it does not replace an existing extension or normalize the resulting path.

`wasm_index_node.cjs` and `wasm_index_browser.html` run all fourteen current
families twice in opposite orders. Index checks compare three complete original
Lua rows and five additional padding/token/path cases, physical error lines,
invalid UTF-8 and padding, five literal suffix cases, both editable entry fields,
atomic replacement errors, copied collections and source release. Native `index`
and `c_api_index` suites pass. Remaining batch/stream workflows and final combined
platform acceptance remain open.

## Build and verification

Use Rust 1.94 and wasm-bindgen CLI 0.2.129, matching the exact optional crate
version. The project was created with Cargo; the optional dependency was added
with `cargo add wasm-bindgen@=0.2.129 --target wasm32-unknown-unknown --optional`.
The `wasm` feature activates that dependency on the WASM target.

```text
cargo build --target wasm32-unknown-unknown --features wasm --lib
wasm-bindgen target/wasm32-unknown-unknown/debug/shiro_rs.wasm --target nodejs --out-dir bindings-node
node tests/wasm_interchange_node.cjs bindings-node
wasm-bindgen target/wasm32-unknown-unknown/debug/shiro_rs.wasm --target web --out-dir bindings-web
```

Serve the project and web bindings over HTTP, mounting web bindings at
`/bindings/`, then open `tests/wasm_interchange_browser.html`. Node and browser
use the same verifier twice on one initialized module. The verified result is
9000 exact rawfloat values, seven unchanged Lua states, 13 unchanged Lua label
rows and four complete document kinds. The Lua numerical comparison retains
the native verifier's absolute tolerance of 1e-14. State fields and rawfloat
bits are exact. The feature-disabled build exposes none of these six owners
or three standalone functions at that initial checkpoint. The expanded
feature-disabled verifier now covers 15 owners, including parameter collections
and the decoded model owner.

`tests/wasm_phones_node.cjs` and `wasm_phones_browser.html` run the original
interchange checks together with all seven unchanged Lua phone cases. They check
115 complete states, five model definitions and all four option fields twice
on the same module. The same native absolute tolerance of 1e-14 applies to the
Lua numerical references. This includes all native topology alternatives and
short-phone cases, shared duration constraints and every output stream count.

## Rawfloat and labels

`rawfloat_read(bytes, maximum_samples)` returns a copied `Float32Array`.
`rawfloat_write(values)` returns a copied `Uint8Array` of little-endian binary32
values. Signed zero, infinities, NaN payload bits and subnormals survive the
roundtrip. Partial final scalars and insufficient sample budgets fail through
native validation. Empty streams are supported.

`Label(start, end, name)` exposes mutable binary64 `start`/`end` and UTF-8 `name`.
It retains raw native field values; label transformations enforce native
interval checks. `Labels` exposes construction, `parse`, `length`, copied
`get`, copied `push`, indexed `replace`, `clear`, `cloned` and byte `write`.
Writing uses native tab-separated seconds and CRLF endings. Invalid output
names fail. Cloning and child retrieval are independent of source lifetime.

`labels.to_states(phoneMap, hop)` returns a `States` owner.
`Labels.from_states(states, hop, includeStates)` returns a `Labels` owner.
These invoke the original native boundary rounding, phoneme grouping,
interleaved state-label option and metadata validation. `label_output_path`
retains the legacy extension/separator rules without performing filesystem I/O.

## Complete JSON owners

`PhoneMap`, `States`, `SegmentationDocument` and `ModelDefinition` accept their
native JSON schemas in the constructor. `json` returns the complete native
document, `replace` validates JSON into a replacement before assigning it, and
`cloned` makes an independent owner. `States` uses an ordered JSON array.
Every field can be changed through complete JSON replacement, including array
lengths. Phone maps and segmentation documents retain flattened attributes at
every native level. State owners retain optional duration/output/jump fields
and all metadata entries. Model definitions retain their native defaulting and
known-field schema. `ModelDefinition.build` constructs the native model. Remaining
inference/training/audio workflow calculations are still required work.

Errors become JavaScript exceptions. Array mutation does not alter Rust owners.
Ordinary inputs are borrowed for a synchronous call; `.free()` releases an
owner when no longer needed. WebAssembly panic/allocation-abort recovery is
not promised.

## Phone expansion and initial segmentation

`PhoneMapOptions` provides the native defaults and mutable `states_per_phone`,
`streams`, optional `topology` and `weak_skips`. `cloned` copies all four fields.
An absent topology uses `undefined`; a present empty string remains present.
Invalid editable values are rejected by calculations through native validation.

`PhoneMap.create(text, options)` expands phone-set text into disjoint ordered
duration/emission IDs. The returned map copies option strings and does not
borrow its source options. `map.to_definition(dimensions, hop)` returns a
complete `ModelDefinition`, preserving per-stream state counts and intersecting
shared duration constraints. `map.initial(phones, frames)` accepts an ordered
JavaScript string array and returns `States`. It retains native flat boundary
rounding, all topology and weak-skip edges, probabilities, duration/output IDs
and phoneme/local-index metadata. None of these calculations mutate the map.
`ModelDefinition.build` constructs a model from this result. The remaining
inference, training and audio workflows are subsequent binding tasks.

## Model construction, complete parameters and codecs

`ModelDefinition.build()` returns an independent binary32 `Model` using the
native SHIRO defaults, stream layouts, mixture counts, weights and duration
constraints. All native definition validation remains active. The original C
construction fixture is reproduced byte for byte.

`Model` exposes stream/duration counts, copied `stream`/`duration`, child
replacement through `set_stream`/`set_duration`, complete `replace(streams,
durations)`, cloning and structural validation. The constructor creates the
native empty model. `dimensions()` calls the native SHIRO importer helper and
returns copied ordered u32 dimensions, rejecting absent streams/emissions.
Complete replacements permit growth, shrinkage and empty collections.

| Owner | Complete native fields and access |
| --- | --- |
| `Duration` | Typed `values`/`set_values` in mean, variance, variance-floor order; signed `constraints`/`set_constraints` in minimum, maximum, fixed-mean order; clone |
| `Gaussian` | Dimensions, component count, copied weights/means/variances/variance floors, checked complete replacement and clone |
| `Stream` | Copied typed weight, weight replacement, emission count/copied child/replacement, complete replacement from `Gaussians`, clone |
| `Model` | Streams and durations through copied children, child replacement and complete replacement from `Streams`/`Durations`, clone |

`Gaussians`, `Streams` and `Durations` expose construction, length, copied get,
copied push, indexed replacement, clear and clone. Returned owners and arrays
remain valid after sources are released. Scalar storage uses typed arrays so
raw NaN payloads, infinities and signed zeros remain accessible without JSON
conversions. Structural parameter replacement validates lengths before mutation;
native calculations enforce their own numeric validity at use.

`Model.read`, `read_with_limits`, `read_prefix`, `read_prefix_with_limits`,
`write` and `write_with_encoding` reuse the native model codecs. The explicit
limit is the cumulative array-entry budget. Prefix decoding returns
`DecodedModel`, with bigint byte position, copied `parameters()` and consuming
`into_parameters()`. Strict reading rejects trailing data; prefix reading can
walk sequential models. Encoding 0 retains variance floors, encoding 1 is the
historical schema and rejects nonzero floors. The default writer uses encoding
0. Invalid codes fail.

`tests/wasm_models_node.cjs` and `wasm_models_browser.html` verify six original C
models, including the bundled historical model, with 91,151 compared wire bytes
including model construction. Every parameter field is compared by raw bits
through cloning, serialization and prefix recovery. Exact budgets and
one-entry-short failures are checked; every truncated prefix of the small
construction fixture and selected boundaries of the larger models fail.
Whole-model growth/shrinkage, copied dimensions, child lifetimes and atomic
invalid edits are included. The model, phone and interchange families run twice
on the same module, reversing their order on the second pass. This checkpoint
does not establish full inference/training or combined final acceptance.

The implementation follows the official wasm-bindgen documentation for
[exported Rust types](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/exported-rust-types.html)
and [copied numeric vectors](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-slices.html).

## Observation and segmentation import

`Observation.read_rawfloat(bytes, dimensions, maximum_frames)` invokes the
native SHIRO importer. `from_model_rawfloat` obtains the ordered stream
dimensions from a model before importing. The budget limits complete frames;
partial scalars or frames fail. Stream samples are deinterleaved without
changing binary32 values. `Observation` exposes frame/stream counts, copied
frames and stream owners, checked child replacement, complete shape replacement,
validation, cloning and native serialization through `write`.

`ObservationStream` exposes dimensions, copied values, complete replacement and
cloning. `ObservationStreams` provides length, copied get/push, indexed
replacement, clear and cloning. Whole observation edits validate the resulting
shape before mutation. Empty observations with positive dimensions are supported.

`Segmentation.from_states(states, model)` invokes the native SHIRO importer,
including time truncation and binary32 residual transition arithmetic. Explicit
delta-1 entries are excluded before appending the ordinary forward transition.
The owner exposes every native field: boundaries, duration states, per-stream
output states and outgoing jump groups. State-array setters check lengths;
outgoing replacement validates structural transition constraints atomically.
`JumpGroup` exposes copied signed deltas and binary32 probabilities, length and
cloning. Complete segmentation replacement, validation, cloning and native
serialization are available. These setters retain native structural semantics;
numeric validity is checked by the operation that uses the data.

`tests/wasm_data_node.cjs` and `wasm_data_browser.html` run all four current
families twice on the same initialized module in opposite orders. Data checks
compare all 12 fixture frames by raw bits against the interleaved source,
original C boundaries and three jump probabilities, plus self/backward jumps,
explicit forward-jump handling and fractional times. They check IEEE payloads,
70 streams, large budgets with small inputs, invalid import shapes, all field
edits, copied arrays, source release and atomic failed edits. Native
`initialization` and `c_api_samples` integration suites provide related importer
and model-initialization regression coverage. Final
combined platform acceptance remain in progress.

## Isolated grouping

`IsolatedGroups.split(model, observation, states)` uses the native shared
grouping operation for isolated alignment and training. It splits at a phone
name change or a non-increasing local state index. Observation intervals are
capped at the available frames; local times retain the native truncated boundary
conversion. Transitions leaving a group are removed, while explicit ordinary
forward entries and all retained jump attributes survive. Original state
positions, original frame positions, all samples and full state metadata are
retained in independent snapshots.

`IsolatedGroup` exposes all four native fields: mutable `first_state` and
`first_frame`, copied `observation()` and `states()`, and setters for those
owners. Construction and editing permit arbitrary native group values, with
validation left to operations that consume the data. `cloned()` retains every
field. `IsolatedGroups` supports empty construction, length, copied get/push,
indexed replacement, clear and cloning.

`tests/wasm_isolation_node.cjs` and `wasm_isolation_browser.html` verify all five
current families twice in opposite orders. The grouping checks use the original
C isolated-alignment state fixture and acoustic input for three variants,
covering six groups and 108 scalar samples. They compare complete local states,
metadata, sample bits and original positions; exercise boundary capping,
fractional times, self/backward jumps, out-of-group filtering and same-phone
index reset; and check all field edits, collection operations, source release,
maximum u32 positions and invalid inputs. Related native `c_api_isolation` and
`rest_cli` suites check the shared operation and original trained model bytes.
Final combined acceptance is still required.

## Dataset ownership and initialization

`Dataset` retains both complete native arrays, accessible through copied
`observations()` and `segmentations()`, independent setters and complete
replacement. Unpaired arrays are representable, matching native public fields;
the consuming operation validates pairing. `Observations` and `Segmentations`
provide empty construction, length, copied get/push, indexed replacement,
clear and clone. Document loading uses the in-memory feature-file inputs
described below.

`Model.initialize(dataset, options)` invokes the native SHIRO initializer and
returns an independent model. `InitializationOptions` exposes all three native
editable fields: `flat_start`, `globally_tied` and `variance_floor_ratio`.
Defaults are false, false and binary32 0.1, respectively; cloning retains every
field. Source parameters and datasets remain unchanged on success or failure.
The shared native implementation preserves flat-start rounding and boundary
capping and fixes the upstream overwritten segment count for corpus fallback.

`tests/wasm_initialization_node.cjs` and `wasm_initialization_browser.html` run
all six current families twice in opposite orders. Initialization compares five
original C model files, totaling 1,440 output bytes, including all four option
combinations and the ten-frame flat-start case. Reloaded results remain exact.
The multi-file case compares the complete original C reference after applying
only the documented fallback-duration correction. Tests exercise both complete
dataset arrays, cloning, growth/shrinkage, independent child owners, all option
fields, invalid settings, unpaired samples and invalid intervals. The related
native `initialization` and `c_api_initialization` suites pass as regression
coverage. Final combined acceptance remains open.

## Document feature-file loading

`FeatureFiles` stores independent byte snapshots addressed by exact UTF-8
filenames. It provides construction, length, sorted copied names, copied get,
set/replacement, remove, clear and cloning. Filenames are not normalized or
case-folded; repeated set replaces that filename's contents. Repeated document
entries resolve the same bytes independently and retain document order.

`Dataset.load(document, model, files, maximum_frames)` loads every document
entry into the complete paired arrays. `Datasets.load_training_files(document,
model, files, maximum_frames, isolated)` returns one dataset per file; each
contains either the embedded sample or its ordered isolated groups. `Datasets`
also provides all standard copied collection operations. Loading validates
model dimensions, per-file frame budgets, rawfloat frames and states using the
same native assembly functions as OS-backed loading. Empty documents return
empty datasets after model validation. Missing filenames and malformed inputs
throw without changing any input or previously returned owner.

`tests/wasm_loading_node.cjs` and `wasm_loading_browser.html` run all seven
current families twice in opposite orders. They verify ordered and repeated
files, UTF-8 names, complete observation and segmentation bytes, six isolated
samples, boundary capping and local transition filtering, file replacement,
independent snapshots, exact names, missing and late malformed files, frame
budgets, empty documents and source release. A document-loading-to-initialization
path produces the exact original C aligned model. Native initialization and
training suites cover the shared OS-backed assembly, including original model
bytes and likelihood reports. These bindings expose data loading without OS
file APIs; final combined acceptance remains in progress.

## Training and progress

`Model.train(files, options)` returns an independent `TrainingResult` containing
the complete model and every ordered iteration report. `train_with_progress`
also calls a synchronous JavaScript function with an independently owned
`IterationReport` after each completed model update. The callback may retain,
edit or free that report; it must free retained owners when finished. Callback
exceptions stop before the next iteration and propagate the original JavaScript
value unchanged. Input owners remain borrowed for the call and must not be
mutated or freed during a callback. A callback return value is ignored.

`TrainingOptions` exposes all 13 native fields with setters, defaults and clone:
`iterations`, `duration_mode`, `hsmm_temperature`, `duration_weight`,
`state_radius`, `duration_extra`, `duration_extra_factor`,
`geometric_temperature`, `pruning_slope`, `termination_threshold`,
`deterministic_annealing`, `mean_frame_likelihood` and `workers`. Duration mode
0 selects HSMM and 1 selects geometric HMM; other values fail. As in native
training, the per-iteration annealing schedule replaces both supplied inference
temperatures. The standard browser target has no OS threads. For workers above
one, it executes per-file estimates sequentially while preserving the native
worker path's independent statistics and ordered reduction. Native builds
continue to use OS workers. This changes scheduling, not the reduction order.

| Owner | Complete native fields and access |
| --- | --- |
| `FileLikelihoods` | Typed copied values, full replacement and clone |
| `LikelihoodRows` | Ordered rows with all copied collection operations |
| `IterationReport` | Mutable iteration, temperature and mean log likelihood; copied rows and replacement; arbitrary constructor and clone |
| `IterationReports` | Ordered reports with all copied collection operations |
| `TrainingResult` | Copied model and iterations, replacement of each field, arbitrary constructor and clone |

The callback uses `js-sys` 0.3.106, added as an optional target dependency with
Cargo and activated by the `wasm` feature. Exception handling follows the
official [Function API](https://wasm-bindgen.github.io/wasm-bindgen/api/js_sys/struct.Function.html)
and [Result boundary](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/result.html).
Remaining SHIRO operations and final cross-platform acceptance are
still required before claiming completion.

`tests/wasm_training_node.cjs` and `wasm_training_browser.html` run all eight
current families twice in opposite orders. Nine C reference cases cover embedded
and isolated HMM/HSMM, annealing and frame-mean likelihood reporting. All 2,592
model bytes match exactly; 18 reports and 27 likelihood values satisfy the
existing 1e-5 absolute tolerance. Tests retain callback owners after the call,
compare every report field against returned results, verify temperature and
aggregate arithmetic, convergence stopping and zero iterations, callback
exception identity and immediate stopping, subsequent recovery, arbitrary
report/result editing, copied collection ownership and repeated worker-path
results. Native unit tests compare the browser's ordered estimates against
actual OS workers for both duration modes and verify fallible progress stopping.
The existing native `training`, `c_api_training` and `rest_cli` suites continue
to pass. This checkpoint does not refresh the full platform matrix or establish
final cross-crate acceptance.

## State and document alignment

`Model.align_states(observation, states, options)` returns independent complete
states. `align_document(document, files, maximum_frames, options)` returns a
complete independent document, retaining file order, filenames and all document,
file and state attributes. The document path shares native assembly and prepares
selected duration distributions once for the corpus. It resolves exact names
through `FeatureFiles` and enforces the supplied per-file frame budget. Native
OS-backed alignment retains its existing maximum and process-relative filenames.

`AlignmentOptions` exposes all nine native fields with defaults, setters and
clone: `duration_mode`, `isolated`, `hsmm_temperature`, `duration_weight`,
`state_radius`, `duration_extra`, `duration_extra_factor`,
`geometric_temperature` and `pruning_slope`. Mode 0 uses explicit HSMM durations;
mode 1 uses geometric HMM. Options retain native numeric validation, including
permitted zero temperature. Isolated alignment shares grouping and boundary
capping; explicit and isolated outputs remove jump attributes as in native
materialization, while embedded geometric output retains them. Unused explicit
duration densities are not prepared, and HMM alignment does not require them.

`tests/wasm_alignment_node.cjs` and `wasm_alignment_browser.html` run all nine
current families twice in opposite orders. Nine original C modes provide 40
state comparisons, covering embedded/isolated HMM/HSMM and pruning options.
Additional assertions cover nested state/file/document metadata, independent
results, reused corpus preparation, native no-path errors, zero temperature,
invalid selected and unused uninitialized durations, capped intervals, local
jump filtering, repeated-phone resets, invalid identity/interval/options,
missing or malformed files and frame budgets. Native `alignment` and
`c_api_alignment` regression suites validate the shared OS-backed path. This
does not establish the remaining feature/audio/model workflows or final combined
platform acceptance.

## Distribution untying and summary output

`Model.untie(document)` duplicates duration and emission distributions in
file/state order and rewrites all document model references. It returns an
independent `UntiedModel` with all three native fields: copied `model()`, copied
`segmentation()` and copied `assignments()`. Each field has a complete setter;
the owner also supports an arbitrary native-value constructor and clone.
`Assignment` exposes mutable state/file/segment indices, construction and clone.
`Assignments` provides length, copied get/push, indexed replacement, clear and
clone. These arbitrary owners retain native validation boundaries rather than
enforcing corpus constraints during field editing.

`UntiedModel.write_summary()` returns the native summary byte sequence. All
assignment locations and optional phone/index metadata are checked before
output; empty metadata yields only the three assignment indices. Metadata state
indices retain native truncating signed conversion. Untying does not open feature
files. Times, jumps, full metadata and unknown document attributes are retained.
The existing native correction preserves stream weights lost by the C tool.

`tests/wasm_untying_node.cjs` and `wasm_untying_browser.html` run all ten current
families twice in opposite orders. They compare the original C model's 521 bytes,
JSON document and 60 summary bytes, six original and twelve multi-file assignments,
complete result and assignment field editing, weighted parameter preservation,
independence of repeated distributions, nested metadata/jumps, arbitrary u32
indices, missing or invalid references, optional and invalid summary metadata,
empty documents, cloning and source release. The weighted result reproduces the
C model after changing only its two corrected weights back to C's values. Native
`untying` and `c_api_untying` suites provide related regression coverage. Remaining
feature/audio/utterance operations and final platform acceptance remain open.

## Feature extraction

`Features.extract(signal, options)` invokes the native extractor.
`FeatureOptions` exposes all 12 native editable fields and cloning: kind,
order, channels, frame length, fractional hop, sample rate, minimum bandwidth,
warp, DC inclusion, energy mode, delta and acceleration. Kind 0/1/2 selects
MFCC/MFBE/PLPCC; energy 0/1/2 selects none/RMS/decibels. Other codes fail.
Defaults match native options. `Features` retains all three public fields:
mutable frames and columns, copied values and complete value replacement.
Arbitrary shapes are representable as in the native struct; extraction validates
its own inputs. Construction and cloning retain binary32 storage independently.

`wasm_features_node.cjs` and `wasm_features_browser.html` run all eleven current
families twice in opposite orders. Feature checks consume all 72 original C
records and 6,399 values, including 42 matching nonfinite results, using the
existing 2e-5 normalized-error threshold. They also verify defaults, option
cloning, empty signals, invalid codes/settings/nonfinite input, independent
arrays, arbitrary shapes, all result fields and IEEE payloads. Related native
`features` and `c_api_features` suites pass. Audio and utterance workflows and
final combined platform acceptance remain open.

## Utterance segmentation

`SegmentedUtterances.split_features(features, filename, options, source)` runs
the native in-memory utterance workflow. `UtteranceOptions` retains all five
editable native settings and defaults: utterance count, hop seconds, minimum
silence/voicing seconds and iterations. `UtteranceModelSource.fresh()`,
`.initialized(model)` and `.trained(model)` select all three native paths.
Sources own independent model copies, expose kind 0/1/2 and an optional copied
model, and support cloning. Fresh models initialize and train; initialized
models train; trained models align without estimation.

`SegmentedWave.split` accepts a Wave, filename, dimensions, feature kind 0/1/2,
options, source and synchronous uniform callback. `split_with_sequence` takes
a mutable DitherSequence instead. The native pipeline resamples to 16 kHz,
adds level-0.01 dither and extracts MFCC/MFBE/PLPCC with RMS energy. Invalid
feature setup consumes no draws; callback failure stops with its original
JavaScript exception value. Other validation and calculation boundaries match
the native workflow.

All ten SegmentedUtterances fields are preserved: phonemap, definition, phones,
initial segmentation, optional uninitialized/initialized models, final model,
iteration reports, alignment and labels. Child owners are copied, setters borrow
and copy complete values, and each optional model has separate set/clear methods.
Phones use a JSON string array with atomic replacement, retaining arbitrary
strings. Construction accepts model/map/definition/initial/aligned owners; the
remaining collections start empty and optional models absent, and every field
can be replaced independently. SegmentedWave retains audio, features and
utterances with complete setters, construction and independent cloning.

`wasm_utterances_node.cjs` and `wasm_utterances_browser.html` run all thirteen
families twice in opposite orders. They compare all three original C intermediate
models, totaling 1,386 bytes, full reference documents and five C/Lua labels.
The waveform pipeline matches all 64,000 C audio samples exactly and compares
520 feature values within the existing 2e-5 normalized-error gate; maximum
observed error is 8.493661880493164e-6. Coverage includes all source variants,
all feature kinds, complete reports/results/field replacement, zero iterations,
nondefault timing and floors, UTF-8 filenames, callback consumption/errors,
invalid inputs and source release. Related native `utterances` and
`c_api_utterances` suites pass. Remaining batch/index/stream workflows and final
combined platform acceptance remain open.

Ownership follows the official wasm-bindgen
[exported Rust types guide](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/exported-rust-types.html):
borrowed parameters remain usable and returned owners have independent storage.

## Audio conversion

`Wave.read(bytes, maximum_frames)` uses the native bounded WAV decoder. Its
sample rate, bits per sample, channels, encoding and samples remain accessible
and editable. Encoding 0 selects PCM and 1 selects float; other codes fail.
The constructor and clone retain independent samples and arbitrary native field
values. Decoding selects the left channel, as in the native implementation.

`AudioOptions` exposes all five native settings and cloning: normalization,
dither level, optional output sample rate, boundary policy and kernel policy.
The default output rate is absent; explicit zero is invalid. Boundary 0/1 selects
include-first/legacy-skip-first and kernel 0/1 selects stable/legacy. Unknown
codes fail. Normalization precedes dither, which precedes resampling. Nonpositive
dither levels do not call the random source. Silent normalization preserves
signed zero.

`Audio.prepare(wave, options, uniform)` accepts a synchronous JavaScript function
returning a number in [0, 1]. Invalid draws stop processing immediately. A thrown
JavaScript value is returned with its original identity; partial audio is not
published. `Audio.prepare_with_sequence` accepts a mutable `DitherSequence`,
whose `windows()` and `linux_gnu()` factories retain the original seed-one
runtime sequences. `next_uniform()` advances the sequence; preparation consumes
one draw per input sample when dither is enabled. Audio sample rate and samples
support full editing, arbitrary construction and independent cloning.

`wasm_audio_node.cjs` and `wasm_audio_browser.html` run all twelve current families
twice in opposite orders. Five original C conversions compare 1,286 samples at
the existing 2e-7 normalized-error gate, with zero observed error. Both runtime
sequences compare 128 exact uniform draws and 128 exact signed noise values.
The complete original C dithered WAV is checked bit for bit. Tests also cover
all fields, optional-rate presence, both policies, mutable sequence consumption,
callback order/errors/recovery, frame budgets, truncated WAVs, copied arrays,
source release, empty signals and arbitrary IEEE audio values. Native `audio`
and `c_api_audio` suites pass. Remaining batch/utterance/stream workflows and
final combined platform acceptance remain open.
