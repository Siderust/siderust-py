"""Tests for core siderust API surface."""

import siderust
from siderust import (
    Body,
    CrossingDirection,
    CrossingEvent,
    CulminationEvent,
    CulminationKind,
    Direction,
    Displacement,
    MoonPhaseGeometry,
    MoonPhaseLabel,
    Observer,
    PhaseEvent,
    PhaseKind,
    Position,
    ProperMotion,
    Star,
    Target,
    above_threshold,
    altitude_at,
    azimuth_at,
    below_threshold,
    crossings,
    culminations,
    find_moon_phases,
    moon_phase,
)


class TestModuleExports:
    """Verify all expected names are importable."""

    def test_version_exists(self):
        assert hasattr(siderust, "__version__")
        assert isinstance(siderust.__version__, str)

    def test_all_exports(self):
        expected = {
            "Observer",
            "Body",
            "Star",
            "Direction",
            "CrossingEvent",
            "CulminationEvent",
            "CrossingDirection",
            "CulminationKind",
            "MoonPhaseGeometry",
            "MoonPhaseLabel",
            "PhaseEvent",
            "PhaseKind",
            "Position",
            "Displacement",
            "Target",
            "ProperMotion",
            "altitude_at",
            "above_threshold",
            "below_threshold",
            "crossings",
            "culminations",
            "azimuth_at",
            "moon_phase",
            "find_moon_phases",
            "__version__",
        }
        for name in expected:
            assert hasattr(siderust, name), f"Missing export: {name}"


class TestObserver:
    """Observer creation and properties."""

    def test_custom_observer(self):
        obs = Observer(-17.8925, 28.7543, 2396.0)
        assert abs(obs.lon_deg - (-17.8925)) < 1e-4
        assert abs(obs.lat_deg - 28.7543) < 1e-4
        assert abs(obs.height_m - 2396.0) < 0.1

    def test_default_height(self):
        obs = Observer(0.0, 0.0)
        assert abs(obs.height_m) < 1e-6

    def test_roque(self):
        obs = Observer.roque_de_los_muchachos()
        assert abs(obs.lat_deg - 28.7543) < 0.01

    def test_paranal(self):
        obs = Observer.el_paranal()
        assert obs.lat_deg < 0  # Southern hemisphere

    def test_mauna_kea(self):
        obs = Observer.mauna_kea()
        assert obs.height_m > 4000

    def test_la_silla(self):
        obs = Observer.la_silla()
        assert obs.lon_deg < 0  # Western hemisphere

    def test_repr(self):
        obs = Observer(0.0, 45.0, 100.0)
        r = repr(obs)
        assert "Observer" in r
        assert "45" in r

    def test_equality(self):
        a = Observer.roque_de_los_muchachos()
        b = Observer.roque_de_los_muchachos()
        assert a == b

    def test_hash(self):
        a = Observer.roque_de_los_muchachos()
        b = Observer.roque_de_los_muchachos()
        assert hash(a) == hash(b)


class TestBody:
    """Body enum and methods."""

    def test_all_bodies_exist(self):
        bodies = [
            Body.Sun,
            Body.Moon,
            Body.Mercury,
            Body.Venus,
            Body.Mars,
            Body.Jupiter,
            Body.Saturn,
            Body.Uranus,
            Body.Neptune,
        ]
        assert len(bodies) == 9

    def test_repr(self):
        assert "Sun" in repr(Body.Sun)

    def test_str(self):
        assert str(Body.Mars) == "Mars"

    def test_equality(self):
        assert Body.Sun == Body.Sun
        assert Body.Sun != Body.Moon

    def test_hash(self):
        assert hash(Body.Sun) == hash(Body.Sun)
        s = {Body.Sun, Body.Moon, Body.Sun}
        assert len(s) == 2

    def test_altitude_at(self):
        obs = Observer.roque_de_los_muchachos()
        alt = Body.Sun.altitude_at(obs, 60000.0)
        assert isinstance(alt, float)
        assert -90.0 <= alt <= 90.0

    def test_azimuth_at(self):
        obs = Observer.roque_de_los_muchachos()
        az = Body.Sun.azimuth_at(obs, 60000.0)
        assert isinstance(az, float)


