# Latest native and binding execution checkpoint

This checkpoint records executable evidence for the five agreed x86 targets.
It supplements the original native gate, which passed before bindings, and
the separate per-function, field, configuration and numerical audits. The final
requirement audit is recorded in `requirements-verification.md`; counts alone
do not establish it.

## Source and invocation

The completed full native refresh used ciglet
`b3fe894b2dcc149453957b24fdca630fe80681ec`, liblrhsmm
`25c1dc14212429a8eeecbf65e6eba398b7564d1d` and SHIRO
`1fb77db7f5097442120251b43a8cb06c7a7ab0bf`. Each runner verified clean Git
state and the same source commit before and after its package execution.

```text
cargo test --locked --target TARGET -- --include-ignored --nocapture
cargo test -p shiro-rs-capi --locked  --target TARGET -- --include-ignored --nocapture
```

Liblrhsmm additionally ran:

```text
cargo test -p shiro-rs-capi --locked --no-default-features  --target TARGET -- --include-ignored --nocapture
```

All tests use the Debug profile. Host rendering uses actual Gnuplot. Required
Lua/SPTK protocol fixtures exercise process arguments, streams and failures;
actual SPTK math is checked separately in `sptk-host-verification.md`. Real
speech, original C numerical corpora, historical model loading, initialization,
HMM/HSMM learning and save/reload/inference run in the full suites.

## Full native and Rust-driven C ABI execution

The following counts apply independently to every row in the target table.
All 35 configurations completed with zero failed and zero ignored tests.
Documentation tests are included, including intentional compile-fail cases.

| Package | Native/default | C API/default | C API/no serialization |
| --- | ---: | ---: | ---: |
| ciglet-rs | 198 | 463 | Not applicable |
| liblrhsmm-rs | 151 | 259 | 202 |
| shiro-rs | 62 | 129 | Not applicable |

| Executed target | Native and C API/default | Liblrhsmm no serialization |
| --- | --- | --- |
| x86_64-pc-windows-msvc | Pass, all three packages | Pass |
| i686-pc-windows-msvc | Pass, all three packages | Pass |
| x86_64-pc-windows-gnu | Pass, all three packages | Pass |
| x86_64-unknown-linux-gnu | Pass, all three packages | Pass |
| i686-unknown-linux-gnu | Pass, all three packages | Pass |

## Actual external callers

Every existing `tests/c_api*smoke.c` is compiled with optimization and enabled
assertions, then executed against freshly built default-feature C ABI shared
libraries. Liblrhsmm callers define `LIBLRHSMM_RS_SERIALIZATION`, so codec and
training-codec branches actually run. Required SHIRO Lua/SPTK protocol tools
and actual Gnuplot rendering are configured explicitly.

| Package | C programs per target | Python programs per 64-bit target |
| --- | ---: | ---: |
| ciglet-rs | 7 | 7 |
| liblrhsmm-rs | 33 | 33 |
| shiro-rs | 20 | 20 |

All five targets pass their C programs. Windows MSVC x86_64, Windows GNU
x86_64 and Linux GNU x86_64 pass every Python program. Python 32-bit, static
linking and a full Release-profile matrix are not claimed by this checkpoint.
The GNU liblrhsmm run additionally recompiles and passes the eight
serialization-dependent programs after its initial runner omitted that define.

The initial Linux caller build used a shared artifact directory containing a
dependency build without C ABI exports and failed at linking. That result was
excluded. The corrected runner builds each independent package in its own
target directory and verifies the shared library's actual C ABI exports before
compiling callers: ciglet 646, liblrhsmm 791, SHIRO 229. No calculation or ABI
implementation was changed to address the artifact-selection failure. Cargo's
[build-cache documentation](https://doc.rust-lang.org/cargo/reference/build-cache.html)
describes `CARGO_TARGET_DIR` and `--target-dir` output isolation.

## Composed bindings and subsequent changes

`cross-crate-verification.md` records three independently instantiated WASM
modules in actual Node.js and browser execution, and three independently loaded
C ABI libraries in Python on all three 64-bit targets. It includes real speech,
original features/states, full model/data transport, both learning modes and
reload/inference. Independent C learned-parameter reference tests remain in
their per-family suites rather than being replaced by self-roundtrips.

Subsequent tracked additions publish those already executed portable
verification scripts and refresh stale documentation/configuration ledgers.
Liblrhsmm's C builder resize test module moved to the end of its source file
to satisfy Clippy. The complete production code and moved tests are identical
after newline normalization; both precision-specific tests pass afterward.
No algorithms, exported ABI declarations, dependency manifests, locks or
reference fixtures changed after the full matrix. All three packages pass
native and WASM Clippy with warnings denied and formatting checks. The updated
liblrhsmm configuration test explicitly checks the standard JavaScript PI
mapping; all eighteen families pass twice in both serialization configurations
in actual Node.js and browser runs.

Deferred Tier 1 ARM64 execution remains unverified. This does not remove those
targets from the intended platform scope. The final full-task requirement audit
passes within the agreed execution scope. The subsequent sequential
initialization/HMM-bootstrap/HSMM-refinement regression also passes on all
five targets; see `training-compatibility.md`. This adds one native integration
test to the recorded 62-test SHIRO checkpoint without changing production code.
