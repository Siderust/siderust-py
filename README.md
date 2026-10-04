# siderust — Astrometry & Astrodynamics for Python

Python bindings for the [siderust](https://github.com/Siderust/siderust) Rust
library, providing observation planning, coordinate queries, ephemeris access,
and moon-phase computation — all backed by Rust for performance and precision.

## Installation

```bash
pip install siderust
```

**From source** (requires Rust toolchain and [maturin](https://www.maturin.rs)):

```bash
git clone --recurse-submodules https://github.com/Siderust/siderust-py.git
cd siderust-py
pip install maturin
maturin develop
```

## Quick Start

```python
from siderust import Observer, Body, Star, crossings, CrossingDirection

# 1. Pick an observatory
obs = Observer.roque_de_los_muchachos()

# 2. Find sunrise and sunset over one day (MJD 60 000)
events = crossings(Body.Sun, obs, 60000.0, 60001.0, threshold_deg=0.0)
for e in events:
    label = "Sunrise" if e.direction == CrossingDirection.Rising else "Sunset"
    print(f"{label} at MJD {e.mjd:.6f}")

# 3. Check star visibility at midnight
vega = Star.catalog("Vega")
alt = vega.altitude_at(obs, 60000.75)
print(f"Vega altitude at midnight: {alt:.2f}°")
```

## Features

| Area | API |
|------|-----|
| **Observers** | `Observer(lon, lat, h)`, `.roque_de_los_muchachos()`, `.el_paranal()`, `.mauna_kea()`, `.la_silla()` |
| **Bodies** | `Body.Sun`, `.Moon`, `.Mercury` … `.Neptune` |
| **Stars** | `Star.catalog("Vega")`, `Star.from_ra_dec(name, ra, dec)` |
| **Directions** | `Direction(ra_deg, dec_deg)` — fixed ICRS position |
| **Altitude** | `altitude_at()`, `above_threshold()`, `below_threshold()` |
| **Events** | `crossings()`, `culminations()` |
| **Azimuth** | `azimuth_at()` |
| **Moon phase** | `moon_phase(jd)`, `find_moon_phases(start, end)` |

All heavy computation happens in compiled Rust. Python is used only for
orchestration and display.

Downstream Rust/PyO3 extensions should use the documented
[`siderust_py::interop`](doc/developers/interop.md) API to exchange canonical
Python `Observer` and `Direction` objects safely across extension boundaries.

## API Reference

### Observer

```python
# Custom location
obs = Observer(lon_deg=-17.89, lat_deg=28.75, height_m=2396.0)

# Predefined sites
obs = Observer.roque_de_los_muchachos()
obs = Observer.el_paranal()
obs = Observer.mauna_kea()
obs = Observer.la_silla()

obs.lon_deg   # longitude
obs.lat_deg   # latitude
obs.height_m  # height above ellipsoid
```

### Body

```python
Body.Sun.altitude_at(observer, mjd)  # degrees
Body.Sun.azimuth_at(observer, mjd)   # degrees
```

### Star

```python
vega = Star.catalog("Vega")       # built-in catalog
s = Star.from_ra_dec("X", 10, 20) # custom RA/Dec

vega.name           # "Vega"
vega.ra_deg          # right ascension
vega.dec_deg         # declination
vega.distance_ly     # distance in light-years
vega.altitude_at(obs, mjd)
vega.azimuth_at(obs, mjd)
```

### Free Functions

```python
altitude_at(target, observer, mjd)                           # degrees
azimuth_at(target, observer, mjd)                            # degrees
above_threshold(target, observer, start, end, threshold)     # [(s,e), ...]
below_threshold(target, observer, start, end, threshold)     # [(s,e), ...]
crossings(target, observer, start, end, threshold)           # [CrossingEvent]
culminations(target, observer, start, end)                   # [CulminationEvent]
moon_phase(jd, observer=None)                                # MoonPhaseGeometry
find_moon_phases(start_mjd, end_mjd)                         # [PhaseEvent]
```

`target` can be a `Body`, `Star`, or `Direction`.

## License

AGPL-3.0 — see [LICENSE](LICENSE).
