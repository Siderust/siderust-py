"""SPICE DE440 lunar surface observation binding tests.

Reference CSV and kernel excerpts are independent CSPICE products from
Siderust PR #125 (`tests/fixtures/lunar_de440`). Expected values are never
regenerated with the Siderust implementation under test.

Exact-ET tolerances match the Rust core:
  matrix 5e-14, position/range 2e-7 km, alt/az 1e-10 deg.
"""

from __future__ import annotations

import csv
import gc
import json
import math
import os
from pathlib import Path

import pytest

from siderust import LunarObserver, SpiceContext
from siderust.direction import ICRS

FIXTURES = Path(__file__).resolve().parent / "fixtures" / "lunar_de440"
KERNEL_DIR = os.environ.get("SIDERUST_LUNAR_KERNEL_DIR")

TOL_ANGLE = 1e-10
TOL_RANGE = 2e-7


def angular_delta_deg(a: float, b: float) -> float:
    d = (a - b + 180.0) % 360.0 - 180.0
    return abs(d)


def load_excerpt_context() -> SpiceContext:
    """Match Rust `excerpt()`: SPK/BPC/FK/LSK plus synthetic rotation FK last."""
    return SpiceContext.from_paths(
        [
            FIXTURES / "excerpt.bsp",
            FIXTURES / "excerpt.bpc",
            FIXTURES / "moon_de440_220930.tf",
            FIXTURES / "naif0012.tls",
            FIXTURES / "test_rotation.tf",
        ]
    )


FRAME_NAMES = {
    31011: "MOON_ME",
    31010: "MOON_PA",
    31009: "MOON_ME_DE440_ME421",
    31008: "MOON_PA_DE440",
    139999: "TEST_LUNAR_ROTATED",
}


class TestSpiceContext:
    def test_from_kernels_paths_and_provenance(self):
        spice = load_excerpt_context()
        assert spice.spk_count >= 1
        assert spice.has_lsk
        prov = list(spice.provenance)
        assert any("excerpt.bsp" in p or p.endswith("excerpt.bsp") for p in prov)
        assert spice == spice
        assert hash(spice) == hash(spice)

    def test_pathlib_and_str(self):
        spice = SpiceContext.from_kernels(
            spk=str(FIXTURES / "excerpt.bsp"),
            bpc=FIXTURES / "excerpt.bpc",
            fk=str(FIXTURES / "moon_de440_220930.tf"),
        )
        assert spice.spk_count == 1

    def test_missing_kernel(self, tmp_path: Path):
        with pytest.raises(FileNotFoundError):
            SpiceContext.from_kernels(
                spk=tmp_path / "missing.bsp",
                bpc=FIXTURES / "excerpt.bpc",
                fk=FIXTURES / "moon_de440_220930.tf",
            )

    def test_malformed_non_daf_rejected(self, tmp_path: Path):
        bad = tmp_path / "bad.txt"
        bad.write_bytes(b"\xff\xfe\x00\x01")
        with pytest.raises(ValueError):
            SpiceContext.from_kernels(
                spk=bad,
                bpc=FIXTURES / "excerpt.bpc",
                fk=FIXTURES / "moon_de440_220930.tf",
            )


