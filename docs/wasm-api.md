# Browser and Node.js WebAssembly API

The optional `wasm` feature exposes native computations through independent
owners and copied byte/typed arrays. Full SHIRO bindings and final combined
acceptance are still in progress. This checkpoint implements rawfloat, all
native label and phone-map operations, model construction/IO/parameter access,
observation and segmentation import with complete owned field access,
isolated grouping,
dataset owners, document loading and model initialization,
and complete JSON document owners used by
the remaining workflows. CLI invocation, OS files and external Lua/SPTK
processes remain native as specified by ADR 0007.

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
and model-initialization regression coverage. Inference, training and final
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
Training workflows and final combined acceptance are still required.

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
coverage. Full training, inference and final combined acceptance remain open.

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
file APIs; training and final combined acceptance remain in progress.
