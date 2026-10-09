# Model creation fixture

`rest-c-hmm-bootstrap-hsmm.hsmm` and its likelihood rows extend the corrected-C
training corpus with two HSMM updates starting from the two-iteration C HMM
bootstrap model. Reproduce them with `tests/generate_training_reference.py`
using the same pinned source and disclosed inference corrections as the other
rest fixtures. `tests/bootstrap_training.rs` checks the complete sequential
initialization/bootstrap/refinement/save/reload/inference workflow. See
`docs/training-compatibility.md` for exact scope and numerical bounds.

The `init-*` fixtures compare shiro-init to original C across aligned,
flat, tied and combined modes. The ten-frame fixture checks C float-duration
rounding, and the two-file fixture exposes the original fallback-duration
count bug. `init-input.bin` contains interleaved two/one-dimensional streams.
Generation, driver and precision details are in
`docs/initialization-compatibility.md`.

`phones-input.txt` and `phones-original.json` cover the original phone-map,
model-definition and initial-segmentation tools. The seven cases include
four named topologies, an unknown topology, weak skips and one/two-state
phones. See `docs/phonemap-compatibility.md` for generation and the separate
byte-identical original-C model comparison.

The `labels-*` fixtures compare both label conversion tools to the unchanged
original Lua programs. Only the segmentation fixture's machine-specific
filename was normalized. Generation, source pin and comparison criteria
are in `docs/label-compatibility.md`.

The three `c-fextr-*.bin` files contain the original Lua/C batch extractor
outputs for `c-audio-input.wav`. `index-original.txt` and
`index-original.json` compare the original Lua index loader with Rust,
including padding, empty phoneme fields and consecutive literal spaces.
See `docs/batch-compatibility.md` for source pins, generation, sizes,
measured differences and the separate actual SPTK comparison.

`modeldef.json` covers two independent streams, default mixture count and stream weight, multiple mixtures, and duration constraints. `empty-c.hsmm` was generated from it by the original `shiro-mkhsmm.c` at SHIRO commit `203ef7b71bf382c8b5ce3f86b8116f63265e2711`, using its bundled cJSON and liblrhsmm commit `1df92da4b77377f4725509e7240a9107ed4c063b` built with `FP_TYPE=float`. The CLI test requires identical binary output from the Rust command.


The synthetic c-audio-input.wav and its original-C plain/normalized/upsampled/downsampled/dither outputs verify the shiro-wav2raw command. The two DTH1 fixtures verify actual Windows and Linux GNU C-runtime draws and unchanged ciglet randu results. Generation commands, byte layout, scoped tolerances and intentional corrections are documented in docs/audio-compatibility.md. tests/dither_oracle.c is the original-header driver. These fixtures contain synthetic data and do not establish real-audio acceptance.

## CMU SLT real speech

`cmu-slt-arctic_a0001.wav` through `cmu-slt-arctic_a0003.wav` are unchanged
CMU US SLT ARCTIC recordings. Their copyright and permission notice is retained
in `../../LICENSES/CMU-ARCTIC.txt`. Corresponding `.param` files and
`cmu-slt-c-{hmm,hsmm}.json` are derived C reference outputs for this port.
The model, phone inventory and selected index rows come from upstream SHIRO.
See `../../docs/real-audio-compatibility.md` for source URLs, checksums, numerical
limits, model schema conversion, and the independent C/Lua reproduction command.
