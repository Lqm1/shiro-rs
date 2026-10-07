# Native acceptance checkpoint

Status: the combined execution matrix below has passed. Complete native
acceptance remains open because the installed bundled fastapprox interface
audit found additional unimplemented functions. Bindings have not started.

The current production implementations are ciglet `a16950d`, liblrhsmm
`40a6e17` and SHIRO `70799ff`. Liblrhsmm `1731f2e` subsequently adds only a
Python diagnostic, C optimization reports and documentation; it changes no
Rust source, manifest or reference fixture used by the running native suites.

## Combined execution evidence

Every package is tested with `cargo test --locked --target TARGET --
--include-ignored --nocapture`, using default features and the Debug profile.
The three repositories remain independent; SHIRO resolves its companion crates
through the existing path dependencies. Gnuplot is required through
`CIGLET_REQUIRE_GNUPLOT`, its executable is supplied through
`CIGLET_TEST_GNUPLOT`, and its directory is added to PATH so the default-name
process API is exercised too. Lua and SPTK test executables are supplied through
the existing SHIRO test environment variables.

| Target | ciglet | liblrhsmm | SHIRO |
| --- | --- | --- | --- |
| x86_64-pc-windows-msvc | 176 passed | 120 passed | 58 passed |
| i686-pc-windows-msvc | 176 passed | 120 passed | 58 passed |
| x86_64-pc-windows-gnu | 176 passed | 120 passed | 58 passed |
| x86_64-unknown-linux-gnu | 176 passed | 120 passed | 58 passed |
| i686-unknown-linux-gnu | 176 passed | 120 passed | 58 passed |

Ciglet's count comprises 173 integration tests and three LF preparation unit
tests. Liblrhsmm's count comprises 117 integration tests, one duration-bound
unit test and two compile-fail Rustdoc tests. SHIRO has 58 integration tests.
Counts are evidence of execution, not a substitute for functional coverage.
The current counts must not be combined with unrelated older revisions to
declare a final five-target result.

Initial runners supplied Gnuplot's explicit executable but omitted its PATH
directory. The required default-name invocation correctly failed on Windows
and Linux. Those original processes terminated before corrected runners were
started, and their failed logs were retained separately. The corrected Windows
runs pass the actual SVG rendering and default-name process test. This was a
runner configuration correction; no library code or assertion was changed.

## Workflow evidence and remaining audit

SHIRO's full suite exercises historical model loading/re-encoding, original-C
model and parameter comparisons, reproducible real-speech feature extraction
and alignment, initialization, HMM and HSMM learning, save/reload/inference,
isolated and embedded training, tying/untying, skip transitions, pruning, DAEM,
ordered parallel statistics and external Lua/SPTK extractors. Detailed input,
license and numerical evidence remains in the corresponding compatibility
documents and tests, including `real-audio-compatibility.md` and
`training-compatibility.md`.

The own-header inventories contain 246 ciglet rows, 103 liblrhsmm symbol rows
and 14 SHIRO tool rows. Ciglet's two macro formal-parameter rows are explicitly
not public symbols; the remaining own-header rows are marked implemented.
The installed `external/fastapprox-all.h` additionally defines 44 scalar and
44 four-lane functions. A separate bundled inventory now tracks them. These
functions were not covered by the original own-header count. All 44 scalar
functions now have Rust mappings; all 44 four-lane functions remain pending.
The scalar comparison also reproduced invalid negative/overflowing Lambert
exponential refinements. Rust now corrects those results while preserving
positive finite source bits, with independent high-precision comparison in
ciglet's `tests/lambert_exp.rs`. Pending rows
must be implemented before declaring complete native acceptance. The inventories
alone do not establish complete macro/configuration/structure coverage. Header
and ownership audits, numerical limitations and intentional source corrections
must be assessed alongside their test evidence.

The latest optimized-C comparison independently checks geometric/HSMM inference
and one expectation step at both precisions, with effective fast-math macros
recorded. All 14 geometric boundary and 18 HSMM path records per precision
remain unchanged in that corpus, while some scores/statistics change. This does
not prove arbitrary near-tie decisions or replace repeated-learning/real-audio
workflow evidence. See liblrhsmm's `docs/optimized-c-numerics.md` and reports.

The completed matrix applies to the production revisions named above, before
new bundled hyperbolic/sigmoid operations. Subsequent additions require their
own validation and a final combined revision check. All fifteen package logs
were checked for exact counts, absence of failures and zero ignored tests.

The remaining acceptance work includes implementing the pending installed
bundled-header functions, reviewing public-definition/numerical evidence against the original
accepted requirements, and making the final requirement-by-requirement decision.
C ABI and browser/Node WebAssembly bindings for all three packages have not
started. Deferred ARM64 Tier 1 execution remains separately unverified.
