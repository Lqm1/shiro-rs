# shiro-rs

An idiomatic Rust reimplementation of the complete SHIRO toolkit, including its original C and Lua workflows. Upstream: https://github.com/Sleepwalking/SHIRO.

The port is in progress. The package links the local `ciglet-rs` and `liblrhsmm-rs` Rust packages. Typed model definitions and `shiro-mkhsmm -c modeldef.json` are implemented, with binary output checked against the original C tool. The feature API now implements MFCC/MFBE/PLPCC with DC/energy and dynamic features, compared to the original xxcc pipeline. The shiro-xxcc command retains the original options and reads rawfloat files or piped stdin, writing rawfloat features to stdout. The shiro-wav2raw command reads WAV files, normalizes, dithers and resamples before writing rawfloat. It retains the original Windows/Linux GNU default dither sequences, corrects silent normalization, and provides explicit legacy resampling and opt-in seeded Rust dither. See docs/audio-compatibility.md for measured differences. Alignment/training orchestration and four remaining command-line tools are not implemented yet. No C ABI or WebAssembly bindings are implemented yet.

The three repositories are independent sibling Cargo packages. Local path dependencies permit development before publishing versions.

`shiro-fextr` now handles indexed WAV conversion and all three bundled xxcc
extractors in Rust. The bundled SPTK workflow and custom Lua callbacks have
host adapters. See `docs/batch-compatibility.md` for presets, original options,
compatibility corrections, numerical comparisons and host verification limits.
`shiro-lab2seg` and `shiro-seg2lab` implement timed label conversion and
phoneme/state alignment output. See `docs/label-compatibility.md` for the
original rounding, grouping, JSON schema and decimal-format differences.
`shiro-mkpm`, `shiro-pm2md` and `shiro-mkseg` implement phone-set expansion,
model definitions and equally spaced segmentation with the original phone
topologies and skip probabilities. See `docs/phonemap-compatibility.md` for
Lua/C comparisons and the corrected shared-duration constraint handling.
`shiro-init` supports aligned and flat initialization, global emission tying,
model-shaped rawfloat loading and relative variance floors. The original C
models compare byte for byte on the reference corpus, with documented fixes
for malformed feature files and multi-file fallback duration statistics.
See `docs/initialization-compatibility.md`. Four other SHIRO tools remain pending.

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
```

Licensed under GPL-3.0-or-later. SHIRO source is copyright 2017-2018 Kanru Hua. Audited upstream commit: `203ef7b71bf382c8b5ce3f86b8116f63265e2711`.
