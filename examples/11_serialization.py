"""
Serialization Example

Mirrors: siderust/examples/11_serde_serialization.rs

Demonstrates Python serialization for siderust types:
- to_dict() / from_dict() for all major types (JSON-compatible)
- pickle support via __reduce__ (Position, SphericalPosition, Orbit, ProperMotion)
- JSON round-trips using the standard json module
- File I/O with JSON

Run with: python examples/11_serialization.py
"""

import json
import os
import pickle
import tempfile

from siderust import (
    Body,
    Comet,
    Direction,
    Observer,
    Orbit,
    Position,
    ProperMotion,
    SphericalPosition,
    Star,
)

J2000 = 2451545.0


def roundtrip_json(d: dict) -> dict:
    """Serialize to JSON string and back."""
    return json.loads(json.dumps(d))


def roundtrip_pickle(obj):
    """Serialize to pickle bytes and back."""
    return pickle.loads(pickle.dumps(obj))


def section_time_values():
    """1) Time values (JulianDate from tempoch)."""
    print("1) TIME VALUES")
    print("--------------")

    from tempoch import JulianDate

    jd = JulianDate(J2000)
    mjd = jd.to_mjd()

    time_data = {
        "j2000": jd.value,
        "mjd": mjd.value,
        "timeline": [jd.value, jd.value + 1.0, jd.value + 7.0],
    }

    json_str = json.dumps(time_data, indent=2)
    print(json_str)

    recovered = json.loads(json_str)
    print(
        f"Roundtrip check: j2000={recovered['j2000']:.1f}, "
        f"timeline_len={len(recovered['timeline'])}\n"
    )


def section_coordinates():
    """2) Coordinate objects."""
    print("2) COORDINATE OBJECTS")
    print("---------------------")

    # Cartesian position
    geo_pos = Position(6371.0, 0.0, 0.0, frame="ICRS", center="Geocentric", unit="km")
    pos_dict = geo_pos.to_dict()
    print(f"Position dict: {json.dumps(pos_dict, indent=2)}")

    recovered_pos = Position.from_dict(roundtrip_json(pos_dict))
    print(f"Roundtrip check: x={recovered_pos.x:.1f} km\n")

    # Spherical position
    helio_sph = SphericalPosition(
        lon_deg=120.0,
        lat_deg=5.0,
        distance=1.2,
        frame="EclipticMeanJ2000",
        center="Heliocentric",
        unit="au",
    )
    sph_dict = helio_sph.to_dict()
    print(f"SphericalPosition dict: {json.dumps(sph_dict, indent=2)}")

    recovered_sph = roundtrip_pickle(helio_sph)
    print(f"Pickle roundtrip: lon={recovered_sph.lon_deg:.4f}°\n")

    # Direction
    direction = Direction(ra_deg=120.0, dec_deg=22.5)
    dir_dict = direction.to_dict()
    print(f"Direction dict: {json.dumps(dir_dict)}")

    recovered_dir = Direction.from_dict(roundtrip_json(dir_dict))
    print(f"Roundtrip check: RA={recovered_dir.ra_deg:.4f}°\n")

    # Observer
    obs = Observer.roque_de_los_muchachos()
    obs_dict = obs.to_dict()
    print(f"Observer dict: {json.dumps(obs_dict, indent=2)}")

    recovered_obs = Observer.from_dict(roundtrip_json(obs_dict))
    print(f"Roundtrip check: lon={recovered_obs.lon_deg:.4f}°\n")


def section_body_objects():
    """3) Body-related objects."""
    print("3) BODY-RELATED OBJECTS")
    print("-----------------------")

    # Star
    sirius = Star.catalog("Sirius")
    star_dict = sirius.to_dict()
    print(f"Star dict: {json.dumps(star_dict, indent=2)}")

    recovered_star = Star.from_dict(roundtrip_json(star_dict))
    print(f"Roundtrip check: {recovered_star.name}, RA={recovered_star.ra_deg:.4f}°\n")

    # Orbit
    earth_orbit = Orbit(
        semi_major_axis_au=1.0,
        eccentricity=0.0167,
        inclination_deg=0.00005,
        lon_ascending_node_deg=-11.26,
        arg_perihelion_deg=102.95,
        mean_anomaly_deg=100.46,
        epoch_jd=J2000,
    )
    orbit_dict = earth_orbit.to_dict()
    print(f"Orbit dict: {json.dumps(orbit_dict, indent=2)}")

    recovered_orbit = Orbit.from_dict(roundtrip_json(orbit_dict))
    print(f"Roundtrip check: a={recovered_orbit.semi_major_axis_au:.6f} AU")

    # Orbit pickle
    orbit_pickled = roundtrip_pickle(earth_orbit)
    print(f"Pickle roundtrip: a={orbit_pickled.semi_major_axis_au:.6f} AU\n")

    # Comet (Halley)
    halley = Comet.halley()
    halley_pos = halley.kepler_position(J2000)
    snapshot = {
        "name": halley.name,
        "epoch": J2000,
        "orbit": halley.orbit.to_dict(),
        "heliocentric_ecliptic": halley_pos.to_dict(),
    }
    print("Halley snapshot JSON:")
    print(json.dumps(snapshot, indent=2))

    recovered = json.loads(json.dumps(snapshot))
    recovered_orbit = Orbit.from_dict(recovered["orbit"])
    recovered_pos = Position.from_dict(recovered["heliocentric_ecliptic"])
    print(
        f"Roundtrip check: {recovered['name']} @ JD {recovered['epoch']:.1f}, "
        f"r={recovered_pos.distance():.6f} AU\n"
    )


