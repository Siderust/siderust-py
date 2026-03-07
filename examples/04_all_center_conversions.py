"""
All Center Conversions Example

Mirrors: siderust/examples/04_all_center_conversions.rs

Demonstrates all standard center-shift pairs:
  Barycentric <-> Heliocentric
  Barycentric <-> Geocentric
  Heliocentric <-> Geocentric

Run with: python examples/04_all_center_conversions.py
"""

from siderust import Position

CENTERS = ["Barycentric", "Heliocentric", "Geocentric"]


def show_center_conversion(
    jd: float, src: Position, to_center: str
) -> None:
    """Convert src to to_center, then back, and report the round-trip error."""
    out = src.to_center(to_center, jd)
    back = out.to_center(src.center, jd)
    dx = src.x - back.x
    dy = src.y - back.y
    dz = src.z - back.z
    err = (dx * dx + dy * dy + dz * dz) ** 0.5

    print(
        f"{src.center:<12} -> {to_center:<12} "
        f"out=({out.x:+.9f}, {out.y:+.9f}, {out.z:+.9f})  "
        f"roundtrip={err:.3e}"
    )


def main():
    jd = 2_460_000.5
    print(f"Center conversion demo at JD(TT) = {jd:.1f}\n")

    # Start with a Barycentric EclipticMeanJ2000 position
    p_bary = Position(
        0.40, -0.10, 1.20,
        frame="EclipticMeanJ2000", center="Barycentric", unit="au",
    )

    # Convert to other centers
    p_helio = p_bary.to_center("Heliocentric", jd)
    p_geo = p_bary.to_center("Geocentric", jd)

    # ── Standard center shifts ───────────────────────────────────────
    print("── Standard center shifts ─────────────────────────────────────────────")

    # Barycentric source
    for target in CENTERS:
        show_center_conversion(jd, p_bary, target)

    print()

    # Heliocentric source
    for target in CENTERS:
        show_center_conversion(jd, p_helio, target)

    print()

    # Geocentric source
    for target in CENTERS:
        show_center_conversion(jd, p_geo, target)

    # ── Combined frame + center transforms ───────────────────────────
    print("\n── Combined frame + center transforms ─────────────────────────────────")

    frames = [
        "EclipticMeanJ2000",
        "EquatorialMeanJ2000",
        "ICRS",
    ]

    for frame in frames:
        src = p_bary.to_frame(frame, jd)
        for target_center in ["Heliocentric", "Geocentric"]:
            out = src.to_center(target_center, jd)
            back = out.to_center(src.center, jd)
            dx = src.x - back.x
            dy = src.y - back.y
            dz = src.z - back.z
            err = (dx * dx + dy * dy + dz * dz) ** 0.5
            print(
                f"  {frame:<24} {src.center:<12} -> {target_center:<12} "
                f"roundtrip={err:.3e}"
            )


if __name__ == "__main__":
    main()
