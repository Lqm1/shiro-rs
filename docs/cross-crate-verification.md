# Cross-crate speech verification

The three independent packages transport copied arrays and complete serialized
objects between their bindings. A Rust, C ABI or WebAssembly owner belongs to
the library that created it. These checks never pass an owner pointer to a
different library.

This composed workflow supplements each package's complete public-function
inventory, configuration audits and per-family numerical reference tests. It
does not replace them or reduce the scope to functionality used by SHIRO.

## Node.js

Generate each package's Node.js module using the commands in its `wasm-api.md`.
The generated directories must contain `ciglet_rs.js`, `liblrhsmm_rs.js` and
`shiro_rs.js`, respectively. Liblrhsmm needs its default serialization feature.
From the SHIRO repository, run:

```text
node tests/three-crate-speech-node.cjs CIGLET_NODE LIBLRHSMM_NODE SHIRO_NODE tests/fixtures node-result.json
```

All paths are command arguments. No sibling checkout location is hard-coded.
The runner exits unsuccessfully on a failed check and writes JSON only after
the complete workflow succeeds.

## Browser

Generate each package's browser module with wasm-bindgen's `web` target. Serve
those directories and the existing SHIRO fixture directory locally:

```text
node tests/serve-three-crate-speech.cjs CIGLET_WEB LIBLRHSMM_WEB SHIRO_WEB
```

An optional fourth argument selects a different fixture directory. The default
is `tests/fixtures` beside the server script. The server binds only to
`127.0.0.1` and prints its ephemeral port. Open that local address and click
**Run real-speech verification**. A successful complete run displays `PASS`
and the JSON report. Stop the server after verification.

The browser and Node runners use the same `three-crate-speech-checks.mjs`.
They instantiate three independent WebAssembly modules. Browser threads are
not claimed by this check. CLI invocation, OS file operations and external
Lua/SPTK execution retain their native implementation under ADR 0007.

## C ABI through Python

Build all three packages with their `c-api` feature. Keep liblrhsmm's default
serialization feature. Select shared libraries matching the Python process's
architecture, then run:

```text
python tests/three-crate-speech-python.py --ciglet CIGLET_LIBRARY --liblrhsmm LIBLRHSMM_LIBRARY --shiro SHIRO_LIBRARY --fixtures tests/fixtures --output python-result.json
```

Library paths may be `.dll` or `.so` files. Assertions are required; the runner
rejects Python's `-O` mode. It writes JSON after all checks pass. Each standalone
optimized C smoke program remains a separate acceptance check, including
SHIRO's nine original-C training/model/likelihood cases. Python composition
does not stand in for an actual compiled C caller.

Build each package in its own output directory. An explicitly shared target
directory can also contain dependency builds with different features; verify
that the selected shared library exports its C ABI. For example, run
`cargo build -p shiro-rs-capi --locked  --lib --target-dir target/c-api` in
each independent checkout. Cargo documents output isolation through
[`--target-dir` and `CARGO_TARGET_DIR`](https://doc.rust-lang.org/cargo/reference/build-cache.html).

## Input and checked behavior

The licensed CMU US SLT ARCTIC fixtures and historical model are described in
`real-audio-compatibility.md` and `tests/fixtures/README.md`.

Both composed verifiers exercise all three recordings, independent ciglet WAV
decoding, copied audio samples, SHIRO preprocessing and MFCC/delta/acceleration,
historical model loading, complete legacy byte roundtrip, complete feature and
state output, standalone liblrhsmm model/observation/segmentation decoding,
corpus initialization, two HMM and two HSMM updates, ordered likelihood reports
and CSV, saved model transport, reload and inference for every recording.

The WebAssembly verifier also recomputes complete HMM and HSMM inference with
the independently instantiated liblrhsmm module and compares binary segmentation
output to SHIRO. The Python verifier uses liblrhsmm for complete codecs and
model validation; it does not perform that extra independent inference check.
All successful paths explicitly release their owned objects.

Features compare all 74,268 values against the original C corpus using the
existing normalized error bound of `5e-5`. Historical inference compares all
600 states to original C output. Learned model transport and reload checks are
interoperability checks. Independent learned-parameter and likelihood oracles
remain the existing native and binding training-family reference corpora.

The Python workflow uses original C HMM states as initialization input. The
WebAssembly workflow builds flat phone-map initial states. Their learned
likelihoods therefore need not agree. Each runner checks its own complete
reports and reload results; it does not compare those two different setups.

## Recorded runtime results

The verified implementation revisions are ciglet
`b3fe894b2dcc149453957b24fdca630fe80681ec`, liblrhsmm
`25c1dc14212429a8eeecbf65e6eba398b7564d1d` and SHIRO
`1fb77db7f5097442120251b43a8cb06c7a7ab0bf`. The portable runners change path
selection only, plus the Python assertion-mode guard.

Actual Node.js and browser runs of the portable runners both pass with three
independent modules, three files, 74,268 feature values, maximum feature error
`0.000042498111724853516`, 600 historical states and four training iterations.
Their mean log likelihoods are
`[-9259.3095703125, -6943.681640625, -9154.5771484375, -7888.49072265625]`.

Actual Python calls to the three Windows MSVC x86_64 and GNU x86_64 libraries
pass with three independent libraries, maximum feature error
`0.000043489038944244385`, 600 original-C HSMM states and four iterations.
Both yield mean log likelihoods
`[-9693.423828125, -6390.32275390625, -9154.578125, -7888.44384765625]`.
The portable Python runner was rerun successfully against MSVC x86_64.
Actual Linux GNU x86_64 Python composition also passes with the same metrics
and likelihoods. All five latest-source native and external C caller target
gates pass. The completed full requirement decision is recorded in
`requirements-verification.md`. Python 32-bit execution is not claimed.
