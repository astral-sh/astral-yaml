use astral_yaml_dev::check_recursive_edits;

#[test]
fn composed_edits_match_an_independent_recursive_graph() {
    for seed in 0u8..=u8::MAX {
        for stride in [1, 5, 7] {
            let mut data = vec![seed];
            for step in 0u8..24 {
                data.extend([
                    seed.wrapping_add(step * stride) % 13,
                    seed.wrapping_mul(13).wrapping_add(step.wrapping_mul(23)),
                    step,
                ]);
            }
            check_recursive_edits(&data);
        }
    }
}
