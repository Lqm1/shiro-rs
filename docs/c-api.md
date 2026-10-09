# C interface

Build the optional interface with `cargo build --features c-api`. The package
produces Rust, shared C and static C libraries from the same library target.
The generated declarations are in `include/shiro_rs.h`; ABI version is 1.
The current header declares 223 exports, twenty-nine opaque owner types, twenty-seven
settings, report and IO descriptors, and progress and uniform callback types.

## Current coverage

### Complete batch extraction fields and variants

Twenty-two additional exports expose all three native presets, complete batch
options, every extractor variant and all output paths. Preset codes zero, one
and two select MFCC-DA, MFCC-DAE and PLPCC-DA. Feature settings are derived from
the native preset. Batch options retain every audio field and complete UTF-8
input suffix, including empty strings and NUL. Native numerical validation
remains in extraction. Options have native defaults, full independent snapshots,
deep clone and release.

Extractor owners retain either the complete native preset, all three SPTK program
paths or the Lua interpreter, script and executable directory. Variant-specific
getters return range errors with unchanged outputs when that field is absent.
All path snapshots preserve native units, including non-Unicode paths. Outputs
retain raw, parameter and optional MFCC paths, including absence versus a present
empty path. Arbitrary construction follows the native public fields without
filesystem restrictions; snapshots and clones remain independent.

File extraction delegates the full native workflow, including process execution,
write order, output suffixes and caller-controlled uniform draws. Output storage
is validated before file or callback effects. Failed output slots remain
unchanged; files and RNG draws already consumed follow native behavior and are
not rolled back.

Focused Windows x86_64 Rust tests pass five cases and Clippy passes. They compare
all three presets with original C raw/features, all feature/audio fields with
native settings, non-Unicode/empty/NUL/repeated input ownership, complete output
paths, optional presence, native option processing, controlled draw counts and
validation/failure side effects. Actual Lua and SPTK protocol-fixture workflows
pass, including process failures. Optimized assertion-enabled C and Python ctypes
callers pass all twenty-two exports with full variant/field and original-fixture
checks. The independent source audit confirms native code and all 146 previous
C declarations are unchanged; the existing optional audio conversion is shared
through module visibility only. The full five-target matrix passes 53 suites and
110 tests per target, with no failures or ignored tests. All seventeen optimized
C caller families pass all five targets, and all seventeen Python families pass
the three 64-bit targets. Both feature variants pass formatting, Clippy and
Rustdoc with warnings denied. Feature-disabled builds exclude all 168 exports,
and generated-header verification passes. Protocol fixtures do not establish actual SPTK
version compatibility on those platforms. Utterance workflows and all WASM
interfaces remain required.

### Direct native streams

Seven additional exports call native rawfloat reading/writing, multistream
observation reading, label output, untied-model summary output and buffered index
reading directly. An explicit flush operation completes the borrowed writer
interface. Read/write callbacks report partial transfer counts with statuses
zero success, one interrupted and any other value IO error. Invalid counts are
rejected. Native operations retain their own retry and failure behavior, and
never implicitly flush or close the caller's stream. Consumed input and partial
output are retained after failure; independent output owners are published only
after successful native processing.

The buffered reader calls the caller's fill and infallible consume callbacks
directly. It adds no buffer or read-ahead. Fill returns a borrowed initialized
readable range valid until the next fill/consume, with an empty range indicating
EOF. All callback contexts remain independent and live for the synchronous
operation. Callbacks must return normally and must not release or modify active
owners or output slots. The library retains no callback or context.

Focused Windows x86_64 Rust tests pass five cases comparing complete native
results, callback sequences, requested transfer sizes and consumed/emitted bytes
with independent native Read/Write/BufRead implementations. They cover original
fixtures, full binary32 bits, multistream data, one-byte fragmentation,
interruption, EOF, budgets, partial scalars, invalid counts/buffer ranges, late
IO failures, label-name/summary validation and retained output owners. Clippy
passes. Optimized assertion-enabled C and Python ctypes callers pass all seven
exports with actual callbacks and original fixtures. The native-source audit
confirms existing functions and declarations are unchanged. The full five-target
matrix passes 52 suites and 105 tests per target, with no failed or ignored tests.
All sixteen optimized C caller families pass all five targets, and all sixteen
Python families pass the three 64-bit targets. Both feature variants pass
formatting, Clippy and Rustdoc with warnings denied. Feature-disabled builds
exclude all 146 exports, and generated-header verification passes.

### Complete index fields and phoneme padding