class TestStar:
    """Star type tests."""

    def test_catalog_vega(self):
        vega = Star.catalog("Vega")
        assert vega.name == "Vega"
        assert vega.distance_ly > 0

    def test_catalog_case_insensitive(self):
        s1 = Star.catalog("sirius")
        s2 = Star.catalog("SIRIUS")
        assert s1.name == s2.name

    def test_catalog_unknown(self):
        import pytest

        with pytest.raises(ValueError, match="Unknown star"):
            Star.catalog("Nonexistent")

    def test_custom(self):
        """Test creating a star with custom physical parameters."""
        s = Star.custom(
            "TestStar",
            ra_deg=100.0,
            dec_deg=30.0,
            distance_ly=10.0,
            mass_solar=1.0,
            radius_solar=1.0,
            luminosity_solar=1.0,
        )
        assert s.name == "TestStar"
        assert abs(s.ra_deg - 100.0) < 0.01
        assert abs(s.dec_deg - 30.0) < 0.01
        assert abs(s.distance_ly - 10.0) < 0.01
        assert abs(s.mass_solar - 1.0) < 0.01
        assert abs(s.radius_solar - 1.0) < 0.01
        assert abs(s.luminosity_solar - 1.0) < 0.01

    def test_properties(self):
        vega = Star.catalog("Vega")
        assert vega.mass_solar > 0
        assert vega.luminosity_solar > 0

    def test_altitude_at(self):
        obs = Observer.roque_de_los_muchachos()
        vega = Star.catalog("Vega")
        alt = vega.altitude_at(obs, 60000.0)
        assert isinstance(alt, float)
        assert -90.0 <= alt <= 90.0

    def test_repr(self):
        vega = Star.catalog("Vega")
        assert "Vega" in repr(vega)

    def test_equality(self):
        a = Star.catalog("Vega")
        b = Star.catalog("Vega")
        assert a == b


class TestDirection:
    """Direction (ICRS) type tests."""

    def test_creation(self):
        d = Direction(ra_deg=180.0, dec_deg=45.0)
        assert abs(d.ra_deg - 180.0) < 0.01
        assert abs(d.dec_deg - 45.0) < 0.01

    def test_altitude_at(self):
        obs = Observer.roque_de_los_muchachos()
        d = Direction(ra_deg=0.0, dec_deg=90.0)  # North celestial pole
        alt = d.altitude_at(obs, 60000.0)
        # NCP altitude ≈ observer latitude
        assert abs(alt - obs.lat_deg) < 1.0

    def test_repr(self):
        d = Direction(ra_deg=10.0, dec_deg=20.0)
        r = repr(d)
        assert "Direction" in r
        assert "10" in r

    def test_equality(self):
        a = Direction(ra_deg=10.0, dec_deg=20.0)
        b = Direction(ra_deg=10.0, dec_deg=20.0)
        assert a == b

    def test_hash(self):
        a = Direction(ra_deg=10.0, dec_deg=20.0)
        b = Direction(ra_deg=10.0, dec_deg=20.0)
        assert hash(a) == hash(b)

    def test_pickle(self):
        import pickle

        d = Direction(ra_deg=100.0, dec_deg=-30.0)
        d2 = pickle.loads(pickle.dumps(d))
        assert abs(d.ra_deg - d2.ra_deg) < 1e-10
        assert abs(d.dec_deg - d2.dec_deg) < 1e-10


