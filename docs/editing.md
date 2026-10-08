# Editing YAML

Editing is disabled by default. Enable `editable` in the crate you use:

```toml
[dependencies]
astral-yaml-serde = { version = "0.1.0", features = ["editable"] }
```

`astral_yaml_serde::Editable<T>` owns one YAML document and a read-only typed view.
`T` needs `DeserializeOwned`, but not `Serialize` or `Clone`. Unknown fields remain
in the syntax and output. For untyped editing, enable `astral-yaml`'s `editable`
feature and use `astral_yaml::edit::EditableDocument`.

```rust
use astral_yaml_serde::Editable;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config { timeout: u64 }

let mut config = Editable::<Config>::parse("timeout: 30 # seconds\n")?;
config.edit(|editor| editor.root_mut().field("timeout")?.set(60))?;
assert_eq!(config.value().timeout, 60);
assert_eq!(config.source(), "timeout: 60 # seconds\n");
# Ok::<(), astral_yaml_serde::Error>(())
```

The [configuration example](../crates/astral-yaml-serde/examples/edit-config.rs)
also edits collections and comments:

```sh
cargo run -p astral-yaml-serde --features editable --example edit-config
```

## Editing values

Start at `root_mut()`. Use `field("name")` for string keys and `item(index)` for
sequence elements. `get("name")` returns `None` for a missing key. Key lookup
rejects non-mappings and ambiguous keys. Names are literal, so periods, slashes,
and brackets have no special meaning. Numeric, boolean, and application-tagged
keys do not match their string spellings. Navigation does not follow aliases.

| Operation | Behavior |
| --- | --- |
| `set(value)` | Replaces the selected node with a serialized Rust value. |
| `set_entry(name, value)` | Replaces an existing string-keyed value or appends an entry. Selects its value. |
| `remove_field(name)` | Removes an entry. Returns whether it existed. |
| `push(value)` / `insert(index, value)` | Inserts an item and selects it. Index `len` appends. |
| `remove_item(index)` | Removes an item. Later indices reflect the removal. |
| `reborrow()` | Temporarily borrows a cursor, retaining the parent for further edits. |

Only the supplied value needs `Serialize`. Strings are data: `set("false")`
writes a string, while `set(false)` writes a boolean. Setters retain the
existing anchor, outer comments, and compatible style. The new value supplies
tags.

Use `set_scalar_style()` or `set_collection_style()` for explicit style
requests. An unrepresentable request fails, except that empty collections always
use flow syntax. `append_comment()`, `replace_comment()`, and `remove_comment()`
edit the selected node's own comments. Comment text excludes `#` and line
breaks. Include a leading space for a comment beginning with `# `.

## Editing syntax

Use `edit_syntax()` or `editor.syntax_mut()` for arbitrary mapping keys, aliases,
and raw YAML fragments. `replace()` takes a fragment's style, tags, and
anchor. `replace_preserving_style()` retains the old anchor and compatible style
when present. Handles belong to one transaction. Replacement retains the
selected handle and invalidates its old descendants. Deleted and foreign handles
fail.

`comment_position(handle, index)` captures a comment's structural position. Use
it with `insert_comment_at()` to retain a property header or collection gap when
removing and reinserting a comment. For a new detached gap comment,
`CommentPosition::BeforeChild(handle)` names a sequence item or mapping key in
the owning collection. Headers may move during rendering. Child gaps retain
their target identity. Positions containing handles belong to one transaction.

Inside a transaction, `node()` returns `Node<Option<Span>>`. `Some(span)`
identifies an original occurrence, even after replacement. An omitted value has
an empty span. Inserted syntax has `None`. Committed nodes have fresh spans.

## Transactions

Cursors see earlier edits in the same callback, including inserted collections.
Intermediate states need not deserialize as `T`. Aliases may temporarily lack a
definition if later edits remove the aliases or restore their target's anchor.

Commit reparses the output and checks scalar text/types, ordered structure, tags,
anchors, alias targets, requested styles, and comment ownership, placement, and
order. Typed editing also deserializes a fresh `T`. Any error leaves the source and typed value unchanged. Failed mutations prevent
commit even if the callback catches their errors. The callback result is returned
only after commit.

Untyped transactions perform the syntax checks without Serde's type or duplicate
key validation. `value()` and `syntax()` always expose committed state.

## Source preservation

Loading a document or replacement fragment normalizes physical CR and CRLF line
endings to LF. All editing spans and diagnostics refer to that normalized
source. Escaped carriage returns remain scalar data. Ordinary parsing and
deserialization retain the original input. Editing accepts a single UTF-8
document. It does not preserve UTF-16/32 encoding or support cross-document
operations.

`source()` borrows the committed text, `to_yaml()` clones it, and `into_yaml()`
returns it without cloning. Apart from line-ending normalization, unchanged
source is reproduced exactly. Canceled edits do not force rewrites.

Edits rewrite the selected node or an enclosing collection when YAML syntax or
comment boundaries require it. Outside those regions, source is copied
unchanged. Rewritten regions retain entry order, scalar text and types, tags,
anchors, alias targets, comments, and compatible styles. Whitespace and scalar
spelling may change. Block content uses two-space indentation. Directives and
existing document markers are retained. Additional markers or flow delimiters
may be needed to separate comments from scalar content.

Replacement retains a node's outer comments and merges fragment comments at
their structural positions. Deletion removes comments owned by the deleted
subtree. Collection-owned detached comments stay before the same following
child. Deleting that child moves them before the next child, or to the
collection's end. Whole collection replacement retains these positions by index.
Unrepresentable comment attachments fail. Root comments may move to document
markers while retaining ownership, placement, and order.

Aliases retain target identity. Renaming a definition's anchor also updates its
aliases, including those outside the definition's rewrite. Removing a referenced
anchor or rebinding an alias to another definition fails.

`EditableDocument::normalized_yaml()` renders the whole root without changing the
document, subject to the same content, alias, and comment checks. Edits clone the
syntax, reparse the output, and deserialize the entire typed view. Parser,
traversal, insertion, and output limits bound this work.
