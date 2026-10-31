"""Tests for the canonical Python time facade and dependency contract."""

import types
from importlib import import_module
from importlib.metadata import distribution

import tempoch

import siderust
from siderust import time


def test_time_is_an_importable_public_module():
    assert isinstance(time, types.ModuleType)
    assert import_module("siderust.time") is time
    assert "time" in siderust.__all__


def test_exports_reuse_canonical_tempoch_types_and_functions():
    expected = set(tempoch.__all__) - {"__version__"}
    assert set(time.__all__) == expected
    for name in expected:
        assert getattr(time, name) is getattr(tempoch, name)


def test_typed_time_values_with_existing_mjd_float_api():
    jd = time.JulianDate.j2000()
    assert jd.value == 2451545.0
    mjd = jd.to_mjd()
    assert isinstance(mjd, time.ModifiedJulianDate)
    assert mjd.value == 51544.5

    observer = siderust.Observer.roque_de_los_muchachos()
    altitude = siderust.Body.Sun.altitude_at(observer, mjd.value)
    assert -90.0 <= altitude <= 90.0


def test_time_and_astronomy_interval_helpers_remain_distinct():
    assert time.intersect_periods is tempoch.intersect_periods
    assert time.intersect_periods is not siderust.intersect_periods


def test_published_distribution_declares_tempoch():
    requirements = distribution("siderust").requires or []
    assert any(requirement.lower().startswith("tempoch") for requirement in requirements)
