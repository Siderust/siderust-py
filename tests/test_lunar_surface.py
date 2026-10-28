"""Lunar-surface Alt/Az and selenocentric coordinate tests.

Reference values and tolerances mirror Siderust v0.16.0
`tests/body_surface_horizontal.rs` and `examples/22_moon_surface_quickstart.rs`.
CSPICE composite constants come from the upstream offline validation
(CSPICE N0067 / SpiceyPy 8.3.0, pck00011.tpc); ENU geometry is independent.
"""

from __future__ import annotations

import math
import pickle

import pytest

from siderust import Body, LunarObserver, Observer, Position, SphericalPosition
from siderust.direction import ICRS, Horizontal

J2000_TT = 2_451_545.0


def assert_near(actual: float, expected: float, tol: float, label: str) -> None:
    assert abs(actual - expected) < tol, f"{label}: got {actual}, expected {expected} (tol {tol})"


def angular_delta_deg(a: float, b: float) -> float:
    d = abs(a - b) % 360.0
    return min(d, 360.0 - d)


def assert_horizontal_close(a, b, alt_tol: float = 1e-9, az_tol: float = 1e-9) -> None:
    assert_near(a.alt_deg, b.alt_deg, alt_tol, "alt")
    assert angular_delta_deg(a.az_deg, b.az_deg) < az_tol


class TestLunarObserverConstruction:
    def test_site_properties(self):
        site = LunarObserver(20.0, -3.5, 10.0)
        assert_near(site.lon_deg, 20.0, 1e-12, "lon")
        assert_near(site.lat_deg, -3.5, 1e-12, "lat")
        assert_near(site.height_m, 10.0, 1e-12, "height")
        assert_near(site.equatorial_radius_m, 1_737_400.0, 1e-6, "radius")
        assert site.latitude_convention == "planetocentric"
        assert site.body_name == "Moon"

    def test_longitude_normalization(self):
        site = LunarObserver(190.0, 0.0)
        assert_near(site.lon_deg, -170.0, 1e-12, "lon wrap")

    def test_rejects_non_finite(self):
        with pytest.raises(ValueError, match="finite"):
            LunarObserver(float("nan"), 0.0)
        with pytest.raises(ValueError, match="finite"):
            LunarObserver(0.0, float("inf"))

    def test_rejects_invalid_latitude(self):
        with pytest.raises(ValueError):
            LunarObserver(0.0, 95.0)

    def test_rejects_degenerate_height(self):
        with pytest.raises(ValueError):
            LunarObserver(0.0, 0.0, -1_737_400.0)

    def test_earth_observer_unchanged(self):
        earth = Observer(-17.89, 28.75, 2396.0)
        assert abs(earth.lon_deg - (-17.89)) < 1e-9
        assert not isinstance(earth, LunarObserver)

    def test_dict_roundtrip(self):
        site = LunarObserver(15.0, 42.0, 100.0)
        assert LunarObserver.from_dict(site.to_dict()) == site


class TestLunarObserverEqualityHash:
    """Exact equality must agree with hashing (Python data-model invariant)."""

    def test_equal_observers_share_hash(self):
        a = LunarObserver(20.0, -3.5, 0.0)
        b = LunarObserver(20.0, -3.5, 0.0)
        assert a == b
        assert hash(a) == hash(b)

    def test_approximate_neighbours_are_not_equal(self):
        # Regression: former approx __eq__ would accept 1e-7 height diffs while
        # __hash__ used exact bits, violating a == b ⇒ hash(a) == hash(b).
        a = LunarObserver(20.0, -3.5, 0.0)
        b = LunarObserver(20.0, -3.5, 1e-7)
        assert a != b
        assert hash(a) != hash(b)

    def test_different_observers(self):
        a = LunarObserver(20.0, -3.5, 0.0)
        b = LunarObserver(21.0, -3.5, 0.0)
        assert a != b

    def test_signed_zero_equality_and_hash(self):
        pos = LunarObserver(0.0, 0.0, 0.0)
        neg = LunarObserver(-0.0, -0.0, -0.0)
        assert pos == neg
        assert hash(pos) == hash(neg)

    def test_normalized_longitude_equivalence(self):
        east = LunarObserver(190.0, 0.0, 0.0)
        west = LunarObserver(-170.0, 0.0, 0.0)
        assert east.lon_deg == pytest.approx(-170.0)
        assert east == west
        assert hash(east) == hash(west)

    def test_dict_and_set_membership(self):
        site = LunarObserver(20.0, -3.5, 12.0)
        twin = LunarObserver(20.0, -3.5, 12.0)
        mapping = {site: "ok"}
        assert mapping[twin] == "ok"
        assert twin in {site}

    def test_pickle_roundtrip(self):
        site = LunarObserver(15.0, 42.0, 100.0)
        for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
            restored = pickle.loads(pickle.dumps(site, protocol=protocol))
            assert restored == site
            assert hash(restored) == hash(site)


