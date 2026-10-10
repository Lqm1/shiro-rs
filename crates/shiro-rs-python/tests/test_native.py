"""Native Python workflows checked against original SHIRO C artifacts."""
from pathlib import Path

import pytest
import shiro_rs as api

FIXTURES = Path(__file__).resolve().parents[3] / "tests" / "fixtures"


@pytest.mark.parametrize("mode,iterations,fixture", [
    (0, 1, "rest-c-hsmm-one"),
    (0, 2, "rest-c-hsmm-two"),
    (1, 2, "rest-c-hmm"),
])
def test_train_save_reload_infer(mode, iterations, fixture, tmp_path):
    model = api.Model.read((FIXTURES / "init-c-aligned.hsmm").read_bytes())
    document = api.SegmentationDocument((FIXTURES / "align-c-isolated.json").read_text())
    files = api.FeatureFiles()
    files.set("input.f", (FIXTURES / "init-input.bin").read_bytes())
    datasets = api.Datasets.load_training_files(document, model, files, 12, False)
    options = api.TrainingOptions()
    options.duration_mode = mode
    options.iterations = iterations
    options.termination_threshold = 0
    options.pruning_slope = 0.8
    reports = []
    result = model.train_with_progress(datasets, options, reports.append)
    trained = result.model()
    wire = trained.write()
    assert isinstance(wire, bytes)
    assert wire == (FIXTURES / f"{fixture}.hsmm").read_bytes()
    assert len(reports) == iterations
    path = tmp_path / "trained.hsmm"
    trained.write_file(str(path), 0)
    restored = api.Model.read_file(str(path))
    assert restored.write() == wire
    alignment_options = api.AlignmentOptions()
    alignment_options.duration_mode = mode
    aligned = restored.align_document(document, files, 12, alignment_options)
    assert '"file_list"' in aligned.json()


def test_callback_exception_identity_and_recovery():
    model = api.Model.read((FIXTURES / "init-c-aligned.hsmm").read_bytes())
    document = api.SegmentationDocument((FIXTURES / "align-c-isolated.json").read_text())
    files = api.FeatureFiles()
    files.set("input.f", (FIXTURES / "init-input.bin").read_bytes())
    datasets = api.Datasets.load_training_files(document, model, files, 12, False)
    options = api.TrainingOptions()
    failure = RuntimeError("stop training")

    def stop(report):
        assert report.iteration == 0
        raise failure

    with pytest.raises(RuntimeError) as caught:
        model.train_with_progress(datasets, options, stop)
    assert caught.value is failure
    assert model.train(datasets, options).model().write()


def test_audio_features_and_native_file_extraction(tmp_path):
    wave = api.Wave.read((FIXTURES / "utterances-input.wav").read_bytes(), 100000)
    options = api.AudioOptions()
    options.output_sample_rate = 16000
    options.dither_level = 0.01
    audio = api.Audio.prepare_with_sequence(wave, options, api.DitherSequence.linux_gnu())
    assert api.rawfloat_write(audio.samples()) == (FIXTURES / "utterances-c-audio.bin").read_bytes()
    feature_options = api.FeatureOptions()
    feature_options.order = 12
    feature_options.hop = 1600
    feature_options.sample_rate_hz = 16000
    feature_options.energy = 1
    features = api.Features.extract(audio.samples(), feature_options)
    reference = api.rawfloat_read((FIXTURES / "utterances-c-features.bin").read_bytes(), 520)
    assert features.frames == 40
    assert features.columns == 13
    for actual, expected in zip(features.values(), reference, strict=True):
        assert abs(actual - expected) <= 2e-5 * max(1, abs(expected))
    stem = tmp_path / "speech"
    stem.with_suffix(".wav").write_bytes((FIXTURES / "utterances-input.wav").read_bytes())
    outputs = api.batch_extract_file(str(stem), api.BatchOptions(), api.Extractor.native(0), lambda: 0.5)
    assert Path(outputs.raw).is_file()
    assert Path(outputs.parameters).is_file()


def test_explicit_close_is_checked():
    options = api.TrainingOptions()
    assert not options.is_closed
    options.close()
    options.close()
    assert options.is_closed
    with pytest.raises(RuntimeError):
        _ = options.iterations


def test_version_and_module_metadata():
    assert api.__version__ == "0.1.0"
    assert api.Model.__module__ == "shiro_rs"


def test_failed_dither_callback_leaves_no_output_files(tmp_path):
    stem = tmp_path / "speech"
    stem.with_suffix(".wav").write_bytes((FIXTURES / "utterances-input.wav").read_bytes())
    options = api.BatchOptions()
    audio = options.audio()
    audio.dither_level = 0.01
    options.set_audio(audio)
    failure = RuntimeError("stop before output")
    calls = []

    def stop():
        calls.append(1)
        raise failure

    with pytest.raises(RuntimeError) as caught:
        api.batch_extract_file(str(stem), options, api.Extractor.native(0), stop)
    assert caught.value is failure
    assert calls == [1]
    assert sorted(path.name for path in tmp_path.iterdir()) == ["speech.wav"]


def test_native_files_dataset_and_alignment(tmp_path):
    import json
    data_path = tmp_path / "input.f"
    data_path.write_bytes((FIXTURES / "init-input.bin").read_bytes())
    source = json.loads((FIXTURES / "align-c-isolated.json").read_text())
    for entry in source["file_list"]:
        entry["filename"] = str(data_path)
    document = api.SegmentationDocument(json.dumps(source))
    model = api.Model.read((FIXTURES / "init-c-aligned.hsmm").read_bytes())
    dataset = api.Dataset.load_files(document, model, 12)
    assert dataset.observations().length > 0
    datasets = api.Datasets.load_training_paths(document, model, 12, False)
    assert datasets.length > 0
    aligned = model.align_document_files(document, api.AlignmentOptions())
    assert json.loads(aligned.json())["file_list"][0]["filename"] == str(data_path)



def test_installed_type_stub_matches_public_runtime_api():
    import ast
    stub = Path(api.__file__).with_name("__init__.pyi")
    assert stub.is_file()
    assert stub.with_name("py.typed").is_file()
    for definition in ast.parse(stub.read_text()).body:
        if isinstance(definition, (ast.ClassDef, ast.FunctionDef)):
            exported = getattr(api, definition.name)
            if isinstance(definition, ast.ClassDef):
                for member in definition.body:
                    if isinstance(member, ast.FunctionDef):
                        assert hasattr(exported, member.name), f"{definition.name}.{member.name}"
