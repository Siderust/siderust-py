"""Astronomical time primitives from the canonical tempoch Python package.

Exports refer to the exact objects from tempoch. No duplicate PyO3 types or
independent time-scale conversions are defined here.
"""

from tempoch import (
    ConversionError,
    InvalidIntervalError,
    JulianDate,
    ModifiedJulianDate,
    NonFiniteTimeError,
    TimePeriod,
    TimeScale,
    convert_timescale,
    intersect_periods,
    tai_minus_utc,
)

__all__ = [
    "JulianDate",
    "ModifiedJulianDate",
    "TimePeriod",
    "TimeScale",
    "NonFiniteTimeError",
    "InvalidIntervalError",
    "ConversionError",
    "convert_timescale",
    "tai_minus_utc",
    "intersect_periods",
]
