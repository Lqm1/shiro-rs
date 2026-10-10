# @shiro-rs/wasm

Browser bindings for shiro-rs. Install with `npm install @shiro-rs/wasm`.

```javascript
import init, * as api from '@shiro-rs/wasm/web';
await init();
// Use api after initialization. See the repository's WASM API documentation.
```

`@shiro-rs/wasm` is the same web entrypoint. Use `@shiro-rs/wasm/bundler`
with a WebAssembly-aware bundler such as webpack 5 with asyncWebAssembly enabled.
The bundler entrypoint is initialized by the bundler and has no default init function.
Both entrypoints export the same computational API; the web initializer is additional.
The web .wasm asset must be served relative to its generated JavaScript module.

Node.js users should install `@shiro-rs/node` for the native implementation.
No nodejs, deno, no-modules or source-phase-import module build is distributed.
GPL-3.0-or-later. See LICENSE and the tagged repository for source and attribution.
