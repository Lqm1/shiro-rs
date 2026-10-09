# Actual SPTK host verification

The optional SPTK and custom Lua extractors retain native host execution.
The protocol fixtures in `tests/sptk_fixture.c` exercise argument forwarding,
streams and process failures. Actual SPTK verification below separately checks
the real algorithms; protocol fixtures do not substitute for it.

All five agreed Rust targets were executed against actual SPTK 3.9 programs:

| Rust caller | Host programs | Original Lua, Rust SPTK adapter, Rust Lua adapter |
| --- | --- | --- |
| x86_64-pc-windows-msvc | Original SPTK 3.9 built for Windows x86 | Byte-identical outputs |
| i686-pc-windows-msvc | Same Windows x86 programs | Byte-identical outputs |
| x86_64-pc-windows-gnu | Same Windows x86 programs | Byte-identical outputs |
| x86_64-unknown-linux-gnu | Debian SPTK 3.9-3 amd64 programs | Byte-identical outputs |
| i686-unknown-linux-gnu | Same Linux amd64 programs | Byte-identical outputs |

A 32-bit caller may launch the host's 64-bit external program. This table
describes Rust caller execution, not five separately built SPTK architectures.
The input stem includes spaces and a dotted suffix. All three output files
compare completely, rather than only checking exit codes or frame counts.

| Output | Bytes | SHA256 |
| --- | ---: | --- |
| `.raw` | 1028 | `40cd155a0341b0ba0680249bfe777b49510ad7589f344864eb7c67e593a22fb5` |
| `.mfcc` | 192 | `87030d969b2a527eba58819cc988c1c2930acf7a7b685dca86cf729342adaee8` |
| `.param` | 576 | `ae0cec6978e43ddd40699cd30e6c6fd7674ee9c8e482a7add87c1ae84f2f9d31` |

## Reproduction

Build `shiro-fextr` for the target under test. Use the unchanged pinned SHIRO
`extractors/extractor-sptk-mfcc12-da-16k.lua`, Lua 5.4.9, actual binary32 SPTK 3.9
`frame`, `mfcc` and `delta` programs, and the tracked synthetic fixtures:

```text
python tests/verify-actual-sptk.py --cli SHIRO_FEXTR --sptk SPTK_PROGRAM_DIRECTORY --lua LUA --upstream-extractor ORIGINAL_EXTRACTOR --fixtures tests/fixtures --harness tests/run-original-sptk-extractor.lua --output RESULTS_DIRECTORY --target TARGET
```

The harness invokes the original extractor without modifying it. The verifier
runs original Lua, the Rust SPTK host adapter and the Rust custom-Lua adapter,
compares all bytes, and records actual executable/extractor hashes in a
separate `result.json` directory. Python `-O` is rejected. Temporary output is
retained to inspect failures and reproduce the report.

SPTK source came from the [Debian primary archive](https://deb.debian.org/debian/pool/main/s/sptk/).
`sptk_3.9.orig.tar.gz` SHA256 is
`94a8c4e9a43b853a5ce6693aa259c62af15bf641346d991e2b5c5d88705184ce`.
Windows executables were built with the original root, lib and bin
`Makefile.mak` files using MSVC's x86 environment. Only `frame.exe`, `mfcc.exe`
and `delta.exe` were selected for this comparison.

| Program | Windows x86 SHA256 | Linux amd64 SHA256 |
| --- | --- | --- |
| frame | `5e6fe365f189319f8e658ab3803eb048c8a5d389de4c6e513e80be1328bcee1a` | `bf021097935a9c6c6a8dee6b3868659c02e7c0e6e71a8ee0813c43742f2a27a2` |
| mfcc | `945d18d085003ba1d22d343f42458b75c2d25b719dbf41672cd9a68c0d544f30` | `646921d59adb06b4f5f9de7e3b3f105e7bfa8fb6b9e2d557730472d5afe65ffb` |
| delta | `7d0ef919cf1d178fb9b28531985737d033d7fca22c62d8fcd66b3f63a85b6585` | `c5823e0faca3216746e2e2f918d71ca911dda6cd74176da569d1ff83e23187bb` |

The unchanged original Lua extractor SHA256 is
`7573ea88c5fb2747bc04f10552acba91a96d005d3dda8dd1085d72dcae4ef0eb`.
SHIRO implementation revision is `1fb77db7f5097442120251b43a8cb06c7a7ab0bf`.
These synthetic host checks supplement real-speech, model and learning
acceptance; they do not establish those separate requirements by themselves.
