# Random dependency lock and notices

The sibling ciglet-rs now depends on rand 0.10.3 and rand_chacha 0.10.0.
Cargo.lock reflects those path-dependency requirements without changing existing
resolved versions. Rand is already a direct SHIRO dependency; its prior MIT
notice and rand_core notice remain retained.

New resolved packages have retained MIT notices:

| Package | Version | Notice |
| --- | --- | --- |
| rand_chacha | 0.10.0 | LICENSES/rand_chacha-MIT.txt and COPYRIGHT |
| ppv-lite86 | 0.2.21 | LICENSES/ppv-lite86-MIT.txt |
| zerocopy | 0.8.61 | LICENSES/zerocopy-MIT.txt |
| zerocopy-derive | 0.8.61 | LICENSES/zerocopy-derive-MIT.txt |
| syn | 2.0.119 | LICENSES/syn-2.0.119-MIT.txt |

The optional zerocopy derive path adds a second syn major version to the lock;
existing clap/serde derive dependencies retain syn 3.0.6. These additions do not
alter CLI syntax, codecs or numerical operation bodies. Complete distribution
notice/feature configuration review remains part of the final native audit.
