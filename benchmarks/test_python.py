from io import BytesIO, StringIO
from pathlib import Path

import pytest
from pytest_codspeed import BenchmarkFixture

import astral_yaml


@pytest.fixture(params=["small-config", "uv-ci", "airflow-compose"])
def document(request: pytest.FixtureRequest) -> str:
    if request.param == "small-config":
        return "name: café\nports: [8000, 8443]\nratio: 0.75\nenabled: true\n"
    fixtures = Path(__file__).resolve().parents[1] / "fixtures" / "real-world"
    return (fixtures / f"{request.param}.yaml").read_text(encoding="utf-8")


def test_loads(benchmark: BenchmarkFixture, document: str) -> None:
    benchmark(astral_yaml.loads, document)


def test_load(benchmark: BenchmarkFixture, document: str) -> None:
    source = document.encode()

    @benchmark
    def read() -> None:
        astral_yaml.load(BytesIO(source))


def test_dumps(benchmark: BenchmarkFixture, document: str) -> None:
    value = astral_yaml.loads(document)
    benchmark(astral_yaml.dumps, value)


def test_dump(benchmark: BenchmarkFixture, document: str) -> None:
    value = astral_yaml.loads(document)

    @benchmark
    def write() -> None:
        astral_yaml.dump(value, StringIO())
