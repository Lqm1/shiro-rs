# Corpus re-estimation API

`training::train` and `train_with_progress` re-estimate an immutable source
model. Each input Dataset represents one original file and contains either
one embedded observation/segmentation pair or its ordered isolated groups.
The result owns the updated model and per-iteration reports. Reports expose
temperature, corpus mean and one row of group likelihoods per file.

This checkpoint implements the training calculation API. The shiro-rest
CLI, JSON-to-isolated-group loading, likelihood-file export and wavsplit
orchestration are still pending. It is not the full native acceptance gate.

## Preserved calculation rules

Defaults are one iteration, ordinary HSMM duration update, pruning radius
5, HMM slope 0.3, duration extra 30, threshold 1 and one worker. The selected
inference options remain explicit values rather than process-global state.
DAEM uses sqrt(binary32(iteration + 1) / binary32(total iterations)), with
the square root evaluated in binary64 and stored as binary32, as in C.
Without DAEM, temperature is one. Both inference option temperatures are
set by this schedule for every iteration.

Each group's optional mean-frame likelihood divides by that group's frame
count. The file likelihood adds each group value divided by the number of
groups. The corpus mean adds file values, divides by file count, then by
temperature, retaining binary32 operation order. Updating occurs before
checking convergence. From the second iteration onward, positive threshold
stops when current mean is below previous mean plus threshold. Nonpositive
threshold disables this check. Zero iterations return the unchanged model.
Existing boundaries are capped at observation frame count on cloned
segmentations; observation buffers remain borrowed without copying.

## Parallel statistics

Scoped threads borrow the model and prepared samples. Each file has its own
accumulator; results are reduced in original file order. C's estimate.c
protects statistics collection with an OpenMP critical region, but SHIRO's
total_lh addition has no reduction or protection. Rust's ordered reduction
removes that likelihood race and makes statistic collection order stable.
Every started worker is explicitly joined,
including when another worker fails. Spawn failures and worker panics are
reported as errors. The model update validates accumulator shape and finite
values, including reduction overflow, before replacing parameters.

Repeated parallel runs are bit-identical on the test corpus. Compared with
sequential accumulation over four repeated files and two iterations, the
all five native runs have maximum normalized parameter difference 7.027076e-7;
iteration likelihoods are equal. The scoped bound is 2e-6. This difference
comes from grouping binary32 additions by file instead of adding every
contribution directly to a shared sequential accumulator. It is not a
general tolerance claim for real corpora or other model sizes.

The thread lifetimes and explicit join handling follow official Rust docs:
https://doc.rust-lang.org/stable/std/thread/fn.scope.html and
https://doc.rust-lang.org/stable/std/thread/struct.ScopedJoinHandle.html.

## Corrected C references

The pinned SHIRO shiro-rest.c and cli-common.h have only the dependency
include prefix changed. The pinned liblrhsmm inference.c is staged with
two previously disclosed corrections:

1. Backward destination bounds use state count and exclude negative indices.
2. The duration posterior's initial predecessor applies only at time/state zero.

No other inference recurrence is changed. The staged translation unit is
linked before the unchanged binary32 Debug archive, replacing its original
inference object. The tool and staged inference use gcc -O2, without -Ofast.
Other archive objects remain unsanitized. These are corrected-C references,
not claims of equivalence to the buggy unmodified implementation.

The corpus has twelve frames, two streams and two three-state groups.
One/two HSMM iterations, three DAEM iterations, two HMM iterations and two
isolated iterations produce byte-identical 288-byte models. Mean-frame
reporting produces the same model as ordinary one-iteration training.
Likelihood rows are compared with C's six-decimal output, using an absolute
1e-5 bound, or 1e-6 for the mean-frame case. Convergence, zero iterations,
source/input immutability, boundary capping, malformed files/settings,
repeatable parallel reduction and worker-error propagation have independent
regressions. The fixture likelihoods can decrease, so no monotonic-likelihood
or real-audio convergence guarantee is inferred from these synthetic cases.

`tests/generate_training_reference.py` reproduces all six C command cases.
Pass `--shiro`, `--include` and `--archive` for the pinned checkouts/archive.
`--check` compares without writing. It verifies exactly one occurrence of
each source correction before patching. The existing ignored-fread C
warnings are retained. Separate original-tool commands verify that default
convergence stops after two updates and zero iterations preserve model bytes.

## Native validation

All 46 current integration tests pass on Windows MSVC x86_64/i686,
Windows GNU x86_64 and Linux GNU x86_64/i686, including optional host
extractor tests. The final full runs use `--include-ignored --nocapture`.
Formatting, Clippy with warnings denied and whitespace checks pass.
The reference generator's `--check` run reproduces all six command cases
and both stopping checks. An earlier default-target development build
emitted an incremental-cache access-denied warning; final target-specific
test logs have no compiler warnings. No cache repair is claimed.

Complete SHIRO workflows, remaining dependency functions, public precision
and build coverage, real-audio/native acceptance and all bindings remain
in scope and unfinished.
