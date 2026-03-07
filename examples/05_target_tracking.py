"""
Target Tracking Example

Mirrors: siderust/examples/05_target_tracking.rs

Demonstrates:
- Trackable objects: tracking bodies, stars, and directions over time
- Target snapshots: coupling positions with epochs
- Keplerian orbit propagation via Orbit and Comet
- Proper motion application to stellar coordinates

Run with: python examples/05_target_tracking.py
"""

from siderust import (
    Body,
    Comet,
    Direction,
    Orbit,
    ProperMotion,
    Star,
    Target,
    apply_proper_motion,
)

J2000 = 2451545.0
JULIAN_YEAR = 365.25


def section_trackable_objects():
    """1) Track bodies, stars, and directions at specific epochs."""
    print("1) Trackable objects (ICRS direction, star, Sun, planet, Moon)")

    jd = J2000
    jd_next = jd + 1.0

    # Directions are time-invariant
    fixed_icrs = Direction(ra_deg=120.0, dec_deg=22.5)
    t1 = fixed_icrs.track(jd)
    t2 = fixed_icrs.track(jd_next)
    print(
        f"  ICRS direction is time-invariant: "
        f"RA {t1.direction.ra_deg:.3f} -> {t2.direction.ra_deg:.3f}, "
        f"Dec {t1.direction.dec_deg:.3f} -> {t2.direction.dec_deg:.3f}"
    )

    # Star tracking (returns ICRS direction)
    sirius = Star.catalog("Sirius")
    sirius_target = sirius.track(jd)
    print(
        f"  Sirius via track(): RA {sirius_target.direction.ra_deg:.3f}°, "
        f"Dec {sirius_target.direction.dec_deg:.3f}°"
    )

    # Body tracking (returns position)
    sun = Body.Sun.track(jd)
    mars = Body.Mars.track(jd)
    moon = Body.Moon.track(jd)

    print(f"  Sun barycentric distance: {sun.position.distance():.6f} AU")
    print(f"  Mars barycentric distance: {mars.position.distance():.6f} AU")
    print(f"  Moon geocentric distance: {moon.position.distance():.1f} km")
    print()


def section_target_snapshots():
    """2) Create and update Target snapshots."""
    print("2) Target snapshots for arbitrary sky objects")

    jd = J2000
    jd_next = jd + 1.0

    # Create a target from a tracked body
    mars_target = Body.Mars.track(jd)
    print(
        f"  Mars target at JD {mars_target.time:.1f}: r = {mars_target.position.distance():.6f} AU"
    )

    # Update with new position at next epoch
    mars_next = Body.Mars.track(jd_next)
    mars_target.update(mars_next.position, jd_next)
    print(
        f"  Mars target updated to JD {mars_target.time:.1f}: "
        f"r = {mars_target.position.distance():.6f} AU"
    )

    # Comet target (orbit propagated with Kepler)
    halley = Comet.halley()
    halley_pos = halley.kepler_position(jd)
    halley_target = Target(halley_pos, jd)
    print(
        f"  Halley target at JD {halley_target.time:.1f}: "
        f"r = {halley_target.position.distance():.6f} AU"
    )

    # Custom orbit for a satellite-like object
    demo_orbit = Orbit(
        semi_major_axis_au=1.0002,
        eccentricity=0.001,
        inclination_deg=0.1,
        lon_ascending_node_deg=35.0,
        arg_perihelion_deg=80.0,
        mean_anomaly_deg=10.0,
        epoch_jd=jd,
    )
    demo_pos = demo_orbit.kepler_position(jd)
    demo_target = Target(demo_pos, jd)
    print(
        f"  DemoSat target at JD {demo_target.time:.1f}: "
        f"r = {demo_target.position.distance():.6f} AU"
    )
    print()


def section_proper_motion():
    """3) Apply proper motion to stellar targets."""
    print("3) Target with proper motion (stellar-style target)")

    jd = J2000

    # Proper motion for a Betelgeuse-like star (µα⋆, µδ in mas/yr)
    pm = ProperMotion(pm_ra_mas_yr=27.54, pm_dec_mas_yr=10.86)

    # Get Betelgeuse's catalog direction as starting point
    betelgeuse = Star.catalog("Betelgeuse")
    original_dir = betelgeuse.track(jd).direction

    print(
        f"  Betelgeuse-like target at J2000: "
        f"RA {original_dir.ra_deg:.6f}°, Dec {original_dir.dec_deg:.6f}°"
    )

    # Apply proper motion 25 years into the future
    jd_future = jd + 25 * JULIAN_YEAR
    moved_dir = apply_proper_motion(original_dir, pm, jd_future)

    print(
        f"  After 25 years:                  "
        f"RA {moved_dir.ra_deg:.6f}°, Dec {moved_dir.dec_deg:.6f}°"
    )
    print()


def section_orbit_comets():
    """4) Orbit and Comet objects."""
    print("4) Orbit and Comet objects")

    jd = J2000

    # Earth-like orbit
    earth_orbit = Orbit(
        semi_major_axis_au=1.0,
        eccentricity=0.0167,
        inclination_deg=0.00005,
        lon_ascending_node_deg=-11.26,
        arg_perihelion_deg=102.95,
        mean_anomaly_deg=100.46,
        epoch_jd=jd,
    )
    print(f"  Earth orbit: {earth_orbit}")
    print(f"  Period: {earth_orbit.period_years():.3f} years")
    earth_pos = earth_orbit.kepler_position(jd)
    print(f"  Position at J2000: r = {earth_pos.distance():.6f} AU")

    # Preset comets
    for comet in [Comet.halley(), Comet.encke(), Comet.hale_bopp()]:
        pos = comet.kepler_position(jd)
        print(
            f"  {comet.name}: a={comet.orbit.semi_major_axis_au:.3f} AU, "
            f"e={comet.orbit.eccentricity:.6f}, "
            f"P={comet.period_years():.1f} yr, "
            f"r(J2000)={pos.distance():.3f} AU"
        )

    # Custom comet
    custom_orbit = Orbit(
        semi_major_axis_au=3.0,
        eccentricity=0.6,
        inclination_deg=12.0,
        lon_ascending_node_deg=45.0,
        arg_perihelion_deg=200.0,
        mean_anomaly_deg=0.0,
        epoch_jd=jd,
    )
    custom_comet = Comet("MyComet", custom_orbit, tail_length_km=500_000.0)
    pos = custom_comet.kepler_position(jd)
    print(
        f"  {custom_comet.name}: a={custom_comet.orbit.semi_major_axis_au:.3f} AU, "
        f"P={custom_comet.period_years():.1f} yr, "
        f"r(J2000)={pos.distance():.3f} AU"
    )
    print()


def main():
    print("Target + Trackable examples")
    print("===========================\n")

    section_trackable_objects()
    section_target_snapshots()
    section_proper_motion()
    section_orbit_comets()

    print("=== Example Complete ===")


if __name__ == "__main__":
    main()
