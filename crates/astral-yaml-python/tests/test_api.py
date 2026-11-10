import io
import math
import sys

import pytest

import astral_yaml as yaml


def test_round_trip() -> None:
    data = yaml.loads('ports: [80, 443]\nlabels: [on, yes, "42"]\nenabled: true')
    assert data == {"ports": [80, 443], "labels": ["on", "yes", "42"], "enabled": True}

    restored = yaml.loads(yaml.dumps(data))
    assert restored == data
    assert list(restored) == ["ports", "labels", "enabled"]


def test_tuple_as_sequence() -> None:
    assert yaml.dumps((True, "true", None)) == '[true, "true", null]\n'


def test_numeric_types() -> None:
    integer = 2**256 + 1
    big, whole, zero, nan = yaml.loads(yaml.dumps([integer, 1.0, -0.0, math.nan]))
    assert big == integer
    assert isinstance(whole, float) and whole == 1.0
    assert math.copysign(1, zero) == -1
    assert math.isnan(nan)


@pytest.mark.parametrize("value", [5e-324, 1e20, sys.float_info.max])
def test_exponential_float_round_trip(value: float) -> None:
    restored = yaml.loads(yaml.dumps(value))
    assert isinstance(restored, float) and restored == value


def test_python_integer_limit() -> None:
    limit = sys.int_info.str_digits_check_threshold
    previous = sys.get_int_max_str_digits()
    try:
        sys.set_int_max_str_digits(limit)
        with pytest.raises(yaml.YAMLError, match="limit"):
            yaml.loads("0" * limit + "1")
        with pytest.raises(yaml.YAMLError, match="limit"):
            yaml.dumps(10**limit)
    finally:
        sys.set_int_max_str_digits(previous)


def test_file_objects() -> None:
    source = io.BytesIO("name: café".encode("utf-16"))
    data = yaml.load(fp=source)
    assert data == {"name": "café"}

    output = io.StringIO()
    assert yaml.dump(obj=data, fp=output) is None
    assert output.getvalue() == yaml.dumps(obj=data)


def test_bytearray_input() -> None:
    assert yaml.loads(data=bytearray(b"[]")) == []


@pytest.mark.parametrize("source", ["# no documents\n", "---\n"])
def test_empty_stream(source: str) -> None:
    assert yaml.loads(source) is None


def test_mapping_keys_round_trip() -> None:
    data = {"é" * 2048: "long", 1: "first", "1": "second"}
    document = yaml.dumps(data)
    assert document.startswith("{? ")
    assert yaml.loads(document) == data


def test_alias_identity() -> None:
    data = yaml.loads("first: &one [1]\nsecond: *one\nthird: [1]")
    copied = yaml.loads(yaml.dumps(data))
    assert copied["first"] is copied["second"]
    assert copied["first"] is not copied["third"]
    assert copied["first"] == copied["third"]


def test_recursive_containers() -> None:
    data = yaml.loads("&root {children: [*root]}")
    assert data["children"][0] is data
    copied = yaml.loads(yaml.dumps(data))
    assert copied["children"][0] is copied


def test_alias_graph() -> None:
    source = ["- &a0 []"]
    for i in range(1, 40):
        source.append(f"- &a{i} [*a{i - 1}, *a{i - 1}]")
    data = yaml.loads("\n".join(source))
    rendered = yaml.dumps(data)
    assert len(rendered) < 4096
    copied = yaml.loads(rendered)
    for parent, child in zip(copied[1:], copied):
        assert parent[0] is child
        assert parent[1] is child


def test_exception_types() -> None:
    assert issubclass(yaml.YAMLError, ValueError)
    with pytest.raises(TypeError, match="cannot serialize"):
        yaml.dumps(object())


def test_io_error() -> None:
    class BrokenWriter:
        def write(self, text: str) -> int:
            raise OSError("write failed")

    with pytest.raises(OSError, match="write failed"):
        yaml.dump([1], BrokenWriter())