class TestSpiceLunarObserverConstruction:
    def test_height_and_radial_sites(self):
        spice = load_excerpt_context()
        height = LunarObserver.from_spice(spice, "MOON_ME", 32.3, -89.0, height_m=600.0)
        assert height.backend == "spice"
        assert height.frame == "MOON_ME"
        assert height.frame_id == 31011
        assert height.site_kind == "height"
        assert height.height_m == pytest.approx(600.0)
        assert height.radius_m is None
        assert height.equatorial_radius_m == pytest.approx(1_737_400.0)

        radial = LunarObserver.from_spice(spice, "MOON_ME", 32.3, -89.0, radius_m=1_738_000.0)
        assert radial.site_kind == "radial"
        assert radial.radius_m == pytest.approx(1_738_000.0)
        assert radial.height_m == pytest.approx(0.0)
        assert radial.equatorial_radius_m == pytest.approx(1_738_000.0)

    def test_rejects_conflicting_height_and_radius(self):
        spice = load_excerpt_context()
        with pytest.raises(ValueError, match="not both"):
            LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, height_m=1.0, radius_m=1_737_400.0)

    def test_rejects_invalid_frame_and_site(self):
        spice = load_excerpt_context()
        with pytest.raises(ValueError):
            LunarObserver.from_spice(spice, "J2000", 0.0, 0.0, radius_m=1_737_400.0)
        with pytest.raises(ValueError):
            LunarObserver.from_spice(spice, "IAU_MOON", 0.0, 0.0, radius_m=1_737_400.0)
        with pytest.raises(ValueError):
            LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, radius_m=-1.0)
        with pytest.raises(ValueError):
            LunarObserver.from_spice(spice, "MOON_ME", 0.0, 95.0, height_m=0.0)

    def test_pa_versus_me_frames(self):
        spice = load_excerpt_context()
        me = LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, radius_m=1_737_400.0)
        pa = LunarObserver.from_spice(spice, "MOON_PA", 0.0, 0.0, radius_m=1_737_400.0)
        assert me.frame_id != pa.frame_id
        assert me != pa

    def test_analytical_not_equal_spice(self):
        spice = load_excerpt_context()
        analytical = LunarObserver(0.0, 0.0, 0.0)
        spice_obs = LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, height_m=0.0)
        assert analytical != spice_obs
        assert analytical.backend == "analytical"
        assert spice_obs.backend == "spice"

    def test_context_lifetime_after_gc(self):
        spice = load_excerpt_context()
        observer = LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, radius_m=1_737_400.0)
        del spice
        gc.collect()
        sun = observer.observe_sun_et(819720000.0)
        assert sun.range is not None
        assert sun.range_unit == "km"
        assert sun.backend == "spice"

    def test_serialization_guards(self):
        spice = load_excerpt_context()
        observer = LunarObserver.from_spice(spice, "MOON_ME", 10.0, -5.0, height_m=0.0)
        data = observer.to_dict()
        assert data["backend"] == "spice"
        with pytest.raises(TypeError, match="SPICE"):
            LunarObserver.from_dict(data)
        with pytest.raises(TypeError, match="pickle|SPICE"):
            import pickle

            pickle.dumps(observer)


class TestSpiceObservationParity:
    def test_reference_csv_exact_et(self):
        spice = load_excerpt_context()
        # Also load synthetic rotation if present — reference rows use DE440 FK ids.
        path = FIXTURES / "reference.csv"
        maxima = [0.0] * 3
        with path.open(newline="") as fh:
            reader = csv.DictReader(fh)
            for row in reader:
                et = float(row["et"])
                lon = float(row["lon"])
                lat = float(row["lat"])
                radius_km = float(row["radius_km"])
                target_id = int(float(row["target"]))
                frame_id = int(float(row["frame"]))
                frame = FRAME_NAMES.get(frame_id)
                if frame is None:
                    pytest.fail(f"unsupported reference frame id {frame_id}")
                observer = LunarObserver.from_spice(
                    spice,
                    frame,
                    lon,
                    lat,
                    radius_m=radius_km * 1000.0,
                )
                assert observer.frame_id == frame_id
                if target_id == 10:
                    obs = observer.observe_sun_et(et)
                    assert obs.target == "Sun"
                else:
                    obs = observer.observe_earth_et(et)
                    assert obs.target == "Earth"

                alt_err = abs(obs.alt_deg - float(row["alt"]))
                az_err = angular_delta_deg(obs.az_deg, float(row["az"]))
                range_err = abs(obs.range - float(row["range"]))
                maxima[0] = max(maxima[0], alt_err, az_err)
                maxima[1] = max(maxima[1], range_err)
                assert alt_err < TOL_ANGLE
                assert az_err < TOL_ANGLE
                assert range_err < TOL_RANGE
                assert obs.above_horizon == (float(row["above"]) == 1.0)
                assert obs.range_unit == "km"
                assert obs.epoch_et_tdb_seconds == pytest.approx(et)
                assert obs.backend == "spice"
                assert list(obs.kernels)

        # Determinism: repeat one observation.
        observer = LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, radius_m=1_737_400.0)
        a = observer.observe_sun_et(819720000.0)
        b = observer.observe_sun_et(819720000.0)
        assert a.alt_deg == b.alt_deg
        assert a.az_deg == b.az_deg
        assert a.range == b.range

    def test_jd_tt_path_runs(self):
        spice = load_excerpt_context()
        observer = LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, radius_m=1_737_400.0)
        et_mid = 821_000_000.0  # safely inside excerpt coverage
        sun_et = observer.observe_sun_et(et_mid)
        assert math.isfinite(sun_et.alt_deg)
        # JD TT ≈ J2000 + et/86400; choose mid-coverage so TT→TDB stays inside.
        jd_tt = 2_451_545.0 + et_mid / 86400.0
        sun_tt = observer.observe_sun(jd_tt)
        assert math.isfinite(sun_tt.alt_deg)
        assert sun_tt.range_unit == "km"
        # End-to-end TT→TDB is not identical ET (tempoch TDB−TT ≈ 1e-4 s here).
        assert abs(sun_tt.alt_deg - sun_et.alt_deg) < 1e-4
        assert abs(sun_tt.epoch_et_tdb_seconds - et_mid) < 2e-4
        with pytest.raises(ValueError):
            observer.observe_sun(2_451_545.0)  # J2000 outside excerpt BPC

    def test_rejects_nonfinite_epochs(self):
        spice = load_excerpt_context()
        observer = LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, radius_m=1_737_400.0)
        with pytest.raises(ValueError, match="finite"):
            observer.observe_sun(float("nan"))
        with pytest.raises(ValueError, match="finite"):
            observer.observe_sun_et(float("inf"))

    def test_out_of_coverage(self):
        spice = load_excerpt_context()
        observer = LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, radius_m=1_737_400.0)
        with pytest.raises(ValueError):
            observer.observe_sun_et(819720000.0 - 1.0)

    def test_spice_rejects_direction_and_position(self):
        spice = load_excerpt_context()
        observer = LunarObserver.from_spice(spice, "MOON_ME", 0.0, 0.0, height_m=0.0)
        with pytest.raises(ValueError, match="not supported"):
            observer.observe_direction(ICRS(0.0, 90.0), 2451545.0)
        with pytest.raises(ValueError, match="not supported"):
            observer.observe_icrs_vector(0.0, 0.0, 1.0, 2451545.0)

    def test_poles_and_longitude_wrap_in_reference(self):
        spice = load_excerpt_context()
        # Spot-check known site classes from the first rows / polar rows.
        with (FIXTURES / "reference.csv").open(newline="") as fh:
            rows = list(csv.DictReader(fh))
        polar = [r for r in rows if abs(float(r["lat"])) >= 89.0]
        assert polar, "expected polar sites in reference.csv"
        for row in polar[:4]:
            observer = LunarObserver.from_spice(
                spice,
                FRAME_NAMES[int(float(row["frame"]))],
                float(row["lon"]),
                float(row["lat"]),
                radius_m=float(row["radius_km"]) * 1000.0,
            )
            obs = (
                observer.observe_sun_et(float(row["et"]))
                if int(float(row["target"])) == 10
                else observer.observe_earth_et(float(row["et"]))
            )
            assert angular_delta_deg(obs.az_deg, float(row["az"])) < TOL_ANGLE


