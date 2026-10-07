# Native acceptance checkpoint

Status: the native gate passes for the five agreed x86 targets and the recorded
compatibility corpora. All public-functionality mappings are implemented.
The subsequent C ABI and browser/Node WebAssembly gate remains required.

## Final native phase execution

The completed production snapshot is ciglet `4f8a2cd`, liblrhsmm `1731f2e`,
and SHIRO `45a3420`. All three runners terminated successfully. Every package
runs `cargo test --locked --target TARGET -- --include-ignored --nocapture`
with default features, the Debug profile and required host fixtures.

| Target | ciglet | liblrhsmm | SHIRO |
| --- | --- | --- | --- |
| x86_64-pc-windows-msvc | 195 passed | 120 passed | 58 passed |
| i686-pc-windows-msvc | 195 passed | 120 passed | 58 passed |
| x86_64-pc-windows-gnu | 195 passed | 120 passed | 58 passed |
| x86_64-unknown-linux-gnu | 195 passed | 120 passed | 58 passed |
| i686-unknown-linux-gnu | 195 passed | 120 passed | 58 passed |

All fifteen logs have exact expected counts, zero failures and zero ignored
tests. Ciglet has 192 integration tests plus three LF unit tests. Liblrhsmm
has 117 integration tests, one duration unit test and two compile-fail Rustdoc
tests. SHIRO has 58 integration tests and fourteen executable targets.
Gnuplot actual rendering/default-name launch, external Lua/SPTK subprocess
fixtures, real speech, model decoding/re-encoding and learning workflows run.
The conditioning correction separately passes five-target Debug/Release focused
suites. Both changed packages pass Clippy/Rustdoc with warnings denied and
formatting/whitespace checks.

The completed public/configuration/ownership mappings and requirement decision
are recorded in `native-requirements-audit.md`. Subsequent frontend diagnostic
files and documentation change no Rust source, manifest, original fixture or
regression bound used by this snapshot. `frontend-optimization-audit.md` records
the remaining synthetic frontend optimizer comparison, including the six silent
delta nonfinite categories restored by disabling finite-math-only.

The user's beta platform list was rechecked through Context7, Mintlify and the
official rustc book. These five targets are Tier 1. i686 Windows MSVC is now
listed without host tools, which does not change the executable target checks.
Deferred Tier 1 ARM64 execution is not claimed. Primary reference:
https://doc.rust-lang.org/beta/rustc/platform-support.html.

This completes the native intermediate phase. All computational/public
functionality remains in scope for the corresponding binding coverage.
The complete task remains open until C/Python and browser/Node verification
passes for bindings in all three existing packages.

## Completed bundled-function revision snapshot

Production revisions ciglet `a838175`, liblrhsmm `1731f2e` and SHIRO `85e5127`
pass the same full default-feature Debug invocation described below on every
required target. All fifteen package logs have zero failed and zero ignored
tests. Actual Gnuplot, default-name process launch, external Lua/SPTK extractors,
real speech, model interoperability and learning workflows are enabled.

| Target | ciglet | liblrhsmm | SHIRO |
| --- | --- | --- | --- |
| x86_64-pc-windows-msvc | 193 passed | 120 passed | 58 passed |
| i686-pc-windows-msvc | 193 passed | 120 passed | 58 passed |
| x86_64-pc-windows-gnu | 193 passed | 120 passed | 58 passed |
| x86_64-unknown-linux-gnu | 193 passed | 120 passed | 58 passed |
| i686-unknown-linux-gnu | 193 passed | 120 passed | 58 passed |

Ciglet now has 190 integration and three unit tests. The other package counts
retain their historical composition below. The liblrhsmm compile-fail Rustdoc
tests intentionally emit E0502 and pass by proving stale-cache mutation cannot
compile. Those diagnostics are not production compiler failures.

A later independent optimizer audit reproduces severe original Lanczos kernel
conditioning near zero. Ciglet now adds separately selectable Stable/Legacy
kernel behavior and SHIRO routes its default/legacy modes accordingly.
Those production changes postdate the completed matrix above. Their focused
five-target Debug/Release tests pass; the completed snapshot must not
be claimed as full execution evidence for the subsequent fix. Each configuration
has thirteen ciglet and nine SHIRO passes, with zero failed or ignored tests.
See ciglet's `docs/dsp-optimization-audit.md` for measurements and regressions.

The historical completed execution snapshot uses ciglet `a16950d`, liblrhsmm
`40a6e17` and SHIRO `70799ff`. Liblrhsmm `1731f2e` subsequently adds only a
Python diagnostic, C optimization reports and documentation; it changes no
Rust source, manifest or reference fixture used by the running native suites.

## Historical combined execution evidence

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
functions now have Rust mappings. Ten four-lane exponential/logarithm/power
functions and the conversion/index/splat/bit helpers now have safe Rust
mappings verified against the original SSE2 header. Eight four-lane
hyperbolic/sigmoid functions now also have source-order verified mappings.
Twelve four-lane trigonometric functions now retain their vector-specific
sign-order and cosine-equality behavior. Fourteen four-lane special functions
now also preserve audited source order and valid negative gamma/digamma domains,
with independent per-lane Lambert correction verification. All 88 bundled
function mappings are implemented. Rust array splats also correct the reproduced original signed
integer macro's accidental high-word sign extension.
The scalar comparison also reproduced invalid negative/overflowing Lambert
exponential refinements. Rust now corrects those results while preserving
positive finite source bits, with independent high-precision comparison in
ciglet's `tests/lambert_exp.rs`. All function rows now have implemented mappings.
The inventories
alone do not establish complete macro/configuration/structure coverage. Header
and ownership audits, numerical limitations and intentional source corrections
must be assessed alongside their test evidence.

The latest optimized-C comparison independently checks geometric/HSMM inference
and one expectation step at both precisions, with effective fast-math macros
recorded. All 14 geometric boundary and 18 HSMM path records per precision
remain unchanged in that corpus, while some scores/statistics change. This does
not prove arbitrary near-tie decisions or replace repeated-learning/real-audio
workflow evidence. See liblrhsmm's `docs/optimized-c-numerics.md` and reports.

The historical 176/120/58 matrix applies to its production revisions, before
new bundled hyperbolic/sigmoid operations. Subsequent additions require their
own validation and a final combined revision check. All fifteen package logs
were checked for exact counts, absence of failures and zero ignored tests.

The historical pending matrix and requirement review are resolved by the final
native phase execution and `native-requirements-audit.md` above. C ABI and
browser/Node WebAssembly bindings for all three packages remain the next phase.
Deferred ARM64 Tier 1 execution remains separately unverified.
