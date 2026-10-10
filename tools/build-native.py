"""Build one release target; never infer runtime coverage from compilation."""
from pathlib import Path
import argparse
import shutil
import release

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("target", choices=[row["target"] for row in release.CONFIG["targets"]])
args = parser.parse_args()
target = args.target
release.check()
command = ["cargo", "zigbuild" if "linux" in target else "build", "--locked", "--release",
           "--target", target + (".2.17" if "linux" in target else ""),
           "-p", release.PROJECT, "-p", release.PROJECT + "-capi", "-p", release.PROJECT + "-node"]
release.run(*command)
host = next(line.removeprefix("host: ") for line in release.run("rustc", "-vV", capture=True).splitlines() if line.startswith("host: "))
runtime_tested = target == host or (host == "x86_64-pc-windows-msvc" and target in {"i686-pc-windows-msvc", "x86_64-pc-windows-gnu"})
if runtime_tested:
    release.run("cargo", "test", "--locked", "--target", target, "-p", release.PROJECT, "-p", release.PROJECT + "-capi")
binary_name = release.PROJECT.replace("-", "_")
extension = ".dll" if "windows" in target else ".dylib" if "apple" in target else ".so"
prefix = "" if "windows" in target else "lib"
source = release.ROOT / "target" / target / "release" / (prefix + binary_name + "_node" + extension)
row = next(row for row in release.CONFIG["targets"] if row["target"] == target)
output = release.DIST / "addons"
output.mkdir(parents=True, exist_ok=True)
shutil.copy2(source, output / f"{binary_name}.{row['suffix']}.node")
release.archive_native(target, runtime_tested)
release.write_json(output / f"{target}.json", {**release.source_record(), "target": target,
                   "build_verified": True, "runtime_tested": runtime_tested,
                   "environment": release.build_environment()})
