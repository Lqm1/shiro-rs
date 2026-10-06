# Model creation fixture

`modeldef.json` covers two independent streams, default mixture count and stream weight, multiple mixtures, and duration constraints. `empty-c.hsmm` was generated from it by the original `shiro-mkhsmm.c` at SHIRO commit `203ef7b71bf382c8b5ce3f86b8116f63265e2711`, using its bundled cJSON and liblrhsmm commit `1df92da4b77377f4725509e7240a9107ed4c063b` built with `FP_TYPE=float`. The CLI test requires identical binary output from the Rust command.
