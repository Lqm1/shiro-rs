"""Reproduce CMU SLT features and alignments with the pinned C/Lua toolbox."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

MODEL_SHA256 = "2467701cd9732eb702819d7418100d499966ea5f02fdc57977ea63a31cc00743"
WAVE_SHA256 = [
    "7862eb2cccb56875910f6bf46f9b8d26dac36e7672829aabe3956b0837ae122e",
    "d09c9367d7e756cb5f6854d4e8a279e6e6e543fafeb4d04b32757c639f7f38d3",
    "0b32e00846e132826f46d7e5cabb2d8b370a5dc654bac0447cb63285f81cc413",
]


def run(arguments, directory):
    result = subprocess.run([str(arg) for arg in arguments], cwd=directory, capture_output=True)
    if result.returncode:
        message = result.stderr.decode(errors="replace")
        raise RuntimeError(f"{arguments[0]} exited {result.returncode}: {message}")
    return result.stdout


def normalize(content):
    document = json.loads(content)
    for item in document["file_list"]:
        item["filename"] = item["filename"].replace("\\", "/").split("/")[-1]
    return document


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolbox", type=Path, required=True)
    parser.add_argument("--current-model", type=Path, required=True,
                        help="historical model saved with variance-floor slots; see the compatibility document")
    parser.add_argument("--write", action="store_true",
                        help="replace derived reference fixtures; the default only checks")
    args = parser.parse_args()
    toolbox = args.toolbox.resolve()
    model = args.current_model.resolve()
    if hashlib.sha256(model.read_bytes()).hexdigest() != MODEL_SHA256:
        raise ValueError("current-schema reference model changed")
    fixtures = Path(__file__).resolve().parent / "fixtures"
    with tempfile.TemporaryDirectory(prefix="shiro-real-audio-reference-") as temporary:
        root = Path(temporary)
        for index in range(3):
            name = f"arctic_a{index + 1:04d}"
            source = fixtures / ("cmu-slt-" + name + ".wav")
            if hashlib.sha256(source.read_bytes()).hexdigest() != WAVE_SHA256[index]:
                raise ValueError(f"waveform changed: {source}")
            shutil.copyfile(source, root / (name + ".wav"))
            run([toolbox / "shiro-wav2raw", "-r", "16000", root / (name + ".wav")], root)
            content = run([toolbox / "shiro-xxcc", "-l", "512", "-p", "80", "-m", "12",
                           "-s", "16", "-da", root / (name + ".raw")], root)
            (root / (name + ".param")).write_bytes(content)
            target = fixtures / ("cmu-slt-" + name + ".param")
            if args.write:
                target.write_bytes(content)
            elif target.read_bytes() != content:
                raise ValueError(f"C features changed: {target.name}")
        # Original Lua on Linux retains CR in the bare sil token. Normalize
        # only the temporary text input; the upstream fixture stays unchanged.
        phoneset = root / "phoneset.csv"
        phoneset.write_bytes((fixtures / "cmu-arpabet-phoneset.csv").read_bytes().replace(b"\r\n", b"\n"))
        phonemap = run([toolbox / "lua", toolbox / "shiro-mkpm.lua",
                       phoneset, "-s", "5", "-S", "3"], root)
        (root / "phonemap.json").write_bytes(phonemap)
        initial = run([toolbox / "lua", toolbox / "shiro-mkseg.lua",
                       fixtures / "cmu-slt-index.csv", "-m", root / "phonemap.json",
                       "-d", root, "-e", ".param", "-n", "36", "-L", "sil", "-R", "sil"], root)
        try:
            json.loads(initial)
        except ValueError as error:
            raise ValueError(f"Lua produced invalid segmentation: {initial[:2000]!r}") from error
        (root / "initial.json").write_bytes(initial)
        for mode, arguments in [
            ("hmm", ["-s", root / "initial.json", "-g"]),
            ("hsmm", ["-s", root / "hmm.json", "-p", "10", "-d", "50"]),
        ]:
            content = run([toolbox / "shiro-align", "-m", model, *arguments], root)
            (root / (mode + ".json")).write_bytes(content)
            target = fixtures / ("cmu-slt-c-" + mode + ".json")
            if args.write:
                target.write_text(json.dumps(normalize(content), indent=2) + "\n", encoding="utf-8")
            elif normalize(target.read_bytes()) != normalize(content):
                raise ValueError(f"C alignment changed: {target.name}")
        print("All three C feature files and both complete alignment documents match.")


if __name__ == "__main__":
    main()
