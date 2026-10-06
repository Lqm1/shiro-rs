# Preserve external Lua extractors through an optional interpreter

Reimplement the bundled feature-extraction workflows in Rust and preserve arbitrary existing Lua extractors through an optional externally installed Lua interpreter. Dropping the extension mechanism would break existing custom pipelines, while embedding an interpreter would add an unnecessary runtime dependency to ordinary native workflows. External interpreter availability is required only for the compatibility mode.
