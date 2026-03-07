"""
siderust: Astrometry & Astrodynamics for Python

This package provides Python bindings for the siderust Rust library,
enabling observation planning, coordinate transforms, altitude/azimuth
queries, ephemeris access, and moon-phase computation — all backed by
Rust for performance and precision.
"""

# Import from the Rust extension module
from siderust._siderust import (
    # Core types
    Observer,
    Body,
    Star,
    Direction,
    Position,
    Displacement,
    SphericalPosition,
    # Event types
    CrossingEvent,
    CulminationEvent,
    CrossingDirection,
    CulminationKind,
    # Moon phase types
    MoonPhaseGeometry,
    MoonPhaseLabel,
    PhaseEvent,
    PhaseKind,
    # Orbit / target / ephemeris types
    Orbit,
    Comet,
    ProperMotion,
    Target,
    RuntimeEphemeris,
    # Free functions
    altitude_at,
    above_threshold,
    below_threshold,
    crossings,
    culminations,
    azimuth_at,
    intersect_periods,
    moon_phase,
    find_moon_phases,
    apply_proper_motion,
    # Twilight constants
    TWILIGHT_HORIZON,
    TWILIGHT_CIVIL,
    TWILIGHT_NAUTICAL,
    TWILIGHT_ASTRONOMICAL,
    # Version
    __version__,
)

__all__ = [
    # Core types
    "Observer",
    "Body",
    "Star",
    "Direction",
    "Position",
    "Displacement",
    "SphericalPosition",
    # Event types
    "CrossingEvent",
    "CulminationEvent",
    "CrossingDirection",
    "CulminationKind",
    # Moon phase types
    "MoonPhaseGeometry",
    "MoonPhaseLabel",
    "PhaseEvent",
    "PhaseKind",
    # Orbit / target / ephemeris types
    "Orbit",
    "Comet",
    "ProperMotion",
    "Target",
    "RuntimeEphemeris",
    # Free functions
    "altitude_at",
    "above_threshold",
    "below_threshold",
    "crossings",
    "culminations",
    "azimuth_at",
    "intersect_periods",
    "moon_phase",
    "find_moon_phases",
    "apply_proper_motion",
    # Twilight constants
    "TWILIGHT_HORIZON",
    "TWILIGHT_CIVIL",
    "TWILIGHT_NAUTICAL",
    "TWILIGHT_ASTRONOMICAL",
    # Version
    "__version__",
]
