use astral_yaml_dev::{Value, fixtures::manifest};

#[test]
fn real_world_values_comments_and_serialization() {
    for fixture in manifest() {
        let name = &fixture.file;
        let source = fixture.source();

        if !fixture.valid_syntax() {
            assert!(astral_yaml_serde::from_str::<Value>(&source).is_err());
            continue;
        }

        let stream = astral_yaml::parse(&source).unwrap();
        let values: Result<Vec<Value>, _> = (0..stream.documents().len())
            .map(|index| astral_yaml_serde::from_document(&stream, index))
            .collect();

        if let Some(expected) = fixture.serde_error {
            assert_eq!(values.unwrap_err().message, expected, "{name}");
            continue;
        }

        let values = values.unwrap_or_else(|error| panic!("{name}: {error}"));

        if stream.documents().len() == 1 {
            let wrapped: astral_yaml_serde::Spanned<'_, Value> =
                astral_yaml_serde::from_document(&stream, 0).unwrap();
            let comments: Vec<_> = wrapped
                .comments
                .iter()
                .map(|attached| attached.comment.clone())
                .collect();
            assert_eq!(comments, stream.comments(), "root comments: {name}");
        }

        for value in values {
            let yaml = astral_yaml_serde::to_string(&value).unwrap();
            assert_eq!(
                astral_yaml_serde::from_str::<Value>(&yaml).unwrap(),
                value,
                "{name}"
            );
        }

        assert!(
            astral_yaml_serde::from_document::<Value>(&stream, stream.documents().len()).is_err()
        );
    }
}
