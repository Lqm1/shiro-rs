# Model untying compatibility

`untying::untie` creates a duration and one emission distribution per stream
for every state occurrence, traversing files and states in their original
order. The new zero-based global index replaces every duration/emission
reference. Typed assignments retain the file/local-state correspondence.
All distributions own independent storage. Times, jumps, metadata, filenames
and unknown JSON attributes remain unchanged. Observation files are not read.

`shiro-untie` retains `-m`, `-s`, optional `-o` segmentation output, optional
`-O` summary output and `-h`. The model is binary stdout. The shared reader
also accepts `-m -` for a piped binary model. Existing input/output path
aliases and duplicate optional output paths are rejected after canonical
resolution. All formats are validated before creating optional files; an
I/O failure can still leave an earlier optional file written.

The summary contains global-state, file and local-state indices, followed
by the optional phoneme and local phoneme-state index from ext metadata.
Integer conversion retains truncation of finite fractional metadata indices.
Rust writes portable LF lines, matching the Linux C reference; Windows C
text-mode files translate LF to CRLF. JSON comparisons are semantic, since
indentation, object order and numeric spelling may differ. These text-format
differences do not change fields or summary tokens.

## Original C evidence

The reference uses SHIRO 203ef7b71bf382c8b5ce3f86b8116f63265e2711,
its shiro-untie.c, cli-common.h and cJSON.c, plus the pinned liblrhsmm
binary32 Debug archive. Only external/liblrhsmm include prefixes are changed
to locate the staged dependency. The tool is compiled with gcc -O2, without
-Ofast. Its pre-existing readall ignored-fread warning is retained.

The input is the existing initialized two-stream model and two three-state
phones used for alignment tests. The output has six durations, six emission
states per stream, 521 serialized bytes and six summary rows. Rust matches
all model bytes, complete parsed JSON and summary bytes exactly. It rereads
the C model using the Rust codec. `tests/generate_untying_reference.py`
reproduces the fixtures with `--shiro`, `--include` and `--archive`; `--check`
compares without writing. The tracked C helper creates a weighted input
using original model read/write functions, without changing the tool.

## Corrections and independent checks

The original tool allocates new streams with lrh_create_empty_stream, whose
default weight is one, and never copies the original weights. A C-generated
input with weights 0.25 and 1.75 produces the same original output bytes as
the unit-weight input, independently reproducing the reset. Rust retains
both weights. Resetting only the corrected weights to one reproduces every
original C output byte. Before/after untying emission tables are equal with
the retained weights. Tests also modify one cloned distribution and verify
that other occurrences and the source remain unchanged.

Additional cases cover two-file ordering, unknown attributes, jumps,
missing metadata, empty corpora, invalid duration/output references, stream
count mismatch, malformed summary metadata and propagated writer errors.
Summary rows are validated before writing any bytes. Empty metadata emits
the three bare indices instead of dereferencing missing C JSON nodes.
CLI tests cover both optional outputs, model stdin, help and input alias
rejection. No binding or full real-audio/training acceptance is claimed.

The file-output APIs follow official Rust documentation:
https://doc.rust-lang.org/stable/std/io/trait.Write.html and
https://doc.rust-lang.org/stable/std/fs/fn.canonicalize.html.

## Native validation

All 41 current integration tests pass on Windows MSVC x86_64/i686,
Windows GNU x86_64 and Linux GNU x86_64/i686, including optional host
extractor tests. The full runs use `--include-ignored --nocapture`.
The reference generator's `--check` run reproduced all four fixtures
and the original stream-weight reset. Formatting, Clippy with warnings
denied and whitespace checks pass. These checks cover the native untying
checkpoint; they do not prove completion of the full port.

SHIRO re-estimation and wavsplit orchestration, remaining dependency
functions, public precision/build coverage and all bindings remain pending.
