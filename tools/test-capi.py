"""Compile existing C consumers and execute them and ctypes consumers when runnable."""
import argparse
import shutil
import struct
import sys

import release as r

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("target")
args = parser.parse_args()
target = args.target
directory = r.ROOT / "target" / target / "release"
include = r.ROOT / "crates" / f"{r.PROJECT}-capi/include"
name = r.PROJECT.replace("-", "_") + "_capi"
host = next(line.removeprefix("host: ") for line in r.run("rustc", "-vV", capture=True).splitlines() if line.startswith("host: "))
runnable = host == target or (host == "x86_64-pc-windows-msvc" and target in {"i686-pc-windows-msvc", "x86_64-pc-windows-gnu"})
records = []
for source in sorted((r.ROOT / "tests").glob("c_api*smoke.c")):
    output = directory / (source.stem + (".exe" if "windows" in target else ""))
    if target.endswith("msvc"):
        r.run("cl", "/nologo", "/std:c11", f"/I{include}", str(source), str(directory / f"{name}.dll.lib"), f"/Fe:{output}", f"/Fo:{directory / (source.stem + '.obj')}")
    else:
        compiler = "gcc" if "windows" in target else "cc"
        # Linux i686 is build-only on the x86_64 runner; Zig supplies its C/sysroot.
        command = ["zig", "cc", "-target", "x86-linux-gnu.2.17"] if target.startswith("i686-unknown-linux") else [compiler]
        r.run(*command, "-std=c11", "-I" + str(include), str(source), "-L" + str(directory),
              "-l" + name, "-lm", *([] if "windows" in target else ["-Wl,-rpath," + str(directory)]), "-o", str(output))
    arguments = []
    can_run = runnable
    if source.name == "c_api_plot_figure_smoke.c":
        program = shutil.which("gnuplot")
        can_run = can_run and program is not None
        plot = r.ROOT / "target/capi-plot-consumer"
        plot.mkdir(parents=True, exist_ok=True)
        arguments = [program or "gnuplot", str(plot)]
    if can_run:
        r.run(str(output), *arguments)
    records.append({"consumer": source.name, "compiled": True, "runtime_tested": can_run})

python_matches = runnable and (struct.calcsize("P") == (4 if target.startswith("i686") else 8))
extension = ".dll" if "windows" in target else ".dylib" if "apple" in target else ".so"
library = directory / (("" if "windows" in target else "lib") + name + extension)
for source in sorted((r.ROOT / "tests").glob("c_api*smoke.py")):
    arguments = []
    can_run = python_matches
    if source.name == "c_api_plot_figure_smoke.py":
        program = shutil.which("gnuplot")
        can_run = can_run and program is not None
        arguments = [program or "gnuplot", str(r.ROOT / "target/capi-plot-python")]
    if can_run:
        r.run(sys.executable, str(source), str(library), *arguments)
    records.append({"consumer": source.name, "runtime_tested": can_run})
r.write_json(r.DIST / "addons" / f"capi-{target}.json", {**r.source_record(), "target": target, "consumers": records})
