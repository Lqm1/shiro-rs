"""Reproduce wavsplit fixtures with the documented corrected-C/Lua toolbox."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def normalized(document):
    if "file_list" in document:
        for item in document["file_list"]:
            item["filename"] = "sample.param"
    if "dur_attr" in document:
        document["dur_attr"].sort(key=lambda item: item["index"])
    return document


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolbox", type=Path, required=True,
                        help="directory containing original Lua and the reference executables")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    toolbox = args.toolbox.resolve()
    for name in ["lua", "shiro-wavsplit.lua", "shiro-pm2md.lua", "shiro-mkseg.lua",
                 "shiro-seg2lab.lua", "shiro-wav2raw", "shiro-xxcc", "shiro-mkhsmm",
                 "shiro-init", "shiro-rest", "shiro-align"]:
        if not (toolbox / name).is_file():
            raise FileNotFoundError(toolbox / name)
    fixtures = Path(__file__).resolve().parent / "fixtures"
    with tempfile.TemporaryDirectory(prefix="shiro-utterance-reference-") as temporary:
        root = Path(temporary)
        shutil.copyfile(fixtures / "utterances-input.wav", root / "sample.wav")
        environment = dict(os.environ, PATH=str(toolbox) + os.pathsep + os.environ["PATH"])
        subprocess.run([str(toolbox / "lua"), str(toolbox / "shiro-wavsplit.lua"),
                        "./sample.wav", "-n", "2", "-N", "2"],
                       cwd=root, env=environment, check=True)
        stages = {
            "audio.bin": "raw", "features.bin": "param",
            "uninit.hsmm": "uninit.hsmm", "flat.hsmm": "flat.hsmm",
            "trained.hsmm": "trained.hsmm", "labels.txt": "txt",
            "phonemap.json": "phonemap", "definition.json": "modeldef",
            "initial.json": "init.segm", "aligned.json": "aligned.segm",
        }
        for name, suffix in stages.items():
            target = fixtures / ("utterances-c-" + name)
            content = (root / ("sample." + suffix)).read_bytes()
            if name.endswith(".json"):
                document = normalized(json.loads(content))
                if args.check:
                    if normalized(json.loads(target.read_bytes())) != document:
                        raise ValueError(f"reference changed: {name}")
                else:
                    target.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
            elif args.check:
                if target.read_bytes() != content:
                    raise ValueError(f"reference changed: {name}")
            else:
                target.write_bytes(content)
            print(f"{name}: corrected C/Lua reference verified")


if __name__ == "__main__":
    main()
