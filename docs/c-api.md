# C interface

Build the optional interface with `cargo build --features c-api`. The package
produces Rust, shared C and static C libraries from the same library target.
The generated declarations are in `include/shiro_rs.h`; ABI version is 1.
The current header declares 31 exports, five opaque owner types and one settings
descriptor.

## Current coverage

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
implemented. Rawfloat and
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