class TestAnalyticalRegression:
    def test_analytical_api_unchanged(self):
        site = LunarObserver(20.0, -3.5, 0.0)
        assert site.backend == "analytical"
        hz = site.observe_direction(ICRS(0.0, 90.0), 2_451_545.0)
        assert abs(hz.alt_deg - 17.765928422856998) < 1e-9
        earth = site.observe_earth(2_460_000.5)
        assert earth.range_unit == "km"
        sun = site.observe_sun(2_460_000.5)
        assert sun.range_unit == "au"
        assert LunarObserver.from_dict(site.to_dict()) == site


@pytest.mark.skipif(not KERNEL_DIR, reason="Set SIDERUST_LUNAR_KERNEL_DIR for full-kernel tests")
class TestOptionalFullKernels:
    def test_external_reference(self):
        directory = Path(KERNEL_DIR)
        spice = SpiceContext.from_de440(directory)
        with (FIXTURES / "external_reference.csv").open(newline="") as fh:
            for row in csv.DictReader(fh):
                frame = FRAME_NAMES[int(float(row["frame"]))]
                observer = LunarObserver.from_spice(
                    spice,
                    frame,
                    float(row["lon"]),
                    float(row["lat"]),
                    radius_m=float(row["radius_km"]) * 1000.0,
                )
                obs = (
                    observer.observe_sun_et(float(row["et"]))
                    if int(float(row["target"])) == 10
                    else observer.observe_earth_et(float(row["et"]))
                )
                assert abs(obs.alt_deg - float(row["alt"])) < TOL_ANGLE
                assert angular_delta_deg(obs.az_deg, float(row["az"])) < TOL_ANGLE
                assert abs(obs.range - float(row["range"])) < TOL_RANGE


def test_provenance_manifest_present():
    data = json.loads((FIXTURES / "provenance.json").read_text())
    assert data["generator"].startswith("SpiceyPy")
    assert "source_sha256" in data
