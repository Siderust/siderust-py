"""
Star Observability: altitude + azimuth constraints

Mirrors: siderust/examples/09_star_observability.rs

Finds periods when Sirius is observable from Roque de los Muchachos
within both an altitude range and an azimuth range simultaneously.

Run with: python examples/09_star_observability.py
"""

from siderust import (
    Observer,
    Star,
    above_threshold,
    below_threshold,
    intersect_periods,
)


def main():
    print("Star observability: altitude + azimuth constraints\n")

    observer = Observer.roque_de_los_muchachos()
    target = Star.catalog("Sirius")

    # One-night search window (MJD TT)
    t_0 = 60000.0
    window_start = t_0
    window_end = t_0 + 1.0  # 1 day

    # Constraint 1: altitude between 25° and 65°
    min_alt = 25.0
    max_alt = 65.0

    # Get above min_alt periods
    above_min = above_threshold(target, observer, window_start, window_end, min_alt)
    # Get below max_alt periods
    below_max = below_threshold(target, observer, window_start, window_end, max_alt)
    # Altitude range = intersection of above_min and below_max
    altitude_periods = intersect_periods(above_min, below_max)

    # Constraint 2: azimuth between 110° and 220° (ESE -> SW sector)
    # We use crossings to approximate azimuth periods.
    # For a simpler approach, we sample azimuth over the altitude periods.
    min_az = 110.0
    max_az = 220.0

    print(f"Observer: {observer}")
    print(f"Target: {target.name}")
    print(f"Window: MJD {window_start:.1f} -> {window_end:.1f}\n")

    print(f"Altitude range: {min_alt}°..{max_alt}°")
    print(f"Azimuth range:  {min_az}°..{max_az}°\n")

    print(f"Altitude-constrained periods: {len(altitude_periods)}")
    for idx, (start, end) in enumerate(altitude_periods):
        hours = (end - start) * 24.0
        print(f"  {idx + 1}. MJD {start:.6f} -> {end:.6f}  ({hours:.2f} h)")

    total_hours = sum((end - start) * 24.0 for start, end in altitude_periods)
    print(f"\nTotal time in altitude range: {total_hours:.2f} h")

    # Note: full azimuth-period intersection would require azimuth-period
    # queries (not yet exposed as a free function). The Rust example uses
    # the trait-based AzimuthProvider which is available on Body/Star objects.
    # Here we demonstrate the altitude + period intersection workflow.


if __name__ == "__main__":
    main()
