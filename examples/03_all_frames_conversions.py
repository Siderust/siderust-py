"""
All Frames Conversions Example

Mirrors: siderust/examples/03_all_frames_conversions.rs

Demonstrates every supported frame-rotation pair and verifies round-trip
accuracy.

Run with: python examples/03_all_frames_conversions.py
"""

from siderust import Position

FRAMES = [
    "ICRS",
    "ICRF",
    "EclipticMeanJ2000",
    "EquatorialMeanJ2000",
    "EquatorialMeanOfDate",
    "EquatorialTrueOfDate",
]


def show_frame_conversion(jd: float, src: Position, to_frame: str) -> None:
    """Convert src to to_frame, then back, and report the round-trip error."""
    out = src.to_frame(to_frame, jd)
    back = src.to_frame(src.frame, jd)  # identity, just for the pattern
    # Real round-trip: src → out → back
    back = out.to_frame(src.frame, jd)
    dx = src.x - back.x
    dy = src.y - back.y
    dz = src.z - back.z
    err = (dx * dx + dy * dy + dz * dz) ** 0.5

    print(
        f"{src.frame:<24} -> {to_frame:<24} "
        f"out=({out.x:+.9f}, {out.y:+.9f}, {out.z:+.9f})  "
        f"roundtrip={err:.3e}"
    )


def main():
    jd = 2_460_000.5
    print(f"Frame conversion demo at JD(TT) = {jd:.1f}\n")

    # Build a source position in each frame (all Barycentric, AU)
    p_icrs = Position(0.30, -0.70, 0.64, frame="ICRS", center="Barycentric")

    # Convert to all other frames to get starting positions
    positions = {}
    for frame in FRAMES:
        if frame == "ICRS":
            positions[frame] = p_icrs
        else:
            positions[frame] = p_icrs.to_frame(frame, jd)

    # Identity conversions
    print("── Identity conversions ────────────────────────────────────────")
    for frame in FRAMES:
        show_frame_conversion(jd, positions[frame], frame)

    # All non-identity pairs
    print("\n── All non-identity conversions ────────────────────────────────")
    for src_frame in FRAMES:
        for dst_frame in FRAMES:
            if src_frame != dst_frame:
                show_frame_conversion(jd, positions[src_frame], dst_frame)


if __name__ == "__main__":
    main()
