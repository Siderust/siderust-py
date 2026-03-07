"""
Night Events Example

Mirrors: siderust/examples/06_night_events.rs

Shows crossing events and below-threshold periods for the Sun using
civil/nautical/astronomical/horizon thresholds.

Run with: python examples/06_night_events.py
"""

from siderust import (
    TWILIGHT_ASTRONOMICAL,
    TWILIGHT_CIVIL,
    TWILIGHT_HORIZON,
    TWILIGHT_NAUTICAL,
    Body,
    CrossingDirection,
    Observer,
    below_threshold,
    crossings,
)

# MJD epoch: 2400000.5 JD
_JD_TO_MJD = 2_400_000.5


def jd_to_mjd(jd: float) -> float:
    return jd - _JD_TO_MJD


def mjd_to_iso(mjd: float) -> str:
    """Approximate MJD (TT) to a human-readable UTC string."""
    # MJD 0 = 1858-11-17 00:00:00 UTC
    import datetime as dt

    base = dt.datetime(1858, 11, 17, tzinfo=dt.timezone.utc)
    utc = base + dt.timedelta(days=mjd)
    return utc.strftime("%Y-%m-%dT%H:%M:%S")


def print_events_for_type(
    observer: Observer,
    start_mjd: float,
    end_mjd: float,
    name: str,
    threshold: float,
) -> None:
    events = crossings(Body.Sun, observer, start_mjd, end_mjd, threshold)
    downs = sum(1 for e in events if e.direction == CrossingDirection.Setting)
    rises = sum(1 for e in events if e.direction == CrossingDirection.Rising)

    print(f"{name:18s} threshold {threshold:>8.1f}° -> {len(events):2d} crossing(s)")

    for ev in events:
        if ev.direction == CrossingDirection.Setting:
            label = "night-type down (Sun setting below threshold)"
        else:
            label = "night-type raise (Sun rising above threshold)"
        print(f"  - {label} at {mjd_to_iso(ev.mjd)}")

    print(f"  summary: down={downs} raise={rises}")


def print_periods_for_type(
    observer: Observer,
    start_mjd: float,
    end_mjd: float,
    name: str,
    threshold: float,
) -> None:
    periods = below_threshold(Body.Sun, observer, start_mjd, end_mjd, threshold)
    print(f"{name:18s} night periods (Sun < {threshold}°): {len(periods)}")

    for start, end in periods:
        hours = (end - start) * 24.0
        print(f"  - {mjd_to_iso(start)} -> {mjd_to_iso(end)} ({hours:.1f} h)")


def main():
    # Default: Greenwich, one week from 2023-10-01
    lat_deg = 51.4769
    lon_deg = 0.0
    height_m = 0.0

    observer = Observer(lon_deg, lat_deg, height_m)

    # One week starting 2023-10-01 00:00 UTC
    # JD of 2023-10-01 00:00 UTC ≈ 2460219.5
    jd_start = 2_460_219.5
    start_mjd = jd_to_mjd(jd_start)
    end_mjd = start_mjd + 7.0

    night_types = [
        ("Horizon", TWILIGHT_HORIZON),
        ("Civil", TWILIGHT_CIVIL),
        ("Nautical", TWILIGHT_NAUTICAL),
        ("Astronomical", TWILIGHT_ASTRONOMICAL),
    ]

    print("Night events over one week")
    print("==========================")
    print(f"Site: lat={lat_deg}° lon={lon_deg}° height={height_m} m")
    print(f"Window: MJD {start_mjd:.1f} -> {end_mjd:.1f}\n")

    print("1) Night-type crossing events")
    for name, thr in night_types:
        print_events_for_type(observer, start_mjd, end_mjd, name, thr)

    print("\n2) Night periods per night type")
    for name, thr in night_types:
        print_periods_for_type(observer, start_mjd, end_mjd, name, thr)


if __name__ == "__main__":
    main()
