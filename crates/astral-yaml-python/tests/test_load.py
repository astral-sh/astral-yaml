from collections.abc import Callable
from unittest.mock import Mock

import pytest

import astral_yaml as yaml


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


@pytest.mark.parametrize("load", [yaml.loads, yaml.load])
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
