"""Contract tests against a separately compiled PyO3 consumer extension."""

import _siderust_interop_consumer as consumer
import pytest
import siderust._siderust as canonical_extension

import siderust


def test_bridge_protocol_matches_canonical_extension():
    assert canonical_extension._bridge_protocol_version == 1
    assert consumer.bridge_protocol_version() == 1


def test_bridge_rejects_incompatible_protocol(monkeypatch):
    monkeypatch.setattr(canonical_extension, "_bridge_protocol_version", 999)

    with pytest.raises(
        ImportError,
        match=r"incompatible siderust bridge protocol: expected 1, found 999",
    ):
        consumer.observer_parts(siderust.Observer(0.0, 0.0))


def test_bridge_rejects_missing_protocol(monkeypatch):
    monkeypatch.delattr(canonical_extension, "_bridge_protocol_version")

    with pytest.raises(
        ImportError,
        match=r"does not expose a valid _bridge_protocol_version; expected bridge protocol 1",
    ):
        consumer.direction_parts(siderust.Direction(0.0, 0.0))


def test_observer_cross_extension_round_trip():
    observer = siderust.Observer(-17.8925, 28.7543, 2396.0)

    assert consumer.observer_parts(observer) == pytest.approx((-17.8925, 28.7543, 2396.0))
    result = consumer.observer_round_trip(observer)

    assert type(result) is siderust.Observer
    assert (result.lon_deg, result.lat_deg, result.height_m) == pytest.approx(
        (-17.8925, 28.7543, 2396.0)
    )


def test_direction_cross_extension_round_trip():
    direction = siderust.Direction(83.633, 22.014)

    assert consumer.direction_parts(direction) == pytest.approx((83.633, 22.014))
    result = consumer.direction_round_trip(direction)

    assert type(result) is siderust.Direction
    assert (result.ra_deg, result.dec_deg) == pytest.approx((83.633, 22.014))


@pytest.mark.parametrize(
    ("function", "wrong_value"),
    [
        (consumer.observer_parts, object()),
        (consumer.observer_round_trip, siderust.Direction(10.0, 20.0)),
        (consumer.direction_parts, object()),
        (consumer.direction_round_trip, siderust.Observer(10.0, 20.0)),
    ],
)
def test_bridge_rejects_noncanonical_types(function, wrong_value):
    with pytest.raises(TypeError):
        function(wrong_value)


@pytest.mark.parametrize("field", ["lon_deg", "lat_deg", "height_m"])
@pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
def test_bridge_rejects_non_finite_observer_parts(field, value):
    parts = {"lon_deg": 0.0, "lat_deg": 0.0, "height_m": 0.0}
    parts[field] = value

    with pytest.raises(ValueError, match=rf"{field} must be finite"):
        consumer.observer_from_parts(
            parts["lon_deg"], parts["lat_deg"], parts["height_m"]
        )


@pytest.mark.parametrize("field", ["ra_deg", "dec_deg"])
@pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
def test_bridge_rejects_non_finite_direction_parts(field, value):
    parts = {"ra_deg": 0.0, "dec_deg": 0.0}
    parts[field] = value

    with pytest.raises(ValueError, match=rf"{field} must be finite"):
        consumer.direction_from_parts(parts["ra_deg"], parts["dec_deg"])