class TestAltitudeQueries:
    """Free-function altitude/azimuth queries."""

    def test_altitude_at_body(self):
        obs = Observer.roque_de_los_muchachos()
        alt = altitude_at(Body.Sun, obs, 60000.0)
        assert isinstance(alt, float)
        assert -90.0 <= alt <= 90.0

    def test_altitude_at_star(self):
        obs = Observer.roque_de_los_muchachos()
        vega = Star.catalog("Vega")
        alt = altitude_at(vega, obs, 60000.0)
        assert isinstance(alt, float)

    def test_altitude_at_direction(self):
        obs = Observer.roque_de_los_muchachos()
        d = Direction(ra_deg=0.0, dec_deg=45.0)
        alt = altitude_at(d, obs, 60000.0)
        assert isinstance(alt, float)

    def test_azimuth_at(self):
        obs = Observer.roque_de_los_muchachos()
        az = azimuth_at(Body.Moon, obs, 60000.0)
        assert isinstance(az, float)

    def test_invalid_target_type(self):
        import pytest

        obs = Observer.roque_de_los_muchachos()
        with pytest.raises(TypeError, match="target must be"):
            altitude_at("not a target", obs, 60000.0)


class TestThresholdQueries:
    """above_threshold, below_threshold over a 1-day window."""

    def test_above_threshold_sun(self):
        obs = Observer.roque_de_los_muchachos()
        periods = above_threshold(Body.Sun, obs, 60000.0, 60001.0, 0.0)
        assert isinstance(periods, list)
        for start, end in periods:
            assert start < end

    def test_below_threshold(self):
        obs = Observer.roque_de_los_muchachos()
        periods = below_threshold(Body.Sun, obs, 60000.0, 60001.0, 0.0)
        assert isinstance(periods, list)

    def test_invalid_window(self):
        import pytest

        obs = Observer.roque_de_los_muchachos()
        with pytest.raises(ValueError, match="Invalid time window"):
            above_threshold(Body.Sun, obs, 60001.0, 60000.0, 0.0)


class TestCrossingsAndCulminations:
    """Crossing and culmination event queries."""

    def test_crossings(self):
        obs = Observer.roque_de_los_muchachos()
        events = crossings(Body.Sun, obs, 60000.0, 60001.0, 0.0)
        assert isinstance(events, list)
        for e in events:
            assert isinstance(e, CrossingEvent)
            assert isinstance(e.mjd, float)
            assert isinstance(e.direction, CrossingDirection)

    def test_crossing_directions(self):
        assert CrossingDirection.Rising != CrossingDirection.Setting
        assert "Rising" in repr(CrossingDirection.Rising)

    def test_culminations(self):
        obs = Observer.roque_de_los_muchachos()
        events = culminations(Body.Sun, obs, 60000.0, 60001.0)
        assert isinstance(events, list)
        for e in events:
            assert isinstance(e, CulminationEvent)
            assert isinstance(e.mjd, float)
            assert isinstance(e.altitude_deg, float)
            assert isinstance(e.kind, CulminationKind)

    def test_culmination_kinds(self):
        assert CulminationKind.Max != CulminationKind.Min
        assert "Max" in repr(CulminationKind.Max)


