"""Contract tests against a separately compiled PyO3 consumer extension."""

import _siderust_interop_consumer as consumer
import pytest

import siderust


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
