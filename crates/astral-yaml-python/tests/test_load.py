import io
from collections.abc import Callable
from unittest.mock import Mock

import astral_yaml as yaml
import pytest


def test_python_scalar_construction() -> None:
    assert yaml.loads("[! 1, 010, 0x10]") == ["1", 10, 16]


@pytest.mark.parametrize(
    ("data", "message"),
    [
        ("---\nfirst\n---\nsecond\n", "expected one YAML document"),
        ("{true: first, 1: second}", "duplicate mapping key"),
        ("{.nan: a, .NaN: b}", "duplicate mapping key"),
        ("{[1]: a}", "collection keys cannot be used in a Python dict"),
        ("!custom text", "unsupported YAML tag"),
        ("!!int []", "tag does not match node kind"),
        ("1e10000", "float out of range"),
        ("[", "at bytes"),
        (b"\xff", "invalid UTF-8"),
        ("!!bool x".encode("utf-16-le"), r"at bytes 14\.\.16"),
        ("\ud800", "surrogates"),
    ],
)
def test_load_diagnostics(data: str | bytes, message: str) -> None:
    with pytest.raises(yaml.YAMLError, match=message):
        yaml.loads(data)


@pytest.mark.parametrize("load", [yaml.loads, yaml.load, yaml.loads_all, yaml.load_all])
def test_wrong_input_type(load: Callable[[object], object]) -> None:
    with pytest.raises(TypeError):
        load(object())


def test_short_reads() -> None:
    reader = Mock()
    reader.read.side_effect = ["fir", "st", ""]
    assert yaml.load(reader) == "first"

    reader.read.side_effect = [b"first", ""]
    with pytest.raises(TypeError, match="mixed text and binary"):
        yaml.load(reader)


@pytest.mark.parametrize(
    ("data", "expected"),
    [
        ("", []),
        ("# no documents\n", []),
        ("---\n", [None]),
        ("--- []\n", [[]]),
        ("--- {mode: on}\n---\n--- [1, true]\n", [{"mode": "on"}, None, [1, True]]),
        (b"--- first\n--- second\n", ["first", "second"]),
        (bytearray(b"--- 1\n--- 2\n"), [1, 2]),
    ],
)
def test_loads_all(data: str | bytes | bytearray, expected: list[object]) -> None:
    assert yaml.loads_all(data=data) == expected


def test_load_all_files() -> None:
    source = "--- café\n--- 2\n"
    assert yaml.load_all(fp=io.StringIO(source)) == ["café", 2]
    assert yaml.load_all(fp=io.BytesIO(source.encode("utf-16"))) == ["café", 2]


def test_load_all_short_reads() -> None:
    reader = Mock()
    reader.read.side_effect = ["--- fir", "st\n--", "- second\n", ""]
    assert yaml.load_all(reader) == ["first", "second"]

    reader.read.side_effect = ["--- first\n", OSError("read failed")]
    with pytest.raises(OSError, match="read failed"):
        yaml.load_all(reader)


def test_load_all_alias_identity() -> None:
    # https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases
    first, second = yaml.loads_all("--- [&items [], *items]\n--- [&items [], *items]\n")
    assert first[0] is first[1]
    assert second[0] is second[1]
    assert first[0] is not second[0]


@pytest.mark.parametrize(
    ("data", "message"),
    [
        ("--- 0\n--- [\n", "at bytes"),
        ("--- &items []\n--- *items\n", "unknown alias"),
        ("--- 0\n--- !custom text\n", "unsupported YAML tag"),
        ("--- 0\n--- !!bool x\n".encode("utf-16-le"), r"at bytes 34\.\.36"),
    ],
)
def test_load_all_later_document_errors(data: str | bytes, message: str) -> None:
    with pytest.raises(yaml.YAMLError, match=message):
        yaml.loads_all(data)
