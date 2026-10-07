# Alignment compatibility

`alignment::align_states` accepts an owned model's borrowed parameters, a
multistream observation and JSON states. `align_document` reads the stored
process-relative feature paths and prepares emission/duration distributions
once for the corpus. Both return new results without modifying caller inputs.
The CLI retains `-m`, `-s`, `-g`, `-p`, `-P`, `-d`, `-i` and `-h`; `-m -`
uses the shared binary-stdin model reader. Output is segmentation JSON.

Explicit durations use boundary-pruned emissions and HSMM Viterbi. Geometric
durations use linear-pruned emissions and HMM Viterbi. Defaults remain radius
5, slope 0.3 and duration extra 30. The Rust radius accepts fractional values,
matching the library's floating-point setting, whereas the old CLI uses atoi.
Library options also expose temperature and duration weighting explicitly.

The embedded HMM retains original state order and jump attributes. HSMM and
isolated output select the inferred occurrences and omit jump attributes,
matching json_from_seg_shuffle. Duration/emission IDs and all ext metadata
are copied from each selected source state. Unknown state/file/document
attributes are additionally retained instead of silently discarded. This
does not change the legacy fields read by the C importer.

## Original C reference

The pinned SHIRO source is 203ef7b71bf382c8b5ce3f86b8116f63265e2711.
Reference compilation uses its shiro-align.c, cli-common.h and cJSON.c,
with only the missing external/liblrhsmm include prefix changed to
liblrhsmm. It links the pinned liblrhsmm Debug binary32 archive and uses
gcc -O2 for the tool. No -Ofast comparison is claimed.

The model and observation are the existing 288-byte init-c-aligned.hsmm
and 144-byte init-input.bin fixtures. Embedded states use the original
three-state sequence with two extra jumps and extended metadata. Isolated
states comprise two three-state phones over the same twelve frames.
The four cases per input are HSMM defaults, HMM slope 0.8, HSMM radius 2
and duration extra 8, and HMM slope 0.5. An additional four-state fixture
tests the default HMM slope. All nine cases compare complete JSON state
values, including boundaries, source identities and retained attributes.
Tests also load emitted states through the shared segmentation importer.

`tests/generate_alignment_reference.py` regenerates all nine output fixtures
or checks them without writing when `--check` is set. Run it with Python 3
on a C build host, passing `--shiro` for the pinned SHIRO checkout,
`--include` for the parent of the pinned liblrhsmm headers, and `--archive`
for its binary32 static archive. It stages only the two include-prefix
changes, compiles the original tool and runs the checked input fixtures.

## Corrections

- The C grouping helper splits only on a strictly decreasing local state
  index. Rust also splits on a phoneme-name change or an equal index reset,
  so consecutive one-state phones receive their own frame intervals.
- The C isolated transition filter uses the jump-list ordinal instead of
  the source-state index and writes accepted entries into uncompressed
  slots. Rust checks the actual destination, excludes cross-phone edges
  and compacts the retained JSON transitions before probability import.
  Mixed binary64/binary32 residual arithmetic is still preserved.
- Isolated C copying can read past the observation when JSON time exceeds
  its frame count. Rust caps end boundaries before slicing and rejects
  nonpositive intervals. Invalid metadata, empty states/observations and
  invalid dimensions return errors.
- For three states, slope 0.3 becomes an integer radius of zero. C emits
  fabricated HMM boundaries despite no surviving path. Rust retains the
  library's explicit no-path error. The valid original-C HMM references
  use wider slopes; the separate four-state reference covers the default.
- Misspelled and incomplete input diagnostics are replaced with errors
  from the validated model, JSON and rawfloat readers. No partial corpus
  output is written if an alignment fails.

Safe slices and fallible indexing follow the official Rust slice API:
https://doc.rust-lang.org/stable/std/primitive.slice.html.

## Native validation

All 37 current integration tests, including optional Lua/SPTK host tests,
pass on Windows MSVC x86_64/i686, Windows GNU x86_64 and Linux GNU
x86_64/i686. All five full runs use `--include-ignored --nocapture`.
The reference generator's `--check` run reproduced all nine C outputs.
Formatting, Clippy with warnings denied and whitespace checks pass.
An earlier default-target development build emitted incremental-cache
access-denied warnings; the five final target-specific test logs contain
no compiler warnings. No cache repair is claimed.

Full real-audio/pretrained-model acceptance, training/untie/wavsplit tools,
remaining dependency functions and all bindings are still incomplete.
