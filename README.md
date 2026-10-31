# siderust-py

[![PyPI](https://img.shields.io/pypi/v/siderust.svg)](https://pypi.org/project/siderust/)
[![Docs.rs](https://docs.rs/siderust-py/badge.svg)](https://docs.rs/siderust-py)
[![CI](https://github.com/Siderust/siderust-py/actions/workflows/ci.yml/badge.svg)](https://github.com/Siderust/siderust-py/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)
[![Python 3.9+](https://img.shields.io/badge/python-3.9%2B-blue)](https://www.python.org/)

Astrometry and astrodynamics for Python, powered by the
[siderust](https://github.com/Siderust/siderust) Rust library through PyO3.

`siderust-py` provides Python-native access to observation planning,
coordinate queries, ephemerides, solar-system bodies, stellar targets,
moon-phase calculations, and related astronomy utilities while keeping the
numerical implementation in Rust.

The PyPI badge refers to the Python distribution (`siderust`). The docs.rs
badge documents the companion `siderust-py` Rust crate for cross-extension
interoperability. The Python package is imported as `siderust`.

## Features

- **Rust-backed astronomy** with Python orchestration and Pythonic result types
- **Astronomical time primitives** from the automatically installed dependency, exposed via `siderust.time`
- **Observers and observing sites**, including predefined major observatories
- **Lunar surface observers** with geometric Alt/Az for ICRS directions,
  finite-distance selenocentric targets, and optional DE440 SPICE-backed
  Sun/Earth observations (`SpiceContext`, `LunarObserver.from_spice`)
- **Solar-system bodies and stars** with altitude and azimuth queries
- **Coordinate types** including directions, positions, spherical positions,
  selenocentric centres, and lunar body-fixed frames
- **Observation events** such as threshold crossings and culminations
- **Visibility windows** above or below configurable altitude thresholds
- **Twilight thresholds** for horizon, civil, nautical, and astronomical twilight
- **Moon phase geometry and events**
- **Targets and proper motion** for epoch-aware stellar tracking
- **Orbit, comet, and runtime ephemeris** bindings
- **PyO3 interoperability** for exchanging canonical `Observer`, `direction.ICRS` and `direction.Horizontal`
  objects across independently compiled Rust extensions

## Installation

Install the Python package with pip:

```bash
pip install siderust
```

This installs `tempoch` automatically. Its original Python types and functions
are also accessible through `siderust.time`; a separate installation is not
required.

To build from source, install a Rust toolchain and Maturin:

```bash
git clone https://github.com/Siderust/siderust-py.git
cd siderust-py

python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate

python -m pip install --upgrade pip
python -m pip install "maturin>=1.9.4,<2"
python -m pip install -e .
```

## Quick Start

```python
from siderust import Body, CrossingDirection, Observer, Star, crossings

# Pick an observatory.
observer = Observer.roque_de_los_muchachos()

# Find sunrise and sunset over one day (MJD 60 000).
events = crossings(
    Body.Sun,
    observer,
    60000.0,
    60001.0,
    threshold_deg=0.0,
)

for event in events:
    label = (
        "Sunrise"
        if event.direction == CrossingDirection.Rising
        else "Sunset"
    )
    print(f"{label} at MJD {event.mjd:.6f}")

# Check a star altitude.
vega = Star.catalog("Vega")
altitude = vega.altitude_at(observer, 60000.75)
print(f"Vega altitude: {altitude:.2f}°")
```

## Core API

| Area | Main API |
|------|----------|
| **Time** | `time.JulianDate`, `time.ModifiedJulianDate`, `time.TimePeriod`, `time.TimeScale` |
| **Observers** | `Observer`, predefined observatories |
| **Lunar surface** | `LunarObserver`, `LunarObservation`, `SpiceContext` |
| **Solar system** | `Body`, `Orbit`, `Comet`, `RuntimeEphemeris` |
| **Stars and targets** | `Star`, `Target`, `ProperMotion`, `apply_proper_motion()` |
| **Coordinates** | `direction.ICRS`, `direction.Horizontal`, `Position`, `SphericalPosition` |
| **Altitude / azimuth** | `altitude_at()`, `azimuth_at()`, lunar `observe_direction` / `observe_position` |
| **Visibility** | `above_threshold()`, `below_threshold()` |
| **Events** | `crossings()`, `culminations()`, `intersect_periods()` |
| **Moon phase** | `moon_phase()`, `find_moon_phases()` |
| **Twilight** | `TWILIGHT_HORIZON`, `TWILIGHT_CIVIL`, `TWILIGHT_NAUTICAL`, `TWILIGHT_ASTRONOMICAL` |

### Time primitives

```python
from siderust import Body, Observer
from siderust.time import ModifiedJulianDate

mjd = ModifiedJulianDate(60000.0)
altitude = Body.Sun.altitude_at(Observer.roque_de_los_muchachos(), mjd.value)
```

Siderust's existing observation functions accept MJD numbers, so pass
`.value` when using a `ModifiedJulianDate`. Directly accepting typed time
objects is not yet part of the Python API.

### Observer

```python
from siderust import Observer

observer = Observer(
    lon_deg=-17.89,
    lat_deg=28.75,
    height_m=2396.0,
)

observer = Observer.roque_de_los_muchachos()
observer = Observer.el_paranal()
observer = Observer.mauna_kea()
observer = Observer.la_silla()

print(observer.lon_deg)
print(observer.lat_deg)
print(observer.height_m)
```

### Lunar surface Alt/Az

```python
from siderust import LunarObserver, SpiceContext
from siderust.direction import ICRS

# Analytical IAU_MOON (unchanged from 0.7)
site = LunarObserver(lon_deg=20.0, lat_deg=-3.5)
hz = site.observe_direction(ICRS(0.0, 90.0), jd_tt=2451545.0)
print(hz.alt_deg, hz.az_deg)

# Optional DE440 SPICE path (SPK + binary PCK + FK; no analytical fallback)
spice = SpiceContext.from_de440("kernels")  # documented DE440 filenames
obs = LunarObserver.from_spice(
    spice, frame="MOON_ME", lon_deg=32.3, lat_deg=-89.0, radius_m=1_738_000.0
)
sun = obs.observe_sun(jd_tt=2461358.5)
print(sun.alt_deg, sun.az_deg, sun.range, sun.range_unit, sun.above_horizon)
```

Longitude is east-positive. Analytical sites use the IAU mean sphere
(R = 1737.4 km). Epochs are Julian Date TT (SPICE also accepts exact ET via
`observe_sun_et` / `observe_earth_et`). Results are geometric only. See
[doc/lunar_surface.md](doc/lunar_surface.md),
[examples/14_lunar_surface_altaz.py](examples/14_lunar_surface_altaz.py), and
[examples/15_spice_lunar_de440.py](examples/15_spice_lunar_de440.py).

### Bodies and stars

```python
from siderust import Body, Star

sun_altitude = Body.Sun.altitude_at(observer, 60000.0)
sun_azimuth = Body.Sun.azimuth_at(observer, 60000.0)

vega = Star.catalog("Vega")
custom = Star.from_ra_dec("Target", 10.0, 20.0)

print(vega.name)
print(vega.ra_deg)
print(vega.dec_deg)
print(vega.distance_ly)
```

### Visibility and events

```python
from siderust import (
    Body,
    above_threshold,
    below_threshold,
    crossings,
    culminations,
)

visible = above_threshold(
    Body.Moon,
    observer,
    60000.0,
    60001.0,
    20.0,
)

hidden = below_threshold(
    Body.Moon,
    observer,
    60000.0,
    60001.0,
    20.0,
)

rise_set = crossings(
    Body.Sun,
    observer,
    60000.0,
    60001.0,
    0.0,
)

transits = culminations(
    Body.Sun,
    observer,
    60000.0,
    60001.0,
)
```

`target` arguments accepted by the common observing functions can be a
`Body`, `Star`, or `direction.ICRS`.

## Explicit directions (0.6 migration)

```python
from siderust import direction
from siderust.direction import ICRS, Horizontal

icrs = direction.ICRS(ra_deg=83.633, dec_deg=22.014)
horizontal = direction.Horizontal(alt_deg=45.0, az_deg=180.0)
assert type(icrs) is ICRS
assert type(horizontal) is Horizontal
```

`Direction` → `direction.ICRS`; `HorizontalDirection` → `direction.Horizontal`.
There are no legacy aliases. See the [direction API](doc/direction.md) for
frame conventions, operations, serialization and migration details.

## Examples

The repository includes runnable examples covering the larger API surface:

- [Basic coordinates](examples/01_basic_coordinates.py)
- [Coordinate transformations](examples/02_coordinate_transformations.py)
- [Reference-frame conversions](examples/03_all_frames_conversions.py)
- [Reference-center conversions](examples/04_all_center_conversions.py)
- [Target tracking](examples/05_target_tracking.py)
- [Night events](examples/06_night_events.py)
- [Moon properties](examples/07_moon_properties.py)
- [Solar-system calculations](examples/08_solar_system.py)
- [Star observability](examples/09_star_observability.py)
- [Time periods](examples/10_time_periods.py)
- [Serialization](examples/11_serialization.py)
- [Runtime ephemerides](examples/12_runtime_ephemeris.py)
- [Coordinate operations](examples/13_coordinate_operations.py)
- [Lunar surface Alt/Az](examples/14_lunar_surface_altaz.py)

## Rust / PyO3 interoperability

The project also builds an `rlib` so downstream Rust/PyO3 extensions can
exchange canonical Python `siderust.Observer` and both `siderust.direction.ICRS` / `Horizontal`
objects without relying on duplicate PyO3 class registrations.

Add the published interoperability crate to a Rust extension:

```toml
[dependencies]
pyo3 = "0.30"
siderust-py = "0.8"
```

The public bridge lives under `siderust_py::interop` and uses a versioned,
primitive-only protocol across extension boundaries.

See [Cross-extension interoperability](doc/developers/interop.md) for the
compatibility contract, Cargo setup, and complete examples.

## Relationship with siderust

`siderust-py` is the Python interface to the
[`siderust`](https://github.com/Siderust/siderust) Rust library. Core
astronomy and astrodynamics algorithms remain implemented in Rust; this
repository focuses on Python bindings, Python-facing ergonomics, and safe
cross-extension interoperability.

- Rust core crate: [crates.io/crates/siderust](https://crates.io/crates/siderust)
- Rust core API: [docs.rs/siderust](https://docs.rs/siderust)
- Python/Rust interop crate: [crates.io/crates/siderust-py](https://crates.io/crates/siderust-py)
- Interop API docs: [docs.rs/siderust-py](https://docs.rs/siderust-py)

## Development

Prerequisites:

- Rust stable toolchain
- Python 3.9+
- Maturin 1.9.4 or newer
- pytest
- Ruff

Local setup:

```bash
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate

python -m pip install --upgrade pip
python -m pip install "maturin>=1.9.4,<2" pytest pytest-cov ruff
python -m pip install -e .
```

Common checks used by CI:

```bash
# Python tests
pytest tests/ -v

# Rust tests
cargo test --all-targets

# Formatting
cargo fmt --all -- --check
python -m ruff format --check python tests examples scripts

# Linting
cargo clippy --all-targets --all-features -- -D warnings
python -m ruff check python tests examples scripts
```

The CI matrix validates Python 3.9 through 3.14 and also exercises the
cross-extension bridge contract on Linux, macOS, and Windows.

## Citation

If you use Siderust for Python in scientific work, cite this repository using
[CITATION.cff](CITATION.cff) (also available through GitHub's **Cite this
repository** action). Cite the underlying
[Siderust Rust core](https://github.com/Siderust/siderust/blob/main/CITATION.cff)
and the astronomical models and datasets relevant to your analysis as well.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release notes.

## License

AGPL-3.0 — see [LICENSE](LICENSE).
