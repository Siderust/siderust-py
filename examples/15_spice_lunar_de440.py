"""
DE440 SPICE lunar surface Sun/Earth observations

Beginner-friendly walkthrough of the SPICE-backed lunar observer introduced in
siderust-py 0.8. No SpiceyPy runtime dependency — geometry is evaluated in Rust.

Demonstrates:
1) Locating a compatible downloaded kernel directory
2) Building an immutable SpiceContext
3) Defining a custom lunar landing-site observer in MOON_ME
4) Geometric Sun and Earth Alt/Az with slant range
5) Horizon visibility (target centre, geometric)
6) Actual Moon-centre radial distance (terrain-ready site definition)
7) Exact-ET parity input for CSPICE comparisons
8) How this differs from the analytical IAU_MOON LunarObserver

Download kernels once (large files stay outside Git):

    mkdir -p /tmp/lunar-kernels && cd /tmp/lunar-kernels
    base=https://naif.jpl.nasa.gov/pub/naif/generic_kernels
    curl -fLO "$base/spk/planets/de440s.bsp"
    curl -fLO "$base/pck/moon_pa_de440_200625.bpc"
    curl -fLO "$base/fk/satellites/a_old_versions/moon_de440_220930.tf"
    curl -fLO "$base/pck/pck00011.tpc"
    curl -fLO "$base/lsk/naif0012.tls"

Run with:

    python examples/15_spice_lunar_de440.py [/path/to/kernels]

Default kernel directory: $SIDERUST_LUNAR_KERNEL_DIR or /tmp/lunar-kernels.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

from siderust import LunarObserver, SpiceContext

DEFAULT_DIR = Path(os.environ.get("SIDERUST_LUNAR_KERNEL_DIR", "/tmp/lunar-kernels"))


def main(argv: list[str]) -> int:
    kernel_dir = Path(argv[1]) if len(argv) > 1 else DEFAULT_DIR
    if not kernel_dir.is_dir():
        print(
            f"Kernel directory not found: {kernel_dir}\n"
            "Download the five DE440-compatible NAIF kernels listed in the "
            "module docstring, then re-run."
        )
        return 1

    # 1–2. Immutable kernel context (SPK + binary PCK + FK + text PCK + LSK).
    # Filenames alone do not prove compatibility; use a documented matching set.
    spice = SpiceContext.from_de440(kernel_dir)
    print(f"Loaded SpiceContext from {kernel_dir}")
    print(f"  provenance aliases: {list(spice.provenance)}")

    # 3–4. Site in MOON_ME (Mean-Earth from the loaded DE440 FK), height above
    # the IAU sphere (R = 1737.4 km). Longitude east-positive; planetocentric lat.
    height_site = LunarObserver.from_spice(
        context=spice,
        frame="MOON_ME",
        lon_deg=32.3,
        lat_deg=-89.0,
        height_m=600.0,
    )
    jd_tt = 2_461_358.5
    sun = height_site.observe_sun(jd_tt)
    earth = height_site.observe_earth(jd_tt)
    print(f"\nHeight site in {height_site.frame} (NAIF {height_site.frame_id})")
    print(f"  epoch TT JD {jd_tt}")
    print(
        f"  Sun:   alt={sun.alt_deg:.6f}°, az={sun.az_deg:.6f}°, "
        f"range={sun.range:.3f} {sun.range_unit}, above={sun.above_horizon}"
    )
    print(
        f"  Earth: alt={earth.alt_deg:.6f}°, az={earth.az_deg:.6f}°, "
        f"range={earth.range:.3f} {earth.range_unit}, above={earth.above_horizon}"
    )

    # 5–6. Actual radial distance (e.g. from a DEM/LOLA product). Local up is
    # radial; no terrain normal or occlusion is applied.
    radial = LunarObserver.from_spice(
        context=spice,
        frame="MOON_ME",
        lon_deg=32.3,
        lat_deg=-89.0,
        radius_m=1_738_000.0,
    )
    sun_r = radial.observe_sun(jd_tt)
    print(
        f"\nRadial site (R={radial.radius_m} m): "
        f"Sun alt={sun_r.alt_deg:.6f}°, range={sun_r.range:.3f} km"
    )

    # 7. Exact SPICE ET (TDB seconds past J2000) for same-epoch CSPICE parity.
    et = 820_000_000.0
    sun_et = radial.observe_sun_et(et)
    print(
        f"Exact ET {et}: Sun alt={sun_et.alt_deg:.12f}°, "
        f"az={sun_et.az_deg:.12f}°, range={sun_et.range:.9f} km"
    )

    # 8. Analytical IAU_MOON path remains available and is a different model.
    analytical = LunarObserver(32.3, -89.0, 600.0)
    a_sun = analytical.observe_sun(jd_tt)
    print(
        f"\nAnalytical IAU_MOON at same lon/lat/height: "
        f"Sun alt={a_sun.alt_deg:.6f}°, range={a_sun.range} {a_sun.range_unit}"
    )
    print(
        "Models differ: analytical uses VSOP/ELP + IAU_MOON; "
        "SPICE uses the loaded DE440 SPK/BPC/FK with no analytical fallback."
    )
    assert analytical != radial
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
