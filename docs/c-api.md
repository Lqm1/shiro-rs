# C interface

Build the optional interface with `cargo build --features c-api`. The package
produces Rust, shared C and static C libraries from the same library target.
The generated declarations are in `include/shiro_rs.h`; ABI version is 1.
The current header declares 95 exports, thirteen opaque owner types, ten settings
and report descriptors, and one progress callback type.

## Current coverage

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
training with progress. Rawfloat and
observation byte operations are present; stream callbacks and standalone model
dimension retrieval remain pending. Other entries remain pending. Public types, fields, derived trait
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
generated-header verification and source-preservation checks pass. Generic
summary-stream callbacks and all WASM bindings remain required.

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
verification and source-preservation checks pass. Generic output-stream
callbacks and all WASM bindings remain required.

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

Nineteen exports provide complete ordered training inputs, all thirteen native
settings, training with or without synchronous progress, complete model/report
results, independent snapshots, deep clones and releases. Each input dataset
represents one original file and retains every embedded or isolated group.
The host loader delegates to `dataset::load_training_files`. Results retain
every iteration, temperature, mean likelihood and nested file/group likelihood.

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
Training, isolated groups, ordered per-file datasets and progress
callbacks remain required.

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
argument, model-file and stdout handling. Training and other pending workflows
remain required.

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
Generic stream callbacks and the remaining host workflows are still required.

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

## Primary references

- [Cargo library target types](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field)
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Rust ABI](https://doc.rust-lang.org/reference/abi.html)
- [Rust C representation and target layout](https://doc.rust-lang.org/reference/type-layout.html#the-c-representation)
- [Rust lint attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-attributes)
- [cbindgen 0.29.4 documentation](https://github.com/mozilla/cbindgen/blob/v0.29.4/docs.md)
