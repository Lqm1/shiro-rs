"""Regenerate or check model untying references with the original C tool."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shiro", type=Path, required=True)
    parser.add_argument("--include", type=Path, required=True)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    tests = Path(__file__).resolve().parent
    fixtures = tests / "fixtures"
    with tempfile.TemporaryDirectory(prefix="shiro-untie-reference-") as temporary:
        root = Path(temporary)
        for name in ("shiro-untie.c", "cli-common.h"):
            (root / name).write_text((args.shiro / name).read_text()
                                    .replace("external/liblrhsmm/", "liblrhsmm/"))
        common = [args.cc, "-O2", "-DFP_TYPE=float", "-I" + str(args.include.resolve()),
                  "-I" + str(args.shiro.resolve()), "-I" + str(root)]
        libraries = [str(args.shiro.resolve() / "external/cJSON/cJSON.c"),
                     str(args.archive.resolve()), "-lm"]
        executable = root / "shiro-untie-c"
        weighted_driver = root / "weighted-driver"
        subprocess.run([*common, str(root / "shiro-untie.c"), *libraries,
                        "-o", str(executable)], check=True)
        subprocess.run([*common, str(tests / "untie_weights_oracle.c"), *libraries,
                        "-o", str(weighted_driver)], check=True)
        (root / "model.hsmm").write_bytes((fixtures / "init-c-aligned.hsmm").read_bytes())
        (root / "seg.json").write_bytes((fixtures / "align-c-isolated.json").read_bytes())
        output = subprocess.run([str(executable), "-m", "model.hsmm", "-s", "seg.json",
                                 "-o", "untied.json", "-O", "summary.txt"],
                                cwd=root, capture_output=True, check=True)
        weighted = subprocess.run([str(weighted_driver), "model.hsmm"], cwd=root,
                                  capture_output=True, check=True).stdout
        (root / "weighted.hsmm").write_bytes(weighted)
        reset = subprocess.run([str(executable), "-m", "weighted.hsmm", "-s", "seg.json"],
                               cwd=root, capture_output=True, check=True).stdout
        if reset != output.stdout:
            raise ValueError("original C no longer reproduces the stream-weight reset")
        references = {
            "untie-c.hsmm": output.stdout,
            "untie-c-weighted-input.hsmm": weighted,
            "untie-c-summary.txt": (root / "summary.txt").read_bytes(),
            "untie-c.json": (root / "untied.json").read_bytes(),
        }
        for name, data in references.items():
            target = fixtures / name
            if args.check:
                expected = target.read_bytes()
                equal = json.loads(data) == json.loads(expected) if name.endswith(".json") else data == expected
                if not equal:
                    raise ValueError(f"C reference changed: {name}")
            else:
                target.write_bytes(data)
            print(f"{name}: verified")
        print("weighted model: original C resets both stream weights to one")


if __name__ == "__main__":
    main()
