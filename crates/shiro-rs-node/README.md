# shiro-rs: native Node.js bindings

This directory is the `shiro-rs-node` Cargo package and the local `shiro-rs` npm
package. It belongs to the existing repository workspace.

## Build and use

Install Rust and a supported Node.js version, then run in this directory:

```console
npm ci
npm run build
npm test
```

Use `npm run build:release` for an optimized addon. The napi-rs CLI generates
`index.js`, `index.d.ts`, and the platform-specific `.node` binary. The CLI is
pinned to 3.5.1 because newer tested CLI versions fail their filesystem
transaction rename on the development Windows host; Rust napi-rs remains v3.
The TypeScript declarations are generated from the exported API, including
explicit callback signatures.

```javascript
const api = require("./");
const model = api.Model.read_file("model.hsmm");
model.validate();
model.write_file("copy.hsmm", 0);
model.close();
```

The `version()` function reports the embedded Cargo package version. Inputs use
typed arrays for numeric buffers, `number` for dimensions, and `bigint` for
64-bit integer values. Dimensions must be finite, nonnegative safe integers
within the target's `usize` range. Invalid shapes and indices throw exceptions.

These bindings call the native Rust implementation directly. They do not
load the C ABI or WebAssembly adapter. Objects own their Rust data; results and
arrays are independent copies. Numeric precision follows the chosen `F32` or
`F64` API. Model serialization retains the original binary32 format.

Calls are synchronous. A long computation occupies the calling thread.
Objects release their data automatically when garbage collected. `close()` and
its `free()` alias release data explicitly and are idempotent. Access after
release raises an exception. Ownership-taking APIs, such as `into_*` and
`prepare_owned`, consume their argument; use `cloned()` when retaining it.

Array arguments are copied. Operations that edit a caller-owned array in Rust
return `(result, updated_array)` in these native bindings. They do not edit the
input array. This applies to matrix swaps, selection, and CZT output buffers.
Class setters and methods still update the owning class.

Callback exceptions propagate to the original caller. Callbacks execute on the
calling thread. Re-entering an owner during an exclusive operation raises an
exception; immutable operations can share an owner. Callback arguments and retained context remain owned for as long
as the native owner needs them.

All six packages inherit their repository workspace version. No package is
published by the build or test commands below.

## Local distribution

`npm pack` creates a local tarball containing the loader, TypeScript declarations,
license, and built addon. It is specific to the platform built locally. A
multi-platform public release needs the separate platform packages produced by
the napi-rs release workflow; a local tarball is not that release.

## References

- [napi-rs manual setup](https://napi.rs/docs/introduction/manual-setup)
- [napi-rs classes](https://napi.rs/docs/concepts/class)
- [napi-rs TypeScript overrides](https://napi.rs/docs/concepts/types-overwrite)
- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)

## License

GPL-3.0-or-later; see `LICENSE`.
