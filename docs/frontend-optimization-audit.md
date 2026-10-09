# Original C frontend optimization audit

Baselines are SHIRO `203ef7b71bf382c8b5ce3f86b8116f63265e2711` and ciglet
`895ba9b1c0eabee83d3544208bbc82420efa3206`.
`tests/compare_frontend_optimization.py` stages only the ciglet include path in
the original feature/audio tools, then compiles unchanged algorithms at four
profiles. It records compiler identity, source hashes and effective macros.
All profiles disable floating-point contraction.

Profiles are strict `-O2`, `-Ofast`, explicit `-O3 -ffast-math`, and a diagnostic
`-O3 -ffast-math -fno-finite-math-only`. The last profile tests whether disabling
the finite-value assumption restores the source's nonfinite categories.
It is not a replacement Rust acceptance mode. Final reports are
`frontend-optimization-windows.json` and `frontend-optimization-linux.json`.

Both compilers complete eight original C builds. Clang 21.1.0 through Zig on
Windows leaves fast-math inactive for `-Ofast` alone; the explicit profile is
verified active. GCC 13.3.0 on Linux enables it for both profiles. The
finite-preserving diagnostic records `__FINITE_MATH_ONLY__` zero. Original C
warnings remain visible, including unchecked fread and WAV integer-to-float
conversion. No Rust source, fixture or numerical threshold changed in this audit.

## Feature corpus

The existing oracle covers all 72 feature configurations, 6,399 output values,
MFCC/MFBE/PLPCC, energy/DC, delta/acceleration, integer/fractional hops and odd/even
frame lengths. It contains 42 nonfinite energy/derivative results. Input bytes,
configuration fields, frame counts and column counts are identical in all
profiles. The first diagnostic parser incorrectly required four frames in every
record; fractional hops produce three. That checker was corrected before the
final runs without changing the original driver or fixture.

Strict finite output values match fixture bits on both compilers. All 42
nonfinite categories also match. The checker retains the existing 2e-5 finite
normalized criterion and requires zero category differences before reporting
optimizer effects against the same compiler's strict result.

| Compiler | Explicit fast-math changed finite bits | Maximum normalized finite difference | Nonfinite category changes |
| --- | --- | --- | --- |
| Windows Clang | 3,292 | 1.8703285604715347e-6 | 6 |
| Linux GCC | 2,962 | 9.5367431640625e-7 | 6 |

Both have maximum absolute finite difference 1.9073486328125e-6. These are
measurements for this corpus, not a universal numerical guarantee. The
finite-preserving diagnostic restores every nonfinite category on both compilers
while retaining some finite rounding differences.

The six changed categories occur at records 18, 42 and 66, one per feature
kind. Each is silent input with decibel energy and delta enabled. At frames zero
and two, delta column 25 changes from NaN to negative/positive infinity.
The source dynamic kernel is `[-0.5, 0, 0.5]`; silence gives energy negative
infinity. Strict evaluation includes zero times infinity, producing NaN.
Permitting finite-only transformations removes that contribution. Disabling
finite-math-only restores the strict category, providing a separate compiler
control experiment in addition to inspection of the source expression.

Rust retains the strict source arithmetic and nonfinite categories tested in
`crates/shiro-rs/tests/features.rs`. It does not replace a silent derivative with compiler-specific
infinity to imitate a finite-only optimization. The existing fixture and category
regressions remain authoritative. The unchanged frame/column decisions are
verified separately from finite values and nonfinite categories.

## WAV conversion corpus

The unchanged 257-sample PCM16 fixture exercises plain output, normalization,
two-times upsampling, half-rate downsampling, normalized downsampling and dither.
Each profile launches fresh original commands, preserving the implicit random
seed. The first five strict outputs reproduce existing fixture criteria on both
compilers with zero measured differences. Linux dither also matches its fixture.
Windows dither is compared to the same compiler/runtime's strict command output;
this audit does not reuse the Linux random stream as a Windows baseline.
The separate C-runtime sequence tests already cover both original streams.

Plain, normalized and upsampled output are bit-identical under effective fast
math in both compilers. Downsampling differences are:

| Compiler | Changed samples / 129 | Maximum absolute difference | Normalized-down maximum absolute difference |
| --- | --- | --- | --- |
| Windows Clang | 77 | 4.470348358154297e-8 | 1.7881393432617188e-7 |
| Linux GCC | 68 | 2.9802322387695312e-8 | 1.1920928955078125e-7 |

Normalization reaches unit peak, so the maximum normalized and absolute errors
coincide for these samples. Windows dither changes one sample by
1.4901161193847656e-8; Linux dither is unchanged. No sample count or finite
classification changes. This corpus uses exact half/double ratios and does not
replace ciglet's separate fractional-rate conditioning audit or corrected kernel.

## Acceptance limits and reproduction

These diagnostics complete the previously pending optimized-C comparisons for
the recorded synthetic feature/audio corpora. They do not establish every
compiler's output for arbitrary nonfinite data or real-speech repeated learning.
Real speech and numerical learning evidence remain in
`real-audio-compatibility.md` and `training-compatibility.md`. Compiler-specific
optimization differences are documented without relaxing Rust regression bounds.
Final native acceptance still needs the current combined revision execution
matrix and requirement decision. Bindings have not started.

```text
python tests/compare_frontend_optimization.py --shiro /path/to/SHIRO --ciglet /path/to/ciglet --cc gcc --report /path/to/report.json
python tests/compare_frontend_optimization.py --shiro C:/path/to/SHIRO --ciglet C:/path/to/ciglet --cc zig cc --target x86_64-windows-gnu --report C:/path/to/report.json
```

The official GCC optimization options distinguish finite-math assumptions from
contraction control. The reference was consulted through Context7 and Mintlify
during the dependency optimizer audit:
https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html.
