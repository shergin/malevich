import re
from pathlib import Path

import pytest

import supremus

HERE = Path(__file__).resolve()
CRATE = HERE.parents[2] / "Cargo.toml"
PYPROJECT = HERE.parents[1] / "pyproject.toml"


def version_in(path: Path) -> str:
    match = re.search(r'^version = "([^"]+)"', path.read_text(encoding="utf-8"), re.MULTILINE)
    assert match, path
    return match.group(1)


def test_engine_version_names_the_crate_it_was_built_from() -> None:
    if not CRATE.exists():
        pytest.skip("the crate manifest lives in the repository checkout")
    assert supremus.ENGINE_VERSION == version_in(CRATE)


def test_package_version_matches_pyproject() -> None:
    if not PYPROJECT.exists():
        pytest.skip("pyproject lives in the repository checkout")
    assert supremus.__version__ == version_in(PYPROJECT)


def test_native_errors_become_supremus_errors() -> None:
    with pytest.raises(supremus.SupremusError) as caught:
        supremus.Plot.from_document({"version": 1, "kind": "plot", "spec": {"layers": [{"Nope": {}}]}}).validate()
    assert caught.value.code == "Format"
    assert isinstance(caught.value, Exception)
    assert str(caught.value)


def test_all_exports_resolve() -> None:
    for name in supremus.__all__:
        assert hasattr(supremus, name), name
