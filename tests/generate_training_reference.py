"""Reproduce re-estimation fixtures with two disclosed C inference corrections."""
import argparse
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
    library = args.include.resolve() / "liblrhsmm"
    with tempfile.TemporaryDirectory(prefix="shiro-training-reference-") as temporary:
        root = Path(temporary)
        for name in ("shiro-rest.c", "cli-common.h"):
            (root / name).write_text((args.shiro / name).read_text()
                                    .replace("external/liblrhsmm/", "liblrhsmm/"))
        inference = (library / "inference.c").read_text()
        corrections = [
            ("if(dstst < nt)", "if(dstst >= 0 && dstst < nseg)"),
            ("if(i == 0) pprev = 0;", "if(t == 0 && j == 0) pprev = 0;"),
        ]
        for original, corrected in corrections:
            if inference.count(original) != 1:
                raise ValueError(f"unexpected original source: {original}")
            inference = inference.replace(original, corrected)
        (root / "inference.c").write_text(inference)
        executable = root / "shiro-rest-c"
        subprocess.run([
            args.cc, "-O2", "-DFP_TYPE=float", "-I" + str(args.include.resolve()),
            "-I" + str(library), "-I" + str(args.shiro.resolve()),
            str(root / "shiro-rest.c"), str(root / "inference.c"),
            str(args.shiro.resolve() / "external/cJSON/cJSON.c"),
            str(args.archive.resolve()), "-lm", "-o", str(executable),
        ], check=True)
        (root / "model.hsmm").write_bytes((fixtures / "init-c-aligned.hsmm").read_bytes())
        (root / "input.f").write_bytes((fixtures / "init-input.bin").read_bytes())
        (root / "seg.json").write_bytes((fixtures / "align-c-isolated.json").read_bytes())
        cases = [
            ("hsmm-one", []), ("hsmm-two", ["-n", "2", "-t", "0"]),
            ("daem", ["-n", "3", "-D", "-t", "0"]),
            ("hmm", ["-n", "2", "-g", "-P", "0.8", "-t", "0"]),
            ("isolated", ["-n", "2", "-i", "-t", "0"]), ("mean", ["-M"]),
            ("isolated-mean", ["-n", "2", "-i", "-M", "-t", "0"]),
            ("isolated-hmm", ["-n", "2", "-i", "-g", "-P", "0.8", "-t", "0"]),
            ("isolated-daem", ["-n", "3", "-i", "-D", "-t", "0"]),
        ]
        for name, flags in cases:
            output = subprocess.run([
                str(executable), "-m", "model.hsmm", "-s", "seg.json", "-l", "likelihood", *flags,
            ], cwd=root, capture_output=True, check=True)
            model_name = "hsmm-one" if name == "mean" else name
            references = {
                f"rest-c-{model_name}.hsmm": output.stdout,
                f"rest-c-{name}.likelihood": (root / "likelihood").read_bytes(),
            }
            for filename, data in references.items():
                target = fixtures / filename
                if args.check:
                    if target.read_bytes() != data:
                        raise ValueError(f"corrected C reference changed: {filename}")
                else:
                    target.write_bytes(data)
            print(f"{name}: corrected C model and likelihood verified")
        convergence = subprocess.run([
            str(executable), "-m", "model.hsmm", "-s", "seg.json", "-n", "5",
        ], cwd=root, capture_output=True, check=True)
        if convergence.stdout != (fixtures / "rest-c-hsmm-two.hsmm").read_bytes():
            raise ValueError("source convergence no longer stops after two updates")
        zero = subprocess.run([
            str(executable), "-m", "model.hsmm", "-s", "seg.json", "-n", "0",
        ], cwd=root, capture_output=True, check=True)
        if zero.stdout != (root / "model.hsmm").read_bytes():
            raise ValueError("source zero-iteration output changed")
        print("convergence and zero iterations: corrected C behavior verified")


if __name__ == "__main__":
    main()
