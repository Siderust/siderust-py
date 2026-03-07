"""
Coordinate Operations Example

Mirrors: siderust/examples/13_coordinate_operations.rs

Demonstrates angular separation (Vincenty formula), cross-validation
against the cartesian dot product, Euclidean 3D distance, and operations
on spherical/cartesian positions.

Run with: python examples/13_coordinate_operations.py
"""

import math

from siderust import Direction, SphericalPosition


def main():
    print("=== Siderust Coordinate Operations Example ===\n")

    # =========================================================================
    # 1. Angular Separation — Directions (ICRS)
    # =========================================================================
    print("1. ANGULAR SEPARATION (DIRECTIONS)")
    print("-----------------------------------")

    polaris = Direction(ra_deg=37.9546, dec_deg=89.2641)
    sirius = Direction(ra_deg=101.2872, dec_deg=-16.7161)

    sep = polaris.angular_separation(sirius)
    print(f"Polaris  (RA={polaris.ra_deg:.4f}°, Dec={polaris.dec_deg:.4f}°)")
    print(f"Sirius   (RA={sirius.ra_deg:.4f}°, Dec={sirius.dec_deg:.4f}°)")
    print(f"  Angular separation = {sep:.4f}°\n")

    # Nearby pair
    star_a = Direction(ra_deg=10.0, dec_deg=30.0)
    star_b = Direction(ra_deg=10.5, dec_deg=30.5)
    close_sep = star_a.angular_separation(star_b)
    print("Nearby pair (0.5° apart in both RA and Dec):")
    print(f"  Angular separation = {close_sep:.4f}°\n")

    # Self-separation must be 0
    self_sep = polaris.angular_separation(polaris)
    print(f"Self-separation of Polaris = {self_sep:.6f}°  (must be 0)\n")

    # =========================================================================
    # 2. Cross-Validation: Vincenty vs. Cartesian Dot Product
    # =========================================================================
    print("2. CROSS-VALIDATION: SPHERICAL vs. CARTESIAN")
    print("---------------------------------------------")

    polaris_cart = polaris.to_cartesian()
    sirius_cart = sirius.to_cartesian()

    # angle from dot product: cos(θ) = a · b
    dot_val = polaris.dot(sirius)
    angle_rad = math.acos(max(-1.0, min(1.0, dot_val)))
    angle_deg = math.degrees(angle_rad)

    print(
        f"Polaris cartesian: ({polaris_cart[0]:.4f}, {polaris_cart[1]:.4f}, {polaris_cart[2]:.4f})"
    )
    print(f"Sirius  cartesian: ({sirius_cart[0]:.4f}, {sirius_cart[1]:.4f}, {sirius_cart[2]:.4f})")
    print(f"  angle (Cartesian dot)        = {angle_rad:.6f} rad = {angle_deg:.4f}°")
    print(f"  angular_separation (Vincenty) = {sep:.4f}°")
    print(f"  Difference                   = {abs(angle_deg - sep):.2e}°  (near machine epsilon)\n")

    # =========================================================================
    # 3. Dot Product and Perpendicularity
    # =========================================================================
    print("3. DOT PRODUCT")
    print("--------------")

    north_pole = Direction(ra_deg=0.0, dec_deg=90.0)
    equatorial = Direction(ra_deg=0.0, dec_deg=0.0)

    print(f"dot(North Pole, Equatorial point)   = {north_pole.dot(equatorial):.6f}  (must be  0)")

    anti_polaris = Direction(
        ra_deg=polaris.ra_deg + 180.0,
        dec_deg=-polaris.dec_deg,
    )
    print(f"dot(Polaris, anti-Polaris)          = {polaris.dot(anti_polaris):.6f}  (must be -1)\n")

    # =========================================================================
    # 4. Euclidean Distance Between Spherical Positions
    # =========================================================================
    print("4. EUCLIDEAN DISTANCE BETWEEN SPHERICAL POSITIONS")
    print("--------------------------------------------------")

    earth = SphericalPosition(
        lon_deg=100.0,
        lat_deg=0.0,
        distance=1.0,
        frame="EclipticMeanJ2000",
        center="Heliocentric",
        unit="au",
    )
    mars = SphericalPosition(
        lon_deg=200.0,
        lat_deg=2.0,
        distance=1.524,
        frame="EclipticMeanJ2000",
        center="Heliocentric",
        unit="au",
    )

    eu_dist = earth.distance_to(mars)
    ang_sep_pos = earth.angular_separation(mars)

    print("Earth (lon=100°, lat=0°, r=1.000 AU)")
    print("Mars  (lon=200°, lat=2°, r=1.524 AU)")
    print(f"  Euclidean 3D distance  = {eu_dist:.4f} AU")
    print(f"  Angular separation     = {ang_sep_pos:.4f}°\n")

    # =========================================================================
    # 5. Same Operations from Cartesian Positions
    # =========================================================================
    print("5. CARTESIAN POSITION OPERATIONS")
    print("---------------------------------")

    earth_cart = earth.to_cartesian()
    mars_cart = mars.to_cartesian()

    eu_dist_cart = earth_cart.distance_to(mars_cart)
    print(f"Earth cartesian: ({earth_cart.x:.4f}, {earth_cart.y:.4f}, {earth_cart.z:.4f})")
    print(f"Mars  cartesian: ({mars_cart.x:.4f}, {mars_cart.y:.4f}, {mars_cart.z:.4f})")
    print(f"  Euclidean distance = {eu_dist_cart:.4f} AU  (matches spherical)")

    # Vector difference
    diff = earth_cart - mars_cart
    mag = (diff[0] ** 2 + diff[1] ** 2 + diff[2] ** 2) ** 0.5
    print(f"  Earth − Mars vector: ({diff[0]:.4f}, {diff[1]:.4f}, {diff[2]:.4f})")
    print(f"  |Earth − Mars|      = {mag:.4f} AU\n")

    # =========================================================================
    # 6. Ecliptic Directions via SphericalPosition
    # =========================================================================
    print("6. ECLIPTIC DIRECTIONS (SPHERICAL POSITIONS)")
    print("---------------------------------------------")

    vernal_equinox = SphericalPosition(
        lon_deg=0.0,
        lat_deg=0.0,
        distance=1.0,
        frame="EclipticMeanJ2000",
        center="Heliocentric",
    )
    summer_solstice = SphericalPosition(
        lon_deg=90.0,
        lat_deg=0.0,
        distance=1.0,
        frame="EclipticMeanJ2000",
        center="Heliocentric",
    )

    equinox_to_solstice = vernal_equinox.angular_separation(summer_solstice)
    print(
        f"Vernal Equinox → Summer Solstice angular sep = {equinox_to_solstice:.4f}°  (must be 90°)"
    )

    ecliptic_north = SphericalPosition(
        lon_deg=0.0,
        lat_deg=90.0,
        distance=1.0,
        frame="EclipticMeanJ2000",
        center="Heliocentric",
    )
    ecliptic_to_equinox = ecliptic_north.angular_separation(vernal_equinox)
    print(
        "Ecliptic North Pole → Vernal Equinox ang. sep = "
        f"{ecliptic_to_equinox:.4f}°  (must be 90°)\n"
    )

    print("=== Example Complete ===")


if __name__ == "__main__":
    main()