Twelve additional exports introduce independent ordered string and index-entry
owners. String construction accepts an array of byte owners, validates every
complete UTF-8 string, and retains empty strings, embedded NUL, whitespace and
repeated inputs. Count, independent byte snapshots, deep clone and release expose
the full collection. Entry construction accepts every native field: a complete
native path and an ordered phoneme collection. It imposes no filesystem or
phoneme-content restrictions beyond those of the native public fields. Count,
independent path and phoneme snapshots, deep clone and release retain all fields
after source and parent release. An invalid late descriptor or string leaves
the caller's output slot unchanged.

The byte reader delegates `index::read` with complete directory and left/right
padding inputs. Original literal-space tokenization, empty phoneme fields,
skipped blank rows, CRLF, native path joins, invalid UTF-8 and physical-line
format errors follow the native reader. Non-Unicode directory units remain
intact in returned stems. No lossy path conversion is used.

Focused Windows x86_64 Rust, optimized assertion-enabled C and Python tests pass
all twelve exports. They compare every field of the original index fixture and
exercise whitespace/NUL padding, non-Unicode paths, repeated owners, independent
lifetimes, empty collections and retained outputs after errors. Clippy and the
native-source preservation audit pass. The complete five-target matrix passes
51 suites and 100 tests per target, with no failures or ignored tests. All fifteen
C caller families pass all five targets, and all fifteen Python families pass
the three 64-bit targets. Both feature variants pass formatting, Clippy and
Rustdoc with warnings denied; disabled builds exclude all 139 exports and
generated-header checks pass. Direct `BufRead` support is verified by the later
stream checkpoint above. Utterance workflows and all WASM bindings remain required.

### Lossless native host paths

Seven additional exports provide a complete independent native path owner,
native encoding query, platform-byte and UTF-8 constructors, native-byte snapshots,
suffix appending, cloning and release. Native encoding is 1 for Unix bytes and
2 for Windows little-endian 16-bit units; 0 indicates an unavailable encoding.
Lengths are explicit and no terminator is added. Unix non-UTF-8 bytes and Windows
unpaired surrogates remain intact through native construction and retrieval.
Odd Windows byte lengths are rejected. UTF-8 construction validates the entire
string. Embedded NUL is retained in paths and suffixes; native file operations
may impose their own restrictions later.

Suffix appending delegates `index::append_suffix` with the complete path and
validated UTF-8 suffix, preserving every original native unit. Copies and byte
snapshots remain independent after input and parent release. No lossy Unicode
conversion or internal pointer exposure is used. Platform conversions follow
the official Rust `OsStringExt` and `OsStrExt` examples.

Focused Windows x86_64 Rust, optimized assertion-enabled C and Python checks pass
all seven exports. They cover invalid native Unicode, embedded NUL, non-BMP UTF-8,
empty paths, suffixes, clone/snapshot lifetimes and retained outputs on errors.
The complete five-target matrix passes 50 suites and 97 tests per target, with
no failed or ignored tests. All fourteen optimized C caller families pass all
five targets, and all fourteen Python families pass the three 64-bit targets.
Both feature variants pass formatting, Clippy and Rustdoc with warnings denied.
Feature-disabled builds exclude all 127 exports; generated-header and native
source-preservation checks pass. Complete index collections, generic streams and
batch extraction are verified by their later checkpoints above. Utterance
workflows and all WASM bindings remain required.

### Complete audio preparation and legacy dither

Twelve additional exports expose every native audio option, complete decoded
wave header inputs, complete audio results, and both legacy random sequences.
Audio options retain normalization, binary32 dither level, explicit optional
output-rate presence, and independent corrected/legacy boundary and kernel
policies. Absence differs from a present zero rate, which native validation
rejects. A decoded wave descriptor retains rate, bit depth, channels and encoding;
samples are independently owned. Preparation delegates native validation without
adding constraints on header fields that the native operation does not inspect.
A bounded WAVE-byte entry point uses the native decoder before preparation.

Audio constructors retain arbitrary public sample rates and every sample bit.
Cloning and sample snapshots remain independent after input and parent release.
The Windows and Linux GNU constructors create their explicit seed-one sequences;
exclusive next-draw calls validate output storage before advancing state.

Preparation accepts a synchronous uniform callback and caller context. Return
zero and write a finite value in [0,1]; nonzero status or an invalid draw fails
preparation with unchanged output. A null callback is allowed when no draw is
needed and otherwise fails. Callbacks must return normally, retain their context
for the call and avoid mutating or releasing active inputs. Consumption of RNG
state already performed is not rolled back after a later failure. Nonpositive
dither consumes no draws. All output slots are validated before preparation or
callback invocation.

