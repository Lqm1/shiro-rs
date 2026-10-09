# Label and segmentation conversion

`shiro-lab2seg` implements the original index-to-segmentation workflow with
`-m` phonemap, `-d` directory, `-t` hop in seconds, `-e` label suffix and
`-E` feature suffix. Defaults remain `.txt`, `.f`, current directory and
0.01 seconds. Feature files need not exist for this conversion, as in Lua.
The output retains `file_list`, `filename`, `states`, `time`, `dur`, `out`,
`jmp` and `ext`. Each label is divided evenly among its mapped states.
Frame boundaries use the source binary64 operation order and ceiling.
In the fixture, the first three boundaries are 3, 5 and 8; the last boundary
is not simplified to 7 because binary64 division lies slightly above it.
Empty mapped state lists produce no states, retaining source behavior.

`shiro-seg2lab` retains positional segmentation input, `-t` hop, `-e` output
suffix and `-s` state alignment. Output names remove only the final extension
and append the requested suffix. Both slash forms delimit directories, and
a leading dot counts as an extension, preserving Lua's filename rule even
when the document originated on another OS. Consecutive states belong to one phoneme
until the phoneme changes or its local state index stops increasing. This
keeps consecutive occurrences of the same phoneme separate. The original
`-s` output interleaves state rows and completed phoneme rows; Rust keeps
that behavior rather than interpreting the option as state rows only.

The `labels` module provides owned label, phonemap and segmentation types,
parsing, state conversion, grouping and writing. JSON types retain unknown
attributes and all additional `ext` entries, allowing later workflows to
carry metadata without dropping it. Label-only segmentation input can omit
`dur`, `out` and `jmp`, matching the original checkseg requirements.

Label text uses tab separators and CRLF endings. Rust writes shortest
round-trip decimal representations. Lua 5.4 uses fourteen significant
digits and sometimes appends `.0`; spelling can differ while numeric values
remain equivalent within the measured comparison bound. JSON formatting and
key order also differ; consumers must compare numeric/schema content rather
than whitespace. These tools do not change binary `.hsmm` serialization.

## Intentional corrections

Blank label rows no longer silently discard later rows. Repeated spaces in
space-separated labels are accepted. Tab-separated names retain spaces.
Missing fields, invalid numbers, nonfinite or reversed intervals report a
physical line number instead of failing later with a Lua arithmetic error.
Hops must be finite and positive. Generated frame/state indices must fit
the original signed 32-bit format. Segmentation boundaries must be finite
and nondecreasing. Empty segmentations write empty labels rather than
dereferencing a nonexistent final state.

Both commands validate their entire conversion before emitting output.
seg2lab validates every file before creating any label output, and rejects
an output path equal to its segmentation input.
The input check also resolves existing paths so relative aliases cannot
overwrite the segmentation document. Ordinary file I/O failure
can still leave files written earlier in a batch, as with the original.

## Reference fixtures

`labels-phones.json` has a three-state phoneme, a one-state phoneme, two
output streams and an empty-state phoneme. `labels-input.txt` repeats the
three-state phoneme, then changes phoneme. SHIRO commit
`203ef7b71bf382c8b5ce3f86b8116f63265e2711` is the reference.
Generate fixtures with unchanged `shiro-lab2seg.lua`, `shiro-seg2lab.lua`,
cli-common and bundled Lua modules, using Lua 5.4.9:

```text
lua shiro-lab2seg.lua index.csv -d directory -m labels-phones.json > seg.json
lua shiro-seg2lab.lua seg.json -e .phone.txt
lua shiro-seg2lab.lua seg.json -e .state.txt -s
```

The index contains `sample,aa aa bb`; copy `labels-input.txt` to sample.txt.
The generated JSON fixture changes only its machine-specific filename to
`sample.f` and pretty-prints the JSON. State boundaries, indices, jumps and
metadata remain unchanged. Label fixtures preserve the original bytes in
the working tree; tests parse either Git newline representation.

`crates/shiro-rs/tests/labels.rs` compares all seven generated states, three phoneme rows
and ten combined state/phoneme rows to the original tools. Numeric label
comparison uses absolute tolerance 1e-14 seconds to cover Lua decimal
formatting; state boundaries and indices compare exactly. Other checks
cover CLI options, compound suffixes, repeated phonemes, multiple streams,
empty states, metadata retention, malformed input and failure before output.
These synthetic labels do not establish complete alignment/training or
real-audio acceptance.

All 26 integration tests, including optional host extractor tests, passed on
Windows MSVC x86_64/i686, Windows GNU x86_64 and Linux GNU x86_64/i686.
After the final filename and canonical input-alias checks, all three label
tests passed again on each of those five targets. Formatting and Clippy
with warnings denied passed. Eight other SHIRO tools and all bindings remain
pending.

Primary Rust references used for the implementation:
[str](https://doc.rust-lang.org/std/primitive.str.html),
[FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html), and
[Write](https://doc.rust-lang.org/std/io/trait.Write.html).
