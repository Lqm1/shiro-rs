# shiro-rs: native Python bindings

This directory is the `shiro-rs-python` Cargo package. Its distribution name is
`shiro-rs` and its Python import is `shiro_rs`. The extension is a private
`shiro_rs._shiro_rs` module inside a mixed Rust/Python package.

## Build and use

Activate a Python virtual environment and run in this directory:

```console
python -m pip install "maturin>=1.15,<2" pytest
maturin develop
python -m pytest tests
```

Use `maturin build --release` to create an optimized wheel. Maturin configures
PyO3 extension linking; the deprecated `extension-module` Cargo feature is not
used. The wheel includes the English license, `.pyi` declarations, and
`py.typed` marker. `__version__` reports the embedded Cargo package version.

```python
import shiro_rs as api
model = api.Model.read_file("model.hsmm")
model.validate()
model.write_file("copy.hsmm", 0)
model.close()
```

Numeric inputs accept Python sequences. Numeric results are lists; encoded
model, audio, and other byte results are `bytes`. Optional arguments accept
`None`; pass all arguments shown by the typed signature. Data classes use
PyO3's normal thread-safe storage. Random/generation owners containing internal
thread-local state and the plotting process owner are thread-confined.

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

`maturin build` creates a wheel for the selected Python interpreter and target.
`maturin sdist` creates a source distribution containing the local Rust path
dependencies. All three distributions were rebuilt outside the original workspace
with pip wheel. SHIRO retains pinned public Git dependencies for its companion
cores, so building its source distribution requires access to those revisions.

## References

- [PyO3 building and distribution](https://pyo3.rs/main/building-and-distribution.html)
- [PyO3 classes](https://pyo3.rs/main/class.html)
- [Maturin project layout](https://www.maturin.rs/project_layout.html)
- [Maturin configuration](https://www.maturin.rs/config.html)

## License

GPL-3.0-or-later; see `LICENSE`.
