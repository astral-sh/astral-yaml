import io
from typing import Literal

import astral_yaml as yaml
import pytest
from inline_snapshot import snapshot

_NESTED_COLLECTION = {"items": [{"name": "café", "flags": []}, [1, 2], {}]}


def test_block_style() -> None:
    assert yaml.dumps(_NESTED_COLLECTION, style="block") == snapshot("""\
"items":
  - "name": "café"
    "flags": []
  - - 1
    - 2
  - {}
""")


def test_flow_style() -> None:
    assert yaml.dumps(_NESTED_COLLECTION, style="flow") == snapshot(
        '{"items": [{"name": "café", "flags": []}, [1, 2], {}]}\n'
    )


@pytest.mark.parametrize("style", ["block", "flow"])
def test_collection_style_round_trip(style: Literal["block", "flow"]) -> None:
    output = io.StringIO()
    yaml.dump(_NESTED_COLLECTION, output, style=style)
    document = output.getvalue()
    assert document == yaml.dumps(_NESTED_COLLECTION, style=style)
    assert yaml.loads(document) == _NESTED_COLLECTION


@pytest.mark.parametrize(
    ("value", "expected"), [([], "[]\n"), ((), "[]\n"), ({}, "{}\n")]
)
def test_empty_collections(value: object, expected: str) -> None:
    assert yaml.dumps(value) == expected


@pytest.mark.parametrize(("style", "error"), [("BLOCK", ValueError), (None, TypeError)])
def test_invalid_style(style: object, error: type[Exception]) -> None:
    with pytest.raises(error, match="style"):
        yaml.dumps(None, style=style)  # ty: ignore[invalid-argument-type]
    with pytest.raises(error, match="style"):
        yaml.dump(None, None, style=style)  # ty: ignore[invalid-argument-type]


def test_style_is_keyword_only() -> None:
    with pytest.raises(TypeError):
        yaml.dumps(None, "flow")  # ty: ignore[too-many-positional-arguments]
    with pytest.raises(TypeError):
        yaml.dump(None, io.StringIO(), "flow")  # ty: ignore[too-many-positional-arguments]


# https://yaml.org/spec/1.2.2/#822-block-mappings
# The quotes count toward the 1024-character limit.
@pytest.mark.parametrize(
    ("style", "length", "template"),
    [
        ("block", 1022, '"%s":\n  - 1\n'),
        ("block", 1023, '? "%s"\n:\n  - 1\n'),
        ("flow", 1022, '{"%s": [1]}\n'),
        ("flow", 1023, '{? "%s": [1]}\n'),
    ],
)
def test_implicit_key_limit(
    style: Literal["block", "flow"], length: int, template: str
) -> None:
    key = "é" * length
    data = {key: [1]}
    document = yaml.dumps(data, style=style)
    assert document == template % key
    assert yaml.loads(document) == data


def test_distinct_nan_keys() -> None:
    with pytest.raises(yaml.YAMLError, match="duplicate mapping key"):
        yaml.dumps({float("nan"): 1, float("nan"): 2})


def test_collection_key() -> None:
    with pytest.raises(TypeError, match="mapping keys"):
        yaml.dumps({(): 1})


def test_unpaired_surrogate() -> None:
    with pytest.raises(yaml.YAMLError):
        yaml.dumps("\ud800")


def test_missing_writer() -> None:
    with pytest.raises(TypeError, match="write method"):
        yaml.dump(None, None)  # ty: ignore[invalid-argument-type]


def test_short_write() -> None:
    class ShortWriter:
        def write(self, text: str) -> int:
            return len(text) - 1

    with pytest.raises(OSError, match="short write"):
        yaml.dump("αβγ", ShortWriter())
