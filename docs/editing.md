# Editing YAML

`astral_yaml_serde::Editable<T>` owns a YAML document and a read-only typed view.
Edits update the syntax, and a successful transaction deserializes a fresh `T`.
`T` must implement `DeserializeOwned`; it does not need `Serialize` or `Clone`.
Unknown fields remain in the syntax representation and output.

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
also updates a sequence item, inserts an entry, changes collection style, and adds
a comment.
Run it with `cargo run -p astral-yaml-serde --example edit-config`.

## Selecting and editing values

`edit()` exposes an `astral_yaml_serde::edit::Editor` for the candidate document.
Start with `root_mut()`, then select a decoded string key with `field("name")` or
a sequence item with `item(index)`. Key names are literal: periods, slashes, and
brackets have no special meaning. `get("name")` returns `None` for a missing key;
`field("name")` returns an error. Both reject non-mappings and ambiguous keys.
Plain numeric and boolean keys do not match their string spellings.

`set(value)` serializes the supplied Rust value and replaces the selected node.
`set("false")` writes a string; `set(false)` writes a boolean. Strings are never
interpreted as YAML fragments. Only the supplied value needs `Serialize`; `T`
does not. Replacing a sequence with a scalar or mapping is allowed when the final
document still deserializes as `T`.

| Operation | Behavior |
| --- | --- |
| `set_entry(name, value)` | Replaces an existing string-keyed value in place, or appends a missing entry. Returns a cursor to the value. |
| `remove_field(name)` | Removes the entry and returns whether it existed. |
| `push(value)` | Appends a sequence item and returns a cursor to it. |
| `insert(index, value)` | Inserts before the current index; `len` appends. Returns a cursor to the new item. |
| `remove_item(index)` | Removes a sequence item. Subsequent indices reflect the removal. |

Navigation and insertion consume their cursor. Use `reborrow()` to retain a
parent position for more edits. For a document containing a `servers` sequence:

```rust
config.edit(|editor| {
    let mut servers = editor.root_mut().field("servers")?;
    servers.reborrow().item(1)?.set("backup")?;
    servers.reborrow().push("tertiary")?;
    servers.remove_item(0)?;
    Ok(())
})?;
```

The cursor also provides `node()` for inspection, `set_scalar_style()` and
`set_collection_style()` for explicit presentation requests, and
`append_comment()`, `replace_comment()`, and `remove_comment()` for directly owned
comments. Styles and `CommentPlacement` are re-exported from
`astral_yaml_serde::edit`.

## Transactions

`value()` and `syntax()` inspect the committed state. Cursors see the current
candidate: a node can be set twice, and a newly inserted or replaced collection
can be navigated and edited in the same transaction. Intermediate candidates do
not need to deserialize as `T`. Catching a failed mutation inside the callback
does not make the transaction valid: it still rolls back.

The callback can return a value, such as a count of removed entries. `edit()`
returns that value only after the transaction commits.

Before commit, the editor emits and reparses the candidate, checks content,
alias identity, requested styles, and comment ownership, placement, and order,
then deserializes `T`.
Any error leaves the original source and typed value intact. This includes
missing required fields, out-of-range values, and duplicate mapping keys.

Use `edit_syntax()` for syntax handles, arbitrary YAML mapping keys, alias
inspection, and parsed, single-document YAML fragments. The same syntax editor
is available through `editor.syntax_mut()` inside a value-based transaction:

```rust
config.edit_syntax(|syntax| {
    let entries = syntax.mapping_entries(&syntax.root())?;
    syntax.replace(&entries[0].1, "!!int 60")
})?;
```

Handles belong to one transaction. Replacing a node retains its handle and
invalidates handles to its removed descendants. Deleted handles and handles from
previous transactions are rejected. Navigation never follows aliases implicitly;
use the syntax editor to locate a definition explicitly.

For syntax editing without a typed projection, use
`astral_yaml::edit::EditableDocument`. Its transactions perform the same syntax
checks, without Serde type or duplicate-key validation. The existing parser's
`Stream` remains an immutable, borrowed view.

## Preservation

`source()` borrows the committed source, `to_yaml()` clones it, and `into_yaml()`
consumes the document and returns its source without cloning. An unchanged
document is reproduced exactly. Scalar replacements preserve source outside the
replaced range. A structural edit may rewrite its collection, or an enclosing
collection when the surrounding syntax requires it. Outside those regions,
original text is copied.

Rewrites include adjacent separators when YAML assigns them to the replacement,
such as trailing blank lines that would become part of a block scalar's value.

Rewritten regions retain entry order, comments, tags, anchors, alias identity,
and compatible styles. Whitespace and scalar spelling may normalize, but decoded
scalar text is retained, including empty values read by Serde as strings. Block
content uses two-space indentation and the document's first line-ending
convention, defaulting to LF. Empty collections use flow syntax. An explicit
style request fails when it would change the scalar's type or cannot be
represented in its surrounding context. Existing block content placed in a flow
context is emitted in a compatible style.

A node replacement retains its outer comments. Removing an entry or item removes
comments owned by its subtree, including leading comments attached to that entry.
Comments owned by the enclosing collection remain. Comment editing uses text
without `#` or a line break and placement relative to its direct owner. Include a
leading space in the text for a comment beginning with `# `. A requested
attachment that cannot survive emission is rejected rather than reassigned.
Comments retain their order within each owner, including across transactions.

Value setters retain the selected node's anchor and compatible style, with a
fallback when that style cannot represent the replacement. Tags come from the
new value. Lower-level `replace()` takes the fragment's style, tags, and anchor;
`replace_preserving_style()` retains the original anchor and compatible style.

Existing aliases keep their target identity. Edits that remove a referenced
anchor or make an alias resolve to a different definition fail. Anchor renaming,
cross-document operations, and original UTF-16/32 byte encoding preservation are
not supported. The editing API accepts one UTF-8 YAML document.

Value setters serialize only their replacement subtree. Transactions currently
clone the syntax, render changed regions, reparse the result, and deserialize the
entire typed view. Parser, traversal, insertion, and output limits bound their
work. This API favors validation and source preservation over incremental update
performance.

`EditableDocument::normalized_yaml()` is an explicit whole-root rendering
operation. It returns normalized output without changing the document, or an
error if the rendering would change content, aliases, or comment attachments.

## Migrating metadata wrappers

`Spanned<'a, T>` is now `Spanned<T>` and contains only `value` and `span`. Remove
its lifetime argument and any `#[serde(borrow)]` that was needed solely for that
wrapper. `Spanned<T>` implements `DeserializeOwned` when `T` does.

Use `Commented<'a, T>` to borrow comments, or
`Spanned<Commented<'a, T>>` when both metadata sets are needed. The two wrappers
compose in either order. Parent and child comment lists still overlap, and both
wrappers serialize only their values. Comments retain their original source
spans even without a `Spanned` wrapper.

Style information is available in the editable syntax and the existing parser
nodes. There is no `Styled<T>` wrapper. For preserved output, use `Editable<T>`
and its explicit `to_yaml()` method.
