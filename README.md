# astral-yaml

A Rust YAML 1.2 parser under development. The core parser uses only `std`;
serde support lives in a separate crate. Workspace code forbids unsafe Rust.

- `astral-yaml`: parsing, syntax, source spans, and comments.
- `astral-yaml-serde`: serialization and deserialization.
- `astral-yaml-dev`: shared test, fuzz, and benchmark helpers (not published).
- `xtask/`: development commands and benchmarks (not published).

The parser targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). YAML 1.1 schema
compatibility is outside the project's scope. See
[conformance and limits](docs/conformance.md) for supported behavior and
[validation results](TRANSCRIPT.md) for recorded test, fuzz, and benchmark runs.

## Usage

`astral_yaml::parse(source)` returns a stream of syntax nodes with byte ranges,
comments, scalar styles, tags, and anchors. Collection children and aliases refer
to node IDs in the stream's arena. The parser does not expand aliases.

Use `astral_yaml_serde::from_str::<T>(source)` for a `Deserialize` type and
`astral_yaml_serde::to_string(&value)` for a `Serialize` value. The serializer uses
flow style and quotes strings to preserve their type under YAML 1.2 core resolution.

Wrap a value or field in `Spanned<'a, T>` to retain its source range and comments:

```rust
use astral_yaml_serde::{Spanned, from_str};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config<'a> {
    #[serde(borrow)]
    timeout: Spanned<'a, u64>,
}

let source = "timeout: 30 # seconds\n";
let config: Config<'_> = from_str(source).unwrap();
assert_eq!(*config.timeout, 30);
assert_eq!(config.timeout.span.slice(source), Some("30"));
assert_eq!(config.timeout.comments[0].comment.text, " seconds");
```

`Spanned` exposes `value`, `span`, and `comments`, implements `Deref` and `DerefMut`
for `T`, and serializes as the value alone. Each wrapper collects its node's
comments and those of its syntax descendants once, in source order. Parent and
child lists overlap. Each `AttachedComment` contains a borrowed `Comment` and a
`CommentPlacement`: `Leading`, `Inline`, `Trailing`, or `Detached`. Placement is
relative to the comment's direct owner, even in an ancestor's list.

The input must outlive the wrapper, even when `T` is owned. `Spanned` therefore
cannot satisfy `DeserializeOwned`; ordinary owned types still can. Spans include
node properties and delimiters. Alias wrappers use the span and comments at the
`*name` occurrence; comment collection does not follow the alias target.
Serde buffering for `untagged` and `flatten` discards metadata, so wrap the
containing enum or struct instead of fields decoded from a buffer.

For encoded byte input, use `astral_yaml::Source::from_bytes(&bytes)` and parse its
decoded text. UTF-8 is borrowed; UTF-16 and UTF-32 in either byte order are decoded
with size limits. `Source::original_span` maps a parsed UTF-8 span back to the
original encoded bytes.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and checks.
