"""
Lunar surface geometric Alt/Az example

Mirrors the Rust quickstart (`22_moon_surface_quickstart`) and finite-distance
workflow from Siderust 0.16.

Demonstrates:
1) Creating a lunar surface observer (IAU mean sphere).
2) Observing a fixed ICRS direction (no invented parallax).
3) Finite-distance observation of a selenocentric target.
4) Earth and Sun as seen from the lunar site.

Conventions:
- Longitude east-positive; planetocentric latitude; height above R=1737.4 km.
- Azimuth: north = 0°, east = 90° (north through east).
- Altitude: geometric horizon = 0°, zenith = +90°.
- Epoch is Julian Date TT; orientation is NAIF IAU_MOON Mean-Earth
  (`MoonIauMeanEarth`, legacy alias `MoonPrincipalAxes` — not JPL DE MOON_PA).
- Geometric only: no refraction, aberration, light-time, or terrain.

Run with: python examples/14_lunar_surface_altaz.py
"""

from siderust import LunarObserver, Position
from siderust.direction import ICRS

J2000_TT = 2_451_545.0


def main() -> None:
    # 1. Observer on the lunar sphere (lon east, lat planetocentric, height 0).
    site = LunarObserver(lon_deg=20.0, lat_deg=-3.5, height_m=0.0)
    print(f"Lunar site: lon={site.lon_deg}°, lat={site.lat_deg}°")
    print(
        f"  surface: IAU sphere R={site.equatorial_radius_m / 1000.0:.1f} km, "
        f"lat={site.latitude_convention}"
    )
    print("  frame: MoonIauMeanEarth (NAIF IAU_MOON analytical Mean-Earth)")

    # 2. Fixed ICRS direction toward the north celestial pole (unit vector).
    # Direction-only: parallax is not invented.
    ncp = ICRS(ra_deg=0.0, dec_deg=90.0)
    hz = site.observe_direction(ncp, J2000_TT)
    print(
        f"\nGeometric Alt/Az of NCP at J2000 TT: "
        f"alt={hz.alt_deg:.12f}°, az={hz.az_deg:.12f}° (N=0°, E=90°)"
    )

    # Equivalent Cartesian form (same direction).
    hz_vec = site.observe_icrs_vector(0.0, 0.0, 1.0, J2000_TT)
    assert abs(hz_vec.alt_deg - hz.alt_deg) < 1e-12

    # 3. Finite-distance body-fixed target (CSPICE composite reference case).
    # Site-relative ENU offset encoded as a selenocentric MoonIauMeanEarth position.
    near = Position(
        1617.7391512763131,
        642.0177864954426,
        -96.38962730131595,
        frame="MoonIauMeanEarth",
        center="Selenocentric",
        unit="km",
    )
    obs = site.observe_position(near, J2000_TT)
    print(
        f"Finite selenocentric target: alt={obs.alt_deg:.12f}°, "
        f"az={obs.az_deg:.12f}°, range={obs.range:.12f} {obs.range_unit}"
    )

    # 4. Earth and Sun from the lunar site (ephemeris + center transform).
    jd = 2_460_000.5
    earth = site.observe_earth(jd)
    sun = site.observe_sun(jd)
    print(f"\nEpoch TT JD {jd}:")
    print(
        f"  Earth: alt={earth.alt_deg:.4f}°, az={earth.az_deg:.4f}°, "
        f"range={earth.range:.3f} {earth.range_unit}"
    )
    print(
        f"  Sun:   alt={sun.alt_deg:.4f}°, az={sun.az_deg:.4f}°, "
        f"range={sun.range:.6f} {sun.range_unit}"
    )


if __name__ == "__main__":
    main()