class TestSelenocentricCoordinates:
    def test_construct_selenocentric_moon_frame(self):
        p = Position(
            1.0,
            2.0,
            3.0,
            frame="MoonIauMeanEarth",
            center="Selenocentric",
            unit="km",
        )
        assert p.frame == "MoonIauMeanEarth"
        assert p.center == "Selenocentric"

    def test_moon_principal_axes_alias(self):
        p = Position(1.0, 0.0, 0.0, frame="MoonPrincipalAxes", center="Selenocentric", unit="km")
        assert p.frame == "MoonIauMeanEarth"

    def test_center_roundtrip_geo_seleno(self):
        jd = 2_460_000.5
        moon = Body.Moon.geocentric_position(jd)
        seleno = moon.to_center("Selenocentric", jd)
        assert seleno.center == "Selenocentric"
        back = seleno.to_center("Geocentric", jd)
        err = math.sqrt((moon.x - back.x) ** 2 + (moon.y - back.y) ** 2 + (moon.z - back.z) ** 2)
        assert err < 1e-9

    def test_moon_frame_roundtrip_via_icrs(self):
        jd = J2000_TT
        fixed = Position(
            0.0,
            0.0,
            1.0,
            frame="MoonIauMeanEarth",
            center="Selenocentric",
            unit="km",
        )
        icrs = fixed.to_frame("ICRS", jd)
        back = icrs.to_frame("MoonIauMeanEarth", jd)
        err = math.sqrt((fixed.x - back.x) ** 2 + (fixed.y - back.y) ** 2 + (fixed.z - back.z) ** 2)
        assert err < 1e-10

    def test_unknown_center_rejected(self):
        with pytest.raises(ValueError, match="Unknown center"):
            Position(1.0, 0.0, 0.0, center="Martian")

    def test_unknown_frame_rejected(self):
        with pytest.raises(ValueError, match="Unknown frame"):
            Position(1.0, 0.0, 0.0, frame="MOON_PA_DE")


