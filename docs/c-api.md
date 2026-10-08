# C interface

Build the optional interface with `cargo build --features c-api`. The package
produces Rust, shared C and static C libraries from the same library target.
The generated declarations are in `include/shiro_rs.h`; ABI version is 1.

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
- [Rust lint attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-attributes)
- [cbindgen 0.29.4 documentation](https://github.com/mozilla/cbindgen/blob/v0.29.4/docs.md)
