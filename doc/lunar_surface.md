# Lunar surface Alt/Az

Siderust for Python 0.8 exposes geometric altitude and azimuth for observers on
the Moon through two explicit backends:

1. **Analytical** (unchanged from 0.7): VSOP/ELP ephemerides with NAIF
   `IAU_MOON` Mean-Earth orientation (`MoonIauMeanEarth`).
2. **SPICE / DE440**: one immutable `SpiceContext` supplying SPK states, binary
   PCK Type 2 orientation, and FK lunar frames (`MOON_ME`, `MOON_PA`, …).

## Analytical API

```python
from siderust import LunarObserver, Position
from siderust.direction import ICRS, Horizontal

site = LunarObserver(lon_deg=20.0, lat_deg=-3.5, height_m=0.0)
hz: Horizontal = site.observe_direction(ICRS(0.0, 90.0), jd_tt=2451545.0)

earth = site.observe_earth(jd_tt=2460000.5)
sun = site.observe_sun(jd_tt=2460000.5)
```

`LunarObserver` is distinct from Earth `Observer` (WGS84 / ECEF). Do not
reinterpret an Earth geodetic site as a lunar site.

## SPICE / DE440 API

```python
from pathlib import Path
from siderust import SpiceContext, LunarObserver

spice = SpiceContext.from_kernels(
    spk=Path("kernels/de440s.bsp"),
    bpc=Path("kernels/moon_pa_de440_200625.bpc"),
    fk=Path("kernels/moon_de440_220930.tf"),
    pck=Path("kernels/pck00011.tpc"),
    lsk=Path("kernels/naif0012.tls"),
)
# Or, for the documented directory layout:
# spice = SpiceContext.from_de440("kernels")

observer = LunarObserver.from_spice(
    context=spice,
    frame="MOON_ME",
    lon_deg=32.3,
    lat_deg=-89.0,
    radius_m=1_738_000.0,  # actual Moon-centre distance; radial up
)
sun = observer.observe_sun(jd_tt=2461358.5)
earth = observer.observe_earth(jd_tt=2461358.5)
print(sun.alt_deg, sun.az_deg, sun.range, sun.range_unit, sun.above_horizon)

# Exact SPICE ET (TDB seconds past J2000) for CSPICE parity:
sun_et = observer.observe_sun_et(et_tdb_seconds=820000000.0)
```

Kernels are never downloaded automatically. Filenames containing `DE440` do
**not** authenticate mutual compatibility — supply a documented matching set.
See NAIF downloads in [examples/15_spice_lunar_de440.py](../examples/15_spice_lunar_de440.py).

### Ownership

`SpiceContext` is immutable after construction and shared via `Arc`. A
`LunarObserver.from_spice` keeps a cloned handle, so the observer remains valid
after the caller's `spice` variable is deleted. There is no global mutable SPICE
state and no SpiceyPy runtime dependency.

### Model separation

| Name | Meaning |
|------|---------|
| `MoonIauMeanEarth` | Analytical NAIF `IAU_MOON` Mean-Earth |
| `MoonPrincipalAxes` | Legacy **alias** of `MoonIauMeanEarth` (not JPL DE `MOON_PA`) |
| `MOON_ME` / `MOON_PA` | Runtime FK names from the loaded DE440 frame kernel |

A SPICE observer never silently falls back to VSOP/ELP or analytical IAU_MOON.
`observe_direction` / `observe_position` / `observe_icrs_vector` remain
analytical-only and raise `ValueError` on SPICE observers.

### Site geometry

- **Height** (`height_m`): metres above the IAU mean sphere (radius 1737.4 km),
  along the outward sphere normal.
- **Radial** (`radius_m`): actual Moon-centre distance in metres; local up is
  radial. Prefer this when a DEM/LOLA radius is available.
- `height_m` and `radius_m` are mutually exclusive.
- Longitude east-positive; planetocentric latitude; north-positive.
- Terrain occlusion and local terrain normals are **not** implemented.

## Conventions and limitations

- Azimuth: north = 0°, east = 90°, increasing north through east
- Altitude: geometric horizon = 0°, zenith = +90°
- Visibility (`above_horizon`): target **centre** altitude strictly above zero
- Ordinary epochs: **Julian Date TT**. SPICE converts TT→TDB via tempoch.
  Exact-ET methods take TDB seconds past J2000 (SPICE ET) for same-epoch
  comparisons; TT conversion residuals are measured separately.
- **Geometric only** (`abcorr=NONE`): no atmospheric refraction, stellar
  aberration, light-time, relativistic deflection, or terrain masking
- SPICE Sun/Earth slant range is always in **kilometres**
- Analytical Sun range remains in **au**; Earth in km
- Same-kernel CSPICE agreement is not absolute mission-grade pointing accuracy

## Equality, hashing, serialization

Analytical observers keep exact coordinate equality/hash semantics (including
signed zeros and longitude normalization).

SPICE observers additionally require the same frame, site kind, and kernel
provenance aliases. Analytical and SPICE observers at the same lon/lat are
**not** equal. `to_dict` records `backend="spice"`; `from_dict` and pickle
refuse to reconstruct a SPICE observer as analytical.

## Validation provenance

- Analytical tests mirror Siderust v0.16+ body-surface references (CSPICE
  composite `IAU_MOON` + independent ENU).
- SPICE tests reuse independent CSPICE DE440 fixtures from Siderust PR #125
  (`tests/fixtures/lunar_de440`), with exact-ET bounds:
  matrix `5e-14`, position/range `2e-7 km`, altitude/azimuth `1e-10 deg`.
- Optional full-kernel checks: set `SIDERUST_LUNAR_KERNEL_DIR`.

See also:

- [examples/14_lunar_surface_altaz.py](../examples/14_lunar_surface_altaz.py)
- [examples/15_spice_lunar_de440.py](../examples/15_spice_lunar_de440.py)