class TestLunarDirectionAltAz:
    def test_quickstart_ncp_j2000(self):
        site = LunarObserver(20.0, -3.5)
        hz = site.observe_direction(ICRS(0.0, 90.0), J2000_TT)
        assert isinstance(hz, Horizontal)
        assert_near(hz.alt_deg, 17.765928422856998, 1e-9, "NCP alt")
        assert_near(hz.az_deg, 12.045094675187446, 1e-9, "NCP az")

    def test_vector_form_matches_spherical(self):
        site = LunarObserver(20.0, -3.5)
        a = site.observe_direction(ICRS(0.0, 90.0), J2000_TT)
        b = site.observe_icrs_vector(0.0, 0.0, 1.0, J2000_TT)
        assert_near(a.alt_deg, b.alt_deg, 1e-12, "alt parity")
        assert_near(a.az_deg, b.az_deg, 1e-12, "az parity")

    def test_rejects_zero_direction(self):
        site = LunarObserver(0.0, 0.0)
        with pytest.raises(ValueError, match="non-zero"):
            site.observe_icrs_vector(0.0, 0.0, 0.0, J2000_TT)

    def test_rejects_non_finite_epoch(self):
        site = LunarObserver(0.0, 0.0)
        with pytest.raises(ValueError, match="jd_tt"):
            site.observe_icrs_vector(0.0, 0.0, 1.0, float("nan"))

    def test_extreme_magnitude_scalar_multiples_agree(self):
        site = LunarObserver(20.0, -3.5)
        v = (1.0, 2.0, 3.0)
        ref = site.observe_icrs_vector(*v, J2000_TT)
        for scale in (1e-200, 1e-100, 1e100, 1e200):
            scaled = tuple(c * scale for c in v)
            hz = site.observe_icrs_vector(*scaled, J2000_TT)
            assert_horizontal_close(hz, ref, alt_tol=1e-9, az_tol=1e-9)

    def test_axis_aligned_extreme_magnitudes(self):
        site = LunarObserver(20.0, -3.5)
        for axis in ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)):
            ref = site.observe_icrs_vector(*axis, J2000_TT)
            for scale in (1e-200, 1e200):
                hz = site.observe_icrs_vector(*(c * scale for c in axis), J2000_TT)
                assert_horizontal_close(hz, ref)

    def test_rejects_nan_and_infinity_components(self):
        site = LunarObserver(0.0, 0.0)
        for bad in (float("nan"), float("inf"), float("-inf")):
            with pytest.raises(ValueError, match="finite"):
                site.observe_icrs_vector(bad, 0.0, 0.0, J2000_TT)
            with pytest.raises(ValueError, match="finite"):
                site.observe_icrs_vector(0.0, bad, 0.0, J2000_TT)
            with pytest.raises(ValueError, match="finite"):
                site.observe_icrs_vector(0.0, 0.0, bad, J2000_TT)

    def test_vector_matches_icrs_direction_object(self):
        site = LunarObserver(20.0, -3.5)
        # RA=0°, Dec=0° → Cartesian ≈ (1, 0, 0)
        spherical = site.observe_direction(ICRS(0.0, 0.0), J2000_TT)
        cartesian = site.observe_icrs_vector(1.0, 0.0, 0.0, J2000_TT)
        assert_horizontal_close(spherical, cartesian)


class TestPositionDeserializationValidation:
    """from_dict must enforce the same invariants as the public constructors."""

    def test_position_from_dict_rejects_non_finite(self):
        base = {
            "x": 1.0,
            "y": 0.0,
            "z": 0.0,
            "frame": "ICRS",
            "center": "Selenocentric",
            "unit": "km",
        }
        for key in ("x", "y", "z"):
            for bad in (float("nan"), float("inf"), float("-inf")):
                payload = dict(base)
                payload[key] = bad
                with pytest.raises(ValueError, match="finite"):
                    Position.from_dict(payload)

    def test_spherical_from_dict_rejects_non_finite(self):
        base = {
            "lon_deg": 10.0,
            "lat_deg": 20.0,
            "distance": 1.0,
            "frame": "ICRS",
            "center": "Selenocentric",
            "unit": "au",
        }
        for key in ("lon_deg", "lat_deg", "distance"):
            for bad in (float("nan"), float("inf"), float("-inf")):
                payload = dict(base)
                payload[key] = bad
                with pytest.raises(ValueError, match="finite"):
                    SphericalPosition.from_dict(payload)

    def test_from_dict_missing_keys(self):
        with pytest.raises(ValueError, match="missing"):
            Position.from_dict({"x": 1.0, "y": 0.0, "z": 0.0})
        with pytest.raises(ValueError, match="missing"):
            SphericalPosition.from_dict({"lon_deg": 0.0, "lat_deg": 0.0})

    def test_from_dict_invalid_frame_center_unit(self):
        with pytest.raises(ValueError, match="Unknown frame"):
            Position.from_dict(
                {
                    "x": 1.0,
                    "y": 0.0,
                    "z": 0.0,
                    "frame": "MOON_PA_DE",
                    "center": "Selenocentric",
                    "unit": "km",
                }
            )
        with pytest.raises(ValueError, match="Unknown center"):
            Position.from_dict(
                {
                    "x": 1.0,
                    "y": 0.0,
                    "z": 0.0,
                    "frame": "ICRS",
                    "center": "Martian",
                    "unit": "km",
                }
            )
        with pytest.raises(ValueError, match="Unknown unit"):
            Position.from_dict(
                {
                    "x": 1.0,
                    "y": 0.0,
                    "z": 0.0,
                    "frame": "ICRS",
                    "center": "Selenocentric",
                    "unit": "m",
                }
            )

    def test_position_dict_roundtrip_and_alias(self):
        original = Position(
            1.0,
            2.0,
            3.0,
            frame="MoonPrincipalAxes",
            center="Selenocentric",
            unit="km",
        )
        restored = Position.from_dict(original.to_dict())
        assert restored.frame == "MoonIauMeanEarth"
        assert restored.x == original.x
        assert restored.center == original.center
        aliased = Position.from_dict(
            {
                "x": 1.0,
                "y": 0.0,
                "z": 0.0,
                "frame": "MoonPrincipalAxes",
                "center": "Selenocentric",
                "unit": "km",
            }
        )
        assert aliased.frame == "MoonIauMeanEarth"

    def test_spherical_dict_roundtrip_and_pickle(self):
        sph = SphericalPosition(
            30.0,
            -10.0,
            2.0,
            frame="MoonIauMeanEarth",
            center="Selenocentric",
            unit="km",
        )
        restored = SphericalPosition.from_dict(sph.to_dict())
        assert restored.lon_deg == sph.lon_deg
        assert restored.frame == sph.frame
        for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
            pickled = pickle.loads(pickle.dumps(sph, protocol=protocol))
            assert pickled.lon_deg == sph.lon_deg
            assert pickled.frame == sph.frame

    def test_position_pickle_roundtrip(self):
        pos = Position(1.0, 0.0, 0.0, frame="ICRS", center="Selenocentric", unit="km")
        for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
            restored = pickle.loads(pickle.dumps(pos, protocol=protocol))
            assert restored.x == pos.x
            assert restored.frame == pos.frame


