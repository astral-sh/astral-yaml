use astral_yaml::edit::EditableDocument;
use astral_yaml_dev::edit_diff;

pub(super) fn normalize(source: &str) -> String {
    let output = EditableDocument::parse(source)
        .unwrap()
        .normalized_yaml()
        .unwrap();
    assert_eq!(
        EditableDocument::parse(&output)
            .unwrap()
            .normalized_yaml()
            .unwrap(),
        output,
        "normalization must be idempotent: {source:?}",
    );
    edit_diff(source, &output)
}
