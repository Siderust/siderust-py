"""
Moon Phase Properties Example

Mirrors: siderust/examples/07_moon_properties.rs

Shows:
1) Moon phase properties at a given instant.
2) Finding principal phase events (New Moon, First Quarter, etc.).

Run with: python examples/07_moon_properties.py
"""

from siderust import Observer, find_moon_phases, moon_phase

# MJD epoch offset
_JD_TO_MJD = 2_400_000.5


def jd_to_mjd(jd: float) -> float:
    return jd - _JD_TO_MJD


def mjd_to_iso(mjd: float) -> str:
    """Approximate MJD (TT) to a human-readable UTC string."""
    import datetime as dt

    base = dt.datetime(1858, 11, 17, tzinfo=dt.timezone.utc)
    utc = base + dt.timedelta(days=mjd)
    return utc.strftime("%Y-%m-%d %H:%M UTC")


def main():
    # Default: Roque de los Muchachos, starting today-ish
    lat = 28.762
    lon = -17.892
    h_m = 2396.0

    observer = Observer(lon, lat, h_m)

    # Starting from 2023-10-01 00:00 UTC (JD ≈ 2460219.5)
    jd_start = 2_460_219.5
    mjd_start = jd_to_mjd(jd_start)
    mjd_end = mjd_start + 35.0

    # 1) Point-in-time phase properties
    phase = moon_phase(mjd_start)

    print("Moon phase at MJD {:.1f}".format(mjd_start))
    print("==================================")
    print(f"Site: lat={lat:.4f}°, lon={lon:.4f}°, h={h_m:.0f} m")

    print(f"\n  label                 : {phase.label}")
    print(f"  illuminated fraction  : {phase.illuminated_fraction:.4f}")
    print(f"  illuminated percent   : {phase.illuminated_fraction * 100:.2f} %")
    print(f"  phase angle           : {phase.phase_angle_deg:.4f}°")
    print(f"  elongation            : {phase.elongation_deg:.4f}°")
    print(f"  waxing                : {phase.waxing}")

    # 2) Find principal phase events over 35 days
    events = find_moon_phases(mjd_start, mjd_end)
    print(f"\nPrincipal phase events in next 35 days: {len(events)}")
    for ev in events:
        print(f"  - {str(ev.kind):>13s} at {mjd_to_iso(ev.mjd)}")


if __name__ == "__main__":
    main()
