#![cfg(feature = "editable")]

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

#[test]
fn generated_edit_batches_preserve_comments_and_aliases() {
    let mut state = 0x1234_5678_9abc_def0u64;
    for _ in 0..4096 {
        let mut data = [0u8; 97];
        for byte in &mut data {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state.to_le_bytes()[0];
        }
        check_recursive_edits(&data);
    }
}
