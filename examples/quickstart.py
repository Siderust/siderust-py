"""
siderust quickstart — observation planning in 5 minutes.

Run:
    python examples/quickstart.py
"""

from siderust import (
    Observer,
    Body,
    Star,
    Direction,
    altitude_at,
    above_threshold,
    below_threshold,
    crossings,
    azimuth_at,
    moon_phase,
    CrossingDirection,
)


def main():
    print("=" * 60)
    print("siderust quickstart")
    print("=" * 60)

    # ── 1. Create an observer ────────────────────────────────────
    obs = Observer.roque_de_los_muchachos()
    print(f"\n1. Observer: {obs}")

    # ── 2. Sun altitude at a specific time ───────────────────────
    mjd = 60000.0  # 2023-02-25
    sun_alt = Body.Sun.altitude_at(obs, mjd)
    sun_az = Body.Sun.azimuth_at(obs, mjd)
    print(f"\n2. Sun at MJD {mjd}:")
    print(f"   Altitude: {sun_alt:.4f}°")
    print(f"   Azimuth:  {sun_az:.4f}°")

    # ── 3. Find sunrise and sunset ───────────────────────────────
    events = crossings(Body.Sun, obs, mjd, mjd + 1.0, 0.0)
    print(f"\n3. Sun crossings (horizon) over 1 day:")
    for e in events:
        label = "Rise" if e.direction == CrossingDirection.Rising else "Set"
        print(f"   {label}: MJD {e.mjd:.6f}")

    # ── 4. Night-time periods ────────────────────────────────────
    night = below_threshold(Body.Sun, obs, mjd, mjd + 1.0, 0.0)
    print(f"\n4. Night-time (Sun below horizon):")
    for start, end in night:
        hours = (end - start) * 24.0
        print(f"   MJD {start:.4f} – {end:.4f} ({hours:.1f} hours)")

    # ── 5. Star visibility ───────────────────────────────────────
    vega = Star.catalog("Vega")
    print(f"\n5. Star: {vega}")

    if night:
        mid = (night[0][0] + night[0][1]) / 2.0
        vega_alt = vega.altitude_at(obs, mid)
        vega_az = vega.azimuth_at(obs, mid)
        print(f"   At mid-night (MJD {mid:.4f}):")
        print(f"   Altitude: {vega_alt:.4f}°")
        print(f"   Azimuth:  {vega_az:.4f}°")

    # ── 6. Moon phase ────────────────────────────────────────────
    jd = 2460000.0 + mjd - 59999.5  # approximate JD
    phase = moon_phase(2460000.5)
    print(f"\n6. Moon phase: {phase}")

    # ── 7. Planet altitudes ──────────────────────────────────────
    print(f"\n7. Planet altitudes at MJD {mjd}:")
    for body in [Body.Mars, Body.Jupiter, Body.Saturn]:
        alt = altitude_at(body, obs, mjd)
        az = azimuth_at(body, obs, mjd)
        print(f"   {body}: alt={alt:.2f}°, az={az:.2f}°")

    # ── 8. Custom ICRS direction ─────────────────────────────────
    # Galactic center (approximate)
    gc = Direction(ra_deg=266.4, dec_deg=-29.0)
    gc_alt = gc.altitude_at(obs, mjd)
    print(f"\n8. Galactic center: {gc}")
    print(f"   Altitude at MJD {mjd}: {gc_alt:.4f}°")

    # ── 9. Multiple observatories ────────────────────────────────
    print(f"\n9. Sun altitude at different sites (MJD {mjd}):")
    sites = [
        ("Roque", Observer.roque_de_los_muchachos()),
        ("Paranal", Observer.el_paranal()),
        ("Mauna Kea", Observer.mauna_kea()),
        ("La Silla", Observer.la_silla()),
    ]
    for name, site in sites:
        alt = Body.Sun.altitude_at(site, mjd)
        print(f"   {name:12s}: {alt:+.2f}°")

    print("\n" + "=" * 60)
    print("Done!")


if __name__ == "__main__":
    main()
