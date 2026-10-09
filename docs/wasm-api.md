# Browser and Node.js WebAssembly API

The optional `wasm` feature exposes native computations through independent
owners and copied byte/typed arrays. Full SHIRO bindings and final combined
acceptance are still in progress. This checkpoint implements rawfloat and all
native label operations, together with complete JSON document owners used by
the remaining workflows. CLI invocation, OS files and external Lua/SPTK
processes remain native as specified by ADR 0007.

## Build and verification

Use Rust 1.94 and wasm-bindgen CLI 0.2.129, matching the exact optional crate
version. The project was created with Cargo; the optional dependency was added
with `cargo add wasm-bindgen@=0.2.129 --target wasm32-unknown-unknown --optional`.
The `wasm` feature activates that dependency on the WASM target.

```text
cargo build --target wasm32-unknown-unknown --features wasm --lib
wasm-bindgen target/wasm32-unknown-unknown/debug/shiro_rs.wasm --target nodejs --out-dir bindings-node
node tests/wasm_interchange_node.cjs bindings-node
wasm-bindgen target/wasm32-unknown-unknown/debug/shiro_rs.wasm --target web --out-dir bindings-web
```

Serve the project and web bindings over HTTP, mounting web bindings at
`/bindings/`, then open `tests/wasm_interchange_browser.html`. Node and browser
use the same verifier twice on one initialized module. The verified result is
9000 exact rawfloat values, seven unchanged Lua states, 13 unchanged Lua label
rows and four complete document kinds. The Lua numerical comparison retains
the native verifier's absolute tolerance of 1e-14. State fields and rawfloat
bits are exact. The feature-disabled build exposes none of these six owners
or three standalone functions.

## Rawfloat and labels

`rawfloat_read(bytes, maximum_samples)` returns a copied `Float32Array`.
`rawfloat_write(values)` returns a copied `Uint8Array` of little-endian binary32
values. Signed zero, infinities, NaN payload bits and subnormals survive the
roundtrip. Partial final scalars and insufficient sample budgets fail through
native validation. Empty streams are supported.

`Label(start, end, name)` exposes mutable binary64 `start`/`end` and UTF-8 `name`.
It retains raw native field values; label transformations enforce native
interval checks. `Labels` exposes construction, `parse`, `length`, copied
`get`, copied `push`, indexed `replace`, `clear`, `cloned` and byte `write`.
Writing uses native tab-separated seconds and CRLF endings. Invalid output
names fail. Cloning and child retrieval are independent of source lifetime.

`labels.to_states(phoneMap, hop)` returns a `States` owner.
`Labels.from_states(states, hop, includeStates)` returns a `Labels` owner.
These invoke the original native boundary rounding, phoneme grouping,
interleaved state-label option and metadata validation. `label_output_path`
retains the legacy extension/separator rules without performing filesystem I/O.

## Complete JSON owners

`PhoneMap`, `States`, `SegmentationDocument` and `ModelDefinition` accept their
native JSON schemas in the constructor. `json` returns the complete native
document, `replace` validates JSON into a replacement before assigning it, and
`cloned` makes an independent owner. `States` uses an ordered JSON array.
Every field can be changed through complete JSON replacement, including array
lengths. Phone maps and segmentation documents retain flattened attributes at
every native level. State owners retain optional duration/output/jump fields
and all metadata entries. Model definitions retain their native defaulting and
known-field schema. These owners do not yet expose the remaining model/workflow
calculations; those remain required work.

Errors become JavaScript exceptions. Array mutation does not alter Rust owners.
Ordinary inputs are borrowed for a synchronous call; `.free()` releases an
owner when no longer needed. WebAssembly panic/allocation-abort recovery is
not promised.

The implementation follows the official wasm-bindgen documentation for
[exported Rust types](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/exported-rust-types.html)
and [copied numeric vectors](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-slices.html).