Focused Windows x86_64 Rust, optimized assertion-enabled C and Python callers
pass all five original WAVE conversion cases, both 64-draw original C sequences,
the exact signed-noise outputs and original Linux dithered WAVE bytes. Native
comparisons cover every option, unused header fields, corrected and legacy
resampling, optional-rate presence, source preservation and callback failures.
Arbitrary result constructors preserve signed zero, subnormals, NaN payloads and
infinities with independent snapshots. The full five-target matrix passes
49 suites and 95 tests per target with no failed or ignored tests, retaining all
14 original executable tests. All thirteen optimized C caller families pass on
all five targets; all thirteen Python families pass on the three 64-bit targets.
Both feature configurations pass formatting, Clippy and Rustdoc with warnings
denied. Feature-disabled checks confirm all 120 ABI symbols are absent; generated
header and source-preservation verification pass. The initial Rustdoc interval
link and GCC test-variable macro collision were corrected without changing
native algorithms or comparison thresholds. Remaining native workflows and all
WASM bindings are still required.

### Complete feature extraction

Seven additional exports provide every native feature setting and all public
feature matrix fields. The descriptor contains kind, order, channels, frame
length, fractional hop, sample rate, minimum bandwidth, warp, DC inclusion,
optional energy, delta and acceleration. Kind codes are 0 MFCC, 1 MFBE and
2 PLPCC; energy codes are 0 absent, 1 RMS and 2 decibels. Flags accept only 0 or
1. Numeric validation delegates the native extractor. Defaults derive directly
from the native defaults.

The independent result owner retains frame count, column count and all binary32
values. Arbitrary construction preserves every public native field, including
inconsistent dimensions and nonfinite values; downstream consumers retain their
native validation. Info, value snapshots, cloning and release preserve ownership
independence. No internal buffer pointer escapes. Empty extraction retains the
computed column count.

Focused Windows x86_64 Rust, optimized assertion-enabled C and Python callers
pass all 72 original C cases and 6,399 feature values using the native comparison
threshold of normalized error below 2e-5, with independent checks for matching
nonfinite outputs. Rust also compares every nondefault setting, all feature and
energy modes and empty fractional-hop cases against independent native calls.
Tests retain full-width arbitrary dimension fields and signed zero, subnormal,
NaN-payload and infinity value bits after source and parent owners are released.
Invalid codes, numeric inputs and pointers retain outputs. Focused Clippy passes.
The full five-target matrix passes 48 suites and 91 tests per target with no
failed or ignored tests and all 14 original executable tests retained. All
twelve optimized C caller families pass on all five targets; all twelve Python
families pass on the three 64-bit targets. Both feature configurations pass
formatting, Clippy and Rustdoc with warnings denied. Feature-disabled checks
confirm all 108 ABI symbols are absent; generated-header and source-preservation
verification pass. Remaining native workflows and WASM are still required.

### Standalone dimensions and target-width arrays

Six additional exports expose the native ordered model dimensions and an
independent unsigned target-width array with creation, length, checked range
copy, cloning and release. Model dimension retrieval delegates
`dataset::dimensions`, preserving its validation and stream order. Arbitrary
array construction retains every integer bit, including zero and the largest
target-width value; it does not impose model dimension constraints on an array.
Copies validate the entire range before writing and preserve output buffers
on errors. No pointer into internal storage escapes.

Focused Windows x86_64 Rust, optimized assertion-enabled C and Python checks
pass. Original models and a three-stream definition with widths 7, 1 and 19
verify the complete ordered result. Tests cover repeated independent model
snapshots, release of input owners, full-width maximum values, subrange copying,
overflow and bounds failures, null and misaligned input rejection, and empty
owners. The full five-target matrix, including the heterogeneous-width definition
case, passes 47 suites and 88 tests per target with no failed or ignored tests.
All 14 original executable tests remain included. All eleven optimized C caller
families pass on all five targets; all eleven Python families pass on the three
64-bit targets. Both feature configurations pass formatting, Clippy and Rustdoc
with warnings denied. Feature-disabled checks confirm all 101 ABI symbols are
absent; generated-header and source-preservation verification pass. WASM and the
remaining native workflows are still required.

### Standalone isolated groups

Eight additional exports provide standalone native group splitting, an arbitrary
complete-field constructor, length, position retrieval, independent observation
and state snapshots, cloning and release. Both `first_state` and `first_frame`
use target-width integers. Construction deep-copies every public native field
in caller order and accepts empty collections and repeated input owners.
It preserves arbitrary positions without imposing validation absent from the
native public data structure. Splitting delegates native validation, phone-name
and local-index grouping, frame-end capping and local jump filtering.
State metadata and local times are retained even when the final sample interval
is capped at the observation length.

