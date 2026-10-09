# Requirements verification

The current package organization is documented in [workspace layout](workspace-layout.md).
The split and public Git dependency checks are recorded separately in
[workspace split verification](workspace-split-verification.md). Earlier execution
checkpoints retain the source revisions and package organization they actually tested.

This audit applies the accepted requirements and acceptance plan without
reducing the scope to functionality already used by SHIRO. It distinguishes
implementation mappings, executable evidence and deferred platform validation.
The accepted native and binding gates are verified. The added sequential
HMM-bootstrap/HSMM regression passes on all five agreed targets. Deferred
platform execution remains explicitly unverified as authorized.

## Project, implementation and repository requirements

| Requirement | Current-state evidence and decision |
| --- | --- |
| Three independent repositories under the requested project directory | Each of shiro-rs, ciglet-rs and liblrhsmm-rs has its own Git repository, Cargo.toml and Cargo.lock under D:\Desktop\Projects. Cargo metadata reports three flat workspace packages per repository: native, C ABI and WASM. Each package has one library target. The native SHIRO package additionally has fourteen executable targets. Pass. |
| Cargo-generated initial packages | The accepted requirements document records the executed cargo new --lib --edition 2024 --vcs git initialization after authorization. Subsequent source files implement the generated packages. Pass. |
| Modern Rust reimplementation rather than mechanical C translation or delegation | The calculation modules use checked owned storage, borrowed preparation, Result errors, generic binary32/binary64 numerical engines, Rust readers/writers and explicit settings. Cargo manifests have no C build dependency; original C files are reference/caller tests and generated ABI declarations. Core implementation uses Rust, not original C delegates. Pass. |
| Official Rust primary references | Requirements, ownership/configuration audits, and algorithm/API compatibility documents link the official Rust/Cargo/standard-library references used for their implementations. Binding docs additionally link primary wasm-bindgen and cbindgen documentation. Context7 and Mintlify retrieval are recorded in the accepted design and execution history. Pass. |
| Compatible models and existing inputs/outputs | Native and binding codecs preserve historical/current model layouts and all legacy data/set schemas, including original wire quirks and binary32 storage. Original C/Lua reference artifacts compare complete structures, bytes and discrete outputs. Detailed numerical scope remains in per-family reports. Pass. |
| Complete public scope, including standalone dependency functionality and Lua tools | The ciglet own-header and installed fastapprox source inventories, liblrhsmm source inventory and all fourteen SHIRO tools have implemented mappings. Function counts are supplemented by header macros, all public fields/configuration, ownership/view/cache/build/precision audits and executable families. See the exact coverage audits below. Pass for implementation mapping; native/binding execution is assessed separately. |
| Fix bugs and identifier spelling without changing wire names incompatibly | Per-family compatibility documents disclose corrections and retain regression/original-C evidence. Rust identifiers use the documented semantic names; serialized legacy fields and schema quirks remain compatible. No new wire format replaces hsmm. Pass. |
| Established dependencies used only with compatibility validation | Manifests select rustfft, rand, rmp, serde, clap and binding tools; tests retain transform scaling, source arithmetic, RNG conventions, codecs and option behavior. No new dependency was added for this final audit. Performance claims require their separate measurements. Pass. |
| GPL inheritance and attribution | All package metadata declares GPL-3.0-or-later and each repository contains GPL-3.0 LICENSE. Dependency upstream notices and adapted fastapprox/other notices are retained in NOTICE/LICENSES; SHIRO attribution is in README. CMU speech permission and unchanged fixture/model provenance are documented separately. Public publication was not requested or performed. Pass. |
| English tracked content, conventional branches and commits | Current source/docs/tests and repository metadata are English. Unicode test data exercise encoding through escaped inputs. Each current branch is feat/rust-port; latest commits use conventional English prefixes. No codex/ branch was introduced. Pass. |
| Target Rust Tier 1 with agreed staged validation | Five agreed Windows/Linux x86 targets have actual native and C execution, including both Windows x86_64 toolchains. Tier 1 ARM64 execution is explicitly deferred as authorized. Intended support is separate from execution evidence. Pass for the agreed execution gate; deferred platforms are not claimed verified. |

## Source coverage audits

The following audits were inspected alongside the native modules and binding
implementations, rather than treating inventory status strings as proof:

- Ciglet: `docs/public-configuration-audit.md`,
  `public-field-binding-audit.md`, `native-value-field-audit.md`,
  `filterbank-configuration-audit.md`, bundled approximation and numerical
  compatibility documents. The own-header checker matches all five structures
  and twenty fields to pinned upstream source; the native checker matches twenty
  named-field types and sixty-eight fields to every public native module.
  Those sets overlap and are not added as separate original capabilities.
- Liblrhsmm: `docs/header-coverage.md`, `configuration-coverage.md`,
  `configuration-binding-audit.md`, `ownership-coverage.md` and
  `public-api-audit.md`. These cover all nineteen original configuration items,
  selected precision, complete model/data/statistic fields, builders, prepared
  caches, borrowed views, posterior storage, memory pools and all eight codec
  families. Source settings convert to native settings and are used by the
  corresponding computational verifiers.
