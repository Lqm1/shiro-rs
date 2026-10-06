# Maintain three independent package repositories

Maintain `shiro-rs`, `ciglet-rs`, and `liblrhsmm-rs` as independent Git repositories and Cargo packages under `D:\Desktop\Projects`. A shared workspace would simplify coordinated builds and changes, but independent repositories preserve the requested boundaries and allow each library to be distributed independently. The SHIRO package includes its library and command-line targets without creating additional published packages.
