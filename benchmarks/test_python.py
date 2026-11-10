from collections.abc import Callable
from functools import partial
from io import BytesIO, StringIO
from pathlib import Path
from typing import NamedTuple

import pytest
from pytest_codspeed import BenchmarkFixture

import astral_yaml


class YamlAPI(NamedTuple):
    loads: Callable[[str], object]
    load: Callable[[BytesIO], object]
    dumps: Callable[[object], str]
    dump: Callable[[object, StringIO], None]


@pytest.fixture(
    params=[
        pytest.param("astral-yaml", id=pytest.HIDDEN_PARAM),
        "pyyaml",
        "pyyaml-libyaml",
    ]
)
def api(request: pytest.FixtureRequest) -> YamlAPI:
    if request.param == "astral-yaml":
        return YamlAPI(
            astral_yaml.loads, astral_yaml.load, astral_yaml.dumps, astral_yaml.dump
        )

    yaml = pytest.importorskip(
        "yaml", reason="install the compare dependency group for PyYAML"
    )
    if request.param == "pyyaml":
        loader, dumper = yaml.SafeLoader, yaml.SafeDumper
    else:
        loader, dumper = yaml.CSafeLoader, yaml.CSafeDumper
    load = partial(yaml.load, Loader=loader)
    dump = partial(
        yaml.dump,
        Dumper=dumper,
        default_flow_style=True,
        sort_keys=False,
        allow_unicode=True,
    )
    return YamlAPI(load, load, dump, dump)


@pytest.fixture(params=["small-config", "uv-ci", "airflow-compose"])
def document(request: pytest.FixtureRequest) -> str:
    if request.param == "small-config":
        return "name: café\nports: [8000, 8443]\nratio: 0.75\nenabled: true\n"
    fixtures = Path(__file__).resolve().parents[1] / "fixtures" / "real-world"
    return (fixtures / f"{request.param}.yaml").read_text(encoding="utf-8")


def test_loads(benchmark: BenchmarkFixture, document: str, api: YamlAPI) -> None:
    benchmark(api.loads, document)


def test_load(benchmark: BenchmarkFixture, document: str, api: YamlAPI) -> None:
    source = document.encode()

    @benchmark
    def read() -> None:
        api.load(BytesIO(source))


def test_dumps(benchmark: BenchmarkFixture, document: str, api: YamlAPI) -> None:
    value = astral_yaml.loads(document)
    benchmark(api.dumps, value)


def test_dump(benchmark: BenchmarkFixture, document: str, api: YamlAPI) -> None:
    value = astral_yaml.loads(document)

    @benchmark
    def write() -> None:
        api.dump(value, StringIO())
