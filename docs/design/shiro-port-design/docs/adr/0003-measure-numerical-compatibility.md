# Measure numerical compatibility with justified tolerances

Preserve existing file formats and validate discrete inference results while measuring floating-point differences against the original C implementation. Requiring universal bitwise equality would constrain idiomatic implementation and optimization, especially because the original standard build uses `float` and `-Ofast`. Derive tolerances from baseline measurements and investigate discrepancies that change discrete inference results instead of accepting them solely because intermediate numbers are close.
