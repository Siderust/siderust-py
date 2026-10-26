"""Contract tests against a separately compiled PyO3 consumer extension."""

from datetime import datetime, timedelta, timezone

import _siderust_interop_consumer as consumer
import pytest
import siderust._siderust as canonical_extension

import siderust


def test_both_bridges_share_siderust_time_and_period_types():
    offset = timezone(timedelta(hours=5))
    start = datetime(2024, 6, 21, 17, 34, 56, 123456, tzinfo=offset)
    end = start + timedelta(hours=1)
    actual_start, actual_end = consumer.period_round_trip(start, end)
    for actual, expected in ((actual_start, start), (actual_end, end)):
        assert actual.tzinfo is timezone.utc
        assert abs(actual - expected) < timedelta(microseconds=50)


def test_combined_bridge_rejects_naive_datetimes():
    naive = datetime(2024, 6, 21, 12)
    aware = naive.replace(tzinfo=timezone.utc)
    with pytest.raises(ValueError, match="naive"):
        consumer.period_round_trip(naive, aware)


def test_bridge_protocol_matches_canonical_extension():
    assert canonical_extension._bridge_protocol_version == 2
    assert consumer.bridge_protocol_version() == 2


def test_bridge_rejects_incompatible_protocol(monkeypatch):
    monkeypatch.setattr(canonical_extension, "_bridge_protocol_version", 999)

    with pytest.raises(
        ImportError,
        match=r"incompatible siderust bridge protocol: expected 2, found 999",
    ):
        consumer.observer_parts(siderust.Observer(0.0, 0.0))


def test_bridge_rejects_missing_protocol(monkeypatch):
    monkeypatch.delattr(canonical_extension, "_bridge_protocol_version")

    with pytest.raises(
        ImportError,
        match=r"does not expose a valid _bridge_protocol_version; expected bridge protocol 2",
    ):
        consumer.icrs_parts(siderust.direction.ICRS(0.0, 0.0))


def test_observer_cross_extension_round_trip():
    observer = siderust.Observer(-17.8925, 28.7543, 2396.0)

    assert consumer.observer_parts(observer) == pytest.approx((-17.8925, 28.7543, 2396.0))
    result = consumer.observer_round_trip(observer)

    assert type(result) is siderust.Observer
    assert (result.lon_deg, result.lat_deg, result.height_m) == pytest.approx(
        (-17.8925, 28.7543, 2396.0)
    )


def test_direction_cross_extension_round_trip():
    direction = siderust.direction.ICRS(83.633, 22.014)

    assert consumer.icrs_parts(direction) == pytest.approx((83.633, 22.014))
    result = consumer.icrs_round_trip(direction)

    assert type(result) is siderust.direction.ICRS
    assert (result.ra_deg, result.dec_deg) == pytest.approx((83.633, 22.014))


def test_horizontal_cross_extension_round_trip():
    direction = siderust.direction.Horizontal(45.0, 180.0)

    assert consumer.horizontal_parts(direction) == pytest.approx((45.0, 180.0))
    result = consumer.horizontal_round_trip(direction)

    assert type(result) is siderust.direction.Horizontal
    assert (result.alt_deg, result.az_deg) == pytest.approx((45.0, 180.0))


@pytest.mark.parametrize(
    ("function", "wrong_value"),
    [
        (consumer.observer_parts, object()),
        (consumer.observer_round_trip, siderust.direction.ICRS(10.0, 20.0)),
        (consumer.icrs_parts, object()),
        (consumer.icrs_round_trip, siderust.Observer(10.0, 20.0)),
        (consumer.horizontal_parts, object()),
        (consumer.horizontal_round_trip, siderust.direction.ICRS(10.0, 20.0)),
        (consumer.horizontal_round_trip, siderust.Observer(10.0, 20.0)),
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
        consumer.observer_from_parts(parts["lon_deg"], parts["lat_deg"], parts["height_m"])


@pytest.mark.parametrize("field", ["ra_deg", "dec_deg"])
@pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
def test_bridge_rejects_non_finite_direction_parts(field, value):
    parts = {"ra_deg": 0.0, "dec_deg": 0.0}
    parts[field] = value

    with pytest.raises(ValueError, match=rf"{field} must be finite"):
        consumer.icrs_from_parts(parts["ra_deg"], parts["dec_deg"])


@pytest.mark.parametrize("field", ["alt_deg", "az_deg"])
@pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
def test_bridge_rejects_non_finite_horizontal_parts(field, value):
    parts = {"alt_deg": 0.0, "az_deg": 0.0}
    parts[field] = value

    with pytest.raises(ValueError, match=rf"{field} must be finite"):
        consumer.horizontal_from_parts(parts["alt_deg"], parts["az_deg"])


@pytest.mark.parametrize("protocol", [None, "2", -1, 2.5, object()])
def test_bridge_rejects_malformed_protocol(monkeypatch, protocol):
    monkeypatch.setattr(canonical_extension, "_bridge_protocol_version", protocol)
    with pytest.raises(ImportError, match="valid _bridge_protocol_version"):
        consumer.bridge_protocol_version()


def test_provisional_hooks_are_removed():
    for name in (
        "_bridge_direction_from_parts",
        "_bridge_direction_to_parts",
        "_bridge_horizontal_direction_from_parts",
        "_bridge_horizontal_direction_to_parts",
    ):
        assert not hasattr(canonical_extension, name)


@pytest.mark.parametrize(
    ("cls", "round_trip", "from_parts"),
    [
        (siderust.direction.ICRS, consumer.icrs_round_trip, consumer.icrs_from_parts),
        (
            siderust.direction.Horizontal,
            consumer.horizontal_round_trip,
            consumer.horizontal_from_parts,
        ),
    ],
)
def test_canonical_value_contract_across_extensions(cls, round_trip, from_parts):
    import pickle

    obj = cls(12.5, 22.25)
    for result in (round_trip(obj), from_parts(12.5, 22.25)):
        assert isinstance(result, cls)
        assert type(result) is cls
        assert result == obj
        assert hash(result) == hash(obj)
        assert {obj: "canonical"}[result] == "canonical"
        assert type(pickle.loads(pickle.dumps(result))) is cls
        assert result.__class__.__module__ == "siderust.direction"
        assert result.__class__.__qualname__ == cls.__name__


def test_bridge_rejects_both_incompatible_frames_and_duck_types():
    class FakeDirection:
        ra_deg = dec_deg = alt_deg = az_deg = 0.0

    for value in (siderust.direction.Horizontal(0, 0), FakeDirection()):
        with pytest.raises(TypeError):
            consumer.icrs_round_trip(value)
    for value in (siderust.direction.ICRS(0, 0), FakeDirection()):
        with pytest.raises(TypeError):
            consumer.horizontal_round_trip(value)


@pytest.mark.parametrize("function", [consumer.icrs_from_parts, consumer.horizontal_from_parts])
@pytest.mark.parametrize("bad", [None, "10", object()])
def test_bridge_rejects_incorrect_part_types(function, bad):
    with pytest.raises(TypeError):
        function(bad, 0)
    with pytest.raises(TypeError):
        function(0, bad)


@pytest.mark.parametrize("version", [0, 1, 3, 999])
def test_protocol_mismatch_blocks_direction_construction(monkeypatch, version):
    monkeypatch.setattr(canonical_extension, "_bridge_protocol_version", version)
    for function in (consumer.icrs_from_parts, consumer.horizontal_from_parts):
        with pytest.raises(ImportError, match="incompatible siderust bridge protocol"):
            function(0, 0)