def section_targets():
    """4) Target objects."""
    print("4) TARGET OBJECTS")
    print("-----------------")

    jd = J2000

    # Planet target
    mars_target = Body.Mars.track(jd)
    mars_data = {
        "body": "Mars",
        "epoch": mars_target.time,
        "position": mars_target.position.to_dict(),
    }
    print("Mars target JSON:")
    print(json.dumps(mars_data, indent=2))

    recovered = json.loads(json.dumps(mars_data))
    recovered_pos = Position.from_dict(recovered["position"])
    print(
        f"Roundtrip check: {recovered['body']} target JD {recovered['epoch']:.1f}, "
        f"r={recovered_pos.distance():.6f} AU\n"
    )

    # Star target
    sirius_target = Star.catalog("Sirius").track(jd)
    star_data = {
        "star": "Sirius",
        "epoch": sirius_target.time,
        "direction": sirius_target.direction.to_dict(),
    }
    print("Sirius target JSON:")
    print(json.dumps(star_data, indent=2))

    recovered = json.loads(json.dumps(star_data))
    recovered_dir = Direction.from_dict(recovered["direction"])
    print(f"Roundtrip check: {recovered['star']} target, RA={recovered_dir.ra_deg:.4f}°\n")


def section_proper_motion():
    """5) ProperMotion pickle roundtrip."""
    print("5) PROPER MOTION SERIALIZATION")
    print("------------------------------")

    pm = ProperMotion(pm_ra_mas_yr=27.54, pm_dec_mas_yr=10.86)
    pm_pickled = roundtrip_pickle(pm)
    print(
        f"  Original:  pm_ra={pm.pm_ra_mas_yr:.2f}, pm_dec={pm.pm_dec_mas_yr:.2f}, "
        f"µα⋆={pm.mu_alpha_star}"
    )
    print(
        f"  Roundtrip: pm_ra={pm_pickled.pm_ra_mas_yr:.2f}, pm_dec={pm_pickled.pm_dec_mas_yr:.2f}, "
        f"µα⋆={pm_pickled.mu_alpha_star}"
    )
    print()


def section_file_io():
    """6) File I/O with JSON."""
    print("6) FILE I/O")
    print("-----------")

    jd = J2000

    # Build a composite snapshot
    data = {
        "observer": Observer.roque_de_los_muchachos().to_dict(),
        "stars": [Star.catalog(name).to_dict() for name in ["Sirius", "Betelgeuse", "Polaris"]],
        "mars": Body.Mars.track(jd).position.to_dict(),
        "halley_orbit": Comet.halley().orbit.to_dict(),
    }

    out_path = os.path.join(tempfile.gettempdir(), "siderust_serialization_example.json")
    with open(out_path, "w") as f:
        json.dump(data, f, indent=2)

    with open(out_path) as f:
        loaded = json.load(f)

    # Verify roundtrip
    obs = Observer.from_dict(loaded["observer"])
    stars = [Star.from_dict(s) for s in loaded["stars"]]
    mars_pos = Position.from_dict(loaded["mars"])
    orbit = Orbit.from_dict(loaded["halley_orbit"])

    print(f"  Saved and loaded: {out_path}")
    print(f"  Observer: {obs}")
    print(f"  Stars: {', '.join(s.name for s in stars)}")
    print(f"  Mars distance: {mars_pos.distance():.6f} AU")
    print(f"  Halley orbit: {orbit}")
    print()


def main():
    print("=== Siderust Serialization Examples ===\n")

    section_time_values()
    section_coordinates()
    section_body_objects()
    section_targets()
    section_proper_motion()
    section_file_io()

    print("=== Example Complete ===")


if __name__ == "__main__":
    main()
