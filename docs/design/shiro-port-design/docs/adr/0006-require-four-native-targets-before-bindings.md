# Require Windows and Linux x86 validation before bindings

Use the primary Windows x86_64 target as an intermediate development milestone, but require executable native validation on Windows and Linux at both 64-bit and 32-bit before implementing bindings. A Windows-only gate would allow binding work sooner, but the accepted gate establishes cross-platform and pointer-width behavior first. This decision strengthens the binding prerequisite without removing the staged broader Tier 1 support plan.