Focused Windows x86_64 Rust, optimized assertion-enabled C and Python callers
pass. Tests compare original fixture positions and independent rawfloat slices,
full state JSON, nested attributes, crossing-jump filtering and capped final
intervals. Constructors retain full-width maximum position values and complete
observation/state data; snapshots remain usable after parent and input owners
are released. Invalid descriptors, oversized ranges, missing states and
nonpositive intervals retain output slots. The focused Rust suite and Clippy
pass. All five required native targets pass 46 suites and 86 tests per target,
with no failed or ignored tests and all 14 original executable tests retained.
All ten optimized C caller families pass on all five targets; all ten Python
families pass on the three 64-bit targets. Both feature configurations pass
formatting, Clippy and Rustdoc with warnings denied. Feature-disabled checks
confirm all 95 ABI symbols are absent; generated-header and source-preservation
verification pass. WASM remains required.

The initial interface provides 13 exports: the ABI version, creation, length,
copy, clone and release operations for byte and binary32 array owners, and
rawfloat byte decoding and encoding. Each owner keeps an independent copy.
Rawfloat operations call the existing native implementation and preserve every
binary32 bit, including signed zero, subnormals, infinities and NaN payloads.
This foundation does not complete any of the 14 tool workflows. Their C and
WASM status is tracked separately in `binding-inventory.csv`.

The required scope includes all native public functionality and all original
C and Lua tools. No functionality is removed when adding bindings. Browser
WASM uses in-memory computation and model operations; native interfaces retain
CLI, filesystem and external Lua/SPTK operations as specified in ADR 0007.

## Ownership and errors

### Model interface

Five additional exports create a SHIRO-owned complete model from the original
JSON definition, read model bytes with a cumulative array-entry limit, write
model bytes with an explicit schema, clone all parameters and release the owner.
They delegate to `ModelDefinition::build` and the unchanged native model codecs.
Encoding 0 includes variance-floor fields. Encoding 1 uses the older schema and
rejects nonzero variance floors. Reading rejects trailing bytes and accepts both
historical schemas. The interface retains every stream and duration field.

Rust, optimized C and Python callers pass against the
original empty C model, CMU Arctic pretrained model, multistream initialized
model, weighted model, DAEM trained model and utterance trained model. The full
five-target matrix passes 38 suites and 65 tests per target with no failed or
ignored tests. Both C caller families pass all five targets; both Python caller
families pass the three 64-bit targets. Feature isolation and the quality gates
also pass. Model generation covers the computation of `shiro-mkhsmm`; its native
CLI still provides file and stdout handling. Remaining SHIRO workflow bindings
and all WASM bindings remain required.

`native-binding-inventory.csv` separately lists the 35 explicit public native
functions, including methods and host operations. Model definition construction
and complete state-to-segmentation conversion and both alignment operations are
implemented, as are native host dataset loading, initialization and complete
training with progress. Rawfloat and observation byte and stream operations,
standalone dimensions, complete indices and batch extraction are implemented.
The remaining two function mappings are utterance feature and waveform splitting.
Public types, fields, derived trait
behavior and crate reexports require separate review; a function count alone
does not prove complete coverage.

Inputs remain caller-owned. Copy operations write into caller-provided storage;
the interface never returns a pointer into an internal vector. Release accepts
an owner slot, clears it before dropping the owner, and accepts an empty slot.
Clones survive release of their source. Copy ranges are checked before writes.

Status 0 means success, 1 invalid pointer or alignment, 2 invalid address range,
3 a native input, I/O or calculation error, and 4 a caught unwinding panic.
Output slots retain their previous value on failure. Rawfloat decoding rejects
partial scalars and sample counts above the supplied budget.

Callers must provide live, aligned allocations of the stated size, independent
output slots and exclusive writable access. Runtime checks cannot prove these
allocation and aliasing requirements. An owner must be released by the library
that created it. Do not pass owners from another crate's dynamic library.

Unsafe code is restricted to the optional C boundary. Builds without `c-api`
retain the crate's prohibition on unsafe code. Exported functions use the C ABI
and Rust 2024's explicit unsafe attribute for symbol names. Panic containment
applies to unwinding panics; it cannot recover from process aborts.

## Verification

### Untied models and assignment tables

Nine exports expose complete native untying, an independent result owner,
construction from all public model/document/assignment fields, model and document
snapshots, assignment count and field getters, summary bytes, clone and release.
Constructed assignments preserve target-width integers without enforcing summary
validity. Summary generation retains native row validation and publishes bytes
only after success. It does not open observation files.

Focused Windows x86_64 Rust, optimized C and Python callers pass against the
original C model, complete document and summary. All ordered assignment fields
and independent snapshot lifetimes are checked, including maximum-width integer
fields, late invalid assignments and unchanged failure outputs. Rust additionally
compares complete weighted, multi-file results and all metadata against independent
native calls, retaining stream weights corrected in the Rust implementation.
The five-target matrix passes 45 suites and 84 tests per target, with no failed
or ignored tests. All nine C caller families pass all five targets; all nine
Python families pass the three 64-bit targets. Feature-disabled builds exclude
all nine new exports. Formatting, Clippy and Rustdoc with warnings denied,
generated-header verification and source-preservation checks pass. Direct summary
streams are verified by the later stream checkpoint above. All WASM bindings
remain required.

