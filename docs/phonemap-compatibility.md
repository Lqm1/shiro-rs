# Phone maps, model definitions and initial segmentation

`shiro-mkpm` accepts the original phone-set path and `-s` states per phone,
`-S` stream count, `-t` default topology and `-w` weak skips. The defaults
remain three states and three streams. Duration and output state IDs advance
in input order, with the same ID in each stream. `durfloor` and `durceil`
attributes are divided among the states and rounded to milliseconds using
floor(x * 1000 + 0.5) / 1000. Weak skips use probability 0.02.
Unknown phone-set attributes remain ignored, as in Lua. JSON object keys
are sorted for reproducible output; assigned state IDs still follow input
order. `phonemap::create` exposes this operation to Rust callers.

`shiro-pm2md` retains positional phone-map input, `-d` feature dimensions
and `-t` hop, defaulting to 12 dimensions and 0.01 seconds. Duration and
per-stream emission counts use maximum referenced state index plus one,
including independent state tying in different streams. Each stream has
one mixture and weight one. Per-state duration constraints use the source
binary64 division and ceiling to frames. `phonemap::to_definition` returns
the typed original-schema definition without mutating the map.

`shiro-mkseg` accepts index, `-m` map, `-d` directory, `-e` feature suffix,
`-n` complete frame width, and comma-separated `-L`/`-R` phone padding.
The defaults are current directory, `.f` and 36 binary32 samples per frame.
Feature file length must be divisible by four times the complete frame
width. The command uses file metadata rather than reading unused feature
values. The original parsed but unused `-t` option remains accepted.
`segmentation::initial` performs the state expansion and transition setup.

Initial state boundaries use floor(k * frames / states + 0.5), preserving
the source operation order. All original topology behavior is retained:

- type-a, omitted and unknown topology names add no topology edges.
- type-b connects early states directly to the phone's final state.
- type-c adds relative offset-two edges to early states.
- skip-boundary adds edges across boundaries and from the globally
  penultimate state. This includes the original behavior for one-state phones.
- A positive phone skip probability adds an edge from the preceding state
  over the complete next phone, with relative offset state count plus one.

Unspecified extra-edge probabilities are 0.5 * (1 - explicit sum) divided
by the number of all extra edges, including those already assigned. The
ordinary forward edge is implicit and remains outside JSON `jmp`.
The source's unusual denominator is intentional compatibility behavior.

## Corrections and validation

Blank/CRLF phone-set rows no longer end the file or contaminate phone names.
Repeated whitespace is accepted. Duplicate phones and incomplete attribute
pairs are errors rather than silently losing definitions. State/stream counts
must be positive and fit the legacy range. Hop and dimensions must be valid;
constraints must be finite, have applicable state entries and fit i32 after
conversion. Skip probabilities must be between zero and one. Combined
explicit edge probabilities cannot exceed one. Empty initial sequences
remain empty without calculating a meaningless infinite spacing.

Shared duration constraints now intersect deterministically. The largest
active floor and smallest active ceiling apply to a tied duration state.
Nonpositive limits retain the original disabled-limit convention and do not
erase an active ceiling from another phone. Contradictory positive limits
return an error. The source overwrites constraints while iterating an
unordered Lua table: eight independent runs of the same two-phone map
produced either floor/ceiling 2/10 or 3/8. Rust consistently uses 3/8.
This is an intentional correction; tied maps with distinct constraints can
therefore produce a stricter definition than one arbitrary source run.

## Original-source comparisons

The reference is SHIRO `203ef7b71bf382c8b5ce3f86b8116f63265e2711`, using
unchanged mkpm, pm2md, mkseg and bundled Lua modules with Lua 5.4.9.
`phones-input.txt` contains three phones with fractional duration constraints.
`phones-original.json` stores seven original cases: all four named
topologies and an unknown name at four states, plus skip-boundary at one
and two states. Every case has two streams and weak skips enabled.
The segmentation sequence is bb, aa, bb, cc, aa over 41 frames. Only the
machine-specific feature filename is normalized to `sample.f`.

```text
lua shiro-mkpm.lua phones-input.txt -s 4 -S 2 -t type-b -w > map.json
lua shiro-pm2md.lua map.json -d 12 -t 0.01 > definition.json
lua shiro-mkseg.lua index.csv -m map.json -d directory -n 36 -L bb -R aa
```

The index row is `sample,aa bb cc`. `sample.f` is 41 * 36 * 4 zero bytes.
Repeat with each topology and state count. Model-definition duration
constraints are sorted by index for comparison because source order varies.
Tests compare frame boundaries, state IDs and metadata exactly. JSON numeric
attributes and probabilities use an absolute bound 1e-14, covering the Lua
encoder's decimal representation. CLI tests also execute map -> definition
-> mkhsmm -> model reload and compare to the Rust library model builder.

A separate original-C mkhsmm comparison uses the four-state/two-stream
type-b definition. Rust pm2md plus mkhsmm and original Lua pm2md plus C
mkhsmm both produce the same 4853-byte `.hsmm`, SHA256
`64630b66d5b360344860657016dd0d38935d6080ec6c14d5fa3b94a7062c9027`.
This verifies uninitialized model construction interoperability; it does
not establish trained-model or real-audio inference acceptance.

All 29 integration tests pass on Windows MSVC x86_64/i686, Windows GNU
x86_64 and Linux GNU x86_64/i686, including optional Lua and SPTK host tests.
Formatting and Clippy with warnings denied pass. Five SHIRO tools, remaining
ciglet functionality, complete native real-audio/training/inference acceptance
and all bindings remain incomplete.

Primary Rust references:
[BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html),
[f64](https://doc.rust-lang.org/std/primitive.f64.html), and
[Metadata](https://doc.rust-lang.org/std/fs/struct.Metadata.html).
