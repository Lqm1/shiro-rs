# SHIRO Rust reimplementation

Status: design accepted; implementation and the agreed native/binding gates verified. The user authorized project generation and implementation. The final decision and deferred execution scope are recorded in ../../requirements-verification.md. Historical environment observations below retain their original scope.

## Established requirements

- Create the projects under `D:\Desktop\Projects`.
- Use the package names `shiro-rs`, `ciglet-rs`, and `liblrhsmm-rs`.
- Reimplement the original algorithms in idiomatic Rust. Do not mechanically transpile C or replace the existing model architecture with an incompatible design.
- Preserve compatibility with existing models, including `.hsmm` files, and existing inputs and outputs.
- Improve names and correct spelling errors in Rust identifiers. Changes to serialized field names and other compatibility-sensitive identifiers require separate treatment.
- Permit efficiency improvements when the agreed compatibility contract remains satisfied.
- Use official Rust documentation as primary implementation references.
- Generate initial Cargo packages using `cargo new`; do not hand-create package scaffolding.
- Use English for every Git-tracked file, branch name, and commit message. Do not use a `codex/` branch prefix.
- Preserve upstream licensing and attribution. SHIRO source headers specify GPL version 3 or any later version.
- Verify native Rust model loading, inference, and test training end to end before implementing C ABI and WebAssembly bindings for all three packages.

## Verified SHIRO baseline

- Repository: https://github.com/Sleepwalking/SHIRO
- Audited commit: `203ef7b71bf382c8b5ce3f86b8116f63265e2711`.
- SHIRO performs phoneme-to-speech forced alignment and model training.
- Seven C executables are declared in `makefile`: `shiro-mkhsmm`, `shiro-init`, `shiro-rest`, `shiro-align`, `shiro-untie`, `shiro-wav2raw`, and `shiro-xxcc`.
- Lua programs provide feature extraction orchestration, phone-map and model-definition generation, segmentation generation, label conversion, and waveform splitting.
- The repository contains bundled C and Lua utilities, including cJSON. The README's statement about two library dependencies does not mean these bundled utilities can be ignored when inventorying behavior and attribution.
- The standard C build specifies `FP_TYPE=float` and `-Ofast` and links OpenMP.
- `examples/cmu-arctic-all-speakers.hsmm` is a bundled pretrained model, 89,257 bytes at the audited commit.
- `cli-common.h` loads feature data as native binary 32-bit floats and reads models through `lrh_read_model`.
- `shiro-rest` supports HMM and HSMM re-estimation, isolated training, pruning, DAEM, and optional multithreading. These are distinct behaviors that need coverage in a full port.

## Accepted decisions, round 1

1. Create three independent Git repositories and Cargo packages at `D:\Desktop\Projects\shiro-rs`, `D:\Desktop\Projects\ciglet-rs`, and `D:\Desktop\Projects\liblrhsmm-rs`. Each package has one library target. The SHIRO package also contains executable targets corresponding to the original tools.
2. Port all public functionality of the three upstream projects, including the functionality of SHIRO's Lua tools. The dependency scope is not limited to functions called by SHIRO. External Lua extractor execution is settled in round 2.
3. Validate file-format compatibility and discrete inference results. Compare floating-point features, likelihoods, and trained parameters using justified tolerances derived from measurements against the original C implementation. Bitwise equality of every floating-point result is not required. Tie cases and numerical differences that alter discrete inference results require explicit investigation rather than automatic acceptance.

## Accepted decisions, round 2

The three-package split must not remove any original public functionality. Track every public upstream operation against its Rust implementation and verification evidence. This includes standalone DSP, signal generation, plotting integration, observation generation, model manipulation, and data serialization beyond the SHIRO execution path. Platform-specific host integrations remain part of the native scope even when they cannot run inside browser WebAssembly.

