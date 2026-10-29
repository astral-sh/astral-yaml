use test_case::test_case;

use super::{assert_projection, project};
use crate::syntax::DocumentSyntax;
use astral_yaml::{CollectionStyle, comments::CommentPlacement};
use astral_yaml_dev::{formatdoc, indoc};
use std::fmt::Write;

#[derive(Clone, Copy, Debug)]
enum Slot {
    Root,
    Item,
    Value,
    Key,
}

impl Slot {
    fn source(self, body: &str) -> String {
        let mut indented = String::new();
        for line in body.lines() {
            writeln!(indented, "  {line}").unwrap();
        }
        match self {
            Self::Root => body.to_owned(),
            Self::Item => format!("-\n{indented}"),
            Self::Value => format!("outer:\n{indented}"),
            Self::Key => format!("?\n{indented}: value\n"),
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Root => 0,
            Self::Item | Self::Key => 1,
            Self::Value => 2,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Scalar,
    Sequence,
    Mapping,
}

fn body(kind: Kind, properties: usize, mask: usize, leading: bool, block: bool) -> String {
    let mut headers = Vec::new();
    if properties & 1 != 0 {
        headers.push("!<tag:example.com,2026:node>");
    }
    if properties & 2 != 0 {
        headers.push("&node");
    }
    match (kind, block) {
        (Kind::Scalar, _) => headers.push("item"),
        (Kind::Sequence, false) => headers.extend(["[", "  item", "]"]),
        (Kind::Mapping, false) => headers.extend(["{", "  key: value", "}"]),
        (Kind::Sequence, true) => headers.push("- item"),
        (Kind::Mapping, true) => headers.push("key: value"),
    }
    let mut output = if leading {
        "# leading\n".to_owned()
    } else {
        String::new()
    };
    let mut slot = 0;
    for header in headers {
        output.push_str(header);
        if !header.starts_with(' ') {
            if mask & (1 << slot) != 0 {
                write!(output, " # comment {slot}").unwrap();
            }
            slot += 1;
        }
        output.push('\n');
    }
    output
}

fn framed(source: &str, frame: usize, slot: Slot) -> String {
    match frame {
        0 => source.to_owned(),
        1 => {
            let prefix = if matches!(slot, Slot::Root) {
                source
                    .lines()
                    .take_while(|line| line.is_empty() || line.starts_with('#'))
                    .map(|line| line.len() + 1)
                    .sum()
            } else {
                0
            };
            let (prefix, body) = source.split_at(prefix);
            let separator = if matches!(slot, Slot::Root) {
                " "
            } else {
                "\n"
            };
            format!("{prefix}---{separator}{body}...\n")
        }
        _ => formatdoc! {"
            %YAML 1.2
            ---
            {source}...
        "},
    }
}

#[derive(Clone, Copy, Debug)]
struct Case {
    kind: Kind,
    properties: usize,
    mask: usize,
    leading: bool,
    slot: Slot,
    frame: usize,
    header_comment: bool,
    detached_prefix: bool,
}

#[derive(Clone, Copy, Debug)]
enum Edit {
    Replace,
    Reinsert,
    ReinsertUnchanged,
    Fragment,
}

impl Case {
    fn source(self, block: bool) -> String {
        let mut body = body(self.kind, self.properties, self.mask, self.leading, block);
        if self.header_comment {
            let at = body
                .match_indices('\n')
                .nth(usize::from(self.leading))
                .unwrap()
                .0
                + 1;
            body.insert_str(at, "# property\n");
        }
        if self.detached_prefix {
            body.insert_str(0, "# detached\n\n");
        }
        framed(&self.slot.source(&body), self.frame, self.slot)
    }

    fn check(self) {
        let source = self.source(false);
        let original = DocumentSyntax::parse(&source).unwrap();
        let expected = project(&original);
        let headers = self.properties.count_ones()
            + if matches!(self.kind, Kind::Scalar) {
                1
            } else {
                2
            };
        let mut comments = Vec::new();
        if self.leading {
            comments.push((" leading".to_owned(), CommentPlacement::Leading, None));
        }
        comments.extend(
            (0..headers)
                .filter(|bit| self.mask & (1 << bit) != 0)
                .map(|bit| (format!(" comment {bit}"), CommentPlacement::Inline, None)),
        );
        if self.header_comment {
            comments.insert(
                usize::from(self.leading) + usize::from(self.mask & 1 != 0),
                (" property".to_owned(), CommentPlacement::Trailing, None),
            );
        }
        if self.detached_prefix {
            let gap = (!matches!(self.kind, Kind::Scalar)).then_some(0);
            comments.insert(0, (" detached".to_owned(), CommentPlacement::Detached, gap));
        }
        assert_eq!(expected[self.slot.index()].comments, comments, "{source}");
        assert_eq!(
            expected
                .iter()
                .map(|node| node.comments.len())
                .sum::<usize>(),
            comments.len(),
            "{source}"
        );

        // Property headers give these block layouts a known spelling. No flow delimiter
        // is needed for a comment. Verify that witness independently.
        let block = !matches!(self.kind, Kind::Scalar)
            && self.properties != 0
            && self.mask < 1 << self.properties.count_ones();
        if block {
            let witness = DocumentSyntax::parse(&self.source(true)).unwrap();
            assert_projection(&witness, &expected, &source);
        }
        for edit in [
            Edit::Replace,
            Edit::Reinsert,
            Edit::ReinsertUnchanged,
            Edit::Fragment,
        ] {
            self.edit(&source, edit, block);
        }
    }

    fn edit(self, source: &str, edit: Edit, block: bool) {
        let mut document = DocumentSyntax::parse(source).unwrap();
        let mut expected = project(&document);
        let comments = &mut expected[self.slot.index()].comments;
        match edit {
            Edit::Fragment => comments.insert(
                usize::from(self.leading) + usize::from(self.detached_prefix),
                (" imported".to_owned(), CommentPlacement::Leading, None),
            ),
            Edit::ReinsertUnchanged => {}
            _ => {
                for (text, _, _) in comments {
                    text.push_str(" edited");
                }
            }
        }
        document
            .edit_syntax(|editor| {
                let handles = super::handles(editor);
                let owner = &handles[self.slot.index()];
                if matches!(edit, Edit::Fragment) {
                    let fragment = format!(
                        "# imported\n{}",
                        body(self.kind, self.properties, 0, false, false)
                    );
                    editor.replace_preserving_style(owner, &fragment)?;
                } else {
                    for (index, (text, placement, _)) in expected[self.slot.index()]
                        .comments
                        .iter()
                        .enumerate()
                        .rev()
                    {
                        if matches!(edit, Edit::Reinsert | Edit::ReinsertUnchanged) {
                            let position = editor.comment_position(owner, index)?;
                            editor.remove_comment(owner, index)?;
                            editor.insert_comment_at(owner, index, text, *placement, position)?;
                        } else {
                            editor.replace_comment(owner, index, text)?;
                        }
                    }
                }
                if block && !matches!(edit, Edit::ReinsertUnchanged) {
                    editor.set_collection_style(owner, CollectionStyle::Block)?;
                }
                Ok(())
            })
            .unwrap_or_else(|error| panic!("{error}\n{source}\n{self:?} {edit:?} block={block}"));
        assert_projection(&document, &expected, source);
        if matches!(edit, Edit::ReinsertUnchanged) {
            assert_eq!(
                document.source(),
                source,
                "canceled reinsertion retains source"
            );
        }
    }
}

#[test_case(Kind::Scalar)]
#[test_case(Kind::Sequence)]
#[test_case(Kind::Mapping)]
fn bounded_comment_headers_preserve_equivalent_edits(kind: Kind) {
    for properties in 0usize..4 {
        let headers = properties.count_ones() + if matches!(kind, Kind::Scalar) { 1 } else { 2 };
        for mask in 0..1 << headers {
            for (leading, detached_prefix) in
                [(false, false), (false, true), (true, false), (true, true)]
            {
                for slot in [Slot::Root, Slot::Item, Slot::Value, Slot::Key] {
                    for frame in 0..3 {
                        for header_comment in [false, true] {
                            if header_comment && properties == 0 {
                                continue;
                            }
                            Case {
                                kind,
                                properties,
                                mask,
                                leading,
                                slot,
                                frame,
                                header_comment,
                                detached_prefix,
                            }
                            .check();
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn a_nested_suffix_then_inline_comment_requires_a_closing_delimiter() {
    for slot in [Slot::Item, Slot::Value, Slot::Key] {
        for properties in ["", "&node\n", "!local\n", "!local &node\n"] {
            let body = format!(
                "{properties}{}",
                indoc! {r"
                    [
                      item
                    # trailing
                    ] # closing
                "}
            );
            let source = slot.source(&body);
            let mut document = DocumentSyntax::parse(&source).unwrap();
            let error = document
                .edit_syntax(|editor| {
                    let owner = super::handles(editor)[slot.index()].clone();
                    editor.set_collection_style(&owner, CollectionStyle::Block)
                })
                .unwrap_err();
            assert_eq!(document.source(), source);
            insta::allow_duplicates! {
                insta::assert_snapshot!(error.message, @"requested style is not representable in this context");
            }
        }
    }
}
