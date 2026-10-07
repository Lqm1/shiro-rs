"""Compare original C feature/audio frontends across floating-point profiles."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile


def scalars(content):
    if len(content) % 4:
        raise ValueError("Incomplete binary32 scalar")
    return struct.unpack("<" + "f" * (len(content) // 4), content)


def feature_records(content):
    offset = 0

    def take(count):
        nonlocal offset
        end = offset + count
        if count < 0 or end > len(content):
            raise ValueError("Truncated feature oracle")
        result = content[offset:end]
        offset = end
        return result

    def integer():
        return struct.unpack("<I", take(4))[0]

    if take(4) != b"XCC1" or integer() != 72:
        raise ValueError("Unexpected feature corpus format")
    records = []
    for index in range(72):
        identifiers = take(20)
        count = integer()
        if count != 321:
            raise ValueError("Feature input count changed")
        inputs = take(count * 4)
        frames, columns = integer(), integer()
        if frames not in (3, 4) or not 12 <= columns <= 42:
            raise ValueError("Feature dimensions changed")
        records.append((identifiers, inputs, frames, columns, scalars(take(frames * columns * 4))))
    if offset != len(content):
        raise ValueError("Trailing feature output")
    return records


def category(value):
    if math.isnan(value):
        return "nan"
    if math.isinf(value):
        return "positive_infinity" if value > 0 else "negative_infinity"
    return "finite"


def display(value):
    return value if math.isfinite(value) else category(value)


def compare(old, new):
    if len(old) != len(new):
        raise ValueError("Scalar count changed")
    result = {"values": len(old), "finite_pairs": 0, "nonfinite_reference_values": 0,
              "category_changes": 0, "changed_finite_bits": 0,
              "maximum_absolute_error": 0.0, "maximum_normalized_error": 0.0,
              "maximum_difference": None, "first_category_changes": []}
    for index, (first, second) in enumerate(zip(old, new)):
        a, b = category(first), category(second)
        if a != "finite":
            result["nonfinite_reference_values"] += 1
        if a != b:
            result["category_changes"] += 1
            if len(result["first_category_changes"]) < 8:
                result["first_category_changes"].append(
                    {"index": index, "reference": display(first), "actual": display(second)})
        if a != "finite" or b != "finite":
            continue
        result["finite_pairs"] += 1
        if struct.pack("<f", first) != struct.pack("<f", second):
            result["changed_finite_bits"] += 1
        absolute = abs(first - second)
        normalized = absolute / max(1.0, abs(first))
        if normalized > result["maximum_normalized_error"]:
            result["maximum_difference"] = {"index": index, "reference": first, "actual": second}
        result["maximum_absolute_error"] = max(result["maximum_absolute_error"], absolute)
        result["maximum_normalized_error"] = max(result["maximum_normalized_error"], normalized)
    return result


def compare_features(old, new):
    if len(old) != len(new):
        raise ValueError("Feature record count changed")
    reference, actual = [], []
    for first, second in zip(old, new):
        if first[:4] != second[:4]:
            raise ValueError("Feature inputs or dimensions changed")
        reference.extend(first[4])
        actual.extend(second[4])
    return compare(reference, actual)


def checked_fixture(metrics, limit):
    if metrics["category_changes"] or metrics["maximum_normalized_error"] >= limit:
        raise ValueError("Strict output does not reproduce the existing fixture criterion")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shiro", type=Path, required=True)
    parser.add_argument("--ciglet", type=Path, required=True)
    parser.add_argument("--cc", nargs="+", default=["gcc"])
    parser.add_argument("--target")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.target:
        args.cc += ["-target", args.target]
    tests = Path(__file__).resolve().parent
    shiro, ciglet = args.shiro.resolve(), args.ciglet.resolve()
    profiles = {"strict": ["-O2"], "ofast": ["-Ofast"], "fast_math": ["-O3", "-ffast-math"],
                "fast_math_finite": ["-O3", "-ffast-math", "-fno-finite-math-only"]}
    report = {"compiler": subprocess.check_output(args.cc + ["--version"], text=True).splitlines()[0],
              "contraction": "disabled", "source_sha256": {}, "profiles": {},
              "feature_results": [], "audio_results": []}
    for name, path in (("shiro-xxcc.c", shiro / "shiro-xxcc.c"),
                       ("shiro-wav2raw.c", shiro / "shiro-wav2raw.c"),
                       ("ciglet.c", ciglet / "ciglet.c")):
        report["source_sha256"][name] = hashlib.sha256(path.read_bytes()).hexdigest()
    with tempfile.TemporaryDirectory(prefix="shiro-frontend-optimization-") as temporary:
        root = Path(temporary)
        for name, target in (("shiro-xxcc.c", "original_xxcc.c"), ("shiro-wav2raw.c", "original_wav2raw.c")):
            content = (shiro / name).read_text()
            old = '"external/ciglet/ciglet.h"'
            if content.count(old) != 1:
                raise ValueError("Unexpected original include")
            (root / target).write_text(content.replace(old, '"ciglet.h"'))
        shutil.copyfile(tests / "fixtures/c-audio-input.wav", root / "input.wav")
        strict_features = None
        strict_audio = {}
        for profile, flags in profiles.items():
            options = args.cc + flags + ["-ffp-contract=off"]
            macros = subprocess.run(options + ["-dM", "-E", "-x", "c", "-"],
                                    input="", check=True, capture_output=True, text=True).stdout
            selected = [line for line in macros.splitlines()
                        if line.startswith(("#define __FAST_MATH__ ", "#define __FINITE_MATH_ONLY__ "))]
            report["profiles"][profile] = {"flags": flags, "macros": selected}
            if profile == "fast_math" and ("#define __FAST_MATH__ 1" not in selected
                                         or "#define __FINITE_MATH_ONLY__ 1" not in selected):
                raise ValueError("Explicit fast-math is inactive")
            common = ["-DFP_TYPE=float", "-ffunction-sections", "-fdata-sections",
                      "-I" + str(root), "-I" + str(ciglet), str(ciglet / "ciglet.c"),
                      str(ciglet / "external/fftsg_h.c"), str(ciglet / "external/fast_median.c"),
                      str(ciglet / "external/wavfile.c")]
            suffix = ".exe" if os.name == "nt" else ""
            feature_exe = root / ("features-" + profile + suffix)
            subprocess.run(options + common + [str(tests / "xxcc_oracle.c"), "-Wl,--gc-sections",
                           "-lm", "-o", str(feature_exe)], check=True, timeout=120)
            feature_file = root / "features.bin"
            subprocess.run([str(feature_exe), str(feature_file), str(root / "input.raw")],
                           check=True, timeout=30)
            features = feature_records(feature_file.read_bytes())
            baseline = None
            if profile == "strict":
                baseline = compare_features(feature_records((tests / "fixtures/c-xxcc.bin").read_bytes()), features)
                checked_fixture(baseline, 2e-5)
                strict_features = features
            report["feature_results"].append({"profile": profile, "inputs_and_dimensions_identical": True,
                "strict_fixture_comparison": baseline, "metrics": compare_features(strict_features, features)})
            audio_exe = root / ("audio-" + profile + suffix)
            subprocess.run(options + common + [str(root / "original_wav2raw.c"), "-Wl,--gc-sections",
                           "-lm", "-o", str(audio_exe)], check=True, timeout=120)
            cases = [("plain", []), ("normalized", ["-N"]), ("up", ["-r", "32000"]),
                     ("down", ["-r", "8000"]), ("normalized-down", ["-N", "-r", "8000"]),
                     ("dither", ["-d", "0.125"])]
            for name, arguments in cases:
                extension = "." + name + ".raw"
                result = subprocess.run([str(audio_exe), "-e", extension, *arguments, str(root / "input.wav")],
                                        check=True, capture_output=True, timeout=30)
                if result.stdout:
                    raise ValueError("Audio command wrote unexpected stdout")
                values = scalars((root / ("input" + extension)).read_bytes())
                baseline = None
                if profile == "strict":
                    filename = "c-audio-input." + ("dither-linux" if name == "dither" else name) + ".raw"
                    if name != "dither" or os.name != "nt":
                        baseline = compare(scalars((tests / "fixtures" / filename).read_bytes()), values)
                        checked_fixture(baseline, 2e-7)
                    strict_audio[name] = values
                report["audio_results"].append({"profile": profile, "case": name,
                    "strict_fixture_comparison": baseline, "metrics": compare(strict_audio[name], values)})
    args.report.write_text(json.dumps(report, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    print(f"Verified strict frontend criteria and measured {2 * len(profiles)} original C builds in {args.report}")


if __name__ == "__main__":
    main()