4. Reimplement bundled extractor functionality in Rust. Preserve arbitrary existing Lua extractors through an optional compatibility mode that invokes an externally installed Lua interpreter. Ordinary bundled workflows do not require Lua.
5. Actively use established Rust dependencies when they improve maintainability or efficiency without damaging compatibility or causing significant behavioral differences. This permission includes numerical dependencies when validated. Avoid unnecessary dependencies and low-adoption choices without a compelling justification. Reimplementation must not delegate the core functionality to the original C libraries.
6. Target Rust Tier 1 platforms, including targets without host tools, as defined by the official platform-support documentation. Prioritize Windows and Linux on x86 64-bit and 32-bit. Defer other Tier 1 verification when local testing would be disproportionately complex. The explicitly authorized temporary acceptance platform is this machine's primary target, `x86_64-pc-windows-msvc`. Record tested and deferred targets separately; Rust's Tier 1 guarantee is not evidence that these downstream packages have been tested.
7. Fix upstream defects. Preserve compatibility for valid legacy inputs, and document intentional behavioral corrections. Keep externally serialized names unchanged where changing a typo would break compatibility, while correcting Rust identifiers and presenting clearer Rust APIs.

## Platform-support snapshot

The user-specified reference is https://doc.rust-lang.org/beta/rustc/platform-support.html. At the time of this design audit, Tier 1 consists of `aarch64-apple-darwin`, `aarch64-pc-windows-msvc`, `aarch64-unknown-linux-gnu`, `i686-unknown-linux-gnu`, `x86_64-pc-windows-gnu`, `x86_64-pc-windows-msvc`, and `x86_64-unknown-linux-gnu`, plus `i686-pc-windows-msvc` without host tools. Recheck this moving reference when updating the supported toolchain. Beta documentation does not require adopting a beta compiler.

The local compiler host is `x86_64-pc-windows-msvc`. All targets in the audited Tier 1 snapshot already have Rust target libraries installed locally. Installation alone does not establish linker availability or execution support. WSL lists an Ubuntu distribution, which may provide a practical Linux baseline environment after its tools are checked.

## Subsequent decisions

The design decisions have been resolved and the user has authorized implementation.

## Accepted decisions, round 3

8. Use primary Windows x86_64 verification as an intermediate milestone only. Before implementing bindings, pass the native acceptance workflow on Windows and Linux at both 64-bit and 32-bit. Map this requirement to the five x86 Tier 1 targets: `x86_64-pc-windows-msvc`, `x86_64-pc-windows-gnu`, `i686-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, and `i686-unknown-linux-gnu`. Including both Tier 1 Windows x86_64 toolchain variants is the concrete mapping recorded in the final design confirmation. Required workflows include complete public functionality coverage, comparison with original C, existing-model loading, C reading Rust-saved models, real-audio feature extraction and inference, and small-dataset initialization, HMM/HSMM training, saving, reloading, and inference.
9. Implement browser and Node.js WebAssembly bindings for computational functionality, model processing, and training in all three packages. Pass inputs through bytes, arrays, or in-memory objects; keep CLI invocation, OS filesystem operations, and external Lua execution native. Implement C ABI bindings in all three packages and verify their use with C and Python callers. Additional WASI support is outside the agreed scope.

## Primary Rust references

- Cargo package generation: https://doc.rust-lang.org/cargo/commands/cargo-new.html
- Packages and crate targets: https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html
- Cargo workspaces: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
- Target platforms: https://doc.rust-lang.org/beta/rustc/platform-support.html
- Tier guarantees: https://doc.rust-lang.org/rustc/target-tier-policy.html
- Minimum supported Rust version: https://doc.rust-lang.org/cargo/reference/rust-version.html

Context7 and the Mintlify documentation index were queried before selecting these official references. Cargo's documented editions currently include 2024; the calendar year 2026 is not an edition identifier.

## Local environment

The target parent directory exists. Cargo and rustc are installed and report version 1.94.0. WSL Ubuntu has `gcc` and `make` and reports `x86_64`; the inspected PATH did not report Lua or LuaJIT. These checks do not establish that the C baseline or Rust packages compile. The three repositories were generated using cargo new --lib --edition 2024 --vcs git after user authorization.