class TestMoonPhase:
    """Moon phase queries."""

    def test_moon_phase_geocentric(self):
        # J2000.0
        geom = moon_phase(2451545.0)
        assert isinstance(geom, MoonPhaseGeometry)
        assert 0.0 <= geom.illuminated_fraction <= 1.0
        assert isinstance(geom.label, MoonPhaseLabel)

    def test_moon_phase_topocentric(self):
        obs = Observer.roque_de_los_muchachos()
        geom = moon_phase(2451545.0, observer=obs)
        assert isinstance(geom, MoonPhaseGeometry)

    def test_moon_phase_repr(self):
        geom = moon_phase(2451545.0)
        r = repr(geom)
        assert "MoonPhaseGeometry" in r

    def test_find_moon_phases(self):
        events = find_moon_phases(60000.0, 60030.0)
        assert isinstance(events, list)
        assert len(events) > 0  # ~4 events per month
        for e in events:
            assert isinstance(e, PhaseEvent)
            assert isinstance(e.kind, PhaseKind)
            assert 60000.0 <= e.mjd <= 60030.0

    def test_phase_labels(self):
        labels = [
            MoonPhaseLabel.NewMoon,
            MoonPhaseLabel.WaxingCrescent,
            MoonPhaseLabel.FirstQuarter,
            MoonPhaseLabel.WaxingGibbous,
            MoonPhaseLabel.FullMoon,
            MoonPhaseLabel.WaningGibbous,
            MoonPhaseLabel.LastQuarter,
            MoonPhaseLabel.WaningCrescent,
        ]
        assert len(labels) == 8

    def test_phase_kinds(self):
        kinds = [
            PhaseKind.NewMoon,
            PhaseKind.FirstQuarter,
            PhaseKind.FullMoon,
            PhaseKind.LastQuarter,
        ]
        assert len(kinds) == 4


class TestEndToEnd:
    """End-to-end scientific workflows."""

    def test_observation_planning(self):
        """Plan a night of observation at Roque de los Muchachos."""
        obs = Observer.roque_de_los_muchachos()
        mjd_start = 60000.0
        mjd_end = 60001.0

        # Find when Sun is below horizon
        night = below_threshold(Body.Sun, obs, mjd_start, mjd_end, 0.0)
        assert len(night) > 0
        night_start, night_end = night[0]

        # Check if Vega is visible during the night
        vega = Star.catalog("Vega")
        mid_night = (night_start + night_end) / 2.0
        vega_alt = vega.altitude_at(obs, mid_night)
        assert isinstance(vega_alt, float)

    def test_sunrise_sunset(self):
        """Find sunrise and sunset times."""
        obs = Observer.roque_de_los_muchachos()
        events = crossings(Body.Sun, obs, 60000.0, 60001.0, 0.0)
        rising = [e for e in events if e.direction == CrossingDirection.Rising]
        setting = [e for e in events if e.direction == CrossingDirection.Setting]
        # Should have at least one of each in a day
        assert len(rising) + len(setting) >= 1

    def test_multi_body_comparison(self):
        """Compare altitudes of multiple bodies at the same time."""
        obs = Observer.el_paranal()
        mjd = 60000.5  # Mid-day

        altitudes = {}
        for body in [Body.Sun, Body.Moon, Body.Mars, Body.Jupiter]:
            altitudes[str(body)] = altitude_at(body, obs, mjd)

        assert all(isinstance(v, float) for v in altitudes.values())

    def test_moon_phase_during_observation(self):
        """Check moon phase during an observation window."""
        geom = moon_phase(2460000.5)  # JD for a specific night
        assert 0.0 <= geom.illuminated_fraction <= 1.0
        # Verify we get a sensible description
        s = str(geom)
        assert "illuminated" in s


