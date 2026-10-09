mod common;
use common::Directory;
use shiro_rs::{
    batch::{self, Extractor, Options, SptkPrograms},
    index,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn executable(directory: &Path, name: &str) -> PathBuf {
    index::append_suffix(&directory.join(name), std::env::consts::EXE_SUFFIX)
}
#[test]
#[ignore = "requires SHIRO_TEST_SPTK_DIRECTORY protocol fixtures"]
fn sptk_pipeline_arguments_streams_and_failures() {
    let tools = PathBuf::from(
        std::env::var_os("SHIRO_TEST_SPTK_DIRECTORY").expect("SPTK protocol fixtures required"),
    );
    let directory = Directory::new("sptk-protocol");
    let stem = directory.path().join("sample with spaces");
    fs::write(
        index::append_suffix(&stem, ".wav"),
        include_bytes!("../../../tests/fixtures/c-audio-input.wav"),
    )
    .unwrap();
    let normal = SptkPrograms {
        frame: executable(&tools, "frame"),
        mfcc: executable(&tools, "mfcc"),
        delta: executable(&tools, "delta"),
    };
    let output = batch::extract_file(
        &stem,
        &Options::default(),
        &Extractor::Sptk(normal.clone()),
        || panic!(),
    )
    .unwrap();
    assert_eq!(
        fs::read(output.mfcc.unwrap()).unwrap(),
        include_bytes!("../../../tests/fixtures/c-audio-input.plain.raw")
    );
    assert_eq!(
        fs::read(output.parameters).unwrap(),
        include_bytes!("../../../tests/fixtures/c-audio-input.plain.raw")
    );
    for role in ["frame", "mfcc", "delta"] {
        let mut programs = normal.clone();
        let failure = executable(&tools, &format!("{role}-fail"));
        match role {
            "frame" => programs.frame = failure.clone(),
            "mfcc" => programs.mfcc = failure.clone(),
            _ => programs.delta = failure.clone(),
        };
        match batch::extract_file(
            &stem,
            &Options::default(),
            &Extractor::Sptk(programs),
            || panic!(),
        )
        .unwrap_err()
        {
            batch::Error::Process { program, status } => {
                assert_eq!(program, failure);
                assert_eq!(status.code(), Some(7));
            }
            error => panic!("unexpected {error}"),
        }
    }
    let programs = SptkPrograms {
        mfcc: directory.path().join("missing-program"),
        ..normal
    };
    assert!(matches!(
        batch::extract_file(
            &stem,
            &Options::default(),
            &Extractor::Sptk(programs),
            || panic!()
        ),
        Err(batch::Error::Spawn { .. })
    ));
    let input_index = directory.path().join("index.csv");
    fs::write(&input_index, b"sample with spaces,aa\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_shiro-fextr"))
        .arg(input_index)
        .arg("-d")
        .arg(directory.path())
        .args(["-x", "extractor-sptk-mfcc12-da-16k", "--sptk-directory"])
        .arg(tools)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
#[test]
#[ignore = "requires SHIRO_TEST_LUA external interpreter"]
fn existing_lua_signature_modules_global_callback_and_errors() {
    let interpreter =
        PathBuf::from(std::env::var_os("SHIRO_TEST_LUA").expect("Lua interpreter required"));
    let directory = Directory::new("lua-extractor");
    let input_index = directory.path().join("index.csv");
    fs::write(&input_index, b"sample with spaces,aa\n").unwrap();
    fs::write(
        directory.path().join("sample with spaces.wav"),
        include_bytes!("../../../tests/fixtures/c-audio-input.wav"),
    )
    .unwrap();
    fs::write(
        directory.path().join("fixture_helper.lua"),
        "return {value = 'helper'}\n",
    )
    .unwrap();
    let script = directory.path().join("extractor-xxcc-mfcc12-da-16k.lua");
    fs::write(
        &script,
        r#"local count = assert(io.open(arg[2] .. '.loaded', 'ab')); count:write('x'); count:close()
local helper = require('fixture_helper')
return function(try_execute, stem, rawfile, prefix)
    assert(helper.value == 'helper'); assert(try_execute == _G.try_execute); assert(#prefix > 0)
    local input = assert(io.open(rawfile, 'rb')); local bytes = input:read('*a'); input:close()
    local output = assert(io.open(stem .. '.param', 'wb')); output:write(bytes); output:close()
end
"#,
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_shiro-fextr"))
        .arg(&input_index)
        .arg("-d")
        .arg(directory.path())
        .arg("-x")
        .arg(script.with_extension(""))
        .arg("--lua")
        .arg(&interpreter)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fs::read(directory.path().join("sample with spaces.param")).unwrap(),
        include_bytes!("../../../tests/fixtures/c-audio-input.plain.raw")
    );
    assert_eq!(
        fs::read(directory.path().join("sample with spaces.loaded")).unwrap(),
        b"x"
    );
    fs::write(
        &script,
        "return function(try_execute) try_execute('exit 7') end\n",
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_shiro-fextr"))
        .arg(&input_index)
        .arg("-d")
        .arg(directory.path())
        .arg("-x")
        .arg(&script)
        .arg("--lua")
        .arg(&interpreter)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("extractor command failed"));
    fs::write(&script, "return 42\n").unwrap();
    let extractor = Extractor::Lua {
        interpreter,
        script,
        executable_directory: directory.path().to_owned(),
    };
    assert!(matches!(
        batch::extract_file(
            &directory.path().join("sample with spaces"),
            &Options::default(),
            &extractor,
            || panic!()
        ),
        Err(batch::Error::Process { .. })
    ));
}