class TestMoonBodyFixedCardinals:
    """Body-fixed cardinals at the lunar equator (lon=lat=0), via ICRS."""

    @pytest.mark.parametrize(
        ("vec", "alt", "az", "check_az"),
        [
            ((1.0, 0.0, 0.0), 90.0, 0.0, False),  # zenith / +X
            ((0.0, 1.0, 0.0), 0.0, 90.0, True),  # east
            ((0.0, 0.0, 1.0), 0.0, 0.0, True),  # north
            ((0.0, -1.0, 0.0), 0.0, 270.0, True),  # west
            ((0.0, 0.0, -1.0), 0.0, 180.0, True),  # south
            ((-1.0, 0.0, 0.0), -90.0, 0.0, False),  # nadir
        ],
    )
    def test_cardinals(self, vec, alt, az, check_az):
        site = LunarObserver(0.0, 0.0)
        fixed = Position(
            *vec,
            frame="MoonIauMeanEarth",
            center="Selenocentric",
            unit="km",
        )
        icrs = fixed.to_frame("ICRS", J2000_TT)
        hz = site.observe_icrs_vector(icrs.x, icrs.y, icrs.z, J2000_TT)
        assert_near(hz.alt_deg, alt, 1e-6, "alt")
        if check_az:
            assert angular_delta_deg(hz.az_deg, az) < 1e-4


