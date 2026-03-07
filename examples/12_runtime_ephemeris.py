"""
Runtime Ephemeris Example

Mirrors: siderust/examples/12_runtime_ephemeris.rs

Demonstrates how to load a JPL DE4xx BSP file at runtime and query
planetary positions using RuntimeEphemeris.

Unlike compile-time backends, BSP loading lets you use DE440, DE441,
or any compatible ephemeris file without rebuilding the library.

Usage:
    # Load from a local BSP file
    python examples/12_runtime_ephemeris.py /path/to/de440.bsp

    # Without arguments: demonstrates error handling for missing files
    python examples/12_runtime_ephemeris.py

Run with: python examples/12_runtime_ephemeris.py
"""

import sys

from siderust import RuntimeEphemeris

J2000 = 2451545.0


def print_positions(eph: RuntimeEphemeris, label: str) -> None:
    """Print Sun, Earth, and Moon positions at J2000 from a given ephemeris."""
    jd = J2000

    sun = eph.sun_barycentric(jd)
    earth_bary = eph.earth_barycentric(jd)
    earth_helio = eph.earth_heliocentric(jd)
    earth_vel = eph.earth_barycentric_velocity(jd)
    moon = eph.moon_geocentric(jd)

    print(f"=== {label} — positions at J2000 ===")
    print(f"  Sun  (barycentric):  ({sun.x:.6f}, {sun.y:.6f}, {sun.z:.6f}) AU")
    print(f"  Earth (barycentric): ({earth_bary.x:.6f}, {earth_bary.y:.6f}, {earth_bary.z:.6f}) AU")
    print(
        f"  Earth (heliocentric):({earth_helio.x:.6f}, {earth_helio.y:.6f}, {earth_helio.z:.6f}) AU"
    )
    vx, vy, vz = earth_vel
    print(f"  Earth vel:           ({vx:.8f}, {vy:.8f}, {vz:.8f}) AU/day")
    print(f"  Moon  (geocentric):  ({moon.x:.1f}, {moon.y:.1f}, {moon.z:.1f}) km")
    print(f"  Moon  distance:      {moon.distance():.1f} km")
    print()


def demo_load_from_path(path: str) -> None:
    """1) Attempt to load a BSP file from disk."""
    print("────────────────────────────────────────────────────")
    print(f"1) Load from file: {path}")
    print("────────────────────────────────────────────────────")

    try:
        eph = RuntimeEphemeris.from_bsp(path)
        print(f"  ✓ Loaded: {eph}")
        print_positions(eph, "BSP file")
    except OSError as e:
        print(f"  ✗ Failed to load '{path}': {e}")
        print("    Provide a valid DE440/DE441 BSP path as the first argument.")
        print()


def demo_load_from_bytes() -> None:
    """2) Demonstrate error handling when loading invalid bytes."""
    print("────────────────────────────────────────────────────")
    print("2) Load from bytes in memory")
    print("────────────────────────────────────────────────────")

    fake_data = b"this is not a valid BSP file"
    try:
        RuntimeEphemeris.from_bytes(fake_data)
        print("  ✓ Parsed successfully (unexpected!)")
    except ValueError as e:
        print(f"  ✗ Expected error for invalid data: {e}")
    print()


def demo_api_overview() -> None:
    """3) Show the RuntimeEphemeris API surface."""
    print("────────────────────────────────────────────────────")
    print("3) RuntimeEphemeris API overview")
    print("────────────────────────────────────────────────────")
    print()
    print("  Loading methods:")
    print("    eph = RuntimeEphemeris.from_bsp('de440.bsp')   # from file path")
    print("    eph = RuntimeEphemeris.from_bytes(data)         # from bytes")
    print()
    print("  Query methods (all take a Julian Date float):")
    print("    eph.sun_barycentric(jd)       -> Position (AU, EclipticMeanJ2000, Barycentric)")
    print("    eph.earth_barycentric(jd)     -> Position (AU, EclipticMeanJ2000, Barycentric)")
    print("    eph.earth_heliocentric(jd)    -> Position (AU, EclipticMeanJ2000, Heliocentric)")
    print("    eph.earth_barycentric_velocity(jd) -> (vx, vy, vz) tuple (AU/day)")
    print("    eph.moon_geocentric(jd)       -> Position (km, EclipticMeanJ2000, Geocentric)")
    print()
    print("  Notes:")
    print("    - Compatible with DE430, DE440, DE441 BSP files")
    print("    - DE441 (~1.65 GB) covers year -13200 to +17191")
    print("    - DE440 (~120 MB) covers year 1550 to 2650")
    print(
        "    - BSP files are available from JPL: https://ssd.jpl.nasa.gov/planets/eph_export.html"
    )
    print()


def main():
    print("╔══════════════════════════════════════════════════╗")
    print("║     Siderust Runtime Ephemeris Example           ║")
    print("╚══════════════════════════════════════════════════╝")
    print()

    # Get BSP path from command line, or use a default
    bsp_path = None
    for arg in sys.argv[1:]:
        if not arg.startswith("--"):
            bsp_path = arg
            break

    if bsp_path is None:
        bsp_path = "de440.bsp"

    demo_load_from_path(bsp_path)
    demo_load_from_bytes()
    demo_api_overview()

    print("=== Example Complete ===")


if __name__ == "__main__":
    main()
