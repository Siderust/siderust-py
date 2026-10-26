# Spherical directions

```python
from siderust import direction
from siderust.direction import ICRS, Horizontal

icrs = direction.ICRS(ra_deg=83.633, dec_deg=22.014)
horizontal = direction.Horizontal(alt_deg=45.0, az_deg=180.0)
```

`direction` is a real Python module. Both import forms resolve to the same
canonical PyO3 classes, whose `__module__` is `siderust.direction` and whose
`__qualname__` is `ICRS` or `Horizontal`. No generic public direction class,
frame registry or automatic frame coercion is provided. Future frames can add
another small wrapper of the corresponding Rust direction type here.

## Coordinates and operations

| Class | Constructor / read-only properties | Rust spherical type |
| --- | --- | --- |
| `direction.ICRS` | `ra_deg`, `dec_deg` | `direction::ICRS` |
| `direction.Horizontal` | `alt_deg`, `az_deg` | `direction::Horizontal` |

All angles are degrees and must be finite. Rust constructors canonicalize
latitude-like angles to `[-90, 90]` and longitude-like angles to `[0, 360)`;
folding across a pole also shifts longitude by 180 degrees. For example,
`Horizontal(100, 10)` has altitude 80 and azimuth 190. No normalization is
implemented in Python or its bindings.

Horizontal altitude is above the geometric horizon; azimuth is North-clockwise
through East (0 = North, 90 = East). A Horizontal value contains no observer,
epoch, atmosphere, refraction correction or object identity. It cannot identify
a celestial target. It does not expose `track()` or observing methods and is
rejected by Target, proper motion and observing query dispatch. Transformations
requiring an observer must receive that context explicitly; no frame conversion
is performed by either constructor.

`ICRS.angular_separation(other)` returns degrees, `ICRS.dot(other)` returns a
unit-vector scalar product, and `ICRS.to_cartesian()` returns the dimensionless
`(x, y, z)` unit vector in ICRS. These operations delegate to Rust and accept
ICRS operands only. A Cartesian tuple retains no Python frame metadata; callers
must preserve its ICRS interpretation. It is not a spatial Position.

`ICRS.altitude_at(observer, mjd)` and `ICRS.azimuth_at(observer, mjd)` require an
explicit Observer and Modified Julian Date. The common observing query functions
also accept ICRS, Body and Star, with the same explicit context.

## Tracking and proper motion

`ICRS.track(jd)` returns a Target snapshot with a Julian Date and the same ICRS
orientation. `Star.track(jd).direction` is also a canonical ICRS. Target accepts
Position or ICRS, and its `.direction` getter returns ICRS; `.position` remains
reserved for spatial positions. Neither a timestamp nor a direction supplies
an observation origin.

`apply_proper_motion(icrs, pm, jd)` returns ICRS. It retains the existing Rust
core's linear J2000 equatorial catalog propagation, including the `mu_alpha_star`
convention and pole error. The core 0.15 helper takes a J2000 mean-equatorial
position: this existing adapter identifies its catalog RA/Dec with ICRS RA/Dec;
it does not model frame bias, perspective acceleration, parallax or radial
velocity. It is a catalog approximation, not a general frame transform or full
space-motion solution. This release does not redesign that scientific model.

## Value semantics and serialization

Both classes are immutable. Equality compares canonical angular coordinates
exactly in the same frame; it is not a tolerance-based physical equivalence
relation. Signed zeros compare and hash equally. Distinct frames compare unequal.
For approximate comparisons, use angular separation with an explicit tolerance.

Both support `to_dict()` and `from_dict()` with the constructor's keys, and
`pickle` with canonical type identity preserved. Dictionary payloads describe
angles only; callers select the class when deserializing. They contain no
observer or epoch. Repr uses the full public path and constructor keyword names.

## Migration to 0.6.0

```text
Direction           → direction.ICRS
HorizontalDirection → direction.Horizontal
```

Replace `from siderust import Direction` with
`from siderust.direction import ICRS`, or use `from siderust import direction`.
There are no legacy aliases or top-level ICRS / Horizontal exports. Old pickles
referring to `siderust.Direction` must be migrated with the old package before
upgrading (for example, export dictionaries and reconstruct with `ICRS.from_dict`).
The provisional HorizontalDirection name was introduced by this unreleased PR.

Rust consumers must rebuild for bridge protocol 2; see the
[interoperability guide](developers/interop.md).
