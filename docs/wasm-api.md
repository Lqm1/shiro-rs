# Browser and Node.js WebAssembly API

The optional `wasm` feature exposes native computations through independent
owners and copied byte/typed arrays. Full SHIRO bindings and final combined
acceptance are still in progress. This checkpoint implements rawfloat, all
native label and phone-map operations, model construction/IO/parameter access,
and complete JSON document owners used by
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
or three standalone functions at that initial checkpoint. The expanded
feature-disabled verifier now covers 15 owners, including parameter collections
and the decoded model owner.

`tests/wasm_phones_node.cjs` and `wasm_phones_browser.html` run the original
interchange checks together with all seven unchanged Lua phone cases. They check
115 complete states, five model definitions and all four option fields twice
on the same module. The same native absolute tolerance of 1e-14 applies to the
Lua numerical references. This includes all native topology alternatives and
short-phone cases, shared duration constraints and every output stream count.

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
known-field schema. `ModelDefinition.build` constructs the native model. Remaining
inference/training/audio workflow calculations are still required work.

Errors become JavaScript exceptions. Array mutation does not alter Rust owners.
Ordinary inputs are borrowed for a synchronous call; `.free()` releases an
owner when no longer needed. WebAssembly panic/allocation-abort recovery is
not promised.

## Phone expansion and initial segmentation

`PhoneMapOptions` provides the native defaults and mutable `states_per_phone`,
`streams`, optional `topology` and `weak_skips`. `cloned` copies all four fields.
An absent topology uses `undefined`; a present empty string remains present.
Invalid editable values are rejected by calculations through native validation.

`PhoneMap.create(text, options)` expands phone-set text into disjoint ordered
duration/emission IDs. The returned map copies option strings and does not
borrow its source options. `map.to_definition(dimensions, hop)` returns a
complete `ModelDefinition`, preserving per-stream state counts and intersecting
shared duration constraints. `map.initial(phones, frames)` accepts an ordered
JavaScript string array and returns `States`. It retains native flat boundary
rounding, all topology and weak-skip edges, probabilities, duration/output IDs
and phoneme/local-index metadata. None of these calculations mutate the map.
`ModelDefinition.build` constructs a model from this result. The remaining
inference, training and audio workflows are subsequent binding tasks.

## Model construction, complete parameters and codecs

`ModelDefinition.build()` returns an independent binary32 `Model` using the
native SHIRO defaults, stream layouts, mixture counts, weights and duration
constraints. All native definition validation remains active. The original C
construction fixture is reproduced byte for byte.

`Model` exposes stream/duration counts, copied `stream`/`duration`, child
replacement through `set_stream`/`set_duration`, complete `replace(streams,
durations)`, cloning and structural validation. The constructor creates the
native empty model. `dimensions()` calls the native SHIRO importer helper and
returns copied ordered u32 dimensions, rejecting absent streams/emissions.
Complete replacements permit growth, shrinkage and empty collections.

| Owner | Complete native fields and access |
| --- | --- |
| `Duration` | Typed `values`/`set_values` in mean, variance, variance-floor order; signed `constraints`/`set_constraints` in minimum, maximum, fixed-mean order; clone |
| `Gaussian` | Dimensions, component count, copied weights/means/variances/variance floors, checked complete replacement and clone |
| `Stream` | Copied typed weight, weight replacement, emission count/copied child/replacement, complete replacement from `Gaussians`, clone |
| `Model` | Streams and durations through copied children, child replacement and complete replacement from `Streams`/`Durations`, clone |

`Gaussians`, `Streams` and `Durations` expose construction, length, copied get,
copied push, indexed replacement, clear and clone. Returned owners and arrays
remain valid after sources are released. Scalar storage uses typed arrays so
raw NaN payloads, infinities and signed zeros remain accessible without JSON
conversions. Structural parameter replacement validates lengths before mutation;
native calculations enforce their own numeric validity at use.

`Model.read`, `read_with_limits`, `read_prefix`, `read_prefix_with_limits`,
`write` and `write_with_encoding` reuse the native model codecs. The explicit
limit is the cumulative array-entry budget. Prefix decoding returns
`DecodedModel`, with bigint byte position, copied `parameters()` and consuming
`into_parameters()`. Strict reading rejects trailing data; prefix reading can
walk sequential models. Encoding 0 retains variance floors, encoding 1 is the
historical schema and rejects nonzero floors. The default writer uses encoding
0. Invalid codes fail.

`tests/wasm_models_node.cjs` and `wasm_models_browser.html` verify six original C
models, including the bundled historical model, with 91,151 compared wire bytes
including model construction. Every parameter field is compared by raw bits
through cloning, serialization and prefix recovery. Exact budgets and
one-entry-short failures are checked; every truncated prefix of the small
construction fixture and selected boundaries of the larger models fail.
Whole-model growth/shrinkage, copied dimensions, child lifetimes and atomic
invalid edits are included. The model, phone and interchange families run twice
on the same module, reversing their order on the second pass. This checkpoint
does not establish full inference/training or combined final acceptance.

The implementation follows the official wasm-bindgen documentation for
[exported Rust types](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/exported-rust-types.html)
and [copied numeric vectors](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-slices.html).
