"""Public frame, value, serialization and consumer contracts."""

import importlib
import json
import pickle
import subprocess
import sys
import types

import pytest

import siderust
from siderust import Observer, ProperMotion, Star, Target, apply_proper_motion, direction
from siderust.direction import ICRS, Horizontal


def test_real_module_and_public_exports():
    assert isinstance(direction, types.ModuleType)
    assert importlib.import_module("siderust.direction") is direction
    assert set(direction.__all__) == {"ICRS", "Horizontal"}
    assert direction.ICRS is ICRS
    assert direction.Horizontal is Horizontal
    for name in ("Direction", "HorizontalDirection", "ICRS", "Horizontal"):
        assert not hasattr(siderust, name)
    # Exercise the submodule-first import in a fresh interpreter.
    subprocess.run(
        [
            sys.executable,
            "-c",
            "from siderust.direction import ICRS; "
            "from siderust import direction; assert ICRS is direction.ICRS",
        ],
        check=True,
    )


@pytest.mark.parametrize("cls", [ICRS, Horizontal])
def test_identity_serialization_and_immutability(cls):
    obj = cls(12.5, 22.25)
    assert cls.__module__ == "siderust.direction"
    assert cls.__qualname__ == cls.__name__
    assert repr(obj).startswith(f"siderust.direction.{cls.__name__}(")
    restored = cls.from_dict(json.loads(json.dumps(obj.to_dict())))
    assert type(restored) is cls
    assert restored == obj
    for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
        restored = pickle.loads(pickle.dumps(obj, protocol=protocol))
        assert type(restored) is cls
        assert restored == obj
        assert hash(restored) == hash(obj)
    with pytest.raises(AttributeError):
        setattr(obj, next(iter(obj.to_dict())), 30)


@pytest.mark.parametrize("cls", [ICRS, Horizontal])
def test_equality_and_hash_invariants(cls):
    positive, negative = cls(0.0, 0.0), cls(-0.0, -0.0)
    assert positive == negative
    assert hash(positive) == hash(negative)
    assert len({positive, negative}) == 1
    a, nearby, same = cls(10, 20), cls(10 + 1e-11, 20), cls(10, 20)
    assert a != nearby
    assert a == same
    assert {a: "value"}[same] == "value"
    assert a != object()
    assert a != (10, 20)
    assert ICRS(10, 20) != Horizontal(10, 20)


@pytest.mark.parametrize(
    ("obj", "expected"),
    [
        (ICRS(-10, 0), (350, 0)),
        (ICRS(10, 100), (190, 80)),
        (ICRS(360, -90), (0, -90)),
        (Horizontal(100, 10), (80, 190)),
        (Horizontal(-100, 10), (-80, 190)),
        (Horizontal(0, -10), (0, 350)),
        (Horizontal(90, 360), (90, 0)),
    ],
)
def test_core_normalization(obj, expected):
    assert tuple(obj.to_dict().values()) == pytest.approx(expected)


@pytest.mark.parametrize("cls", [ICRS, Horizontal])
@pytest.mark.parametrize("bad", ["10", None, object(), complex(1, 2)])
def test_incorrect_coordinate_types(cls, bad):
    for args in ((bad, 0), (0, bad)):
        with pytest.raises(TypeError):
            cls(*args)
        fields = dict(zip(cls(0, 0).to_dict(), args))
        with pytest.raises(TypeError):
            cls.from_dict(fields)


@pytest.mark.parametrize("cls", [ICRS, Horizontal])
def test_dictionary_validation(cls):
    with pytest.raises(ValueError, match="missing"):
        cls.from_dict({})
    for field in cls(0, 0).to_dict():
        fields = cls(0, 0).to_dict()
        fields[field] = float("nan")
        with pytest.raises(ValueError, match="finite"):
            cls.from_dict(fields)


def test_icrs_core_math_and_frame_rejection():
    x, y, opposite = ICRS(0, 0), ICRS(90, 0), ICRS(180, 0)
    assert x.to_cartesian() == pytest.approx((1, 0, 0))
    assert y.to_cartesian() == pytest.approx((0, 1, 0), abs=1e-15)
    assert ICRS(0, 90).to_cartesian() == pytest.approx((0, 0, 1), abs=1e-15)
    assert x.dot(y) == pytest.approx(0, abs=1e-15)
    assert x.dot(opposite) == pytest.approx(-1)
    assert x.angular_separation(y) == pytest.approx(90)
    assert x.angular_separation(opposite) == pytest.approx(180)
    assert x.angular_separation(x) == 0
    assert x.angular_separation(ICRS(1e-8, 0)) == pytest.approx(1e-8)
    for method in (x.dot, x.angular_separation):
        with pytest.raises(TypeError):
            method(Horizontal(0, 0))


def test_target_star_and_proper_motion():
    original = ICRS(120, 22.5)
    snapshot = original.track(2451545)
    assert snapshot.is_direction and not snapshot.is_position
    assert type(snapshot.direction) is ICRS
    assert snapshot.direction == original
    assert snapshot.time == 2451545
    assert "direction.ICRS" in repr(snapshot)
    target = Target(original, 2451545)
    target.update(ICRS(20, 30), 2451546)
    assert target.direction == ICRS(20, 30)
    assert target.time == 2451546
    with pytest.raises(ValueError):
        _ = target.position
    assert type(Star.catalog("Vega").track(2451545).direction) is ICRS
    pm = ProperMotion(3600000, 0, mu_alpha_star=False)
    propagated = apply_proper_motion(original, pm, 2451545 + 365.25)
    assert type(propagated) is ICRS
    assert propagated.ra_deg == pytest.approx(121)
    assert propagated.dec_deg == pytest.approx(22.5)
    assert apply_proper_motion(original, pm, 2451545) == original
    with pytest.raises(ValueError, match="pole"):
        apply_proper_motion(ICRS(0, 90), ProperMotion(1, 1), 2451545 + 365.25)


def test_horizontal_has_no_implicit_observation_context():
    h = Horizontal(45, 180)
    for attr in ("observer", "epoch", "track", "altitude_at", "azimuth_at"):
        assert not hasattr(h, attr)
    for function, args in (
        (Target, (h, 2451545)),
        (Target(ICRS(0, 0), 2451545).update, (h, 2451546)),
        (apply_proper_motion, (h, ProperMotion(0, 0), 2451545)),
        (siderust.altitude_at, (h, Observer(0, 0), 60000)),
        (siderust.azimuth_at, (h, Observer(0, 0), 60000)),
    ):
        with pytest.raises((TypeError, ValueError)):
            function(*args)


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), float("-inf")])
def test_proper_motion_cannot_create_nonfinite_directions(bad):
    with pytest.raises(ValueError, match="finite"):
        ProperMotion(bad, 0)
    with pytest.raises(ValueError, match="finite"):
        ProperMotion(0, bad)
    with pytest.raises(ValueError, match="jd must be finite"):
        apply_proper_motion(ICRS(0, 0), ProperMotion(0, 0), bad)


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), float("-inf")])
def test_custom_star_cannot_produce_nonfinite_icrs(bad):
    for ra, dec in ((bad, 0), (0, bad)):
        with pytest.raises(ValueError, match="finite"):
            Star.from_ra_dec("invalid", ra, dec)
        with pytest.raises(ValueError, match="finite"):
            Star.from_dict({"name": "invalid", "ra_deg": ra, "dec_deg": dec})
