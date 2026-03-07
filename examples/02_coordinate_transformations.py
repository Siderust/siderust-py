"""
Coordinate Transformations Example

Mirrors: siderust/examples/02_coordinate_transformations.rs

Run with: python examples/02_coordinate_transformations.py
"""

from siderust import Body, Position

# J2000.0 epoch as Julian Date
J2000 = 2_451_545.0


def main():
    print("=== Coordinate Transformations Example ===\n")

    jd = J2000
    print(f"Reference time: J2000.0 (JD {jd:.1f})\n")

    # =========================================================================
    # 1. Frame Transformations (same center)
    # =========================================================================
    print("1. FRAME TRANSFORMATIONS")
    print("------------------------")

    # Start with ecliptic coordinates (heliocentric)
    pos_ecliptic = Position(1.0, 0.0, 0.0,
                            frame="EclipticMeanJ2000",
                            center="Heliocentric", unit="au")
    print("Original (Heliocentric EclipticMeanJ2000):")
    print(f"  X = {pos_ecliptic.x:.6f}")
    print(f"  Y = {pos_ecliptic.y:.6f}")
    print(f"  Z = {pos_ecliptic.z:.6f}\n")

    # Transform to equatorial frame (same heliocentric center)
    pos_equatorial = pos_ecliptic.to_frame("EquatorialMeanJ2000", jd)
    print("Transformed to EquatorialMeanJ2000 frame:")
    print(f"  X = {pos_equatorial.x:.6f}")
    print(f"  Y = {pos_equatorial.y:.6f}")
    print(f"  Z = {pos_equatorial.z:.6f}\n")

    # Transform to ICRS frame
    pos_icrs = pos_equatorial.to_frame("ICRS", jd)
    print("Transformed to ICRS frame:")
    print(f"  X = {pos_icrs.x:.6f}")
    print(f"  Y = {pos_icrs.y:.6f}")
    print(f"  Z = {pos_icrs.z:.6f}\n")

    # =========================================================================
    # 2. Center Transformations (same frame)
    # =========================================================================
    print("2. CENTER TRANSFORMATIONS")
    print("-------------------------")

    # Get Earth's position (heliocentric ecliptic)
    earth_helio = Body.Earth.heliocentric_position(jd)
    print("Earth (Heliocentric EclipticMeanJ2000):")
    print(f"  X = {earth_helio.x:.6f}")
    print(f"  Y = {earth_helio.y:.6f}")
    print(f"  Z = {earth_helio.z:.6f}")
    print(f"  Distance = {earth_helio.distance():.6f} AU\n")

    # Transform to geocentric (Earth becomes origin)
    earth_geo = earth_helio.to_center("Geocentric", jd)
    print("Earth (Geocentric EclipticMeanJ2000) - at origin:")
    print(f"  X = {earth_geo.x:.10f}")
    print(f"  Y = {earth_geo.y:.10f}")
    print(f"  Z = {earth_geo.z:.10f}")
    print(f"  Distance = {earth_geo.distance():.10f} (should be ~0)\n")

    # Get Mars position (heliocentric)
    mars_helio = Body.Mars.heliocentric_position(jd)
    print("Mars (Heliocentric EclipticMeanJ2000):")
    print(f"  X = {mars_helio.x:.6f}")
    print(f"  Y = {mars_helio.y:.6f}")
    print(f"  Z = {mars_helio.z:.6f}")
    print(f"  Distance = {mars_helio.distance():.6f} AU\n")

    # Transform Mars to geocentric
    mars_geo = mars_helio.to_center("Geocentric", jd)
    print("Mars (Geocentric EclipticMeanJ2000) - as seen from Earth:")
    print(f"  X = {mars_geo.x:.6f}")
    print(f"  Y = {mars_geo.y:.6f}")
    print(f"  Z = {mars_geo.z:.6f}")
    print(f"  Distance = {mars_geo.distance():.6f} AU\n")

    # =========================================================================
    # 3. Combined Transformations (center + frame)
    # =========================================================================
    print("3. COMBINED TRANSFORMATIONS")
    print("---------------------------")

    print("Mars transformation chain:")
    print("  Start: Heliocentric EclipticMeanJ2000")

    # Method 1: Step by step
    mars_helio_equ = mars_helio.to_frame("EquatorialMeanJ2000", jd)
    print("  Step 1: Transform frame → Heliocentric EquatorialMeanJ2000")

    mars_geo_equ = mars_helio_equ.to_center("Geocentric", jd)
    print("  Step 2: Transform center → Geocentric EquatorialMeanJ2000")
    print("  Result:")
    print(f"    X = {mars_geo_equ.x:.6f}")
    print(f"    Y = {mars_geo_equ.y:.6f}")
    print(f"    Z = {mars_geo_equ.z:.6f}\n")

    # Method 2: Using .transform() (does both)
    mars_geo_equ_direct = mars_helio.transform("EquatorialMeanJ2000", "Geocentric", jd)
    print("  Or using .transform() directly:")
    print(f"    X = {mars_geo_equ_direct.x:.6f}")
    print(f"    Y = {mars_geo_equ_direct.y:.6f}")
    print(f"    Z = {mars_geo_equ_direct.z:.6f}\n")

    # =========================================================================
    # 4. Barycentric Coordinates
    # =========================================================================
    print("4. BARYCENTRIC COORDINATES")
    print("--------------------------")

    # Get Earth in barycentric coordinates
    earth_bary = Body.Earth.barycentric_position(jd)
    print("Earth (Barycentric EclipticMeanJ2000):")
    print(f"  X = {earth_bary.x:.6f}")
    print(f"  Y = {earth_bary.y:.6f}")
    print(f"  Z = {earth_bary.z:.6f}")
    print(f"  Distance from SSB = {earth_bary.distance():.6f} AU\n")

    # Transform to geocentric
    earth_geo_from_bary = earth_bary.to_center("Geocentric", jd)
    print("Earth (Geocentric, from Barycentric):")
    print(f"  Distance = {earth_geo_from_bary.distance():.10f} (should be ~0)\n")

    # Transform Mars from barycentric to geocentric
    mars_bary = Body.Mars.barycentric_position(jd)
    mars_geo_from_bary = mars_bary.to_center("Geocentric", jd)
    print("Mars (Geocentric, from Barycentric):")
    print(f"  X = {mars_geo_from_bary.x:.6f}")
    print(f"  Y = {mars_geo_from_bary.y:.6f}")
    print(f"  Z = {mars_geo_from_bary.z:.6f}")
    print(f"  Distance = {mars_geo_from_bary.distance():.6f} AU\n")

    # =========================================================================
    # 5. ICRS Frame Transformations
    # =========================================================================
    print("5. ICRS FRAME TRANSFORMATIONS")
    print("-----------------------------")

    star_icrs = Position(100.0, 50.0, 1000.0,
                         frame="ICRS", center="Barycentric", unit="au")
    print("Star (Barycentric ICRS):")
    print(f"  X = {star_icrs.x:.3f}")
    print(f"  Y = {star_icrs.y:.3f}")
    print(f"  Z = {star_icrs.z:.3f}\n")

    # Transform to Geocentric ICRS
    star_gcrs = star_icrs.to_center("Geocentric", jd)
    print("Star (Geocentric ICRS/GCRS):")
    print(f"  X = {star_gcrs.x:.3f}")
    print(f"  Y = {star_gcrs.y:.3f}")
    print(f"  Z = {star_gcrs.z:.3f}")
    print("  (Difference is tiny for distant stars)\n")

    # =========================================================================
    # 6. Round-trip Transformation
    # =========================================================================
    print("6. ROUND-TRIP TRANSFORMATION")
    print("----------------------------")

    original = mars_helio
    print("Original Mars (Heliocentric EclipticMeanJ2000):")
    print(f"  X = {original.x:.10f}")
    print(f"  Y = {original.y:.10f}")
    print(f"  Z = {original.z:.10f}\n")

    # Transform: Helio Ecl → Geo EquatorialMeanJ2000 → Helio Ecl
    temp = original.transform("EquatorialMeanJ2000", "Geocentric", jd)
    recovered = temp.transform("EclipticMeanJ2000", "Heliocentric", jd)

    print("After round-trip transformation:")
    print(f"  X = {recovered.x:.10f}")
    print(f"  Y = {recovered.y:.10f}")
    print(f"  Z = {recovered.z:.10f}\n")

    diff_x = abs(original.x - recovered.x)
    diff_y = abs(original.y - recovered.y)
    diff_z = abs(original.z - recovered.z)
    print("Differences (should be tiny):")
    print(f"  ΔX = {diff_x:.2e}")
    print(f"  ΔY = {diff_y:.2e}")
    print(f"  ΔZ = {diff_z:.2e}\n")

    print("=== Example Complete ===")


if __name__ == "__main__":
    main()
