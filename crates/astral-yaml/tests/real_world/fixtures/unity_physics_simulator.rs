#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("unity-physics-simulator"), @"
       |
    18 |   m_StaticEditorFlags: 0
    19 |   m_IsActive: 1
    20 | --- !u!4 &440985591853936351
       |     ^^^ undefined tag handle
    ");
}
