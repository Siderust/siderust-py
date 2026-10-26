"""Angular orientations with explicit reference frames.

ICRS uses right ascension and declination. Horizontal uses geometric altitude
and North-clockwise azimuth through East. Horizontal carries no observer,
epoch, refraction model or object identity; comparing local directions requires
the caller to establish a common observing context.
"""

from ._siderust import ICRS, Horizontal

__all__ = ["ICRS", "Horizontal"]
