# Workspace split verification

This report covers the split into nine packages across three independent
repositories, their public GitHub publication and the cross-repository dependency
migration. Earlier acceptance reports retain their actual source and execution scope.

## Structure and compatibility

Each repository has three packages initialized with `cargo new --lib` under
`crates/`: the existing native name, a `-capi` adapter and a `-wasm` adapter.
The root is a virtual workspace with `members = ["crates/*"]`, resolver 3 and
the native package as default member. All nine packages retain GPL-3.0-or-later,
Edition 2024 and Rust 1.94. The fourteen SHIRO binary names are preserved.

Computations and model codecs stay in native Rust. C and WASM adapters depend
directly on their own native package and do not call one another.
Native packages forbid unsafe code and have no binding-specific dependencies.
Original calculation sources, reference fixtures and attribution are preserved;
normalized source relocation audits found no unexplained calculation differences.

Ten checked helpers were exposed for adapters without changing their calculations:
three ciglet vector/resampling helpers, liblrhsmm's jump-group validator, and
SHIRO's resolved alignment, batch preparation/output paths, resolved dataset
loaders and fallible progress-callback trainer. Existing `train_with_progress`
remains available; `try_train_with_progress` adds the fallible callback entry point.

All three cbindgen 0.29.4 headers are byte-identical after regeneration.
C symbols and ABI layouts remain unchanged. C artifact base names now end in
`_rs_capi`; callers must select those DLL/static/import-library filenames.
JavaScript keeps the previous `*_rs.js` names through `wasm-bindgen --out-name`.
Liblrhsmm serialization defaults and explicit opt-out behavior are preserved.

## Migration execution matrix

Before the final Git dependency switch, the split working trees passed native
workspace suites with `--locked --workspace -- --include-ignored` on
Windows MSVC64, Windows MSVC32, Windows GNU64 and Linux GNU64.
Each target passed 463 ciglet, 259 liblrhsmm and 130 SHIRO tests, with zero
failures and zero ignored tests. The liblrhsmm native/C packages also passed
202 tests per target with serialization disabled.
Linux GNU32 passed ciglet and liblrhsmm, including liblrhsmm serialization opt-out;
SHIRO's Linux GNU32 refresh was still running at this checkpoint.

Independent optimized C callers, with assertions enabled, passed on all five
x86 targets: 7 ciglet, 33 liblrhsmm and 20 SHIRO programs per target.
The corresponding Python callers passed on Windows MSVC64, Windows GNU64 and
Linux GNU64. Python 32-bit was not tested. Real Gnuplot and the existing
Lua/external-extractor protocol test tools were enabled where required.

Native and wasm32 adapter Clippy passed with warnings denied for all projects.
Rustdoc passed with warnings denied; formatting and whitespace checks passed.
Generated Node.js modules passed all 40 ciglet families, all 18 liblrhsmm
families with and without serialization, and all 16 SHIRO families in two
passes. SHIRO's workflow verifier exercised all fourteen tool computations.

Three independent C DLLs composed a real-speech pipeline with three recordings,
74,268 feature values, the historical 600-state model, both training modes and
four iterations. Three independent WASM modules passed the same pipeline in
Node.js. Maximum feature errors were approximately 4.35e-5 for C/Python and
4.25e-5 for Node; these retain the previously accepted measured tolerance scope.

The new web-target glue is byte-identical to previously browser-verified glue.
Fresh browser automation connections timed out during this migration. New WASM
computations were executed through Node.js; no fresh browser run is claimed.
Other deferred Tier 1 targets retain the existing acceptance reports' limits.

## Public dependency validation

The public dependency repositories use main:

- https://github.com/Lqm1/ciglet-rs at `34208b5fb1757355c638a1c588ec8a7fa49fddc3`.
- https://github.com/Lqm1/liblrhsmm-rs at `d1b32853ccc2381d04af1a1cbd0587a455e992f3`.

An isolated copy of the split SHIRO source resolved both packages from these
GitHub URLs and immutable revisions. Cargo metadata identified Git checkout
sources, not sibling path packages. Every previously locked package version
was retained. The three bootstrap/real-speech tests passed against those Git
sources, including initialization, HMM/HSMM training, save/reload and inference.
`cargo check --locked --workspace --all-targets`, the wasm32 adapter build and
all sixteen Node verification families also passed with the Git dependencies.

The original SHIRO checkout keeps provisional sibling paths until its running
Linux32 migration suite finishes. Final publication and fresh-clone verification
are recorded after applying the validated manifest and lockfile.

## Evidence scope

Scratch logs and JSON reports live in the migration task workspace under
`work/split-*` and `work/git-dependency-preview-*`. The split matrix ran on the
working tree before its publication commit. Old HEAD values in runner logs
are baseline identifiers, not a claim that those old commits contain the split.
Separate pre-existing acceptance reports cover original C/Lua oracle generation,
full native API/configuration coverage and the earlier actual-browser gate.
