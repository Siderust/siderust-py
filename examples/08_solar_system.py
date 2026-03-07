"""
Solar System Module Tour

Mirrors: siderust/examples/08_solar_system.rs

Run with: python examples/08_solar_system.py
"""

from siderust import Body, Position

J2000 = 2_451_545.0
AU_KM = 149_597_870.7


def main():
    jd = J2000

    print("=== Siderust Solar System Module Tour ===\n")
    print(f"Epoch: J2000 (JD {jd:.1f})\n")

    # =========================================================================
    # 1. VSOP87 Positions
    # =========================================================================
    print("1) VSOP87 EPHEMERIDES (HELIOCENTRIC + BARYCENTRIC)")
    print("---------------------------------------------------")

    earth_h = Body.Earth.heliocentric_position(jd)
    mars_h = Body.Mars.heliocentric_position(jd)
    earth_mars = earth_h.distance_to(mars_h)

    print(f"Earth heliocentric distance: {earth_h.distance():.6f} AU")
    print(f"Mars heliocentric distance:  {mars_h.distance():.6f} AU")
    print(f"Earth-Mars separation:       {earth_mars:.6f} AU "
          f"({earth_mars * AU_KM:.0f} km)")

    sun_bary = Body.Sun.barycentric_position(jd)
    print(f"Sun barycentric offset from SSB: {sun_bary.distance():.8f} AU\n")

    # All planets: heliocentric distances
    print("Heliocentric distances at J2000:")
    for body in [Body.Mercury, Body.Venus, Body.Earth, Body.Mars,
                 Body.Jupiter, Body.Saturn, Body.Uranus, Body.Neptune]:
        pos = body.heliocentric_position(jd)
        print(f"  {str(body):<8s} {pos.distance():.6f} AU")
    print()

    # =========================================================================
    # 2. Center Transforms
    # =========================================================================
    print("2) CENTER TRANSFORMS (HELIOCENTRIC -> GEOCENTRIC)")
    print("---------------------------------------------------")

    mars_helio = Body.Mars.heliocentric_position(jd)
    mars_geo = mars_helio.to_center("Geocentric", jd)

    print(f"Mars geocentric distance at J2000: {mars_geo.distance():.6f} AU")
    print(f"Mars geocentric distance at J2000: {mars_geo.distance() * AU_KM:.0f} km\n")

    # =========================================================================
    # 3. Moon
    # =========================================================================
    print("3) MOON")
    print("-------")

    moon_geo = Body.Moon.geocentric_position(jd)
    moon_au = moon_geo.to_unit("au")
    print(f"Moon geocentric distance (ELP2000): {moon_geo.distance():.1f} km "
          f"({moon_au.distance():.6f} AU)\n")

    # =========================================================================
    # 4. Barycentric vs Heliocentric
    # =========================================================================
    print("4) BARYCENTRIC vs HELIOCENTRIC")
    print("-------------------------------")

    for body in [Body.Mercury, Body.Venus, Body.Earth, Body.Mars]:
        helio = body.heliocentric_position(jd)
        bary = body.barycentric_position(jd)
        print(f"  {str(body):<8s} helio={helio.distance():.5f} AU  "
              f"bary={bary.distance():.5f} AU")
    print()

    # =========================================================================
    # 5. Frame + Center combined
    # =========================================================================
    print("5) COMBINED TRANSFORM: HELIO ECL -> GEO EQ")
    print("-------------------------------------------")

    mars_geo_eq = mars_helio.transform("EquatorialMeanJ2000", "Geocentric", jd)
    print(f"Mars (Geocentric EquatorialMeanJ2000):")
    print(f"  X = {mars_geo_eq.x:.6f} AU")
    print(f"  Y = {mars_geo_eq.y:.6f} AU")
    print(f"  Z = {mars_geo_eq.z:.6f} AU")
    print(f"  Distance = {mars_geo_eq.distance():.6f} AU\n")

    print("=== End of example ===")


if __name__ == "__main__":
    main()
