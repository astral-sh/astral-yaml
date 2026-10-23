#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("unity-physics-simulator"), @"
    unity-physics-simulator.yaml:20:5: undefined tag handle
    bytes 488..491
      18 |   m_StaticEditorFlags: 0
      19 |   m_IsActive: 1
      20 | --- !u!4 &440985591853936351
         |     ^^^
      21 | Transform:
    ");
}
