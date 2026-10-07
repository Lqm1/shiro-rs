# Real speech compatibility

The regression corpus contains the first three recordings from the CMU US SLT
ARCTIC corpus. These are unchanged PCM16 mono WAV files at 16 kHz. The retained
CMU permission notice is `LICENSES/CMU-ARCTIC.txt`. Derived feature files,
selected index rows and normalized alignment documents are reference fixtures
created for this port. They are not original corpus annotations.

## Sources and provenance

The archive is
`http://festvox.org/cmu_arctic/packed/cmu_us_slt_arctic.tar.bz2`.
Its SHA-256 is
`7c173297916acf3cc7fcab2713be4c60b27312316765a90934651d367226b4ea`.
The downloaded COPYING file is
`http://festvox.org/cmu_arctic/cmu_arctic/cmu_us_slt_arctic/COPYING`.
Its SHA-256 is
`469adc0262e10d84fb9658fa6181c883586415b5692f6bfbc2fde65dfc63f5c5`.
The generator checks the SHA-256 of each unchanged waveform before use.

The model, phone inventory and index originate in SHIRO revision
`203ef7b71bf382c8b5ce3f86b8116f63265e2711`. The fixture model is an unchanged
copy of `examples/cmu-arctic-all-speakers.hsmm`, SHA-256
`5e5b56996817c058296a50e13b9719a928fd197ecdfb6341b139eeab604520f1`.
The index contains exactly the first three rows of `index-cmu-arctic-slt.csv`.
SHIRO materials retain their upstream GPL-3.0-or-later licensing.

## Model and numerical checks

The supplied model uses the historical GMM schema without variance-floor
slots. Rust reads it and saves that schema byte for byte. The current pinned C
reader expects floor slots. For C comparison, Rust saves the same model in the
current schema with zero floors, without changing weights, means, variances or
durations. The resulting SHA-256 is
`2467701cd9732eb702819d7418100d499966ea5f02fdc57977ea63a31cc00743`.

The model requires five states per phone and three streams of twelve dimensions.
Ten unused duration states for `ax` and `q` have zero variance. Alignment now
prepares only referenced duration densities. HMM inference prepares no explicit
duration densities. Strict all-state preparation still rejects invalid densities,
and referenced invalid densities still fail HSMM inference. No model repair or
replacement distribution is used to obtain these results.

The feature settings are a 512-sample frame, 80-sample hop, twelve MFCCs at
16 kHz, and delta and acceleration channels. There is no dither or energy
coefficient. C and Rust WAV conversion produced identical rawfloat sample bytes.
The initial Windows MSVC64 comparison measured maximum differences of
`4.2259693e-5`, `4.348904e-5`, and `3.2365322e-5`, using
`abs(rust - c) / max(1, abs(c))`. The regression bound is `5e-5` for this corpus.
It is not a general bound for all recordings or coefficient magnitudes.

The C oracle includes the two inference corrections documented in
`training-compatibility.md`. HMM inference uses default pruning. Subsequent HSMM
inference uses radius ten and fifty extra duration frames. C features with C
inference and Rust features with Rust inference yield equal complete state
JSON for these three files. This checks algorithm compatibility, not recognition
accuracy against manually annotated phonetic boundaries.

`tests/real_audio.rs` also builds a fresh model, performs flat globally tied
initialization with variance-floor ratio one, and trains two DAEM iterations in
each of HMM and HSMM mode. HMM training uses pruning slope 0.8. It checks finite
per-file reports, changed model parameters, byte-stable saving after reload,
identical inference before and after reload, and complete label coverage. This
training test is an end-to-end Rust regression; it does not assert numeric
identity with C training on these recordings.

## Reproducing the C references

Save the historical fixture with `Model::write_to` to obtain the current-schema
file described above. Build the pinned Linux C/Lua toolbox as described in the
training and utterance compatibility documents. It must contain original Lua,
`shiro-mkpm.lua`, `shiro-mkseg.lua`, their Lua support files, unchanged
`shiro-wav2raw` and `shiro-xxcc`, and corrected `shiro-align`.

Run:

```text
python tests/generate_real_audio_reference.py --toolbox /path/to/toolbox --current-model /path/to/current.hsmm
```

The default verifies all derived artifacts. `--write` regenerates only derived
feature and alignment fixtures. The generator uses original Lua to construct
phone maps and initial segmentation independently of the Rust implementation.
Feature byte equality in this generator applies to the pinned C compiler/build;
Rust-to-C comparisons use the measured tolerance above.

The original phone inventory has CRLF line endings. Linux Lua does not remove
the carriage return from the bare `sil` row, so the generator converts line
endings to LF in a temporary text copy. The tracked upstream inventory remains
unchanged. Rust accepts both line-ending conventions.
