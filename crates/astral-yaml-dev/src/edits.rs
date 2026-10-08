/// Renders a before/after YAML diff for edit snapshots.
///
/// Fixed labels and context keep snapshots independent of file paths. Append `⏎`
/// only to lines ending in whitespace so editors cannot trim it. Double literal
/// trailing `⏎` characters to distinguish them from the whitespace marker.
/// Show carriage returns as `␍` so line endings survive snapshot normalization.
/// Prefix literal `␍` and `␛` characters with `␛` to distinguish them from markers.
#[must_use]
pub fn edit_diff(before: &str, after: &str) -> String {
    let diff = similar::TextDiff::from_lines(before, after)
        .unified_diff()
        .header("before.yaml", "after.yaml")
        .context_radius(3)
        .to_string();
    let mut rendered = String::new();
    for line in diff.split_terminator('\n') {
        for character in line.chars() {
            match character {
                '\r' => rendered.push('␍'),
                '␍' | '␛' => {
                    rendered.push('␛');
                    rendered.push(character);
                }
                _ => rendered.push(character),
            }
        }
        if line.ends_with(char::is_whitespace) && !line.ends_with('\r') {
            rendered.push('⏎');
        } else {
            for _ in line.chars().rev().take_while(|&c| c == '⏎') {
                rendered.push('⏎');
            }
        }
        rendered.push('\n');
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::edit_diff;

    #[test]
    fn carriage_returns_are_visible_in_diff_snapshots() {
        insta::assert_snapshot!(edit_diff("key: old\r\n", "key: new\r\n"), @"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -key: old␍
        +key: new␍
        ");

        insta::assert_snapshot!(edit_diff("key: old\r\n", "key: new\n"), @"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -key: old␍
        +key: new
        ");

        insta::assert_snapshot!(edit_diff("key: old\n", "key: ␍␛\r\n"), @"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -key: old
        +key: ␛␍␛␛␍
        ");

        insta::assert_snapshot!(edit_diff("key: old\r", "key: new\r"), @"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -key: old␍+key: new␍
        ");
    }

    #[test]
    fn trailing_whitespace_survives_editor_cleanup() {
        let before = "key: old\n\nnext: retained\n";
        let without_whitespace = edit_diff(before, "key: new\n\nnext: retained\n");
        for whitespace in [" ", "  ", "\t"] {
            let after = format!("key: new{whitespace}\n\nnext: retained\n");
            let diff = edit_diff(before, &after);

            assert!(diff.lines().all(|line| line == line.trim_end()));
            assert_ne!(diff, without_whitespace);
        }
    }

    #[test]
    fn literal_markers_remain_distinct_from_whitespace_markers() {
        let before = "key: old\n";
        for whitespace in [" ", "  ", "\t"] {
            let after = format!("key: new{whitespace}\n");
            let literal_marker = format!("key: new{whitespace}⏎\n");

            assert_ne!(
                edit_diff(before, &after),
                edit_diff(before, &literal_marker)
            );
        }
        assert_ne!(
            edit_diff(before, "key: new⏎\n"),
            edit_diff(before, "key: new⏎⏎\n")
        );
    }
}