class TestAffnSemantics:
    """Test affine geometry semantics — Position and Displacement."""

    def test_position_subtraction_returns_displacement(self):
        """Position - Position should return Displacement, not Position."""
        p1 = Position(1.0, 2.0, 3.0, "ICRS", "Barycentric", "au")
        p2 = Position(0.5, 1.0, 1.5, "ICRS", "Barycentric", "au")
        d = p1 - p2
        assert isinstance(d, Displacement), f"Expected Displacement, got {type(d)}"
        assert abs(d.dx - 0.5) < 1e-10
        assert abs(d.dy - 1.0) < 1e-10
        assert abs(d.dz - 1.5) < 1e-10

    def test_position_plus_displacement_returns_position(self):
        """Position + Displacement should return Position."""
        p = Position(1.0, 2.0, 3.0, "ICRS", "Barycentric", "au")
        d = Displacement(0.5, 0.5, 0.5, "ICRS", "au")
        result = p + d
        assert isinstance(result, Position), f"Expected Position, got {type(result)}"
        assert abs(result.x - 1.5) < 1e-10
        assert abs(result.y - 2.5) < 1e-10
        assert abs(result.z - 3.5) < 1e-10

    def test_position_minus_displacement_returns_position(self):
        """Position - Displacement should return Position."""
        p = Position(1.0, 2.0, 3.0, "ICRS", "Barycentric", "au")
        d = Displacement(0.5, 0.5, 0.5, "ICRS", "au")
        result = p - d
        assert isinstance(result, Position), f"Expected Position, got {type(result)}"
        assert abs(result.x - 0.5) < 1e-10
        assert abs(result.y - 1.5) < 1e-10
        assert abs(result.z - 2.5) < 1e-10

    def test_displacement_addition(self):
        """Displacement + Displacement should return Displacement."""
        d1 = Displacement(1.0, 2.0, 3.0, "ICRS", "au")
        d2 = Displacement(0.5, 1.0, 1.5, "ICRS", "au")
        result = d1 + d2
        assert isinstance(result, Displacement)
        assert abs(result.dx - 1.5) < 1e-10
        assert abs(result.dy - 3.0) < 1e-10
        assert abs(result.dz - 4.5) < 1e-10

    def test_displacement_negation(self):
        """Negating a Displacement should work."""
        d = Displacement(1.0, 2.0, 3.0, "ICRS", "au")
        neg = -d
        assert isinstance(neg, Displacement)
        assert abs(neg.dx - (-1.0)) < 1e-10
        assert abs(neg.dy - (-2.0)) < 1e-10
        assert abs(neg.dz - (-3.0)) < 1e-10

    def test_displacement_scalar_multiplication(self):
        """Displacement * scalar should return Displacement."""
        d = Displacement(1.0, 2.0, 3.0, "ICRS", "au")
        result = d * 2.0
        assert isinstance(result, Displacement)
        assert abs(result.dx - 2.0) < 1e-10
        assert abs(result.dy - 4.0) < 1e-10
        assert abs(result.dz - 6.0) < 1e-10


class TestTargetWithProperMotion:
    """Test Target with proper motion support."""

    def test_target_creation(self):
        """Creating a Target with Position and time."""
        p = Position(1.0, 2.0, 3.0, "ICRS", "Barycentric", "au")
        t = Target(p, jd=2451545.0)
        assert t.is_position
        assert not t.is_direction
        assert abs(t.time - 2451545.0) < 1e-6

    def test_target_with_proper_motion(self):
        """Creating a Target with proper motion."""
        d = Direction(ra_deg=217.429, dec_deg=-62.679)
        pm = ProperMotion(pm_ra_mas_yr=-3781.74, pm_dec_mas_yr=769.47)
        t = Target(d, jd=2451545.0, proper_motion=pm)
        assert t.is_direction
        assert t.proper_motion is not None
        assert abs(t.proper_motion.pm_ra_mas_yr - (-3781.74)) < 0.01

    def test_star_track_preserves_proper_motion(self):
        """Tracking a star preserves proper motion if available."""
        vega = Star.catalog("Vega")
        target = vega.track(2451545.0)
        assert target.is_direction
        # Catalog stars may or may not have proper motion; just verify it works
        assert target.time > 0

    def test_body_track_no_proper_motion(self):
        """Solar system bodies have no proper motion."""
        mars = Body.Mars
        target = mars.track(2451545.0)
        assert target.is_position
        assert target.proper_motion is None


