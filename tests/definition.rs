use shiro_rs::definition::ModelDefinition;

#[test]
fn retains_legacy_defaults_and_constraints() {
    let definition: ModelDefinition = serde_json::from_str(
        r#"{
        "ndurstate":3,
        "streamdef":[{"nstate":3,"ndim":2},{"nstate":2,"ndim":1,"nmix":2,"weight":0.5}],
        "dur_attr":[{"index":1,"floor":2,"ceil":10}]
    }"#,
    )
    .unwrap();
    let model = definition.build().unwrap();
    assert_eq!(model.streams[0].mixtures.len(), 3);
    assert_eq!(model.streams[0].mixtures[0].weights, [1.0]);
    assert_eq!(model.streams[1].mixtures[0].weights, [0.5, 0.5]);
    assert_eq!(model.streams[1].weight, 0.5);
    assert_eq!(
        (model.durations[1].minimum, model.durations[1].maximum),
        (2, 10)
    );
    assert_eq!(model.durations[0].minimum, -1);
}

#[test]
fn invalid_dimensions_and_indices_are_errors() {
    for json in [
        r#"{"ndurstate":1,"streamdef":[{"nstate":1,"ndim":0}]}"#,
        r#"{"ndurstate":1,"streamdef":[{"nstate":1,"ndim":2}],"dur_attr":[{"index":1}]}"#,
        r#"{"ndurstate":1,"streamdef":[{"nstate":1,"ndim":2}],"dur_attr":[{"index":0,"floor":10,"ceil":1}]}"#,
    ] {
        assert!(
            serde_json::from_str::<ModelDefinition>(json)
                .unwrap()
                .build()
                .is_err()
        );
    }
    assert!(serde_json::from_str::<ModelDefinition>(r#"{"ndurstate":-1,"streamdef":[]}"#).is_err());
}

#[test]
fn command_produces_an_original_c_model_fixture() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modeldef.json");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_shiro-mkhsmm"))
        .arg("-c")
        .arg(fixture)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, include_bytes!("fixtures/empty-c.hsmm"));
}
