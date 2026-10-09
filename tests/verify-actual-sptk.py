"""Compare actual SPTK through original Lua and both native Rust host paths.

Uses unchanged original C raw samples and the unchanged upstream Lua extractor.
This verifies the real host programs; protocol fixtures are not accepted here.
The caller supplies a freshly built shiro-fextr for the target under test.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

if not __debug__:
    raise RuntimeError('Run verification without Python -O; assertions are required.')

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--cli', required=True, type=Path)
parser.add_argument('--sptk', required=True, type=Path)
parser.add_argument('--lua', required=True, type=Path)
parser.add_argument('--upstream-extractor', required=True, type=Path)
parser.add_argument('--fixtures', required=True, type=Path)
parser.add_argument('--harness', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--target', required=True)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
directory = Path(tempfile.mkdtemp(prefix='actual-sptk-', dir=args.output)).resolve()
assert directory.parent == args.output.resolve()
environment = dict(os.environ)
environment['PATH'] = str(args.sptk.resolve()) + os.pathsep + environment.get('PATH', '')
programs = ['frame', 'mfcc', 'delta']
suffix = '.exe' if os.name == 'nt' else ''
for program in programs:
    assert (args.sptk / (program + suffix)).is_file(), program
reference = directory / 'original Lua'
reference.mkdir()
reference_stem = reference / 'clip with spaces.v1'
raw_reference = args.fixtures / 'c-audio-input.plain.raw'
subprocess.run([str(args.lua), str(args.harness), str(args.upstream_extractor),
                str(reference_stem), str(raw_reference)], env=environment, check=True)
expected = {
    '.raw': raw_reference.read_bytes(),
    '.mfcc': Path(str(reference_stem) + '.mfcc').read_bytes(),
    '.param': Path(str(reference_stem) + '.param').read_bytes(),
}
assert len(expected['.mfcc']) > 0 and len(expected['.mfcc']) % (12 * 4) == 0
assert len(expected['.param']) == len(expected['.mfcc']) * 3
for mode in ['sptk', 'lua']:
    destination = directory / mode
    destination.mkdir()
    stem = destination / 'clip with spaces.v1'
    shutil.copyfile(args.fixtures / 'c-audio-input.wav', Path(str(stem) + '.wav'))
    index = destination / 'index.csv'
    index.write_text('clip with spaces.v1,aa\n', encoding='utf-8')
    command = [str(args.cli), str(index), '-d', str(destination)]
    if mode == 'sptk':
        command += ['-x', 'extractor-sptk-mfcc12-da-16k', '--sptk-directory', str(args.sptk)]
    else:
        command += ['-x', str(args.upstream_extractor), '--lua', str(args.lua)]
    subprocess.run(command, env=environment, check=True)
    for extension, original in expected.items():
        actual = Path(str(stem) + extension).read_bytes()
        assert actual == original, (args.target, mode, extension, len(actual), len(original))
report = {
    'target': args.target,
    'cli': str(args.cli.resolve()),
    'sptk': str(args.sptk.resolve()),
    'extractor_sha256': hashlib.sha256(args.upstream_extractor.read_bytes()).hexdigest(),
    'program_sha256': {name: hashlib.sha256((args.sptk / (name + suffix)).read_bytes()).hexdigest()
                       for name in programs},
    'outputs': {extension: {'bytes': len(value), 'sha256': hashlib.sha256(value).hexdigest()}
                for extension, value in expected.items()},
    'modes': ['original_lua', 'rust_sptk', 'rust_lua'],
    'byte_exact': True,
    'artifacts': str(directory),
}
(directory / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report))
