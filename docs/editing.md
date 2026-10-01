# Editing YAML

`astral_yaml_serde::Editable<T>` owns a YAML document and a read-only typed view.
Edits operate on syntax, and a successful transaction deserializes a fresh `T`.
`T` must implement `DeserializeOwned`; it does not need `Serialize` or `Clone`.
Unknown fields remain in the syntax representation and output.

```rust
use astral_yaml_serde::Editable;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config { timeout: u64 }

let mut config = Editable::<Config>::parse("timeout: 30 # seconds\n")?;
config.edit(|syntax| {
    let entries = syntax.mapping_entries(&syntax.root())?;
    syntax.replace(&entries[0].1, "60")
})?;
assert_eq!(config.value().timeout, 60);
assert_eq!(config.to_yaml(), "timeout: 60 # seconds\n");
# Ok::<(), astral_yaml_serde::Error>(())
```

The [configuration example](../crates/astral-yaml-serde/examples/edit-config.rs)
also inserts and removes entries, changes collection style, and adds a comment.
Run it with `cargo run -p astral-yaml-serde --example edit-config`.

## Transactions

`value()` and `syntax()` inspect the committed state. `edit()` exposes an `Editor`
for the candidate document. Navigate from its root through ordered mapping entries
or sequence items. Mapping keys can be any YAML node, including collections.
Replacement and insertion accept parsed, single-document YAML fragments.

Handles belong to one transaction. Deleted handles and handles from previous
transactions are rejected. Replacing an ancestor and its descendant in the same
transaction is also rejected. Catching a failed mutation inside the callback does
not make the transaction valid: it still rolls back.

Before commit, the editor emits and reparses the candidate, checks content,
alias identity, requested styles, and comment ownership, then deserializes `T`.
Any error leaves the original source and typed value intact. This includes
missing required fields, out-of-range values, and duplicate mapping keys.

For syntax editing without a typed projection, use
`astral_yaml::edit::EditableDocument`. Its transactions perform the same syntax
checks, without Serde type or duplicate-key validation. The existing parser's
`Stream` remains an immutable, borrowed view.

## Preservation

`to_yaml()` returns the committed source. An unchanged document is reproduced
exactly. Scalar replacements preserve source outside the replaced range. A
structural edit may rewrite its collection, or an enclosing collection when the
surrounding syntax requires it. Outside those regions, original text is copied.

Rewritten regions retain entry order, comments, tags, anchors, alias identity,
and compatible styles. Whitespace and scalar spelling may normalize. Block
content uses two-space indentation and the document's first line-ending
convention, defaulting to LF. Empty collections use flow syntax. An explicit
style request fails when it would change the scalar's type or cannot be
represented in its surrounding context. Existing block content placed in a flow
context is emitted in a compatible style.

A node replacement retains its outer comments. Removing an entry or item removes
comments owned by its subtree, including leading comments attached to that entry.
Comments owned by the enclosing collection remain. Comment editing uses text
without `#` or a line break and placement relative to its direct owner. A requested
attachment that cannot survive emission is rejected rather than reassigned.

Existing aliases keep their target identity. Edits that remove a referenced
anchor or make an alias resolve to a different definition fail. Anchor renaming,
cross-document operations, and original UTF-16/32 byte encoding preservation are
not supported. The editing API accepts one UTF-8 YAML document.

Transactions currently clone the syntax, render changed regions, reparse the
result, and deserialize the entire typed view. Parser, traversal, insertion, and
output limits bound their work. This API favors validation and source preservation
over incremental update performance.

`EditableDocument::normalized_yaml()` is an explicit whole-root rendering
operation. It returns normalized output without changing the document.

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
