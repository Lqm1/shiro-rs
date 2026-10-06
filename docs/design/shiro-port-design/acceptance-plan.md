# Acceptance plan

Status: accepted; implementation in progress. Partial execution evidence is recorded in progress reports; the full acceptance gates are not satisfied.

## Baseline and traceability

Pin upstream commit IDs for all three projects. Inventory each public function and command, its Rust counterpart, its verification evidence, and any intentional bug fix. Include standalone dependency functions that SHIRO never calls. Do not claim complete implementation while inventory entries remain unimplemented or unverified.

Use the original C implementation as a numerical reference, with compiler flags and floating-point configuration recorded. If running the C baseline on Linux is more practical, use that baseline and record the cross-platform comparison limits when validating the native Windows Rust implementation. Retain unmodified upstream behavior as evidence for identifying intentional bug corrections.

## Native acceptance gate

The first intermediate milestone runs on `x86_64-pc-windows-msvc`. Map the required Windows/Linux 32-bit and 64-bit gate to the five x86 Tier 1 targets: `x86_64-pc-windows-msvc`, `x86_64-pc-windows-gnu`, `i686-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, and `i686-unknown-linux-gnu`. The inclusion of both Tier 1 Windows x86_64 toolchain variants is explicit in the final confirmation. A compile-only check does not satisfy an executable workflow requirement.

1. Build all three library packages and all SHIRO executable targets; run their unit, integration, and documentation tests and formatting/lint checks.
2. Complete the public-functionality inventory for ciglet, liblrhsmm, and SHIRO, including the original Lua tool functionality.
3. Read the bundled pretrained `.hsmm` model and compare its decoded structure and numerical fields against the original reader.
4. Save Rust-generated models and read them with the original C reader. Compare decoded content, not only success exit codes. Validate C-to-Rust and Rust-to-C interoperability for model definitions, phone maps, segmentation, labels, and feature data where applicable.
5. Process a reproducible real speech sample through waveform loading, preprocessing, feature extraction, existing-model inference, and label output. Compare intermediate features and final boundaries against the C implementation.
6. Process a reproducible small training dataset through initialization, HMM bootstrap training, HSMM re-estimation, alignment, saving, reloading, and inference. Check trained parameters and likelihoods against the baseline using justified tolerances.
7. Cover alternate training and inference behavior, including isolated training, state tying/untying, skip transitions, pruning, DAEM, and serial/parallel execution. Include deterministic synthetic cases where real-data coverage alone does not exercise a behavior.
8. Verify the optional external-Lua extractor mode with a representative custom extractor; bundled Rust workflows must run without Lua.
9. Verify malformed-file and invalid-parameter handling. Record intentional upstream bug corrections and keep regression cases for them.
10. Report target verification separately from implementation coverage. Deferred Tier 1 targets do not become verified merely because they are intended to be supported.

Use a small redistributable real-data fixture with known provenance and sufficient coverage to reproduce the acceptance run. Choose data based on technical adequacy and redistribution terms; do not assume a corpus or model's license solely from the surrounding code repository.

## Numerical comparison policy

Measure absolute and relative errors for each algorithm family and document the chosen thresholds and input coverage. Compare topology, state identifiers, frame counts, and discrete decisions directly. Investigate ties and numerical perturbations that change decisions. Do not raise tolerances merely to make unexplained failures pass.

Validate dependency replacements against the same evidence as handwritten Rust code. Adopt numerical crates only if they preserve algorithm conventions, transform scaling, boundary behavior, serialization precision, and the agreed inference and training behavior.

Performance improvements are encouraged after compatibility checks. Record representative comparisons when making performance claims; no arbitrary speedup target has been accepted.

## Binding acceptance gate

Only start binding implementation after the native gate passes on the required targets.

- Provide C ABI entry points inside each of the three existing packages. Define explicit ownership, release operations, error reporting, and panic containment at the ABI boundary.
- Demonstrate native use from a C caller and Python. Test model loading, inference, and a small training run through the SHIRO ABI, and representative computational operations through the dependency ABIs.
- Provide browser and Node.js WebAssembly bindings inside each of the three existing packages. Receive audio, feature, model, and structured inputs through bytes, arrays, or in-memory objects.
- Exercise model loading, inference, and a small training run through the WebAssembly SHIRO API, plus representative dependency operations. Provide reproducible invocation examples.
- Preserve format interoperability across native Rust, C ABI callers, and WebAssembly callers.
- Keep command-line invocation, operating-system filesystem operations, and external Lua execution in the native layer. Additional WASI support is outside the agreed scope.
- Do not claim that every public operation is bound based only on representative smoke tests. Track exposed computational functionality separately and test coverage explicitly.

## Deferred platform validation

After the first intermediate Windows milestone, complete the required Windows/Linux x86 32-bit and 64-bit execution checks before bindings, including both Tier 1 Windows x86_64 toolchain variants in the concrete plan. Defer Tier 1 ARM64 execution checks when local execution is impractical. Distinguish compile checks from executable tests in the verification matrix.