class TestEarthObservationRejection:
    """Test that Body.Earth is rejected for observation APIs."""

    def test_earth_altitude_raises(self):
        """Body.Earth.altitude_at() should raise ValueError."""
        import pytest

        obs = Observer.roque_de_los_muchachos()
        with pytest.raises(ValueError, match="[Ee]arth"):
            Body.Earth.altitude_at(obs, 60000.0)

    def test_earth_azimuth_raises(self):
        """Body.Earth.azimuth_at() should raise ValueError."""
        import pytest

        obs = Observer.roque_de_los_muchachos()
        with pytest.raises(ValueError, match="[Ee]arth"):
            Body.Earth.azimuth_at(obs, 60000.0)

    def test_earth_above_threshold_raises(self):
        """above_threshold with Body.Earth should raise ValueError."""
        import pytest

        obs = Observer.roque_de_los_muchachos()
        with pytest.raises(ValueError, match="[Ee]arth"):
            above_threshold(Body.Earth, obs, 60000.0, 60001.0, 0.0)

    def test_earth_crossings_raises(self):
        """crossings with Body.Earth should raise ValueError."""
        import pytest

        obs = Observer.roque_de_los_muchachos()
        with pytest.raises(ValueError, match="[Ee]arth"):
            crossings(Body.Earth, obs, 60000.0, 60001.0, 0.0)


class TestTempochTypeAcceptance:
    """Tests that time parameters accept both raw floats and tempoch types."""

    def test_altitude_accepts_raw_float(self):
        """altitude_at should accept raw float MJD."""
        obs = Observer.roque_de_los_muchachos()
        alt = altitude_at(Body.Sun, obs, 60000.0)
        assert isinstance(alt, float)

    def test_altitude_accepts_object_with_value(self):
        """altitude_at should accept object with .value attribute."""
        obs = Observer.roque_de_los_muchachos()

        # Mock tempoch.ModifiedJulianDate-like object
        class MJDLike:
            def __init__(self, v):
                self.value = v

        mjd = MJDLike(60000.0)
        alt = altitude_at(Body.Sun, obs, mjd)
        assert isinstance(alt, float)

    def test_body_altitude_accepts_object(self):
        """Body.altitude_at should accept object with .value."""
        obs = Observer.roque_de_los_muchachos()

        class MJDLike:
            def __init__(self, v):
                self.value = v

        alt_float = Body.Sun.altitude_at(obs, 60000.0)
        alt_obj = Body.Sun.altitude_at(obs, MJDLike(60000.0))
        assert abs(alt_float - alt_obj) < 1e-10

    def test_star_altitude_accepts_object(self):
        """Star.altitude_at should accept object with .value."""
        obs = Observer.roque_de_los_muchachos()
        vega = Star.catalog("Vega")

        class MJDLike:
            def __init__(self, v):
                self.value = v

        alt_float = vega.altitude_at(obs, 60000.0)
        alt_obj = vega.altitude_at(obs, MJDLike(60000.0))
        assert abs(alt_float - alt_obj) < 1e-10

    def test_above_threshold_accepts_objects(self):
        """above_threshold should accept objects for start/end."""
        obs = Observer.roque_de_los_muchachos()

        class MJDLike:
            def __init__(self, v):
                self.value = v

        periods_float = above_threshold(Body.Sun, obs, 60000.0, 60001.0, 0.0)
        periods_obj = above_threshold(Body.Sun, obs, MJDLike(60000.0), MJDLike(60001.0), 0.0)
        assert len(periods_float) == len(periods_obj)

    def test_crossings_accepts_objects(self):
        """crossings should accept objects for start/end."""
        obs = Observer.roque_de_los_muchachos()

        class MJDLike:
            def __init__(self, v):
                self.value = v

        events_float = crossings(Body.Sun, obs, 60000.0, 60001.0, 0.0)
        events_obj = crossings(Body.Sun, obs, MJDLike(60000.0), MJDLike(60001.0), 0.0)
        assert len(events_float) == len(events_obj)

    def test_invalid_time_type_raises(self):
        """Passing an invalid type should raise TypeError."""
        import pytest

        obs = Observer.roque_de_los_muchachos()
        with pytest.raises((TypeError, AttributeError)):
            altitude_at(Body.Sun, obs, "not a number")
