# Workspace layout

This repository contains five packages in a flat Cargo workspace:

| Package | Directory | Responsibility |
| --- | --- | --- |
| `shiro-rs` | `crates/shiro-rs` | Native Rust API and computations and the 14 SHIRO binaries |
| `shiro-rs-capi` | `crates/shiro-rs-capi` | C ABI, ownership boundary and checked-in headers |
| `shiro-rs-wasm` | `crates/shiro-rs-wasm` | Browser and Node.js WebAssembly bindings |
| `shiro-rs-node` | `crates/shiro-rs-node` | Native Node.js addon with napi-rs |
| `shiro-rs-python` | `crates/shiro-rs-python` | Native Python extension with PyO3 and maturin |

The workspace defaults to the native package. `cargo test` and existing SHIRO
binary names continue to select native workflows. Use explicit package selection
for adapters. The native package forbids unsafe code and has no C/WASM binding
features or wasm-bindgen dependency. All adapters depend directly on the native
package; the WASM adapter does not call the C ABI.

```text
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo build --locked -p shiro-rs-capi
cargo build --locked -p shiro-rs-wasm --target wasm32-unknown-unknown
cbindgen --config crates/shiro-rs-capi/cbindgen.toml --crate shiro-rs-capi --output crates/shiro-rs-capi/include/shiro_rs.h
wasm-bindgen --target nodejs --out-name shiro_rs --out-dir target/wasm-node target/wasm32-unknown-unknown/debug/shiro_rs_wasm.wasm
wasm-bindgen --target web --out-name shiro_rs --out-dir target/wasm-web target/wasm32-unknown-unknown/debug/shiro_rs_wasm.wasm
```

Use wasm-bindgen CLI 0.2.129, matching the pinned binding dependency. The native
C artifact base name is now `shiro_rs_capi`, rather than `shiro_rs`. This changes DLL,
import-library and static-library filenames, not C symbols or ABI layouts. Pass
the new artifact path to existing C/Python callers. The generated JavaScript
module retains `shiro_rs.js` through `--out-name`; exported JS names are unchanged.

Rust integration tests live in their owning package. Independent C/Python/JS
callers, reference generators and shared fixtures remain in the root `tests/`
directory. This keeps one authoritative copy of each original reference dataset.
The root `docs/` and license/attribution files apply to all five packages.

Historical acceptance reports retain their original commit and execution scope.
Their feature-based commands do not define the new workspace interface.

## Dependency policy

Adapters use a local workspace path to their own native package. This ensures
that a source edit is tested by its adapters before committing. Cross-repository
dependencies use public GitHub URLs and immutable commit revisions; they do not
require sibling checkout directories. Cargo resolves the native package inside
the dependency repository workspace. No registry publication is required.

Native Node.js and Python setup, public API typing, ownership and local packaging
are documented in their package READMEs and [verification](native-bindings.md).