class TestFiniteDistanceParallax:
    def test_near_target_shifts_altitude(self):
        site = LunarObserver(0.0, 0.0)
        fixed = Position(0.0, 0.0, 1.0, frame="MoonIauMeanEarth", center="Selenocentric", unit="km")
        icrs = fixed.to_frame("ICRS", J2000_TT)
        dir_alt = site.observe_icrs_vector(icrs.x, icrs.y, icrs.z, J2000_TT).alt_deg

        near = Position(
            0.0,
            0.0,
            2_000.0,
            frame="MoonIauMeanEarth",
            center="Selenocentric",
            unit="km",
        )
        near_alt = site.observe_position(near, J2000_TT).alt_deg
        assert_near(dir_alt, 0.0, 1e-6, "direction alt")
        assert near_alt < dir_alt - 10.0

    def test_cspice_composite_body_fixed(self):
        site = LunarObserver(20.0, -3.5)
        target = Position(
            1617.7391512763131,
            642.0177864954426,
            -96.38962730131595,
            frame="MoonIauMeanEarth",
            center="Selenocentric",
            unit="km",
        )
        obs = site.observe_position(target, J2000_TT)
        assert_near(obs.alt_deg, 5.60040918481664, 1e-9, "ENU alt")
        assert_near(obs.az_deg, 78.69006752597979, 1e-9, "ENU az")
        assert_near(obs.range, 51.23475382979799, 1e-9, "range")
        assert obs.range_unit == "km"
        hz = obs.direction()
        assert_near(hz.alt_deg, obs.alt_deg, 1e-12, "direction alt")
        assert_near(hz.az_deg, obs.az_deg, 1e-12, "direction az")

    def test_cspice_icrs_path(self):
        site = LunarObserver(20.0, -3.5)
        # Same body-fixed target rotated with the CSPICE pxform matrix
        # (v_j2000 = M · v_body) from the upstream composite validation.
        m = [
            [0.7842270522119748, -0.6200619150990117, -0.022608671404191448],
            [0.5578471123206701, 0.720556665574856, -0.4118309009425287],
            [0.2716514860154575, 0.31035675139972596, 0.9109797785934671],
        ]
        tf = [1617.7391512763131, 642.0177864954426, -96.38962730131595]
        target = Position(
            m[0][0] * tf[0] + m[0][1] * tf[1] + m[0][2] * tf[2],
            m[1][0] * tf[0] + m[1][1] * tf[1] + m[1][2] * tf[2],
            m[2][0] * tf[0] + m[2][1] * tf[1] + m[2][2] * tf[2],
            frame="ICRS",
            center="Selenocentric",
            unit="km",
        )
        obs = site.observe_position(target, J2000_TT)
        assert_near(obs.alt_deg, 5.60040918481664, 5e-8, "ICRS alt")
        assert_near(obs.az_deg, 78.69006752597979, 5e-8, "ICRS az")
        assert_near(obs.range, 51.23475382979799, 1e-6, "ICRS range")


class TestZenithNadir:
    @pytest.mark.parametrize("lon,lat", [(0.0, 0.0), (20.0, -3.5), (-120.0, 45.0), (30.0, 90.0)])
    def test_zenith_azimuth_conventionally_zero(self, lon, lat):
        site = LunarObserver(lon, lat)
        # Local up ≈ radial for the IAU sphere: observe a nearby radial target.
        r = site.equatorial_radius_m / 1000.0  # km
        lon_r = math.radians(site.lon_deg)
        lat_r = math.radians(site.lat_deg)
        x = (r + 1.0) * math.cos(lat_r) * math.cos(lon_r)
        y = (r + 1.0) * math.cos(lat_r) * math.sin(lon_r)
        z = (r + 1.0) * math.sin(lat_r)
        target = Position(x, y, z, frame="MoonIauMeanEarth", center="Selenocentric", unit="km")
        obs = site.observe_position(target, J2000_TT)
        assert_near(obs.alt_deg, 90.0, 1e-6, "zenith alt")
        assert obs.az_deg == 0.0


class TestEarthAndSunFromMoon:
    def test_observe_earth_and_sun(self):
        site = LunarObserver(20.0, -3.5)
        jd = 2_460_000.5
        earth = site.observe_earth(jd)
        sun = site.observe_sun(jd)
        assert earth.range is not None and earth.range_unit == "km"
        assert sun.range is not None and sun.range_unit == "au"
        assert -90.0 <= earth.alt_deg <= 90.0
        assert 0.0 <= earth.az_deg < 360.0
        assert -90.0 <= sun.alt_deg <= 90.0
        assert sun.range > 0.9  # roughly 1 au


class TestIauMoonOrientationAxes:
    """Independent CSPICE axis references at J2000 (direction cosines)."""

    def test_moon_pole_matches_published_j2000(self):
        ra_deg = 266.85773344495135
        dec_deg = 65.64110274784532
        ra = math.radians(ra_deg)
        dec = math.radians(dec_deg)
        expected = (
            math.cos(dec) * math.cos(ra),
            math.cos(dec) * math.sin(ra),
            math.sin(dec),
        )
        pole = Position(
            0.0,
            0.0,
            1.0,
            frame="MoonIauMeanEarth",
            center="Selenocentric",
            unit="km",
        ).to_frame("ICRS", J2000_TT)
        assert_near(pole.x, expected[0], 1e-10, "pole x")
        assert_near(pole.y, expected[1], 1e-10, "pole y")
        assert_near(pole.z, expected[2], 1e-10, "pole z")
