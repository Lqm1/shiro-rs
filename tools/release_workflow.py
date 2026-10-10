"""Explicit release checkpoints; no account setup or automatic human approvals."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tarfile
import tempfile
import urllib.error
import zipfile

import release as r


def optional_json(url: str) -> dict | None:
    try:
        return r.fetch_json(url)
    except urllib.error.HTTPError as error:
        if error.code != 404:
            raise
        return None


def preflight() -> None:
    sha = os.environ.get("RELEASE_SHA", "")
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("A reviewed full commit SHA is required")
    if os.environ.get("GITHUB_SHA") not in (None, sha):
        raise ValueError("Dispatch the workflow from the release commit itself to preserve OIDC provenance")
    r.check(registry=True, frozen=True)
    phase = os.environ["RELEASE_PHASE"]
    alpha = "-alpha." in r.version()
    if phase != "build" and (phase == "bootstrap-crate") != alpha:
        raise ValueError("Only bootstrap-crate accepts alpha versions")
    if phase != "build":
        environment = json.loads(r.run("gh", "api", f"repos/Lqm1/{r.PROJECT}/environments/release", capture=True))
        if environment.get("name") != "release":
            raise ValueError("The user must configure the release Environment before publication")
    print(f"Publication checkpoint verified: {phase} {sha}")


def metadata() -> None:
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
        output.write(f"sha={r.assert_clean()}\nversion={r.version()}\n")


def require_token() -> None:
    if not os.environ.get("CARGO_REGISTRY_TOKEN"):
        raise ValueError("The user must configure CRATES_IO_BOOTSTRAP_TOKEN in release")
    if r.version() != "0.1.0-alpha.1":
        raise ValueError("Bootstrap is restricted to the initial implemented alpha")


def validate_run(data: dict, sha: str) -> None:
    if (data.get("conclusion") != "success" or data.get("event") != "workflow_dispatch"
            or data.get("path") != ".github/workflows/publish.yml"):
        raise ValueError("Artifacts must come from a successful manual publish.yml build")
    if not re.fullmatch(r"[0-9a-f]{40}", sha) or data.get("head_sha") != sha:
        raise ValueError("Build workflow commit differs from the release source")


def download() -> None:
    identifier = os.environ.get("BUILD_RUN_ID", "")
    if not identifier.isdecimal():
        raise ValueError("A successful build-phase run ID is required")
    repository = f"Lqm1/{r.PROJECT}"
    data = json.loads(r.run("gh", "api", f"repos/{repository}/actions/runs/{identifier}", capture=True))
    validate_run(data, r.run("git", "rev-parse", "HEAD", capture=True))
    if r.DIST.exists():
        raise ValueError("Artifact destination already exists; use a fresh checkout")
    r.run("gh", "run", "download", identifier, "--repo", repository,
          "--name", "verified-release", "--dir", str(r.DIST))
    r.verify_manifest(r.DIST)


def wheel_inventory(target: str, python: str) -> None:
    directory = r.ROOT / f"target/python-{target}-{python}"
    wheels = list(directory.glob("*.whl"))
    if len(wheels) != 1:
        raise ValueError("Expected exactly one version-specific CPython wheel")
    tag = "cp" + python.replace(".", "")
    if f"-{tag}-{tag}-" not in wheels[0].name:
        raise ValueError("Wheel is not version-specific GIL-enabled CPython")
    with zipfile.ZipFile(wheels[0]) as archive:
        metadata = [name for name in archive.namelist() if name.endswith(".dist-info/METADATA")]
        if len(metadata) != 1 or f"Version: {r.version().replace('-alpha.', 'a')}\n" not in archive.read(metadata[0]).decode():
            raise ValueError("Wheel version mismatch")
    r.write_json(directory / "build-record.json", {**r.source_record(), "target": target,
                 "python": python, "runtime_tested": False, "environment": r.build_environment()})


def assemble(inputs: Path) -> None:
    r.DIST.mkdir(parents=True, exist_ok=True)
    for path in inputs.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix in {".tgz", ".zip"} or path.name.endswith(".tar.gz"):
            destination = r.DIST / path.name
        elif path.suffix == ".whl":
            gnu = any("windows-gnu" in part for part in path.parts)
            destination = r.DIST / ("python-gnu" if gnu else "python") / path.name
        elif path.suffix == ".json":
            destination = r.DIST / "build-records" / path.parent.name / path.name
        else:
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        if destination.exists():
            raise ValueError(f"Duplicate artifact: {destination}")
        shutil.copy2(path, destination)
    archive_path = r.DIST / f"{r.PROJECT}-{r.version()}-python-windows-gnu.zip"
    with zipfile.ZipFile(archive_path, "w", zipfile.ZIP_DEFLATED) as archive:
        for path in sorted((r.DIST / "python-gnu").glob("*.whl")):
            archive.write(path, path.name)
        archive.writestr("release-source.json", json.dumps(r.source_record(), indent=2))
    r.pack_node(inputs)
    r.run("cargo", "package", "--locked", "-p", r.PROJECT)
    shutil.copy2(r.ROOT / f"target/package/{r.PROJECT}-{r.version()}.crate", r.DIST)
    bundle_sources()
    r.manifest(r.DIST)
    r.verify_manifest(r.DIST)


def bundle_sources() -> None:
    """Include the exact tracked source and locked dependency sources for GPL recipients."""
    output = r.ROOT / "target/release-sources"
    output.mkdir(parents=True, exist_ok=False)
    for name in r.run("git", "ls-files", "-z", capture=True).split("\0"):
        if name:
            path = Path(name)
            if path.is_absolute() or ".." in path.parts:
                raise ValueError("Invalid tracked source path")
            destination = output / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(r.ROOT / path, destination)
    vendor = output / "vendor"
    configuration = r.run("cargo", "vendor", "--locked", "--versioned-dirs", str(vendor), capture=True)
    (output / ".cargo").mkdir(exist_ok=True)
    # Cargo emits host-specific paths; the source archive must be portable.
    configuration = re.sub(r'(?m)^directory = .*$', 'directory = "vendor"', configuration)
    (output / ".cargo/config.toml").write_text(configuration, encoding="utf-8")
    data = json.loads(r.run("cargo", "metadata", "--locked", "--format-version", "1", capture=True))
    notices = [{"name": p["name"], "version": p["version"], "license": p["license"],
                "repository": p["repository"], "source": p["source"]} for p in data["packages"]]
    r.write_json(output / "THIRD-PARTY-NOTICES.json", notices)
    r.write_json(output / "release-source.json", r.source_record())
    path = r.DIST / f"{r.PROJECT}-{r.version()}-sources.zip"
    archive_sources(output, path)


def archive_sources(source: Path, destination: Path) -> None:
    """Preserve dependency contents even when timestamps precede the ZIP epoch."""
    with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED, strict_timestamps=False) as archive:
        for item in sorted(source.rglob("*")):
            if item.is_file():
                archive.write(item, item.relative_to(source).as_posix())


def test_python(platform: str, directory: Path | None, registry: bool = False) -> None:
    root = (directory or r.DIST / "python").resolve()
    tag = "cp" + str(sys.version_info.major) + str(sys.version_info.minor)
    wheels = [p for p in root.glob("*.whl") if f"-{tag}-{tag}-" in p.name and platform in p.name]
    if len(wheels) != 1:
        raise ValueError("Expected exactly one compatible Python wheel")
    # pip and import run from a fresh temporary directory, outside the repository.
    with tempfile.TemporaryDirectory() as temporary:
        cwd = Path(temporary)
        requirement = f"{r.PROJECT}=={r.version()}" if registry else str(wheels[0])
        r.run(sys.executable, "-m", "pip", "install", "--force-reinstall", "--no-deps",
              "--only-binary=:all:", "--no-cache-dir", "--index-url", "https://pypi.org/simple", requirement, cwd=cwd)
        r.run(sys.executable, "-m", "pip", "install", "pytest", cwd=cwd)
        r.run(sys.executable, "-m", "pytest", "--import-mode=importlib",
              str(r.PYTHON / "tests"), cwd=cwd)
    r.write_json(r.ROOT / f"target/runtime-tests/python-{platform}-{tag}.json",
                 {**r.source_record(), "platform": platform, "python": sys.version,
                  "wheel_sha256": r.digest(wheels[0]), "passed": True})


def package_metadata(path: Path) -> dict:
    with tarfile.open(path) as archive:
        return json.load(archive.extractfile("package/package.json"))


def test_node(target: str, registry: bool = False) -> None:
    row = next(row for row in r.CONFIG["targets"] if row["target"] == target)
    packages = {package_metadata(p)["name"]: p for p in r.DIST.glob("*.tgz")}
    sidecar = f"@{r.PROJECT}/node-{row['suffix']}"
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        r.write_json(root / "package.json", {"name": "release-consumer", "version": "1.0.0", "private": True})
        requirements = ([sidecar + "@" + r.version(), f"@{r.PROJECT}/node@{r.version()}"] if registry else
                        [str(packages[sidecar]), str(packages[f"@{r.PROJECT}/node"])])
        r.run("npm", "install", "--ignore-scripts", "--omit=optional", "--registry", "https://registry.npmjs.org", *requirements, cwd=root)
        # Preserve the test fixture layout while requiring the installed public package.
        tests = root / "crates" / f"{r.PROJECT}-node" / "tests"
        shutil.copytree(r.NODE / "tests", tests)
        shutil.copytree(r.ROOT / "tests/fixtures", root / "tests/fixtures")
        for file in tests.glob("*.cjs"):
            file.write_text(file.read_text().replace("require('..')", f"require('@{r.PROJECT}/node')"), encoding="utf-8")
        r.write_json(tests.parent / "package.json", {"version": r.version()})
        r.run("node", "--test", *[str(p) for p in tests.glob("*.cjs")], cwd=root)
        consumer = tests / "consumer.ts"
        consumer.write_text(consumer.read_text().replace("from '..'", f"from '@{r.PROJECT}/node'"), encoding="utf-8")
        r.run("node", str(r.NODE / "node_modules/typescript/bin/tsc"), "--noEmit", "--strict",
              "--target", "ES2022", "--moduleResolution", "node", str(consumer), cwd=root)
    node_version = r.run("node", "--version", capture=True)
    r.write_json(r.ROOT / f"target/runtime-tests/node-{target}-{node_version}.json",
                  {**r.source_record(), "target": target, "node": node_version, "passed": True})


def verify_attestations() -> None:
    """Verify public signatures, then record registry publisher identities."""
    repository = f"https://github.com/Lqm1/{r.PROJECT}"
    pydata = r.fetch_json(f"https://pypi.org/pypi/{r.PROJECT}/{r.version()}/json")
    for item in pydata["urls"]:
        r.run(sys.executable, "-m", "pypi_attestations", "verify", "pypi",
              "--repository", repository, item["url"])
        provenance = r.fetch_json(f"https://pypi.org/integrity/{r.PROJECT}/{r.version()}/{item['filename']}/provenance")
        publishers = [bundle["publisher"] for bundle in provenance["attestation_bundles"]]
        if not any(p.get("repository") == f"Lqm1/{r.PROJECT}" and p.get("workflow") == "publish.yml"
                   and p.get("environment") == "release" for p in publishers):
            raise ValueError("PyPI publisher identity differs from the reviewed configuration")
    with tempfile.TemporaryDirectory() as temporary:
        cwd = Path(temporary)
        r.write_json(cwd / "package.json", {"name": "provenance-consumer", "version": "1.0.0", "private": True})
        names = [package_metadata(path)["name"] + "@" + r.version() for path in r.DIST.glob("*.tgz")]
        r.run("npm", "install", "--force", "--ignore-scripts", *names, cwd=cwd)
        r.run("npm", "audit", "signatures", cwd=cwd)
    r.write_json(r.DIST / "registry-verification.json", {**r.source_record(), "pypi_attestations_verified": True,
                 "npm_installed_signatures_verified": True})


def stage_npm() -> None:
    if not os.environ.get("ACTIONS_ID_TOKEN_REQUEST_URL") or os.environ.get("NODE_AUTH_TOKEN"):
        raise ValueError("Formal npm staging requires GitHub OIDC without a static token")
    r.verify_manifest(r.DIST)
    paths = sorted(r.DIST.glob("*.tgz"), key=lambda p: package_metadata(p)["name"].endswith("/node"))
    records = []
    previous_run = os.environ.get("NPM_STAGE_RUN_ID", "")
    if previous_run:
        if not previous_run.isdecimal():
            raise ValueError("Invalid npm staging checkpoint run ID")
        repo = f"Lqm1/{r.PROJECT}"
        data = json.loads(r.run("gh", "api", f"repos/{repo}/actions/runs/{previous_run}", capture=True))
        if data.get("head_sha") != r.source_record()["source_commit"] or data.get("path") != ".github/workflows/publish.yml":
            raise ValueError("npm checkpoint belongs to another release")
        with tempfile.TemporaryDirectory() as temporary:
            r.run("gh", "run", "download", previous_run, "--repo", repo, "--name", "npm-stages", "--dir", temporary)
            checkpoint = json.loads((Path(temporary) / "npm-stages.json").read_text())
        if checkpoint["source"] != r.source_record():
            raise ValueError("npm checkpoint source identity mismatch")
        records = checkpoint["stages"]
    r.write_json(r.DIST / "npm-stages.json", {"source": r.source_record(), "stages": records})
    for path in paths:
        metadata = package_metadata(path)
        previous = [item for item in records if item["package"] == metadata["name"]]
        if previous:
            if len(previous) != 1 or previous[0]["sha256"] != r.digest(path):
                raise ValueError("npm staging checkpoint content mismatch")
            continue
        published = optional_json("https://registry.npmjs.org/" + metadata["name"].replace("/", "%2f") + "/" + r.version())
        if published is not None:
            integrity = "sha512-" + r.base64.b64encode(bytes.fromhex(r.digest(path, "sha512"))).decode()
            if published["dist"]["integrity"] != integrity:
                raise ValueError("Published version has different content")
            continue
        result = json.loads(r.run("npm", "stage", "publish", str(path), "--access", "public",
                                  "--provenance", "--ignore-scripts", "--json", capture=True))
        records.append({"package": metadata["name"], "sha256": r.digest(path), "result": result})
        r.write_json(r.DIST / "npm-stages.json", {"source": r.source_record(), "stages": records})
    print("The user must approve sidecars before root packages; no approval is automated")


def prepare_pypi() -> None:
    r.verify_manifest(r.DIST)
    existing = optional_json(f"https://pypi.org/pypi/{r.PROJECT}/{r.version()}/json")
    hashes = {item["filename"]: item["digests"]["sha256"] for item in (existing or {}).get("urls", [])}
    output = r.ROOT / "target/pypi-upload"
    output.mkdir(parents=True, exist_ok=False)
    for path in [*r.DIST.glob("*.tar.gz"), *(r.DIST / "python").glob("*.whl")]:
        if path.name in hashes:
            if hashes[path.name] != r.digest(path):
                raise ValueError("Existing PyPI artifact differs from this build")
        else:
            shutil.copy2(path, output / path.name)
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as stream:
        stream.write(f"pending={'true' if any(output.iterdir()) else 'false'}\n")


def sync_pr() -> None:
    data = json.loads(os.environ["PR"])
    if isinstance(data, list):
        if len(data) != 1:
            raise ValueError("Expected one release preparation PR")
        data = data[0]
    r.run("gh", "pr", "checkout", str(data["number"]))
    r.sync(None)
    if r.run("git", "status", "--porcelain", capture=True):
        r.run("git", "config", "user.name", "github-actions[bot]")
        r.run("git", "config", "user.email", "41898282+github-actions[bot]@users.noreply.github.com")
        r.run("git", "add", "Cargo.toml", "Cargo.lock", "crates")
        r.run("git", "commit", "-m", "chore: synchronize binding release versions")
        r.run("git", "push")


def finalize() -> None:
    r.registry_verify(r.DIST)
    verify_attestations()
    test_node("x86_64-unknown-linux-gnu", registry=True)
    test_python("manylinux2014_x86_64", None, registry=True)
    sha = r.run("git", "rev-parse", "HEAD", capture=True)
    tag = "v" + r.version()
    repository = f"Lqm1/{r.PROJECT}"
    existing = optional_json(f"https://api.github.com/repos/{repository}/git/ref/tags/{tag}")
    if existing:
        r.run("git", "fetch", "origin", "tag", tag)
        if r.run("git", "rev-parse", tag + "^{commit}", capture=True) != sha:
            raise ValueError("Release tag points to a different commit")
    else:
        r.run("gh", "api", f"repos/{repository}/git/refs", "--method", "POST",
              "-f", f"ref=refs/tags/{tag}", "-f", f"sha={sha}")
    published = optional_json(f"https://api.github.com/repos/{repository}/releases/tags/{tag}")
    if not published:
        r.run("gh", "release", "create", tag, "--repo", repository, "--verify-tag", "--draft", "--generate-notes")
    elif not published["draft"]:
        raise ValueError("Release is already public; do not mutate released assets")
    manifest = json.loads((r.DIST / "release-manifest.json").read_text())
    # GNU and MSVC wheels have the same PyPI filename. GNU goes into its own zip.
    artifacts = [str(r.DIST / item["path"]) for item in manifest["artifacts"] if "python-gnu" not in Path(item["path"]).parts]
    if len({Path(path).name for path in artifacts}) != len(artifacts):
        raise ValueError("GitHub asset filenames collide")
    r.run("gh", "release", "upload", tag, "--repo", repository, "--clobber", *artifacts,
          str(r.DIST / "release-manifest.json"), str(r.DIST / "SHA256SUMS"))
    r.run("gh", "release", "upload", tag, "--repo", repository, "--clobber", str(r.DIST / "registry-verification.json"))
    r.run("gh", "release", "edit", tag, "--repo", repository, "--draft=false")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("metadata", "preflight", "require-token", "download", "stage-npm", "prepare-pypi", "sync-pr", "finalize", "test-packaged-core"):
        sub.add_parser(name)
    cmd = sub.add_parser("wheel-inventory")
    cmd.add_argument("target")
    cmd.add_argument("python")
    cmd = sub.add_parser("assemble")
    cmd.add_argument("inputs", type=Path)
    cmd = sub.add_parser("test-node")
    cmd.add_argument("target")
    cmd = sub.add_parser("test-python")
    cmd.add_argument("platform")
    cmd.add_argument("--directory", type=Path)
    args = parser.parse_args()
    if args.command == "wheel-inventory":
        wheel_inventory(args.target, args.python)
    elif args.command == "assemble":
        assemble(args.inputs)
    elif args.command == "test-node":
        test_node(args.target)
    elif args.command == "test-python":
        test_python(args.platform, args.directory)
    else:
        {"metadata": metadata, "preflight": preflight, "require-token": require_token,
         "download": download, "stage-npm": stage_npm, "prepare-pypi": prepare_pypi,
         "sync-pr": sync_pr, "finalize": finalize,
         "test-packaged-core": lambda: r.run("cargo", "test", "--locked", "--lib", "--manifest-path",
             str(r.ROOT / f"target/package/{r.PROJECT}-{r.version()}/Cargo.toml"),
             "--target-dir", str(r.ROOT / "target"))}[args.command]()


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, r.subprocess.CalledProcessError) as error:
        sys.exit(f"Release stopped: {error}")
