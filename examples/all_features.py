"""
siderust all_features — comprehensive feature tour.

Demonstrates every public API: observers, bodies, stars, directions,
altitude/azimuth queries, threshold periods, crossings, culminations,
moon phases, and multi-observatory workflows.

Run:
    python examples/all_features.py
"""

import pickle

from siderust import (
    Body,
    CrossingDirection,
    CulminationKind,
    Direction,
    Observer,
    Star,
    above_threshold,
    altitude_at,
    azimuth_at,
    below_threshold,
    crossings,
    culminations,
    find_moon_phases,
    moon_phase,
)


def section(title: str):
    print(f"\n{'─' * 60}")
    print(f"  {title}")
    print(f"{'─' * 60}")


def main():
    print("=" * 60)
    print("  siderust — all features demo")
    print("=" * 60)

    # ══════════════════════════════════════════════════════════════
    section("1. Observer / Observatory")
    # ══════════════════════════════════════════════════════════════

    # Custom observer
    custom = Observer(lon_deg=-3.7038, lat_deg=40.4168, height_m=650.0)
    print(f"Custom (Madrid): {custom}")

    # Predefined observatories
    roque = Observer.roque_de_los_muchachos()
    paranal = Observer.el_paranal()
    mk = Observer.mauna_kea()
    ls = Observer.la_silla()
    print(f"Roque:    {roque}")
    print(f"Paranal:  {paranal}")
    print(f"Mauna Kea: {mk}")
    print(f"La Silla: {ls}")

    # Properties
    print(f"  Roque lon={roque.lon_deg:.4f}°, lat={roque.lat_deg:.4f}°, h={roque.height_m:.0f}m")

    # Equality & hash
    assert roque == Observer.roque_de_los_muchachos()
    assert hash(roque) == hash(Observer.roque_de_los_muchachos())
    print("  Equality ✓  Hash ✓")

    # ══════════════════════════════════════════════════════════════
    section("2. Body Enum")
    # ══════════════════════════════════════════════════════════════

    all_bodies = [
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
    print(f"Available bodies ({len(all_bodies)}):")
    for b in all_bodies:
        print(f"  {b!r:25s}  str={b!s}")

    # Equality & hash
    assert Body.Sun == Body.Sun
    assert Body.Sun != Body.Moon
    assert len({Body.Sun, Body.Moon, Body.Sun}) == 2
    print("  Equality ✓  Hash ✓")

    # ══════════════════════════════════════════════════════════════
    section("3. Star Catalog")
    # ══════════════════════════════════════════════════════════════

    catalog_names = [
        "Sirius",
        "Vega",
        "Polaris",
        "Canopus",
        "Arcturus",
        "Rigel",
        "Betelgeuse",
        "Procyon",
        "Aldebaran",
        "Altair",
    ]
    for name in catalog_names:
        s = Star.catalog(name)
        print(f"  {s}")

    # Custom star
    custom_star = Star.from_ra_dec("HD 99999", 150.0, -20.0)
    print(f"  Custom: {custom_star}")

    # Error handling
    try:
        Star.catalog("Nonexistent")
        raise AssertionError("Expected Star.catalog to reject an unknown star")
    except ValueError as e:
        print(f"  Expected error: {e}")

    # ══════════════════════════════════════════════════════════════
    section("4. Direction (ICRS)")
    # ══════════════════════════════════════════════════════════════

    ncp = Direction(ra_deg=0.0, dec_deg=90.0)
    scp = Direction(ra_deg=0.0, dec_deg=-90.0)
    print(f"North Celestial Pole: {ncp}")
    print(f"South Celestial Pole: {scp}")

    # Pickle
    d = Direction(ra_deg=123.456, dec_deg=-45.678)
    d2 = pickle.loads(pickle.dumps(d))
    assert abs(d.ra_deg - d2.ra_deg) < 1e-10
    print("  Pickle round-trip ✓")

    # ══════════════════════════════════════════════════════════════
    section("5. Altitude & Azimuth — Instant Queries")
    # ══════════════════════════════════════════════════════════════

    obs = roque
    mjd = 60000.5  # noon-ish

    print(f"At MJD {mjd} from Roque de los Muchachos:")
    for body in [Body.Sun, Body.Moon, Body.Mars]:
        alt = altitude_at(body, obs, mjd)
        az = azimuth_at(body, obs, mjd)
        print(f"  {body!s:10s}  alt={alt:+7.2f}°  az={az:6.2f}°")

    vega = Star.catalog("Vega")
    alt = altitude_at(vega, obs, mjd)
    az = azimuth_at(vega, obs, mjd)
    print(f"  {'Vega':10s}  alt={alt:+7.2f}°  az={az:6.2f}°")

    gc = Direction(ra_deg=266.4, dec_deg=-29.0)
    alt = altitude_at(gc, obs, mjd)
    az = azimuth_at(gc, obs, mjd)
    print(f"  {'Gal.center':10s}  alt={alt:+7.2f}°  az={az:6.2f}°")

    # ══════════════════════════════════════════════════════════════
    section("6. Threshold Periods")
    # ══════════════════════════════════════════════════════════════

    start = 60000.0
    end = 60001.0

    day_periods = above_threshold(Body.Sun, obs, start, end, 0.0)
    night_periods = below_threshold(Body.Sun, obs, start, end, 0.0)
    print(f"Sun above horizon: {len(day_periods)} period(s)")
    for s, e in day_periods:
        print(f"  MJD {s:.6f} – {e:.6f}  ({(e - s) * 24:.1f}h)")
    print(f"Sun below horizon: {len(night_periods)} period(s)")
    for s, e in night_periods:
        print(f"  MJD {s:.6f} – {e:.6f}  ({(e - s) * 24:.1f}h)")

    # ══════════════════════════════════════════════════════════════
    section("7. Crossings (Sunrise/Sunset)")
    # ══════════════════════════════════════════════════════════════

    events = crossings(Body.Sun, obs, start, end, 0.0)
    for e in events:
        label = "Sunrise" if e.direction == CrossingDirection.Rising else "Sunset"
        print(f"  {label:8s} at MJD {e.mjd:.6f}")

    # Also for the Moon
    moon_events = crossings(Body.Moon, obs, start, end, 0.0)
    for e in moon_events:
        label = "Moonrise" if e.direction == CrossingDirection.Rising else "Moonset"
        print(f"  {label:9s} at MJD {e.mjd:.6f}")

    # ══════════════════════════════════════════════════════════════
    section("8. Culminations")
    # ══════════════════════════════════════════════════════════════

    culms = culminations(Body.Sun, obs, start, end)
    for c in culms:
        label = "Transit" if c.kind == CulminationKind.Max else "Nadir"
        print(f"  {label:8s} at MJD {c.mjd:.6f}, alt={c.altitude_deg:+.2f}°")

    # ══════════════════════════════════════════════════════════════
    section("9. Moon Phase")
    # ══════════════════════════════════════════════════════════════

    # Geocentric
    phase = moon_phase(2451545.0)  # J2000.0
    print(f"J2000.0 geocentric: {phase}")
    print(f"  Phase angle: {phase.phase_angle_deg:.1f}°")
    print(f"  Illumination: {phase.illuminated_fraction * 100:.1f}%")
    print(f"  Waxing: {phase.waxing}")

    # Topocentric
    phase_topo = moon_phase(2451545.0, observer=obs)
    print(f"J2000.0 topocentric: {phase_topo}")

    # Phase events
    events = find_moon_phases(60000.0, 60030.0)
    print(f"Phase events in 30-day window: {len(events)}")
    for e in events:
        print(f"  MJD {e.mjd:.4f}: {e.kind}")

    # ══════════════════════════════════════════════════════════════
    section("10. Multi-Observatory Workflow")
    # ══════════════════════════════════════════════════════════════

    print("Sun altitude at different sites (MJD 60000.5):")
    for name, site in [
        ("Roque", roque),
        ("Paranal", paranal),
        ("Mauna Kea", mk),
        ("La Silla", ls),
    ]:
        alt = altitude_at(Body.Sun, site, 60000.5)
        print(f"  {name:12s}: {alt:+7.2f}°")

    # ══════════════════════════════════════════════════════════════
    section("11. Error Handling")
    # ══════════════════════════════════════════════════════════════

    # Invalid star
    try:
        Star.catalog("Nonexistent")
    except ValueError as e:
        print(f"  Star error: {e}")

    # Invalid window
    try:
        above_threshold(Body.Sun, obs, 60001.0, 60000.0, 0.0)
    except ValueError as e:
        print(f"  Window error: {e}")

    # Invalid target type
    try:
        altitude_at("not_a_target", obs, 60000.0)
    except TypeError as e:
        print(f"  Type error: {e}")

    print("\n" + "=" * 60)
    print("  All features demonstrated successfully!")
    print("=" * 60)


if __name__ == "__main__":
    main()
