# Observation loading and model initialization

`shiro-init` retains original `-m` model, `-s` segmentation, `-v` relative
emission variance floor, `-F` uniform-duration start and `-T` global emission
tying. The default variance-floor ratio is 0.1. The command writes binary
`.hsmm` to stdout and diagnostics to stderr. `-m -` reads a binary model from
stdin, preserving the original common reader's pipe mode. `initialization::initialize`
returns a new model without modifying the caller's model or dataset.

The shared `dataset` module connects SHIRO interchange files to the Rust
liblrhsmm API. Rawfloat frames contain every stream's feature vector in order;
the reader splits them into independent time-major streams using the model's
first emission state, as in C. Initialization checks the dimensions of
selected states. Global tying can replace other states with different widths;
the importer does not reject those states before they are replaced.
Feature paths are resolved as written, relative to the process working
directory, matching C rather than rebasing them to the JSON file's directory.
The parser checks complete binary32 scalars, complete frames, checked widths
and caller frame budgets. It preserves sample bit patterns and does not
silently repair nonfinite features; initialization rejects them. Frame-budget
upper bounds are capped by addressable binary32 storage so the default i32
frame limit does not overflow on a 32-bit host and reject small inputs.

Training JSON can omit `time` and `ext`, as in the original C importer. Missing
time initializes to zero. Duration/output IDs are required, must match model
stream counts and must reference existing states. Finite nonnegative frame
boundaries convert with C's truncation rather than label-generation ceiling.
Supplied initialization boundaries are capped at the observation frame count.

Extra transition probabilities convert separately to binary32. The implicit
forward residual preserves C's mixed arithmetic: subtract the original JSON
binary64 probability from the current binary32 residual, then round back to
binary32 after every edge. Extra delta-one entries are omitted before adding
the single implicit forward edge, fixing uninitialized C array entries.
A tiny negative residual caused by binary32 rounding is clamped to zero when
the binary64 input probabilities do not exceed one. Invalid probabilities
and actual excess mass are errors.
Signed and zero relative jump offsets are retained, as accepted by the
original importer and liblrhsmm transition builder.

Flat start computes duration as binary32 frames / states, then floors the
binary64 product of index and that rounded duration, as in C. Ten frames and
three states therefore end at boundary nine. This numeric behavior is retained
for existing initialization results. Global tying directs all observation
statistics to each stream's state zero, then deep-copies the updated GMM to
other states. Duration statistics still use their original state IDs.
Initialization keeps liblrhsmm's normal-duration update, global duration
variance floor 0.05 and emission variance floor 0.01. Afterwards every emission
variance floor becomes its variance times `-v`. Zero-mean duration states get
the corpus average duration and its square as fallback mean and variance.

## Corrected source defects

The C file-size expression is `fsize % stride * 4`, so divisibility by the
complete byte width is not checked. The original command accepts a three-byte
input for a three-scalar model, returns success and writes a 288-byte model
after silently using zero frames. Rust rejects that input before model output.

The C importer also uses a fixed 64-element stack dimension array. Rust's
reader has a regression with 70 independent streams and no fixed array limit.
This test does not execute the unsafe C case.

Original initialization overwrites `total_num_states` for every file while
summing frames. Two copies of the twelve-frame/three-state fixture therefore
give unused duration state three mean/variance 8/64. Rust sums all state counts
and gives 4/16. The test compares every remaining model parameter to the
original two-file model, with no other difference.

Explicit forward entries in C allocate a slot but leave it uninitialized.
The Rust importer omits them, preserving the intended ordinary transition.
This finding comes from source inspection; the invalid memory contents are
not used as a numerical reference.

## Original-source fixtures

SHIRO is pinned to `203ef7b71bf382c8b5ce3f86b8116f63265e2711`, liblrhsmm
to `1df92da4b77377f4725509e7240a9107ed4c063b`. The C CLI and cli-common
header are staged with only `external/liblrhsmm/` include prefixes changed
to `liblrhsmm/`. The command links the original float Debug static library
and bundled cJSON; the CLI uses `-O2`, `FP_TYPE=float` and no fast-math.
The original ignored-fread warnings remain; a warning-free C build is not claimed.

`init-definition.json` has four duration states and two output streams of
dimensions two and one. The first three states are observed and the fourth
tests fallback initialization. `init-input.bin` stores twelve frames:
time minus five, time modulo four minus 1.5, and time times 0.25.
`init-segmentation.json` contains boundaries 3, 7, 12 and two extra jumps.
It intentionally omits optional metadata.

```text
shiro-mkhsmm-c -c definition.json > empty.hsmm
shiro-init-c -m empty.hsmm -s seg.json > aligned.hsmm
shiro-init-c -m empty.hsmm -s seg.json -F > flat.hsmm
shiro-init-c -m empty.hsmm -s seg.json -T > tied.hsmm
shiro-init-c -m empty.hsmm -s seg.json -FT -v 0.4 > flat-tied.hsmm
```

All four 288-byte model fixtures compare exactly to Rust, including CLI
binary output and reload. The fifth flat-start fixture uses the first ten
frames and compares exactly. The sixth contains two copies of the original
file and verifies the intentional fallback correction above.

`tests/dataset_oracle.c` invokes the original header's model, observation
and segmentation readers. Compile it with the staged header, original cJSON
and pinned liblrhsmm archive, and run beside the generated empty model,
input.f and seg.json. It reports twelve frames, two streams, boundaries
3/7/12 and transition bit patterns 3ca3d70a, 3e19999a and 3f547ae2 for
offsets 2/3/1. These are actual original-C observations, not duplicated math.
With offsets 0/-1 and probabilities 0.1/0.2, the same C reader reports offsets
0/-1/1 and bit patterns 3dcccccd, 3e4ccccd and 3f333333; Rust compares them
exactly too.

These synthetic comparisons do not establish real-audio training, inference,
remaining SHIRO tools or bindings. Primary Rust references include
[Read](https://doc.rust-lang.org/std/io/trait.Read.html),
[Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html), and
[f32](https://doc.rust-lang.org/std/primitive.f32.html).

All 33 integration tests passed after the 32-bit budget and signed-jump fixes
on Windows MSVC x86_64/i686, Windows GNU x86_64 and Linux GNU x86_64/i686,
including optional Lua/SPTK host tests. After adding stdin model input,
all four initialization tests passed again on every target. Formatting and
Clippy with warnings denied pass. Two Windows MSVC builds emitted an
incremental-cache finalization access-denied warning; their builds and tests
still exited successfully. Four SHIRO tools and the full acceptance gate
remain pending, along with remaining dependency functionality and bindings.
