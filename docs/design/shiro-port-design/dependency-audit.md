# Dependency audit

The delegated audit has completed. Findings below include direct main-agent verification and corrections to the delegated report. An audit is not implementation or test evidence.

## Baselines

- ciglet: https://github.com/Sleepwalking/ciglet at `895ba9b1c0eabee83d3544208bbc82420efa3206`.
- liblrhsmm: https://github.com/Sleepwalking/liblrhsmm at `1df92da4b77377f4725509e7240a9107ed4c063b`.
- SHIRO dependency URLs were independently verified in SHIRO's lowercase `readme.md` by the main agent. SHIRO's standard build uses `FP_TYPE=float`.

## Complete port scope

ciglet includes scalar and complex operations, random generation, vector statistics, root finding, linear algebra, WAV I/O, windows, FFT and related transforms, phase and cepstral analysis, filtering and convolution, LPC, interpolation and resampling, sinusoid synthesis, psychoacoustics, frequency estimation, STFT/ISTFT, filterbanks, spectral envelopes, LF glottal models, and Gnuplot integration. These families are all in scope even when SHIRO does not call them.

liblrhsmm includes model and GMM construction and manipulation, statistics collection, observation and segmentation containers, topology and jump handling, HSMM forward/backward and Viterbi inference, geometric HMM operation, occupancy probabilities, EM re-estimation, DAEM controls, observation generation, numerical helpers, and serialization. Full functionality must be tracked by an upstream-to-Rust inventory rather than inferred from a successful SHIRO demo.

## Serialization findings

The model format uses positional MessagePack arrays without a magic number or version tag. `serial.c` writes floating-point fields with `cmp_write_float` and reads them through C `float`. The compatible on-disk representation is binary32 even when internal computation uses a different precision.

The segmentation writer declares a three-element outer array but writes four children. It also declares each outgoing-jump array with the jump count while writing a separate integer and float per jump. A strict generic MessagePack decoder must not be assumed compatible with these legacy encodings. Preserve legacy-readable behavior through explicit format handling and verify C-to-Rust and Rust-to-C interoperability, including nested segmentation sets. Do not silently apply generic array-arity fixes to the default compatibility format.

This segmentation encoding defect is distinct from the existing `.hsmm` model layout. The defect does not establish that the bundled model is malformed.

The numerical paths are not uniformly binary32 internally. For example, some model probability functions return `double`; verify the actual operation order and precision when implementing compatible Rust calculations.

## Licensing evidence and correction

SHIRO and liblrhsmm source headers specify GPL version 3 or any later version. Preserve their attribution and licensing in the reimplementation.

The delegated report incorrectly concluded that ciglet's BSD variant and copyright holder were unspecified. Direct inspection of `ciglet.h` shows the complete three-clause BSD license and names Kanru Hua, copyright 2016-2019. The absence of a standalone LICENSE file does not mean the repository lacks license text. Preserve that source notice and the applicable bundled-code notices.

Bundled dependency sources have separate notices, including cmp, fastapprox, Ooura FFT, and median filtering code. The vendored `external/wavfile.c` identifies Masanori Morise and modifications by Kanru Hua but does not include a license grant in the inspected header. Trace the matching upstream file and its license when deciding whether to adapt code or use an established compatible Rust WAV implementation. This is a provenance follow-up, not a reason to remove WAV functionality.

## Validation resources and remaining checks

Both dependencies include self-generating C test programs. Their assertion strength and coverage remain to be inspected before treating them as acceptance tests. The dependencies do not supply pretrained voice-model fixtures; SHIRO supplies a pretrained model.

The audit did not fully review all inference, numerical-helper, or memory-pool implementation bodies. These require source-level inventory and algorithm verification during implementation. A delegated report's suggestion that HTS conversion must exist in SHIRO was an inference without evidence and is not adopted as a requirement or verified capability.