- SHIRO: `docs/native-requirements-audit.md`, `wasm-tool-audit.md`,
  the fourteen-tool and thirty-seven-native-function binding inventories,
  complete C/Python caller tests and all individual WASM verifier families.
  The connected tool workflow checks option conversion and output artifacts
  across all fourteen computations; the real-speech composition supplements it.

Counts prove the integrity of the inspected mapping sets, not general numerical
correctness. Every numerical claim retains the actual corpus, measured bound,
compiler/precision conditions and intended corrections in its own document.

## Numbered native acceptance items

| Item | Evidence and decision |
| --- | --- |
| 1. Libraries, all tools, tests and quality checks | The completed latest five-target matrix includes all packages, fourteen CLI targets, unit/integration/doctests and required ignored host tests. All three packages pass warning-denied native/WASM Clippy, Rustdoc and formatting. The new sequential test separately passes all five targets without changing production code. Pass. |
| 2. Complete function/tool inventory | The source coverage audits above include standalone dependency operations, installed helpers, macros, configuration and ownership. All fourteen C/Lua tools are implemented. Pass. |
| 3. Existing model read/parameter comparison | Historical 89,257-byte model and its SHA256 provenance are retained; original reader/parameter oracles and complete historical wire roundtrip are exercised in native, C/Python and WASM families. Pass. |
| 4. Both-direction C format interoperability | All eight liblrhsmm codec schemas and both precisions have original C read/write oracles. Rust-generated training models compare byte-identically to C outputs; definitions, phone maps, states, labels and rawfloat data have independent C/Lua artifacts. Pass. |
| 5. Real speech through labels | Three licensed CMU SLT recordings cover decode/preprocessing/features, historical HMM/HSMM inference and complete label spans. All 74,268 feature values retain the measured 5e-5 normalized bound; all 600 historical states compare discretely. Native and composed binding execution pass. |
| 6. Initialization, HMM bootstrap, HSMM re-estimation, save/reload/inference and C numerical comparison | Existing synthetic initializations, nine training cases and real-speech workflows pass. The final audit additionally adds bootstrap_training.rs, which passes the HMM-trained model into HSMM refinement and compares every saved stage to C reference bytes plus likelihoods at the unchanged 1e-5 bound. The full sequential regression executes successfully on all five agreed targets. Pass. |
| 7. Alternate modes | Independent cases cover isolated/embedded inference and training, tied initialization/untying, weak/skip/backward transitions, fractional pruning, DAEM, duration modes and ordered serial/parallel reduction. Native and binding families pass within their documented corpus limits. |
| 8. Optional Lua extractor; ordinary workflows without Lua | Required host tests execute custom Lua and SPTK protocol adapters. Actual SPTK 3.9 output is byte-identical across all five Rust caller targets. Native MFCC/PLP presets compute without Lua. Pass. |
| 9. Malformed input and bug regressions | Codec/resource/shape/nonfinite/parameter, ownership, interrupted/partial I/O, callback and process-failure cases run in the full suites. Compatibility documents disclose C corrections and retain their references. Pass. |
| 10. Distinguish implementation from target execution | latest-execution-checkpoint.md records each actual target, Debug scope, Python architecture, host-tool architecture and deferred ARM64. No static/Release/Python32/ARM64 runtime claim is inferred. Pass. |

## Binding acceptance

All three interfaces live inside their existing single library crates. C ABI
owners have explicit release operations, output/error contracts and panic
containment. Native safe modules confine unsafe code to feature-gated boundaries.
Actual optimized assert-enabled C callers and Python callers execute each
package's computational families, with SHIRO model loading, inference and
learning compared to C reference models and likelihoods. No requirement for an
additional fourth binding crate or simultaneous three-library C executable was
introduced by the accepted plan.

Browser and Node interfaces receive arrays, bytes and owned structured inputs.
Ciglet's forty families, liblrhsmm's eighteen families in both serialization
configurations, SHIRO's individual/connected tool families and independent
three-module real-speech composition have actual Node/browser evidence.
The updated liblrhsmm configuration aggregate also runs twice in reverse order
on default and no-serialization Node/browser modules. Export symmetry and
feature-disabled checks supplement the computational references; they do not
replace them.

Format interoperability uses the native codecs and original bytes across
Rust, C ABI and WASM. Copied data transport between independent libraries is
verified explicitly; owner pointers never cross independent modules. Native
OS file/process/CLI/Lua operations remain available in Rust/C. The accepted
browser contract leaves those operations native under ADR 0007. Three ciglet
process lifecycle mappings are therefore native-only; no portable computation
is removed. WASI is outside the accepted scope.

Binding acceptance has recorded execution evidence. All accepted requirements
are verified within the agreed execution scope. Tier 1 ARM64 runtime validation
remains deferred; public crate publication, additional WASI support, Python
32-bit execution and a complete static/Release matrix were not acceptance
requirements and are not claimed.
