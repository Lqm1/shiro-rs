# Release procedure

Release tooling builds distributions but never registers accounts, creates npm
organizations, configures Trusted Publishers, or approves npm stages. A missing
user prerequisite stops its checkpoint. Initial preparation has been confirmed by
the owner; registry publication has not started.

## Names and source identity

The core Cargo crate and Python distribution use the repository name. Python
imports replace hyphens with underscores. npm publishes `@<repository>/node`,
eight generated `@<repository>/node-<platform>-<arch>[-<abi>]` sidecars, and
`@<repository>/wasm`. The WASM package exports `/web` and `/bundler`; its root
exports the web entrypoint. Node.js applications should use the native package.
Binding Cargo crates are private and are never uploaded to crates.io.

`tools/release-config.json` is the target inventory. Cargo workspace version is
authoritative. Run `python tools/release.py sync --version X.Y.Z` and review the
metadata/lockfile changes before committing. `release-pr.yml` uses release-plz
for Conventional Commit version and changelog preparation, then synchronizes
the binding metadata. It does not publish, tag or create a GitHub release.
Review binding API changes when choosing the release version; a core API semver
check does not detect Python or JavaScript API changes.

All formal distributions come from a clean, immutable full commit SHA. Dispatch
`publish.yml` from a branch or tag pointing at that same commit, and supply that
SHA as `source_ref`. A mismatch with the workflow's own commit is rejected, since
OIDC provenance identifies the workflow revision. Keep that ref unchanged until
every checkpoint finishes. No source edits are permitted after artifacts freeze.

## User prerequisites

The owner performs these steps. Do not give token values to the agent or put them
in tracked files.

1. Verify crates.io GitHub login and email; verify PyPI/npm email and 2FA.
2. Own all three npm organizations and verify `npm whoami` and `npm org ls`.
3. Create GitHub Environment `release` in every repository. Restrict its deployment
   branches to reviewed release refs. Configure required reviewers if desired.
4. Enable GitHub Actions to create pull requests if using `release-pr.yml`.
5. For the implemented Cargo alpha only, create a short-lived, minimally scoped
   token and store it as Environment secret `CRATES_IO_BOOTSTRAP_TOKEN`.
6. After the alpha exists, register crates.io Trusted Publishing for owner `Lqm1`,
   this repository, `publish.yml`, Environment `release`. Revoke the bootstrap
   token and remove its GitHub secret.
7. Create a PyPI Pending Trusted Publisher using those same four identity fields.
   Pending registration does not reserve the distribution name.
8. Stage real implemented npm alpha packages locally to establish all ten names,
   then configure every package's GitHub trust with the same identity fields.
   Allow `npm stage publish`; disable direct publishing through that trust.
   Require 2FA and disallow long-lived publishing tokens. Never approve the alpha.
9. Review the formal npm stages and approve each OS sidecar before the native root
   package. Approve the WASM package separately. Reject leftover alpha stages.

Name availability is checked again at publication. An earlier 404 is not a
reservation or proof that a name is permitted. Stop on a conflicting owner/name.
The agent must not choose alternate names or substitute static-token publishing.

## First release order

Publish `ciglet-rs` and `liblrhsmm-rs` implemented `0.1.0-alpha.1` cores first.
Their formal `0.1.0` follows only after the owner's crates.io trust registration.
Then replace SHIRO's pinned Git dependencies with those published registry
versions, removing `git` and `rev`, updating the lockfile and rerunning all checks.
Only after that can SHIRO's alpha and formal publication proceed. The release
preflight deliberately rejects SHIRO's current Git dependencies.

Use a reviewed alpha commit for `bootstrap-crate`; this is the only API-token
publication phase. The alpha contains implemented source, never a reservation
stub. Return to a separately reviewed `0.1.0` commit for formal builds. Versions
are synchronized within each project, while future project releases are
independent of one another.

To establish npm names, build distributions from the separately reviewed alpha
commit using the `build` phase. Download `verified-release`, inspect its manifest,
then the owner runs `npm stage publish <tarball> --access public --tag alpha
--ignore-scripts` for each tarball. Configure trust after staging. The owner must
not approve those alpha stages. Formal `0.1.0` is staged anew with OIDC.

