use shiro_rs::index;
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};
#[test]
fn index_matches_original_lua_padding_and_tokens() {
    let expected: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/index-original.json")).unwrap();
    let actual = index::read(
        Cursor::new(include_bytes!("fixtures/index-original.txt")),
        Path::new("data"),
        &["left".into()],
        &["right".into()],
    )
    .unwrap();
    assert_eq!(actual.len(), 3);
    for (actual, expected) in actual.iter().zip(expected.as_array().unwrap()) {
        assert_eq!(
            actual.stem,
            PathBuf::from(expected["path"].as_str().unwrap())
        );
        let phones: Vec<&str> = expected["phonemes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap())
            .collect();
        assert_eq!(actual.phonemes, phones);
    }
    assert_eq!(actual[1].phonemes, ["left", "cc", "", "dd", "right"]);
    assert_eq!(
        index::append_suffix(&actual[1].stem, ".param"),
        Path::new("data").join("sub/clip.two.param")
    );
}
#[test]
fn blank_rows_crlf_and_empty_phone_fields() {
    let entries = index::read(
        Cursor::new(b"first,aa\r\n\r\nsecond,\r\n"),
        Path::new("."),
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].phonemes, ["aa"]);
    assert!(entries[1].phonemes.is_empty());
    assert!(
        index::read(Cursor::new(b"\n\r\n"), Path::new("."), &[], &[])
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        index::read(Cursor::new(b"clip, aa  bb \n"), Path::new("."), &[], &[]).unwrap()[0].phonemes,
        ["", "aa", "", "bb", ""]
    );
}
#[test]
fn malformed_indices_report_physical_line() {
    for input in [
        b"first,aa\n\nbad\n".as_slice(),
        b"first,aa\n\nbad,aa,bb\n".as_slice(),
        b"first,aa\n\n,aa\n".as_slice(),
    ] {
        let error = index::read(Cursor::new(input), Path::new("."), &[], &[]).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("line 3"));
    }
    assert!(index::read(Cursor::new([0xff]), Path::new("."), &[], &[]).is_err());
}
