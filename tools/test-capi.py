"""Compile existing C consumers and execute them and ctypes consumers when runnable."""
import argparse
import hashlib
import os
import shutil
import struct
import sys
import tarfile
import urllib.request

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


def prepare_external_consumers():
    """Build real Lua and the existing SPTK process-protocol fixture."""
    if not runnable:
        return
    tools = directory / "external-consumer-tools"
    tools.mkdir(parents=True, exist_ok=True)
    suffix = ".exe" if "windows" in target else ""
    fixture = tools / ("frame" + suffix)
    if target.endswith("msvc"):
        r.run("cl", "/nologo", "/std:c11", str(r.ROOT / "tests/sptk_fixture.c"),
              f"/Fe:{fixture}", f"/Fo:{tools / 'sptk_fixture.obj'}")
    else:
        r.run("gcc" if "windows" in target else "cc", "-std=c11",
              str(r.ROOT / "tests/sptk_fixture.c"), "-o", str(fixture))
    for name in ("mfcc", "delta", "frame-fail", "mfcc-fail", "delta-fail"):
        shutil.copy2(fixture, tools / (name + suffix))

    # Official archive checksum: https://www.lua.org/ftp/.
    # Platform configuration follows the bundled src/Makefile and luaconf.h.
    archive = tools / "lua-5.4.9.tar.gz"
    checksum = "2335b6c582a52654f94612bf10d2f4672805d05329aa6568b1d8cd9e5c6fb8e6"
    if not archive.exists():
        with urllib.request.urlopen("https://www.lua.org/ftp/lua-5.4.9.tar.gz", timeout=60) as response:
            archive.write_bytes(response.read())
    if hashlib.sha256(archive.read_bytes()).hexdigest() != checksum:
        raise RuntimeError("Official Lua source archive checksum mismatch")
    with tarfile.open(archive) as source:
        source.extractall(tools, filter="data")
    sources = sorted(str(path) for path in (tools / "lua-5.4.9/src").glob("*.c") if path.name != "luac.c")
    lua = tools / ("lua" + suffix)
    if target.endswith("msvc"):
        r.run("cl", "/nologo", "/O2", *sources, f"/Fe:{lua}", f"/Fo:{tools}{os.sep}")
    else:
        flags = (["-DLUA_USE_LINUX", "-ldl"] if "linux" in target else
                 ["-DLUA_USE_MACOSX"] if "apple" in target else [])
        r.run("gcc" if "windows" in target else "cc", "-O2", *sources, *flags, "-lm", "-o", str(lua))
    r.run(str(lua), "-v")
    os.environ["SHIRO_TEST_SPTK_DIRECTORY"] = str(tools)
    os.environ["SHIRO_TEST_LUA"] = str(lua)
    return {"lua_version": "5.4.9", "lua_source_sha256": checksum,
            "sptk": "process-protocol fixture, not SPTK feature mathematics"}


external_tools = prepare_external_consumers()
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
r.write_json(r.DIST / "addons" / f"capi-{target}.json", {**r.source_record(), "target": target,
             "external_tools": external_tools, "consumers": records})
