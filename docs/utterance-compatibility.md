# Utterance segmentation compatibility

`utterances::split_features` implements the original wavsplit model workflow.
`utterances::split_wave` includes WAV preparation and feature extraction in
memory. `shiro-wavsplit` exports the original intermediate suffixes, including
the final `.txt` labels. It does not split the waveform into separate files;
the original tool trains and aligns alternating silence and utterance states.

The original flags are `-n`, `-t`, `-d`, `-f`, `-N`, `-s`, `-v`, `-l`, and `-i`.
The trained model supplied with `-l` takes precedence over `-i` and skips
estimation. An initialized model skips fresh initialization. Loaded models are
borrowed by the library and never changed by the workflow. The CLI rejects
destinations aliasing the WAV or loaded model before opening any outputs.

The default pipeline retains 16 kHz mono audio, 0.01 dither, a 1024-sample
frame, 36 filterbank channels, 12 coefficients plus RMS energy, a 0.1-second
hop, and two distributions for silence/voicing. Minimum durations are in
seconds. Nonpositive minimum durations disable their constraints. Fresh
models use flat, globally tied initialization with variance-floor ratio one,
then DAEM estimation with no convergence stopping and 15 iterations. Duration
search extends by `floor(10 / hop)` frames and state pruning uses
`floor(0.2 * utterances)`. Utterances receive consecutive labels starting at
zero. The library returns each intermediate model, JSON document, and labels.

## Reference evidence

The fixtures use SHIRO commit
`203ef7b71bf382c8b5ce3f86b8116f63265e2711`, the pinned dependency revisions in
the design audit, and unchanged original Lua orchestration. C initialization
and model creation are unchanged; re-estimation and alignment use the two
inference corrections documented in `training-compatibility.md`. This is a
comparison to corrected C, not a claim of equivalence to unsafe source bugs.

`utterances-input.wav` contains four seconds of synthetic PCM16 mono audio at
16 kHz. Sample indices use `time = index / 16000`; intervals [0.6, 1.6) and
[2.3, 3.3) contain `14000 * (sin(2*pi*140*time) +
0.3*sin(2*pi*320*time))`, rounded to the nearest integer; other samples are
zero. Original `shiro-wavsplit.lua ./sample.wav -n 2 -N 2` produces the fixtures.
The feature filename in JSON is normalized to `sample.param`. Model definition
constraint ordering may differ because Lua iterates a hash table; constructing
the models proves the same constraints and bytes.

`tests/generate_utterance_reference.py --toolbox /path/to/reference/toolbox
--check` reruns the Lua workflow and verifies all ten reference artifacts.
The toolbox must contain the unchanged Lua tools, original WAV/xxcc/model
creation/initialization commands, and corrected-C rest/align executables.
The generator checks complete JSON values with normalized filenames and
duration-constraint ordering, and exact bytes for other artifacts.

Tests compare all fresh model stages byte for byte when using C features,
complete initial/aligned JSON values, phone maps, and label times within
1e-14 (Lua's decimal formatting differs). Loaded initialized and trained
models preserve input bytes and reproduce the appropriate outputs.

The waveform test uses the Linux GNU seed-one dither sequence on every target.
The audio samples match C exactly. On Windows MSVC 64bit the maximum normalized
feature difference is 8.076429e-6, below the existing xxcc reference bound of
2e-5. Rust-extracted features produce the same discrete labels, and the saved
model is reloaded and used for alignment. This synthetic case does not prove
real speech accuracy or a general training-parameter tolerance.

CLI tests exercise MFCC/MFBE/PLPCC, fresh and loaded model paths, bare filenames,
spaces, all eleven stage files, model rereading, and failure before any output
for invalid arguments and aliased inputs. Current cross-target validation is
recorded in the progress log; it is not implied by a host-only test run.

## Intentional corrections and limits

The original unquoted shell commands fail for paths with spaces. A bare
filename produces an empty directory argument, causing the original mkseg
call to consume its next option. Native orchestration fixes both cases and
propagates errors. Help is available although the Lua getopt string omits `h`.
Audio uses the corrected first-sample resampling policy described in
`audio-compatibility.md`; the 16 kHz reference does not exercise resampling.
JSON whitespace/order and label decimal text are not byte-identical promises.

Feature dimension one is rejected by the shared feature API, matching the
original xxcc parser: `-m 0` prints `Error: invalid feature order.` and exits
with status one. The original wavsplit shell workflow ignores that failure;
the Rust workflow returns it without writing intermediates.
Real-audio pretrained-model acceptance, the complete dependency inventories,
other Tier 1 platforms, and all bindings remain unfinished.
