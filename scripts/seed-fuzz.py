#!/usr/bin/env python3
"""Seed the parser fuzzer with the pinned official corpus and regression cases."""
from hashlib import sha256
from pathlib import Path

root = Path(__file__).resolve().parent.parent
corpora = [root / "fuzz/corpus" / name for name in ["parse", "serde"]]
for corpus in corpora:
    corpus.mkdir(parents=True, exist_ok=True)
inputs = list((root / "fixtures/yaml-test-suite").rglob("in.yaml"))
inputs += list((root / "fixtures/regressions").glob("*.yaml"))
inputs += list((root / "fixtures/real-world").glob("*.yaml"))
for path in inputs:
    data = path.read_bytes()
    for corpus in corpora:
        (corpus / sha256(data).hexdigest()).write_bytes(data)
print(f"Seeded {len(inputs)} inputs in each of {len(corpora)} corpora")

decoded = root / "fuzz/corpus/decode"
decoded.mkdir(parents=True, exist_ok=True)
for path in inputs:
    text = path.read_bytes().decode("utf-8")
    for encoding in ["utf-8", "utf-16-be", "utf-16-le", "utf-32-be", "utf-32-le"]:
        data = ("\ufeff" + text).encode(encoding)
        (decoded / sha256(data).hexdigest()).write_bytes(data)
print("Seeded all five input encodings")

typed = root / "fuzz/corpus/typed"
typed.mkdir(parents=True, exist_ok=True)
examples = [
    "0", "-128", "127", "255", "256", "-32768", "65535",
    "-2147483648", "4294967295", "-9223372036854775808", "18446744073709551615",
    "-170141183460469231731687303715884105728", "340282366920938463463374607431768211455",
    "true", "null", "-0.0", ".inf", "-.inf", ".nan", "0o77", "0xff", "'é'", '"a\\nb"',
    "[]", "[1, 2]", "[1, 'text']", "{}", "{a: 1}", "{a: 1, a: 2}",
    "Unit", "Newtype", "Tuple", "Struct", "!Unit", "{Unit: null}",
    "!Newtype abc", "!Tuple [1, false]", "!Struct {value: -1}",
    "{name: 'example', count: 1, unknown: [1, 2]}",
    "{unknown: {a: 1, a: 2}}", "&a [*a]",
]
typed_sources = [path.read_bytes() for path in inputs] + [example.encode() for example in examples]
for selector in range(24):
    for source in typed_sources:
        data = bytes([selector]) + source
        (typed / sha256(data).hexdigest()).write_bytes(data)
print("Seeded all 24 typed serde paths")
