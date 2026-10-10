"""Build, inspect and synchronize release artifacts without implicit publishing."""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import platform
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib
import urllib.error
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
CONFIG = json.loads((ROOT / "tools/release-config.json").read_text())
PROJECT = CONFIG["project"]
NODE = ROOT / "crates" / f"{PROJECT}-node"
PYTHON = ROOT / "crates" / f"{PROJECT}-python"
DIST = ROOT / "target/distribution/release"
VERSION_PATTERN = r"\d+\.\d+\.\d+(?:-alpha\.\d+)?"


def run(*args: str, cwd: Path = ROOT, capture: bool = False) -> str:
    command = list(args)
    if command[0] in {"npm", "npx"} and os.name == "nt":
        command[0] += ".cmd"
    result = subprocess.run(command, cwd=cwd, check=True, text=True,
                            stdout=subprocess.PIPE if capture else None)
    return result.stdout.strip() if capture else ""


def read_toml(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def version() -> str:
    return read_toml(ROOT / "Cargo.toml")["workspace"]["package"]["version"]


def write_json(path: Path, data: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def digest(path: Path, algorithm: str = "sha256") -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, algorithm).hexdigest()


def assert_clean() -> str:
    if run("git", "status", "--porcelain", capture=True):
        raise ValueError("Release builds require a clean committed checkout")
    return run("git", "rev-parse", "HEAD", capture=True)


def check(registry: bool = False, frozen: bool = False) -> None:
    current = version()
    if not re.fullmatch(VERSION_PATTERN, current):
        raise ValueError("Only stable or alpha versions are supported")
    for manifest in sorted((ROOT / "crates").glob("*/Cargo.toml")):
        data = read_toml(manifest)
        package = data["package"]
        if package["version"] != {"workspace": True}:
            raise ValueError(f"Version must inherit from workspace: {manifest}")
        if package["name"] != PROJECT and package.get("publish") is not False:
            raise ValueError(f"Only the core crate can be published: {manifest}")
        for name, dependency in data.get("dependencies", {}).items():
            if name == PROJECT and dependency["version"] != current:
                raise ValueError(f"Local core dependency version mismatch: {manifest}")
            if registry and name in CONFIG["core_dependencies"]:
                if any(key in dependency for key in ("git", "rev", "branch", "tag", "path")):
                    raise ValueError(f"Core dependency is not registry-only: {name}")
                resolved = locked_dependency(name)
                if resolved.get("source") != "registry+https://github.com/rust-lang/crates.io-index":
                    raise ValueError(f"Core dependency is not locked to crates.io: {name}")
                published = fetch_json(f"https://crates.io/api/v1/crates/{name}/{resolved['version']}")
                if published["version"]["checksum"] != resolved.get("checksum"):
                    raise ValueError(f"Core dependency checksum mismatch: {name}")
    package = json.loads((NODE / "package.json").read_text())
    if package["name"] != f"@{PROJECT}/node" or package["version"] != current:
        raise ValueError("npm name/version mismatch")
    if package["napi"]["targets"] != [row["target"] for row in CONFIG["targets"]]:
        raise ValueError("npm target inventory mismatch")
    lock = json.loads((NODE / "package-lock.json").read_text())
    if lock["version"] != current or lock["packages"][""]["name"] != package["name"]:
        raise ValueError("npm lockfile metadata mismatch")
    pyproject = read_toml(PYTHON / "pyproject.toml")
    python_manifest = read_toml(PYTHON / "Cargo.toml")
    if python_manifest.get("features", {}).get("cross-windows") != ["pyo3/generate-import-lib"]:
        raise ValueError("Python bindings must enable import libraries for Windows cross builds")
    if pyproject["project"]["name"] != PROJECT or pyproject["project"]["requires-python"] != ">=3.11":
        raise ValueError("Python distribution metadata mismatch")
    wasm = json.loads((ROOT / "crates" / f"{PROJECT}-wasm/package.json").read_text())
    if wasm["name"] != f"@{PROJECT}/wasm" or wasm["version"] != current:
        raise ValueError("WASM distribution metadata mismatch")
    if frozen:
        sha = assert_clean()
        if os.environ.get("RELEASE_SHA") not in (None, sha):
            raise ValueError("Checkout does not match the requested release commit")
    print(f"Release metadata verified: {PROJECT} {current}")


def sync(new_version: str | None) -> None:
    current = new_version or version()
    if not re.fullmatch(VERSION_PATTERN, current):
        raise ValueError("Invalid release version")
    manifest = ROOT / "Cargo.toml"
    text = manifest.read_text()
    text = re.sub(r'(?m)^version = "[^"]+"$', f'version = "{current}"', text)
    manifest.write_text(text, encoding="utf-8")
    for manifest in (ROOT / "crates").glob("*/Cargo.toml"):
        text = manifest.read_text()
        text = re.sub(rf'({re.escape(PROJECT)} = \{{[^\n]*?version = ")[^"]+("[^\n]*\}})',
                      lambda match: match[1] + current + match[2], text)
        manifest.write_text(text, encoding="utf-8")
    for manifest in (NODE / "package.json", ROOT / "crates" / f"{PROJECT}-wasm/package.json"):
        data = json.loads(manifest.read_text())
        data["version"] = current
        write_json(manifest, data)
    lock_path = NODE / "package-lock.json"
    lock = json.loads(lock_path.read_text())
    lock["name"] = f"@{PROJECT}/node"
    lock["version"] = current
    lock["packages"][""]["name"] = lock["name"]
    lock["packages"][""]["version"] = current
    lock["packages"][""]["engines"] = json.loads((NODE / "package.json").read_text())["engines"]
    write_json(lock_path, lock)
    run("cargo", "update", "--workspace")
    check()


def patch_loader(path: Path | None = None) -> None:
    path = path or NODE / "index.js"
    text = path.read_text()
    if "NAPI_RS_FORCE_WASI" in text:
        text, count = re.subn(r"(?ms)^if \(!nativeBinding \|\| process\.env\.NAPI_RS_FORCE_WASI\) \{.*?^\}\n\n(?=if \(!nativeBinding\) \{)", "", text, count=1)
        if count != 1 or "NAPI_RS_FORCE_WASI" in text:
            raise ValueError("napi-rs WASI loader format changed; review native-only packaging")
    marker = "  } else if (process.platform === 'linux') {\n"
    branch = f"""    if (process.arch === 'ia32') {{
      try {{
        return require('./{PROJECT.replace('-', '_')}.linux-ia32-gnu.node')
      }} catch (error) {{
        loadErrors.push(error)
      }}
      try {{
        return require('@{PROJECT}/node-linux-ia32-gnu')
      }} catch (error) {{
        loadErrors.push(error)
      }}
      return null
    }}
"""
    if text.count(marker) != 1:
        raise ValueError("napi-rs loader format changed; review the ia32 adapter")
    if f"@{PROJECT}/node-linux-ia32-gnu" not in text:
        text = text.replace(marker, marker + branch)
    path.write_text(text, encoding="utf-8")


def pack_node(artifact_root: Path) -> None:
    check()
    DIST.mkdir(parents=True, exist_ok=True)
    npm_dir = DIST / "npm"
    run("npx", "--no-install", "napi", "create-npm-dirs", "--npm-dir", str(npm_dir), cwd=NODE)
    generated = ROOT / "target/release-node-generated"
    patch_loader(generated / "index.js")
    package = json.loads((NODE / "package.json").read_text())
    base = DIST / "node-package"
    base.mkdir(parents=True, exist_ok=True)
    for name in ("index.js", "index.d.ts", "README.md", "LICENSE"):
        shutil.copy2((generated if name in {"index.js", "index.d.ts"} else NODE) / name, base / name)
    package["files"] = ["index.js", "index.d.ts", "README.md", "LICENSE", "release-source.json"]
    package.pop("scripts", None)
    package.pop("devDependencies", None)
    package["optionalDependencies"] = {}
    for row in CONFIG["targets"]:
        matches = list(artifact_root.rglob(f"*.{row['suffix']}.node"))
        if len(matches) != 1:
            raise ValueError(f"Expected exactly one addon for {row['target']}, found {len(matches)}")
        directory = npm_dir / row["suffix"]
        filename = directory / matches[0].name
        shutil.copy2(matches[0], filename)
        shutil.copy2(ROOT / "LICENSE", directory / "LICENSE")
        metadata = json.loads((directory / "package.json").read_text())
        metadata["files"] += ["LICENSE", "release-source.json"]
        if row["target"].endswith("linux-gnu"):
            metadata["libc"] = ["glibc"]
        write_json(directory / "package.json", metadata)
        write_json(directory / "release-source.json", source_record())
        package["optionalDependencies"][metadata["name"]] = version()
        run("npm", "pack", "--ignore-scripts", "--pack-destination", str(DIST), cwd=directory)
    write_json(base / "package.json", package)
    write_json(base / "release-source.json", source_record())
    run("npm", "pack", "--ignore-scripts", "--pack-destination", str(DIST), cwd=base)


def source_record() -> dict:
    return {"project": PROJECT, "version": version(),
            "source_commit": run("git", "rev-parse", "HEAD", capture=True),
            "repository": f"https://github.com/Lqm1/{PROJECT}"}


def build_environment() -> dict:
    return {"os": platform.platform(), "machine": platform.machine(), "python": sys.version,
            "rustc": run("rustc", "-vV", capture=True),
            "runner_image": os.environ.get("ImageOS"), "runner_image_version": os.environ.get("ImageVersion"),
            "workflow_run": os.environ.get("GITHUB_RUN_ID")}


def build_wasm() -> None:
    check()
    output = DIST / "wasm-package"
    output.mkdir(parents=True, exist_ok=True)
    run("cargo", "build", "--locked", "--release", "--target", "wasm32-unknown-unknown", "-p", f"{PROJECT}-wasm")
    binary = ROOT / "target/wasm32-unknown-unknown/release" / f"{PROJECT.replace('-', '_')}_wasm.wasm"
    for target in ("web", "bundler"):
        run("wasm-bindgen", str(binary), "--target", target, "--out-name", "index", "--out-dir", str(output / target))
    # A shared binary is only safe when wasm-bindgen produced identical bytes.
    web = output / "web/index_bg.wasm"
    bundler = output / "bundler/index_bg.wasm"
    if digest(web) == digest(bundler):
        shared = output / "shared"
        shared.mkdir(exist_ok=True)
        shutil.move(str(web), shared / "index_bg.wasm")
        bundler.unlink()
        # wasm-bindgen's binary imports ./index_bg.js relative to its own location.
        # Keep that import namespace resolvable by bundlers after sharing the binary.
        (shared / "index_bg.js").write_text("export * from '../bundler/index_bg.js';\n", encoding="utf-8")
        for name in ("web/index.js", "bundler/index.js"):
            path = output / name
            text = path.read_text().replace("./index_bg.wasm", "../shared/index_bg.wasm")
            text = text.replace("new URL('index_bg.wasm', import.meta.url)", "new URL('../shared/index_bg.wasm', import.meta.url)")
            path.write_text(text, encoding="utf-8")
    shutil.copy2(ROOT / "crates" / f"{PROJECT}-wasm/package.json", output / "package.json")
    shutil.copy2(ROOT / "LICENSE", output / "LICENSE")
    shutil.copy2(ROOT / "crates" / f"{PROJECT}-wasm/README.md", output / "README.md")
    write_json(output / "release-source.json", source_record())
    run("npm", "pack", "--ignore-scripts", "--pack-destination", str(DIST), cwd=output)


def archive_native(target: str, runtime_tested: bool = False) -> None:
    row = next(item for item in CONFIG["targets"] if item["target"] == target)
    output = DIST / target
    output.mkdir(parents=True, exist_ok=True)
    build = ROOT / "target" / target / "release"
    capi = PROJECT.replace("-", "_") + "_capi"
    filenames = ([f"{capi}.dll", f"{capi}.lib", f"{capi}.dll.lib"] if target.endswith("msvc") else
                 [f"{capi}.dll", f"lib{capi}.a", f"lib{capi}.dll.a"] if "windows" in target else
                 [f"lib{capi}.dylib", f"lib{capi}.a"] if "apple" in target else
                 [f"lib{capi}.so", f"lib{capi}.a"])
    for filename in filenames:
        shutil.copy2(build / filename, output / filename)
    shutil.copytree(ROOT / "crates" / f"{PROJECT}-capi/include", output / "include", dirs_exist_ok=True)
    for filename in ("LICENSE", "README.md"):
        shutil.copy2(ROOT / filename, output / filename)
    for filename in CONFIG["binaries"]:
        name = filename + (".exe" if "windows" in target else "")
        shutil.copy2(build / name, output / name)
    write_json(output / "release-source.json", {**source_record(), "target": target,
               "runtime_tested": runtime_tested, "platform": row["suffix"]})
    archive = DIST / f"{PROJECT}-{version()}-{target}.zip"
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as stream:
        for path in sorted(output.rglob("*")):
            if path.is_file():
                stream.write(path, path.relative_to(output))


def fetch_json(url: str) -> dict:
    request = urllib.request.Request(url, headers={"User-Agent": "SHIRO-release-tool/0.1 (read-only)"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def manifest(artifact_root: Path) -> None:
    check(registry=True, frozen=True)
    files = []
    for path in sorted(artifact_root.rglob("*")):
        if path.is_file() and (path.suffix in {".tgz", ".whl", ".zip", ".crate"} or path.name.endswith(".tar.gz")):
            files.append({"path": path.relative_to(artifact_root).as_posix(),
                          "sha256": digest(path), "size": path.stat().st_size})
    if not files:
        raise ValueError("No release distributions found")
    write_json(artifact_root / "release-manifest.json", {**source_record(), "artifacts": files,
               "targets": CONFIG["targets"], "python_versions": CONFIG["python_versions"],
               "node_versions": CONFIG["node_versions"], "core_dependencies": core_dependencies(),
               "toolchain": run("rustc", "--version", capture=True),
               "runtime_verification": [json.loads(path.read_text()) for path in sorted((artifact_root / "runtime-tests").glob("*.json"))],
               "build_records": [json.loads(path.read_text()) for path in sorted((artifact_root / "build-records").rglob("*.json"))]})
    (artifact_root / "SHA256SUMS").write_text("".join(f"{item['sha256']}  {item['path']}\n" for item in files), encoding="utf-8")


def core_dependencies() -> dict:
    dependencies = read_toml(ROOT / "crates" / PROJECT / "Cargo.toml")["dependencies"]
    return {name: {"requirement": dependencies[name] if isinstance(dependencies[name], str)
                  else dependencies[name]["version"], **locked_dependency(name)}
            for name in CONFIG["core_dependencies"]}


def locked_dependency(name: str) -> dict:
    packages = [package for package in read_toml(ROOT / "Cargo.lock")["package"] if package["name"] == name]
    if len(packages) != 1:
        raise ValueError(f"Expected one locked core dependency: {name}")
    return {key: value for key, value in packages[0].items() if key in {"version", "source", "checksum"}}


def verify_manifest(artifact_root: Path) -> None:
    data = json.loads((artifact_root / "release-manifest.json").read_text())
    if data["source_commit"] != run("git", "rev-parse", "HEAD", capture=True) or data["version"] != version():
        raise ValueError("Artifact source or version differs from the checkout")
    for item in data["artifacts"]:
        path = (artifact_root / item["path"]).resolve()
        if not path.is_relative_to(artifact_root.resolve()) or digest(path) != item["sha256"]:
            raise ValueError(f"Artifact checksum mismatch: {item['path']}")
    expected = {row["target"] for row in CONFIG["targets"]}
    found = {target for target in expected if any(target in item["path"] and item["path"].endswith(".zip") for item in data["artifacts"])}
    if found != expected:
        raise ValueError("Missing native target archives")
    tarballs = list(artifact_root.rglob("*.tgz"))
    names = {}
    for path in tarballs:
        with tarfile.open(path) as archive:
            metadata = json.load(archive.extractfile("package/package.json"))
            source = json.load(archive.extractfile("package/release-source.json"))
        if metadata["version"] != version() or source != source_record():
            raise ValueError(f"npm artifact provenance mismatch: {path}")
        if metadata["name"] in names:
            raise ValueError("Duplicate npm package")
        names[metadata["name"]] = metadata
    required = {f"@{PROJECT}/node", f"@{PROJECT}/wasm"} | {f"@{PROJECT}/node-{row['suffix']}" for row in CONFIG["targets"]}
    if set(names) != required:
        raise ValueError("Incomplete npm package inventory")
    for row in CONFIG["targets"]:
        if names[f"@{PROJECT}/node"]["optionalDependencies"][f"@{PROJECT}/node-{row['suffix']}"] != version():
            raise ValueError("Optional dependency version mismatch")
        platform, architecture, *_ = row["suffix"].split("-")
        sidecar = names[f"@{PROJECT}/node-{row['suffix']}"]
        if sidecar.get("os") != [platform] or sidecar.get("cpu") != [architecture]:
            raise ValueError("OS sidecar constraints mismatch")
        if platform == "linux" and sidecar.get("libc") != ["glibc"]:
            raise ValueError("Linux sidecar must require glibc")
    template = json.loads((ROOT / "crates" / f"{PROJECT}-wasm/package.json").read_text())
    if names[f"@{PROJECT}/wasm"]["exports"] != template["exports"]:
        raise ValueError("WASM entrypoints differ from the reviewed package exports")
    standard = [path for path in artifact_root.rglob("*.whl") if "python-gnu" not in path.parts]
    gnu = list((artifact_root / "python-gnu").glob("*.whl"))
    if len(standard) != len(CONFIG["wheel_platforms"]) * len(CONFIG["python_versions"]) or len(gnu) != len(CONFIG["python_versions"]):
        raise ValueError("Python standard/GNU wheel count mismatch")
    for path in standard + gnu:
        tags = path.name.split("-")[-3:]
        if tags[0] != tags[1] or not tags[0].startswith("cp"):
            raise ValueError("Only version-specific GIL-enabled CPython wheels are accepted")
    wheel_tags = {path.name.split("-")[-3] + ":" + path.name.split("-")[-1].removesuffix(".whl") for path in artifact_root.rglob("*.whl") if "python-gnu" not in path.parts}
    for platform in CONFIG["wheel_platforms"]:
        for python in CONFIG["python_versions"]:
            tag = "cp" + python.replace(".", "")
            if not any(item.startswith(tag + ":") and platform in item for item in wheel_tags):
                raise ValueError(f"Missing Python wheel: {tag} {platform}")
    if len(list(artifact_root.rglob("*.tar.gz"))) != 1:
        raise ValueError("Expected one Python sdist")
    crates = list(artifact_root.glob("*.crate"))
    if len(crates) != 1:
        raise ValueError("Expected the exact core Cargo distribution")
    with tarfile.open(crates[0]) as archive:
        vcs = json.load(archive.extractfile(f"{PROJECT}-{version()}/.cargo_vcs_info.json"))
        if vcs["git"]["sha1"] != data["source_commit"] or vcs["git"].get("dirty", False):
            raise ValueError("Cargo distribution was not built from the frozen commit")
    if not (artifact_root / f"{PROJECT}-{version()}-sources.zip").is_file():
        raise ValueError("Missing corresponding source and dependency licenses")
    archive_path = artifact_root / f"{PROJECT}-{version()}-python-windows-gnu.zip"
    with zipfile.ZipFile(archive_path) as archive:
        for path in gnu:
            if hashlib.sha256(archive.read(path.name)).hexdigest() != digest(path):
                raise ValueError("GNU Python archive differs from its build outputs")
    print("Release inventory, checksums and npm source records verified")


def registry_verify(artifact_root: Path) -> None:
    verify_manifest(artifact_root)
    for path in artifact_root.rglob("*.tgz"):
        with tarfile.open(path) as archive:
            package = json.load(archive.extractfile("package/package.json"))
        url = "https://registry.npmjs.org/" + package["name"].replace("/", "%2f") + "/" + version()
        published = fetch_json(url)
        expected = "sha512-" + base64.b64encode(bytes.fromhex(digest(path, "sha512"))).decode()
        if published["dist"]["integrity"] != expected:
            raise ValueError(f"Published npm artifact mismatch: {package['name']}")
        if not published["dist"].get("attestations", {}).get("provenance"):
            raise ValueError(f"Missing npm provenance: {package['name']}")
    python_version = version().replace("-alpha.", "a")
    published = fetch_json(f"https://pypi.org/pypi/{PROJECT}/{python_version}/json")
    hashes = {item["filename"]: item["digests"]["sha256"] for item in published["urls"]}
    for path in artifact_root.rglob("*"):
        if path.is_file() and (path.suffix == ".whl" or path.name.endswith(".tar.gz")) and "python-gnu" not in path.parts:
            if hashes.get(path.name) != digest(path):
                raise ValueError(f"Published Python artifact mismatch: {path.name}")
    crate = fetch_json(f"https://crates.io/api/v1/crates/{PROJECT}/{version()}")
    local = next(artifact_root.glob("*.crate"))
    if crate["version"]["checksum"] != digest(local):
        raise ValueError("Published core crate differs from the reviewed distribution")
    print("Published registry distributions match the frozen artifacts")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    command = sub.add_parser("check")
    command.add_argument("--registry", action="store_true")
    command.add_argument("--frozen", action="store_true")
    command = sub.add_parser("sync")
    command.add_argument("--version")
    sub.add_parser("patch-loader")
    sub.add_parser("build-wasm")
    command = sub.add_parser("archive-native")
    command.add_argument("target", choices=[row["target"] for row in CONFIG["targets"]])
    for name in ("pack-node", "manifest", "verify-manifest", "registry-verify"):
        command = sub.add_parser(name)
        command.add_argument("artifact_root", type=Path)
    args = parser.parse_args()
    if args.command == "check":
        check(args.registry, args.frozen)
    elif args.command == "sync":
        sync(args.version)
    elif args.command == "patch-loader":
        patch_loader()
    elif args.command == "build-wasm":
        build_wasm()
    elif args.command == "archive-native":
        archive_native(args.target)
    else:
        {"pack-node": pack_node, "manifest": manifest, "verify-manifest": verify_manifest,
         "registry-verify": registry_verify}[args.command](args.artifact_root)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(f"Release stopped: {error}")
