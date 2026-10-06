# Keep host operations outside browser and Node WebAssembly bindings

Expose computational functionality, model processing, and training through browser and Node.js WebAssembly APIs using in-memory inputs and outputs. Keep CLI invocation, operating-system filesystem operations, and external Lua execution native because the browser execution environment does not provide the original host process and filesystem model. Additional WASI support is outside this scope. C ABI bindings provide native access from C and Python in each existing package.