### Timed labels and conversions

Eleven exports expose complete timed labels, construction, count, scalar and name
snapshots, clone/release, native text parsing, label/state conversion, text output
and legacy output paths. Constructors retain every binary64 time bit and the
complete UTF-8 name, including embedded NUL. Native `Label` permits arbitrary
field values; each transformation retains its own validation. Name snapshots
are independent byte owners and survive source release.

Conversion uses the complete phone map and state owners. Hop remains binary64;
the optional state-row flag accepts codes 0 and 1. Parsing retains native tab,
space and blank-row behavior. Text output uses native round-trip decimals and
CRLF endings. Byte output is published only after every row succeeds. Paths
preserve slash/backslash handling and leading-dot extension behavior.

Focused Windows x86_64 Rust, optimized C and Python callers pass against original
Lua label rows and complete original state sequences. Tests retain signed
zero, subnormals, NaN payloads and infinite constructor fields, all name bytes,
independent owner lifetimes and unchanged outputs after invalid pointers,
indices, flags, hops, parsing and late text-output errors. The five-target matrix
passes 44 suites and 82 tests per target, with no failed or ignored tests. All
eight C caller families pass all five targets, and all eight Python families
pass the three 64-bit targets. Feature-disabled builds exclude all eleven new
exports. Formatting, Clippy and Rustdoc with warnings denied, generated-header
verification and source-preservation checks pass. Direct output streams are
verified by the later stream checkpoint above. All WASM bindings remain required.

### Phone maps and initial segmentation

Eight exports expose a complete phone map owner, native phone expansion defaults,
creation, original JSON read/write, deep clone and release, model-definition
conversion and initial segmentation. The descriptor retains state/stream counts
and the weak-skip flag. Topology uses a separate optional UTF-8 byte owner so an
absent value remains distinct from an empty or unknown topology string.
All map, phone and state JSON attributes are retained. Initial state generation
preserves native phone order, weak skips, topology edges, frame rounding and
state metadata. Definition conversion preserves native tied-constraint handling.

Focused Windows x86_64 Rust, optimized C and Python callers pass against all
original Lua phone-map, definition and segmentation cases. JSON numbers use the
existing binary64 tolerance of 1e-14; strings, arrays and attributes compare
completely. Definition-generated model bytes match exactly. Tests cover defaults,
independent copies, additional metadata and unchanged outputs after errors.
The five-target matrix passes 43 suites and 80 tests per target with no failed
or ignored tests. All seven C caller families pass all five targets, and all
seven Python families pass the three 64-bit targets. Feature-disabled builds
exclude all eight new exports; formatting, Clippy and Rustdoc with warnings
denied, generated-header verification and source-preservation checks pass. Standalone
label conversion, other remaining C operations and all WASM operations remain
required.

### Training interfaces

The training interfaces provide complete ordered inputs, all thirteen native
settings, training with or without synchronous progress, complete model/report
results, independent snapshots, deep clones and releases. Each input dataset
represents one original file and retains every embedded or isolated group.
The host loader delegates to `dataset::load_training_files`. Results retain
every iteration, temperature, mean likelihood and nested file/group likelihood.

`shiro_rs_training_result_create` copies a complete model and a table of report
owners without executing training. Repeated report pointers, empty report tables,
empty file rows and arbitrary binary32 report values are retained independently.
`shiro_rs_training_result_replace` replaces both public native fields only after
copying every input. `shiro_rs_iteration_report_replace` similarly replaces all
scalar fields and nested rows of an owned report. Existing report construction
remains available through `shiro_rs_iteration_report_create`. Replacement must
not target a callback-borrowed report or overlap its input storage. Null tables
are permitted only for zero counts. Invalid ranges/owners retain all destination
fields; constructor failures retain the output slot. Inputs are never retained.

Current Windows x86_64 Rust, optimized assert-enabled C and Python tests cover
arbitrary iteration indices, signed zero, infinity, NaN payloads, subnormals,
empty/repeated rows, ordered repeated reports, invalid nested pointers, oversized
counts, full field replacement, replacement with a different model, source
release and clone independence. Native Rust additionally checks NaN scalar bits.
These current-source operations still require the final five-target refresh.

Mode and boolean flags accept integer codes 0 and 1. Other settings retain native
validation. Both inference temperatures are replaced by the native per-iteration
annealing schedule, including temperature 1 when annealing is disabled.
Zero iterations retain the original model and produce no reports or callbacks.

Progress runs after each model update and before report storage and stopping.
The callback report is read-only and borrowed only for that invocation. It may
be inspected or cloned, but only an owned clone or getter result may be released.
Cloned reports survive the callback and training result. Callbacks must return
normally without unwinding and must not mutate or release participating inputs.
Already delivered notifications are retained if a later iteration fails.

