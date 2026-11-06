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
    data = yaml.loads(yaml.dumps([integer, 1.0, -0.0, -math.inf, math.inf, math.nan]))
    assert data[0] == integer
    assert isinstance(data[1], float)
    assert math.copysign(1, data[2]) == -1
    assert data[3] == -math.inf
    assert data[4] == math.inf
    assert math.isnan(data[5])


def test_python_integer_limit() -> None:
    limit = sys.int_info.str_digits_check_threshold
    previous = sys.get_int_max_str_digits()
    try:
        sys.set_int_max_str_digits(limit)
        with pytest.raises(yaml.YAMLError, match="limit"):
            yaml.loads("1" * (limit + 1))
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


def test_mapping_key_types() -> None:
    assert yaml.loads('{1: first, "1": second}') == {1: "first", "1": "second"}


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
