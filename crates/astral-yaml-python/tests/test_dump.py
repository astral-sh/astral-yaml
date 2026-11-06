import pytest

import astral_yaml as yaml


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