Focused Windows x86_64 Rust, optimized C and Python callers pass against all nine
original C model/likelihood cases. Models match exact wire bytes; likelihoods
use the established absolute tolerance of 1e-5. Tests cover full report fields,
independent lifetimes, defaults, stopping, zero iterations and unchanged failure
outputs. Rust additionally compares complete nondefault settings and ordered
parallel reduction against independent native calls. Python saves and reloads
trained models. Final combined initialization, training and inference acceptance
and every remaining binding are still required. The five-target matrix passes
42 suites and 78 tests per target, with no failed or ignored tests. All six C
caller families pass all five targets; all six Python families pass the three
64-bit targets. Feature isolation, Clippy and Rustdoc with warnings denied,
generated-header verification and source-preservation checks pass. The i686 C
oracle explicitly rounds its arithmetic to binary32 to avoid x87 excess
precision during exact report comparisons. These checks use debug shared Rust
libraries; release/static libraries and 32-bit Python remain unverified.

### Dataset and initialization interfaces

Nine exports provide complete paired dataset construction and host document
loading, sample count, independent observation and segmentation snapshots,
deep clone and release, initializer defaults and native initialization. Dataset
construction accepts repeated input owners and any number of samples, including
zero. All pointer elements are checked before conversion; output is published
only after the whole dataset is built. Samples retain native document order.
Native Dataset contains observations and numeric segmentations; retain the
separate document/state owners when their JSON metadata is needed later.

The initializer descriptor exposes flat start, global tying and variance floor
ratio. Integer flags are checked before native conversion. All sample values and
model parameters are processed by the unchanged native initializer, including
corpus-wide fallback durations, boundary capping and binary32 flat-start rounding.
Models, datasets and failed output slots remain unchanged.

Focused Windows x86_64 Rust, optimized C and Python checks pass four original C
model byte comparisons. Rust also verifies the original ten-frame model, multiple
host samples, the corrected corpus fallback, full observation/segmentation fields,
independent snapshot lifetimes, repeated inputs, empty datasets, late invalid
samples and native/configuration errors. The complete five-target matrix passes
41 suites and 74 tests per target with no failed or ignored tests. All five C
caller families pass all five targets; all five Python caller families pass the
three 64-bit targets. Feature isolation and quality gates pass. The initializer
and document loader cover the computation and host loading of `shiro-init`;
the existing native CLI retains argument, model-file and stdout handling.
Training, isolated groups, ordered per-file datasets and progress callbacks are
verified by their later checkpoints in this document.

### Alignment interface

Three exports provide native defaults, in-memory state alignment and full host
document alignment. The descriptor contains both mode flags, all five HSMM
settings and both geometric settings. Its `repr(C)` layout follows the target
C ABI, including padding and pointer-sized `duration_extra`. Mode and isolated
flags use validated `u32` values rather than caller-controlled Rust enum or bool
representations. Invalid codes return 2. Numerical validation remains native.

The state operation returns an independent complete state owner. The document
operation reads the original UTF-8 document, delegates to the native file loader
and returns complete JSON bytes. Document, file and state metadata is preserved;
filenames keep their native process-relative meaning. Inputs remain unchanged and
failed calls retain output slots.

Focused Windows x86_64 Rust, optimized C and Python tests pass all nine original
C inference cases, including embedded/isolated, HMM/HSMM and pruning variants.
Rust additionally compares nondefault values for every configuration field with
independently constructed native settings and exercises nested state metadata.
Host document tests read real feature files and compare complete JSON metadata.
Pointer, invalid flag, native numerical failure, malformed document and missing
file checks retain outputs. The complete five-target matrix passes 40 suites and
71 tests per target, with no failed or ignored tests. All four C caller families
pass all five targets; all four Python caller families pass the three 64-bit
targets. Feature isolation and quality gates pass. These operations cover the
computation and host loading of `shiro-align`; its existing native CLI retains
argument, model-file and stdout handling. Training is verified by its later
checkpoint; utterance workflows and all WASM interfaces remain required.

### Observation and state interfaces

Ten additional exports introduce independent observation and state-sequence
owners. Rawfloat observation loading accepts either explicit stream dimensions
or all dimensions from a complete model. It uses the native frame budget and
deinterleaving rules. Serialization retains every observation field in the
original MessagePack format. State JSON loading, writing and cloning preserve
optional fields, jumps, `ext` metadata and flattened additional attributes.
State-to-segmentation conversion uses the native truncating boundaries and
mixed binary64/binary32 transition arithmetic. Its serialization preserves the
original array-header mismatches required by C readers.