## Workflow checkpoints

`publish.yml` is manually dispatched with an explicit phase:

| Phase | Required state | Effect |
| --- | --- | --- |
| `bootstrap-crate` | Alpha commit, release Environment and bootstrap secret | Publish only the implemented core alpha |
| `build` | Frozen source; SHIRO registry dependencies available | Build and test, upload `verified-release` |
| `crate` | Stable commit, successful build run, configured Cargo trust | Publish core through OIDC |
| `npm-stage` | Same build run, all ten package trusts configured | Stage tarballs through OIDC with provenance |
| `pypi` | Same build run, Pending Publisher configured | Upload wheels/sdist through OIDC with attestations |
| `finalize` | All registries published and verified | Create the matching tag and publish GitHub Release assets |

For publication, `build_run_id` identifies the successful `build` phase run.
Downloaded artifact source records, inventory and hashes must match the checkout.
The workflow rejects moving branches, failed runs and mismatched source commits.

Partial npm failures upload `npm-stages` even on failure. Supply that run's ID as
`npm_stage_run_id` to resume the recorded stages from the identical source and
tarballs. A stage rejected by the owner is no longer a valid checkpoint; stop and
review before restarting. PyPI resumes only files absent from the registry and
rejects different hashes. Never rebuild or change source to overwrite an existing
formal version. A fix requires a new version. A missing or expired artifact requires
review; do not silently reconstruct a supposedly identical distribution.

## Verification and assets

Builds cover the fixed eight Rust Tier 1 targets. Runtime jobs cover environments
available to GitHub-hosted runners. Node 22, 24 and 26 run installed-package API and
type tests. Standard GIL-enabled CPython 3.11 through 3.15 gets version-specific
wheels; Windows GNU wheels are separated from PyPI wheels. Linux uses
manylinux2014. Free-threaded Python, PyPy and abi3 are outside this initial matrix.
An unavailable or failed build is not dropped from the inventory.

WASM executes existing compatibility oracles through both web and webpack bundler
entrypoints in Chromium, Firefox and WebKit. Identical wasm-bindgen binaries can be
shared; differing binaries are retained. Python sdist is rebuilt separately.
Installed Node/Python checks run outside the repository, without build fallbacks.

`release-manifest.json` records source SHA, version, dependency origin, toolchain,
target inventory and runtime reports. `SHA256SUMS` covers every distribution.
Native archives contain C ABI headers, shared/static libraries and all CLI binaries
where present. Python wheels/sdist, npm tarballs and WASM are attached too. The
source archive includes the tracked source, locked vendored dependency source,
license files and a third-party package/license inventory for GPL recipients.

Finalization compares public npm integrity, PyPI SHA-256 and crates.io checksum
with frozen artifacts. It requires npm provenance, runs npm signature verification
for all ten installed packages, and cryptographically verifies PyPI attestations
and their configured publisher identities. GitHub Releases remain drafts until
those checks pass. Runtime reports distinguish executed tests from build-only
targets; compiling an artifact never marks it runtime tested.

## Official references

- [crates.io Trusted Publishing](https://doc.rust-lang.org/cargo/reference/registry-authentication.html)
- [PyPI Pending Publishers](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/)
- [PyPI attestations](https://docs.pypi.org/attestations/consuming-attestations/)
- [npm Trusted Publishing](https://docs.npmjs.com/trusted-publishers/)
- [npm stage commands](https://docs.npmjs.com/cli/v11/commands/npm-stage/)
- [napi-rs platform package generation](https://napi.rs/docs/cli/create-npm-dirs)
- [wasm-bindgen deployment targets](https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html)
- [maturin GitHub Actions](https://www.maturin.rs/distribution.html)
- [release-plz configuration](https://release-plz.dev/docs/config)

### CPython 3.15 build bootstrap

CPython 3.15.0 is installed with pinned uv 0.13.0 and setup-uv 10.3.0 while
setup-python's version inventory catches up with the stable release. Its virtual
environment lives under the runner temporary directory. CPython 3.11 through
3.14 retain setup-python. Release candidates and free-threaded interpreters are
not substituted. The selected interpreter path is recorded before wheel builds.
