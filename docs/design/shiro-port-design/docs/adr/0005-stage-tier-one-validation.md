# Stage Tier 1 validation from the primary Windows target

Design the packages for Rust Tier 1 targets while allowing the first native acceptance milestone on `x86_64-pc-windows-msvc`, this machine's primary target. Prioritize Windows and Linux x86 targets at both 64-bit and 32-bit, and defer the other Tier 1 targets when local validation is disproportionately complex. This keeps portable design as a requirement without making unavailable execution environments a prerequisite for the initial milestone; publish the actual verification status of each target separately.

ADR 0006 strengthens the gate before binding work. The Windows-only milestone is intermediate and does not authorize starting bindings.
