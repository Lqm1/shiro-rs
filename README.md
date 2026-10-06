# shiro-rs

An idiomatic Rust reimplementation of the complete SHIRO toolkit, including its original C and Lua workflows. Upstream: https://github.com/Sleepwalking/SHIRO.

The port is in progress. The package links the local `ciglet-rs` and `liblrhsmm-rs` Rust packages. Typed model definitions and `shiro-mkhsmm -c modeldef.json` are implemented, with binary output checked against the original C tool. Alignment, training, the remaining command-line tools, feature-extraction orchestration, and label conversion are not implemented yet. No C ABI or WebAssembly bindings are implemented yet.

The three repositories are independent sibling Cargo packages. Local path dependencies permit development before publishing versions.

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
```

Licensed under GPL-3.0-or-later. SHIRO source is copyright 2017-2018 Kanru Hua. Audited upstream commit: `203ef7b71bf382c8b5ce3f86b8116f63265e2711`.
