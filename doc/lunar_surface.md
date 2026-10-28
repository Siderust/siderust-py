# Lunar surface Alt/Az

Siderust for Python 0.7 exposes geometric altitude and azimuth for observers
on the Moon's IAU mean sphere, backed by Siderust Rust 0.16.

## API

```python
from siderust import LunarObserver, Position
from siderust.direction import ICRS, Horizontal

site = LunarObserver(lon_deg=20.0, lat_deg=-3.5, height_m=0.0)
hz: Horizontal = site.observe_direction(ICRS(0.0, 90.0), jd_tt=2451545.0)

target = Position(
    1617.7391512763131,
    642.0177864954426,
    -96.38962730131595,
    frame="MoonIauMeanEarth",
    center="Selenocentric",
    unit="km",
)
obs = site.observe_position(target, jd_tt=2451545.0)
print(obs.alt_deg, obs.az_deg, obs.range, obs.range_unit)

earth = site.observe_earth(jd_tt=2460000.5)
sun = site.observe_sun(jd_tt=2460000.5)
```

`LunarObserver` is distinct from Earth `Observer` (WGS84 / ECEF). Do not
reinterpret an Earth geodetic site as a lunar site.

## Coordinates

| Name | Role |
|------|------|
| `Selenocentric` | Moon-centred origin |
| `MoonIauMeanEarth` | NAIF `IAU_MOON` analytical Mean-Earth body-fixed frame |
| `MoonPrincipalAxes` | Legacy **alias** of `MoonIauMeanEarth` (not JPL DE `MOON_PA`) |

`Position` accepts these centre/frame names and canonicalizes
`MoonPrincipalAxes` → `MoonIauMeanEarth`. Unsupported combinations raise
`ValueError`.

## Conventions and limitations

- Longitude: east-positive
- Latitude: planetocentric on the IAU sphere (radius 1737.4 km)
- Height: metres above that sphere (along the outward normal)
- Azimuth: north = 0°, east = 90°, increasing north through east
- Altitude: geometric horizon = 0°, zenith = +90°
- Epoch: **Julian Date TT** (not MJD). Orientation converts TT→TDB via tempoch
- Zenith/nadir: azimuth is reported as `0°` by convention
- **Geometric only**: no atmospheric refraction, aberration, light-time,
  relativistic deflection, or terrain masking
- Direction-only targets never invent a distance (no artificial parallax)

## Validation provenance

Python tests mirror Siderust v0.16.0 `tests/body_surface_horizontal.rs`,
including the composite CSPICE `IAU_MOON` orientation + independent ENU
reference at J2000 TT (CSPICE N0067 / SpiceyPy 8.3.0, `pck00011.tpc`).

See also [examples/14_lunar_surface_altaz.py](../examples/14_lunar_surface_altaz.py).

Website examples for the published PyPI package are a follow-up in the
[siderust.github.io](https://github.com/Siderust/siderust.github.io) repository
after `siderust` 0.7.0 is released.
