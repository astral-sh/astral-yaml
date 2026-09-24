mod support;

#[test]
fn real_world_documents_match_reference_and_roundtrip() {
    for &(name, source) in support::FIXTURES {
        let values = support::astral(source);
        assert_eq!(values, support::reference(source), "{name}");
        let stream = astral_yaml::parse(source).unwrap();
        astral_yaml_dev::validate_spans(&stream).unwrap();
        for value in values {
            let yaml = astral_yaml_serde::to_string(&value).unwrap();
            assert_eq!(
                astral_yaml_serde::from_str::<yaml_serde::Value>(&yaml).unwrap(),
                value,
                "{name}"
            );
        }
        assert!(
            astral_yaml_serde::from_document::<yaml_serde::Value>(
                &stream,
                stream.documents().len()
            )
            .is_err()
        );
    }
}
