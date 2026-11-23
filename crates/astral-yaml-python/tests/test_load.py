import io
from collections.abc import Callable
from unittest.mock import Mock

import astral_yaml as yaml
import pytest
from inline_snapshot import snapshot


def test_python_scalar_construction() -> None:
    assert yaml.loads("[! 1, 010, 0x10]") == ["1", 10, 16]


def _loads_error(data: str | bytes) -> str:
    with pytest.raises(yaml.YAMLError) as exc:
        yaml.loads(data)
    return str(exc.value)


def test_loads_rejects_multiple_documents() -> None:
    assert _loads_error("---\nfirst\n---\nsecond\n") == snapshot(
        "expected one YAML document at bytes 10..20"
    )


def test_loads_rejects_equal_bool_and_int_keys() -> None:
    assert _loads_error("{true: first, 1: second}") == snapshot(
        "duplicate mapping key at bytes 14..15"
    )


def test_loads_rejects_duplicate_nan_keys() -> None:
    assert _loads_error("{.nan: a, .NaN: b}") == snapshot(
        "duplicate mapping key at bytes 10..14"
    )


def test_loads_rejects_sequence_key() -> None:
    assert _loads_error("{[1]: a}") == snapshot(
        "collection keys cannot be used in a Python dict at bytes 1..4"
    )


def test_loads_rejects_custom_tag() -> None:
    assert _loads_error("!custom text") == snapshot(
        "unsupported YAML tag !custom at bytes 0..12"
    )


def test_loads_rejects_int_tag_on_sequence() -> None:
    assert _loads_error("!!int []") == snapshot(
        "tag does not match node kind at bytes 0..8"
    )


def test_loads_rejects_float_overflow() -> None:
    assert _loads_error("1e10000") == snapshot("float out of range at bytes 0..7")


def test_loads_reports_unterminated_flow_sequence() -> None:
    assert _loads_error("[") == snapshot("unterminated flow sequence at bytes 1..1")


def test_loads_rejects_invalid_utf8() -> None:
    assert _loads_error(b"\xff") == snapshot("invalid UTF-8 at bytes 0..1")


def test_loads_reports_byte_offsets_in_utf16() -> None:
    assert _loads_error("!!bool x".encode("utf-16-le")) == snapshot(
        "scalar does not match its tag at bytes 14..16"
    )


def test_unpaired_surrogate() -> None:
    with pytest.raises(yaml.YAMLError, match="surrogates"):
        yaml.loads("\ud800")


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


def _loads_all_error(data: str | bytes) -> str:
    with pytest.raises(yaml.YAMLError) as exc:
        yaml.loads_all(data)
    return str(exc.value)


def test_loads_all_reports_syntax_error_in_second_document() -> None:
    assert _loads_all_error("--- 0\n--- [\n") == snapshot(
        "unterminated flow sequence at bytes 12..12"
    )


def test_loads_all_rejects_alias_to_previous_document() -> None:
    assert _loads_all_error("--- &items []\n--- *items\n") == snapshot(
        "unknown alias at bytes 24..25"
    )


def test_loads_all_rejects_custom_tag_in_second_document() -> None:
    assert _loads_all_error("--- 0\n--- !custom text\n") == snapshot(
        "unsupported YAML tag !custom at bytes 10..22"
    )


def test_loads_all_reports_byte_offsets_in_second_utf16_document() -> None:
    assert _loads_all_error("--- 0\n--- !!bool x\n".encode("utf-16-le")) == snapshot(
        "scalar does not match its tag at bytes 34..36"
    )