Focused Windows x86_64 Rust, optimized C and Python callers pass. Independent
C/Python oracles compare all deinterleaved binary32 values and the complete
legacy segmentation wire, including ordinary-edge residual probabilities.
Rust tests also compare the original multistream feature input with the native
loader, optional JSON fields, nested metadata, malformed input, frame budgets
and retained outputs after invalid state conversion. The full five-target matrix
passes 39 suites and 68 tests per target with no failed or ignored tests. All
three C caller families pass all five targets; all three Python caller families
pass the three 64-bit targets. Feature isolation and quality gates pass. These
interfaces do not complete alignment or training.
Direct stream callbacks and batch workflows are verified by their later
checkpoints; utterance workflows and all WASM interfaces remain required.

The foundation passes the full native Rust suite and new ABI tests on
x86_64/i686 Windows MSVC, x86_64 Windows GNU and x86_64/i686 Linux GNU:
37 suites and 62 tests per target, with no failed or ignored tests. Optimized,
assert-enabled C callers exercise all 13 exports on all five targets. Python
ctypes callers exercise them on the three 64-bit targets. Tests cover exact
binary32 wire bytes, independent lifetimes, ranges, empty inputs and unchanged
output slots after errors. Feature-disabled Python checks confirm that the
optional exports are absent. Formatting, Clippy and Rustdoc with warnings denied,
generated-header verification and native-source preservation checks pass.

These checks use debug shared Rust libraries. Release libraries, static linking
and Python on 32-bit targets have not been verified by this checkpoint.

## Typed model definitions and report collections

The definition owner retains every native duration count, stream field and
optional integer bound. Its typed constructor and getters preserve target-width
integers and every binary32 weight bit independently of model-build validation.
JSON uses the original schema and defaults; writing nonfinite weights returns a
native-input error rather than replacing them with JSON null. Build delegates to
the existing native definition implementation.

Individual reports can be constructed from complete iteration scalars and ordered
likelihood row owners. Ordered report collections preserve empty rows, repeated
inputs and every floating-point bit. Getters and clones return independent owners.
These fifteen exports extend the header to 183 functions, 25 opaque owners and
21 descriptors with ABI version 1.

Focused Windows x86_64 Rust tests, optimized assert-enabled C and actual Python
ctypes callers pass. All three original definition fixtures retain native defaults;
two generated models match original C fixture bytes exactly. Tests also cover
arbitrary public-field values, optional-bound presence, nested report rows,
independent parent release and retained outputs on errors. The complete five-target
matrix passes 55 suites and 113 tests per target, including all fourteen original
tool tests, with no failed or ignored tests. All eighteen optimized assert-enabled
C caller families pass all five targets; all eighteen Python caller families pass
the three 64-bit targets. Both feature variants pass formatting, Clippy and Rustdoc
with warnings denied; the feature-disabled library exposes none of the 183 C
functions. Generated-header and native-source preservation checks pass.
Full utterance workflows and all three WASM interfaces remain required.

## Typed states and segmentation documents

Typed state construction and getters retain binary64 time bits, optional
target-width duration, optional output rows and jumps, metadata and attributes.
Absent values remain distinct from present empty rows. JSON fields are decoded
independently so attributes with reserved names do not overwrite typed fields.
File owners retain full UTF-8 filenames, ordered states and attributes; document
owners retain ordered files and attributes. Snapshots and clones are independent.

The nineteen new functions extend the generated header to 202 exports. Original
state/document JSON remains available; writers reject nonfinite times instead
of silently emitting null. Typed access preserves those times. Focused Windows
x86_64 tests compare four original C document fixtures with full typed
reconstruction and every native field. Arbitrary-field tests cover nonfinite
times, full-width output values, reserved attributes, Unicode/NUL filenames,
repeated files and independent lifetimes. All seven focused new/existing Rust
tests, Clippy, Rustdoc and actual Python calls of all nineteen exports pass.
All nineteen optimized assert-enabled C caller families pass all five targets;
all nineteen Python caller families pass the three 64-bit targets. The complete
matrix passes 57 suites and 117 tests per target with no failed or ignored tests,
including all fourteen original tool tests. Both feature variants pass formatting,
Clippy and Rustdoc with warnings denied, generated-header verification and the
feature-disabled library's absence of all 202 functions. Source preservation
checks confirm unchanged native algorithms, dependencies, fixtures and all old
183 C declarations; the optional array field visibility and nonfinite state JSON
writer guard are the only changes to existing optional implementation modules.
Full utterance workflows and all WASM interfaces remain required.

## Complete utterance result and workflow interfaces

The utterance owner retains all ten native fields: phone map, typed model
definition, phone sequence, initial segmentation, optional uninitialized and
initialized models, final model, complete iteration reports, alignment and
labels. Construction preserves arbitrary public fields without rebuilding
artifacts. Independent getters, model-stage presence checks, clones and release
retain optional models and complete nested metadata. The segmented-wave owner
retains audio, features and utterance results with the same independent ownership.

