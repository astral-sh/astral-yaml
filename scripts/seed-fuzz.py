#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = []
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
    print("Seeded bounded syntax and value-based edit sequences")


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    inputs = list((root / "fixtures/yaml-test-suite").rglob("in.yaml"))
    inputs += list((root / "fixtures/regressions").glob("*.yaml"))
    inputs += list((root / "fixtures/real-world").glob("*.yaml"))

    seed_parsers(root, inputs)
    seed_encodings(root, inputs)
    seed_typed(root, inputs)
    seed_editable(root, inputs)


if __name__ == "__main__":
    main()
