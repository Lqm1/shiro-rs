"""Regenerate or check alignment fixtures against the pinned upstream C tool."""
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
    fixtures = Path(__file__).resolve().parent / "fixtures"
    with tempfile.TemporaryDirectory(prefix="shiro-align-reference-") as temporary:
        root = Path(temporary)
        for name in ("shiro-align.c", "cli-common.h"):
            source = (args.shiro / name).read_text()
            (root / name).write_text(source.replace("external/liblrhsmm/", "liblrhsmm/"))
        executable = root / "shiro-align-c"
        subprocess.run([
            args.cc, "-O2", "-DFP_TYPE=float", "-I" + str(args.include.resolve()),
            "-I" + str(args.shiro.resolve()), str(root / "shiro-align.c"),
            str(args.shiro.resolve() / "external/cJSON/cJSON.c"),
            str(args.archive.resolve()), "-lm", "-o", str(executable),
        ], check=True)
        (root / "input.f").write_bytes((fixtures / "init-input.bin").read_bytes())
        (root / "model.hsmm").write_bytes((fixtures / "init-c-aligned.hsmm").read_bytes())
        cases = [
            ("hsmm", []), ("hmm", ["-g", "-P", "0.8"]),
            ("hsmm-pruned", ["-p", "2", "-d", "8"]),
            ("hmm-pruned", ["-g", "-P", "0.5"]),
        ]
        requests = [(mode, suffix, flags + (["-i"] if mode == "isolated" else []))
                    for mode in ("embedded", "isolated") for suffix, flags in cases]
        requests.append(("four", "hmm", ["-g"]))
        for mode, suffix, flags in requests:
            source = fixtures / f"align-c-{mode}.json"
            input_path = root / "seg.json"
            input_path.write_bytes(source.read_bytes())
            output = subprocess.run([
                str(executable), "-m", "model.hsmm", "-s", "seg.json", *flags,
            ], cwd=root, capture_output=True, check=True)
            actual = json.loads(output.stdout)
            target = fixtures / f"align-c-{mode}-{suffix}.json"
            if args.check:
                if actual != json.loads(target.read_bytes()):
                    raise ValueError(f"C reference changed: {target.name}")
            else:
                target.write_text(json.dumps(actual, indent=2) + "\n")
            print(f"{target.name}: verified")


if __name__ == "__main__":
    main()