The two workflow entrypoints delegate to native feature segmentation and decoded
wave processing. Inputs retain all five options, fresh/initialized/trained model
sources, full UTF-8 filenames, every decoded wave header field and all three
feature kinds. Uniform callbacks follow the existing audio contract; validation
of output storage precedes native processing or random draws. Numerical workflow
validation remains in the native implementation.

Five focused Windows x86_64 Rust tests call all twenty-one new functions and pass
with Clippy warnings denied. They compare all result fields and every nested
report likelihood with native processing, match original C stage-model fixtures,
exercise all model sources and feature kinds, zero iterations and nondefault
options, decoded-header variations, exact random draw consumption, errors and
independent lifetimes. Arbitrary construction tests preserve nonfinite typed
definition/time/report/label values, reserved attributes, full-width integers and
all four optional-stage combinations without normalization. The header contains
223 functions, 29 opaque owners and 27 descriptors. Actual Windows x86_64 Python
calls of all twenty-one functions pass, including original C stage-model bytes,
complete finite result/report snapshots and reconstruction, all model sources,
nondefault options, modified decoded headers, all feature kinds, callbacks and
independent lifetimes. Python arbitrary-field construction also preserves all
ten fields with nonfinite definition/time/report/label bits, nested and reserved
attributes, full-width integers, repeated/empty report rows, empty/NUL strings
and all four optional-model combinations after releasing the source owners.
Arbitrary segmented-wave construction retains rate zero and an inconsistent
feature matrix without imposing native workflow validation. An optimized,
assert-enabled Windows x86_64 C caller also exercises all twenty-one functions,
original C model/document/label fixtures, complete finite result reconstruction,
all feature kinds, all model sources, nondefault options and decoded headers.
It checks arbitrary nonfinite values, nested attributes, repeated and empty rows,
all optional model combinations and complete waveform results after releasing
source owners. All five native targets pass 58 suites and 122 tests, including
all fourteen original tool tests. Twenty optimized assert-enabled C caller
families pass on all five targets; twenty Python caller families pass on all
three 64-bit targets. Formatting, Clippy and Rustdoc with warnings denied pass
with and without the C interface. All 223 symbols are absent without `c-api`,
and the generated header and original source-preservation audits pass. The
native and tool inventories now record all 35 native and 14 tool C mappings.
All three WASM interfaces and final combined
end-to-end acceptance remain required.

## Primary references

- [Native byte readers](https://doc.rust-lang.org/stable/std/io/trait.Read.html)
- [Native partial writers](https://doc.rust-lang.org/stable/std/io/trait.Write.html)
- [Native buffered readers](https://doc.rust-lang.org/stable/std/io/trait.BufRead.html)
- [Windows owned OS strings](https://doc.rust-lang.org/stable/std/os/windows/ffi/trait.OsStringExt.html)
- [Windows borrowed OS strings](https://doc.rust-lang.org/stable/std/os/windows/ffi/trait.OsStrExt.html)
- [Unix owned OS strings](https://doc.rust-lang.org/stable/std/os/unix/ffi/trait.OsStringExt.html)
- [Unix borrowed OS strings](https://doc.rust-lang.org/stable/std/os/unix/ffi/trait.OsStrExt.html)

- [Cargo library target types](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field)
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Rust ABI](https://doc.rust-lang.org/reference/abi.html)
- [Rust C representation and target layout](https://doc.rust-lang.org/reference/type-layout.html#the-c-representation)
- [Rust lint attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-attributes)
- [cbindgen 0.29.4 documentation](https://github.com/mozilla/cbindgen/blob/v0.29.4/docs.md)

## Likelihood CSV bytes

`shiro_rs_training_result_likelihood_csv_bytes` returns a new independently
owned ShiroRsBytes object containing the same six-decimal likelihood CSV written
by shiro-rest. File/group order and empty rows are preserved. A failed call
retains the output slot; encoding does not modify the training result. The
native method supports arbitrary Write implementations, with interruption,
partial writes and failures covered by tests/likelihood_csv.rs.
`shiro_rs_training_result_write_likelihood_csv_stream` exposes the same writer
through the existing synchronous ShiroRsWriteStream descriptor. It retries
interrupted writes, retains emitted bytes on failure, rejects excessive callback
counts and missing writers, and neither flushes nor retains the callback/context.
An empty report set performs no callback. Actual Windows x86_64 C/Python callers
exercise both codecs on all nine training cases, with 1/13/16384-byte chunks,
injected failure after seven bytes, zero-byte writers and invalid descriptors;
current-source execution on the other required platforms remains pending.
