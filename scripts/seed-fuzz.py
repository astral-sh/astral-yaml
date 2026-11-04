#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = []
#
# [tool.uv]
# require-build-hashes = true
# preview-features = ["build-dependency-hashes"]
# ///
"""Seed all fuzz targets with pinned fixtures and typed deserialization examples."""

from hashlib import sha256
from pathlib import Path

TYPED_EXAMPLES = [
    "0",
    "-128",
    "127",
    "255",
    "256",
    "-32768",
    "65535",
    "-2147483648",
    "4294967295",
    "-9223372036854775808",
    "18446744073709551615",
    "-170141183460469231731687303715884105728",
    "340282366920938463463374607431768211455",
    "true",
    "null",
    "-0.0",
    ".inf",
    "-.inf",
    ".nan",
    "0o77",
    "0xff",
    "'é'",
    '"a\\nb"',
    "[]",
    "[1, 2]",
    "[1, 'text']",
    "{}",
    "{a: 1}",
    "{a: 1, a: 2}",
    "Unit",
    "Newtype",
    "Tuple",
    "Struct",
    "!Unit",
    "{Unit: null}",
    "!Newtype abc",
    "!Tuple [1, false]",
    "!Struct {value: -1}",
    "{name: 'example', count: 1, unknown: [1, 2]}",
    "{unknown: {a: 1, a: 2}}",
    "&a [*a]",
    "[&number 42, *number]",
    "[&items [1, 2], *items]",
    "[&value !Newtype abc, *value]",
    "{empty: }",
    "---\n",
    "# leading\n42 # inline\n\n# detached\n",
    "nested: # header\n  # child\n  value: 1 # inline\n  # tail\n# root tail\n",
    "[ # opening\n # item\n 1,\n # trailing\n] # closing\n",
    "# child\n[ # open\n  old\n] # close\n",
    "# child\n{ # open\n  key: old\n} # close\n",
    "? [a, # key child\n b] # key\n: 1 # value\n",
    "&self [*self # cycle\n]",
    "\ufeff# BOM\nvalue: 1\r\n",
]


def write_seed(corpus: Path, data: bytes) -> None:
    (corpus / sha256(data).hexdigest()).write_bytes(data)


def seed_parsers(root: Path, inputs: list[Path]) -> None:
    corpora = [root / "fuzz/corpus" / name for name in ("parse", "serde")]
    for corpus in corpora:
        corpus.mkdir(parents=True, exist_ok=True)

    for path in inputs:
        data = path.read_bytes()
        for corpus in corpora:
            write_seed(corpus, data)

    # The float fixture repeats an anchored node through aliases. Serializing the
    # resulting Rust value writes each occurrence into the output.
    # https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases
    for path in (root / "fixtures/regressions/serde").glob("*.yaml"):
        write_seed(root / "fuzz/corpus/serde", path.read_bytes())

    print(f"Seeded {len(inputs)} inputs in each of {len(corpora)} corpora")


def seed_encodings(root: Path, inputs: list[Path]) -> None:
    corpus = root / "fuzz/corpus/decode"
    corpus.mkdir(parents=True, exist_ok=True)

    for path in inputs:
        text = path.read_bytes().decode("utf-8")
        for encoding in ("utf-8", "utf-16-be", "utf-16-le", "utf-32-be", "utf-32-le"):
            data = ("\ufeff" + text).encode(encoding)
            write_seed(corpus, data)

    print("Seeded all five input encodings")


def seed_typed(root: Path, inputs: list[Path]) -> None:
    corpus = root / "fuzz/corpus/typed"
    corpus.mkdir(parents=True, exist_ok=True)
    sources = [path.read_bytes() for path in inputs]
    sources.extend(example.encode("utf-8") for example in TYPED_EXAMPLES)

    for selector in range(29):
        for source in sources:
            write_seed(corpus, bytes([selector]) + source)

    print("Seeded all 29 typed serde paths")


def seed_editable(root: Path, inputs: list[Path]) -> None:
    corpus = root / "fuzz/corpus/editable"
    corpus.mkdir(parents=True, exist_ok=True)
    sources = [path.read_bytes() for path in inputs]
    sources.extend(example.encode("utf-8") for example in TYPED_EXAMPLES)
    sources.extend(["é".encode(), b'"\x7f"'])
    operations = [
        bytes(depth + operation for operation in range(8))
        for depth in (0, 32, 64, 96, 128, 160, 192, 224)
    ]
    operations.extend(
        [bytes([15, 15, 23, 15, 31, 15, 6, 15]), bytes([7, 15, 15, 6, 15, 23, 15, 31])]
    )
    for source in sources:
        for sequence in operations:
            write_seed(corpus, sequence + source)
    print("Seeded value edits at the root and nested paths")


def seed_editable_model(root: Path) -> None:
    corpus = root / "fuzz/corpus/editable-model"
    corpus.mkdir(parents=True, exist_ok=True)
    sequences = [
        bytes(value for selector in range(32) for value in (operation, selector))
        for operation in range(22)
    ]
    sequences.append(
        bytes(value for index in range(64) for value in (index % 22, index))
    )
    # Exercise scalar termination on both sides of a flow-style change.
    for operations in ((14, 15, 17), (15, 17, 14), (8, 14, 15), (15, 16, 14)):
        for selector in range(4):
            sequences.append(
                bytes(
                    value for operation in operations for value in (operation, selector)
                )
            )
    # Reopen nested block syntax after adding an inline and detached root suffix.
    sequences.append(bytes([6, 1, 8, 1, 8, 3, 17, 0]))
    # With four-operation batches, remove a suffix while introducing a scalar
    # tail in the next transaction. Preserve the preceding inline header.
    sequences.append(bytes([6, 1, 8, 1, 8, 3, 16, 0, 12, 1, 17, 0, 14, 0, 16, 0]))
    # A suffix marker must follow the complete subtree, including a removed
    # nested comment beyond the final scalar token.
    sequences.append(bytes([8, 3, 15, 0, 8, 3, 6, 0, 12, 1, 14, 0, 16, 0]))
    # Preserve multiline quoted scalars in compact mappings at every depth.
    sequences.append(bytes([18, 0]))
    for controls in range(256):
        for sequence in sequences:
            for text in ("", "false\n\n", "\ufeff\x00é\r\n"):
                header = bytes([controls, len(sequence) // 2 - 1])
                write_seed(corpus, header + sequence + text.encode("utf-8"))
    print("Seeded valid edit sequences with independent value and comment models")


def seed_editable_graph(root: Path) -> None:
    corpus = root / "fuzz/corpus/editable-graph"
    corpus.mkdir(parents=True, exist_ok=True)
    for seed in range(256):
        for stride in (1, 5, 7):
            sequence = bytes(
                value
                for step in range(24)
                for value in (
                    ((seed + step * stride) % 256) % 12,
                    (seed * 13 + step * 23) % 256,
                    step,
                )
            )
            write_seed(corpus, bytes([seed]) + sequence)
    print("Seeded recursive graph edits with complex keys and alias identities")


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    inputs = list((root / "fixtures/yaml-test-suite").rglob("in.yaml"))
    inputs += list((root / "fixtures/regressions").glob("*.yaml"))
    inputs += list((root / "fixtures/real-world").glob("*.yaml"))

    seed_parsers(root, inputs)
    seed_encodings(root, inputs)
    seed_typed(root, inputs)
    seed_editable(root, inputs)
    seed_editable_model(root)
    seed_editable_graph(root)


if __name__ == "__main__":
    main()
