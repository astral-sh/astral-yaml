import io
from typing import Literal
from unittest.mock import Mock, PropertyMock

import astral_yaml as yaml
import pytest


@pytest.mark.parametrize(
    "style", [None, "block", "flow"], ids=["default", "block", "flow"]
)
def test_collection_styles(style: Literal["block", "flow"] | None) -> None:
    data = {"items": [{"name": "café", "flags": [True, None]}, [1, 2], {}], "empty": []}
    block = (
        '"items":\n'
        '  - "name": "café"\n'
        '    "flags":\n'
        "      - true\n"
        "      - null\n"
        "  - - 1\n"
        "    - 2\n"
        "  - {}\n"
        '"empty": []\n'
    )
    flow = '{"items": [{"name": "café", "flags": [true, null]}, [1, 2], {}], "empty": []}\n'

    output = io.StringIO()
    if style is None:
        document = yaml.dumps(data)
        result = yaml.dump(data, output)
    else:
        document = yaml.dumps(data, style=style)
        result = yaml.dump(data, output, style=style)
    assert document == (flow if style == "flow" else block)
    assert result is None
    assert output.getvalue() == document
    assert yaml.loads(document) == data


@pytest.mark.parametrize(
    ("value", "expected"), [([], "[]\n"), ((), "[]\n"), ({}, "{}\n")]
)
def test_empty_collections(value: object, expected: str) -> None:
    assert yaml.dumps(value) == expected


@pytest.mark.parametrize(("style", "error"), [("BLOCK", ValueError), (None, TypeError)])
def test_invalid_style(style: object, error: type[Exception]) -> None:
    writer = Mock()
    type(writer).write = PropertyMock(side_effect=AssertionError("accessed write"))

    with pytest.raises(error, match="style"):
        yaml.dumps(None, style=style)  # ty: ignore[invalid-argument-type]
    with pytest.raises(error, match="style"):
        yaml.dump(None, writer, style=style)  # ty: ignore[invalid-argument-type]


def test_style_is_keyword_only() -> None:
    with pytest.raises(TypeError):
        yaml.dumps(None, "flow")  # ty: ignore[too-many-positional-arguments]
    with pytest.raises(TypeError):
        yaml.dump(None, io.StringIO(), "flow")  # ty: ignore[too-many-positional-arguments]


@pytest.mark.parametrize("style", ["block", "flow"])
@pytest.mark.parametrize("length", [1022, 1023])
def test_implicit_key_limit(style: Literal["block", "flow"], length: int) -> None:
    # https://yaml.org/spec/1.2.2/#822-block-mappings
    # The quotes count toward the 1024-character limit.
    key = "é" * length
    data = {key: [1]}
    document = yaml.dumps(data, style=style)
    if style == "block":
        expected = f'"{key}":\n  - 1\n' if length == 1022 else f'? "{key}"\n:\n  - 1\n'
    else:
        expected = f'{{"{key}": [1]}}\n' if length == 1022 else f'{{? "{key}": [1]}}\n'
    assert document == expected
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
