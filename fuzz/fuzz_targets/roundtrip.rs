use astral_yaml::{ParseOptions, parse};
use astral_yaml_serde::from_str;
use serde::Deserialize;

/// Read a serialized value back, allowing large outputs to exhaust the default
/// reader's size and work budgets. Keep every failure on ordinary fuzz-sized
/// documents visible, including nesting errors.
pub(crate) fn read<'de, T: Deserialize<'de>>(encoded: &'de str) -> Option<T> {
    let limits = ParseOptions::default();
    if encoded.len() > limits.max_input_bytes {
        return None;
    }

    match from_str(encoded) {
        Ok(value) => Some(value),
        // Each serialized node occupies at least one byte.
        Err(error)
            if error.message == "syntax element limit exceeded"
                && encoded.len() >= limits.max_nodes =>
        {
            None
        }
        Err(error)
            if error.message == "expanded scalar byte limit exceeded" && encoded.len() > 65_536 =>
        {
            None
        }
        Err(error)
            if error.message == "deserialization resource limit exceeded"
                && encoded.len() > 65_536 =>
        {
            // Serialized YAML has no aliases. Checking nested mapping keys can
            // revisit descendants at each level. Even allowing four visits per
            // node per level for the wrappers in these targets, a short tree
            // cannot spend the reader's million visits.
            let stream = parse(encoded).unwrap();
            assert!(
                stream.nodes().len().saturating_mul(4 * limits.max_depth) >= 1_000_000,
                "failed to deserialize a small serialized tree: {error}"
            );
            None
        }
        Err(error) => panic!("failed to deserialize serialized YAML: {error}"),
    }
}
