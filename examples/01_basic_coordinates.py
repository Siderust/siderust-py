"""
Basic Coordinates Example

Mirrors: siderust/examples/01_basic_coordinates.rs

Run with: python examples/01_basic_coordinates.py
"""

from siderust import Direction, Position, SphericalPosition


def main():
    print("=== Siderust Basic Coordinates Example ===\n")

    # =========================================================================
    # 1. Cartesian Coordinates
    # =========================================================================
    print("1. CARTESIAN COORDINATES")
    print("------------------------")

    # Create a heliocentric ecliptic position (1 AU along X-axis)
    earth_position = Position(
        1.0, 0.0, 0.0, frame="EclipticMeanJ2000", center="Heliocentric", unit="au"
    )
    print("Earth position (Heliocentric EclipticMeanJ2000):")
    print(f"  X = {earth_position.x:.6f} AU")
    print(f"  Y = {earth_position.y:.6f} AU")
    print(f"  Z = {earth_position.z:.6f} AU")
    print(f"  Distance from Sun = {earth_position.distance():.6f} AU\n")

    # Create a geocentric equatorial position (Moon at ~384,400 km)
    moon_position = Position(
        300_000.0, 200_000.0, 100_000.0, frame="EquatorialMeanJ2000", center="Geocentric", unit="km"
    )
    print("Moon position (Geocentric EquatorialMeanJ2000):")
    print(f"  X = {moon_position.x:.1f} km")
    print(f"  Y = {moon_position.y:.1f} km")
    print(f"  Z = {moon_position.z:.1f} km")
    print(f"  Distance from Earth = {moon_position.distance():.1f} km\n")

    # =========================================================================
    # 2. Spherical Coordinates
    # =========================================================================
    print("2. SPHERICAL COORDINATES")
    print("------------------------")

    # Create a star direction (Polaris approximately) using Direction
    polaris = Direction(ra_deg=37.95, dec_deg=89.26)
    print("Polaris (ICRS Direction):")
    print(f"  Right Ascension = {polaris.ra_deg:.2f}°")
    print(f"  Declination = {polaris.dec_deg:.2f}°\n")

    # Create a spherical position with distance (Betelgeuse at ~500 ly)
    betelgeuse_distance = 500.0 * 9.461e15 / 1.496e11  # Convert ly to AU
    betelgeuse = SphericalPosition(
        lon_deg=88.79,
        lat_deg=7.41,
        distance=betelgeuse_distance,
        frame="ICRS",
        center="Barycentric",
        unit="au",
    )
    print("Betelgeuse (Barycentric ICRS Position):")
    print(f"  Right Ascension = {betelgeuse.lon_deg:.2f}°")
    print(f"  Declination = {betelgeuse.lat_deg:.2f}°")
    print(f"  Distance = {betelgeuse.distance:.1f} AU (~500 ly)\n")

    # =========================================================================
    # 3. Directions (Unit Vectors)
    # =========================================================================
    print("3. DIRECTIONS (UNIT VECTORS)")
    print("----------------------------")

    # Directions are ICRS coordinates (RA/Dec)
    zenith_like = Direction(ra_deg=0.0, dec_deg=90.0)
    print("North celestial pole direction (ICRS):")
    print(f"  RA = {zenith_like.ra_deg:.1f}°")
    print(f"  Dec = {zenith_like.dec_deg:.1f}°\n")

    # =========================================================================
    # 4. Cartesian <-> Spherical Conversion
    # =========================================================================
    print("4. CARTESIAN ↔ SPHERICAL CONVERSION")
    print("-----------------------------------")

    # Start with cartesian
    cart_pos = Position(
        0.5, 0.5, 0.707, frame="EquatorialMeanJ2000", center="Barycentric", unit="au"
    )
    print("Cartesian position:")
    print(f"  X = {cart_pos.x:.3f} AU")
    print(f"  Y = {cart_pos.y:.3f} AU")
    print(f"  Z = {cart_pos.z:.3f} AU\n")

    # Convert to spherical
    sph_pos = cart_pos.to_spherical()
    print("Converted to Spherical:")
    print(f"  RA = {sph_pos.lon_deg:.2f}°")
    print(f"  Dec = {sph_pos.lat_deg:.2f}°")
    print(f"  Distance = {sph_pos.distance:.3f} AU")

    # Convert back to cartesian
    cart_pos_back = sph_pos.to_cartesian()
    print("\nConverted back to Cartesian:")
    print(f"  X = {cart_pos_back.x:.3f} AU")
    print(f"  Y = {cart_pos_back.y:.3f} AU")
    print(f"  Z = {cart_pos_back.z:.3f} AU\n")

    # =========================================================================
    # 5. Type Safety
    # =========================================================================
    print("5. COORDINATE METADATA")
    print("----------------------")

    helio_pos = Position(1.0, 0.0, 0.0, frame="EclipticMeanJ2000", center="Heliocentric", unit="au")
    geo_pos = Position(0.0, 1.0, 0.0, frame="EquatorialMeanJ2000", center="Geocentric", unit="au")

    print("Coordinate metadata prevents mixing incompatible systems:")
    print(f"  Heliocentric EclipticMeanJ2000: {helio_pos}")
    print(f"  Geocentric EquatorialMeanJ2000: {geo_pos}")
    print("\n  Must transform to same center/frame before computing distance!\n")

    # Operations within the same type work fine
    pos1 = Position(1.0, 0.0, 0.0, frame="EclipticMeanJ2000", center="Heliocentric", unit="au")
    pos2 = Position(1.5, 0.0, 0.0, frame="EclipticMeanJ2000", center="Heliocentric", unit="au")
    distance = pos1.distance_to(pos2)
    print("Distance between two Heliocentric EclipticMeanJ2000 positions:")
    print(f"  {distance:.3f} AU\n")

    # =========================================================================
    # 6. Different Centers and Frames
    # =========================================================================
    print("6. CENTERS AND FRAMES")
    print("---------------------")

    print("Reference Centers:")
    print("  Barycentric, Heliocentric, Geocentric\n")

    print("Reference Frames:")
    print("  EclipticMeanJ2000, EquatorialMeanJ2000, ICRS, ICRF,")
    print("  EquatorialMeanOfDate, EquatorialTrueOfDate\n")

    print("=== Example Complete ===")


if __name__ == "__main__":
    main()
