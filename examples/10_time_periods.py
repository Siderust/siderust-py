"""
Time Scales, Formats, and Period Conversions Example

Mirrors: siderust/examples/10_time_periods.rs

Demonstrates tempoch (re-exported via `siderust.time` or as standalone):
- Constructing instants from datetime
- Viewing the same absolute instant on each supported time scale
- JulianDate / ModifiedJulianDate conversions
- TimePeriod construction and duration queries
- convert_timescale for time-scale transformations
- Period intersection through intersect_periods

Run with: python examples/10_time_periods.py
"""

from datetime import datetime, timedelta, timezone

from tempoch import (
    JulianDate,
    TimePeriod,
    TimeScale,
    convert_timescale,
    intersect_periods,
    tai_minus_utc,
)


def print_scale(label: str, value: float, reference_jd: float) -> None:
    """Print a time-scale value and its roundtrip drift from a reference JD."""
    drift_s = (value - reference_jd) * 86400.0
    print(f"   {label:<10} value = {value:>18.9f}  | JD roundtrip drift = {drift_s:>11.3e} s")


def main() -> None:
    print("Time Scales, Formats, and Period Conversions")
    print("============================================\n")

    # Reference UTC instant: 2000-01-02 00:00:00 UTC
    utc_ref = datetime(2000, 1, 2, 0, 0, 0, tzinfo=timezone.utc)
    jd = JulianDate.from_datetime(utc_ref)
    jd_val = jd.value

    print(f"Reference UTC instant: {utc_ref.isoformat()}\n")

    # ── 1) Each supported time scale ──────────────────────────────────
    print("1) Each supported time scale for the same instant:")

    scales = [
        ("JD", TimeScale.JD),
        ("JDE", TimeScale.JDE),
        ("MJD", TimeScale.MJD),
        ("TDB", TimeScale.TDB),
        ("TT", TimeScale.TT),
        ("TAI", TimeScale.TAI),
        ("TCG", TimeScale.TCG),
        ("TCB", TimeScale.TCB),
        ("GPS", TimeScale.GPS),
        ("Unix", TimeScale.UnixTime),
        ("UT", TimeScale.UT),
    ]

    for label, scale in scales:
        val = convert_timescale(jd_val, TimeScale.JD, scale)
        # For roundtrip check, convert back to JD
        back = convert_timescale(val, scale, TimeScale.JD)
        drift_s = (back - jd_val) * 86400.0
        print(f"   {label:<10} value = {val:>18.9f}  | JD roundtrip drift = {drift_s:>11.3e} s")

    # ΔT = TT − UT
    tt_val = convert_timescale(jd_val, TimeScale.JD, TimeScale.TT)
    ut_val = convert_timescale(jd_val, TimeScale.JD, TimeScale.UT)
    delta_t = (tt_val - ut_val) * 86400.0
    print(f"   {'UT':<10} delta_t = {delta_t:.3f} s (TT - UT)\n")

    # ── 2) Time formats / aliases ─────────────────────────────────────
    print("2) Time formats / aliases:")
    print(f"   JulianDate:         {jd}")
    mjd = jd.to_mjd()
    print(f"   ModifiedJulianDate: {mjd}")
    utc_roundtrip = jd.to_datetime()
    print(f"   UTC roundtrip from JD: {utc_roundtrip.isoformat()}")

    # JD <-> MJD round-trip
    jd_back = mjd.to_jd()
    print(f"   MJD -> JD roundtrip:   {jd_back}")
    print()

    # ── 3) Period representations ─────────────────────────────────────
    print("3) Period representations and conversions:")

    jd_end = JulianDate(jd_val + 0.5)
    period = TimePeriod(jd, jd_end)
    print(
        f"   Original period: [{period.start} -> {period.end}]  "
        f"Δ = {period.duration_days():.6f} days"
    )

    # Convert period endpoints through each scale and show results
    for label, scale in scales:
        start_val = convert_timescale(period.start.value, TimeScale.JD, scale)
        end_val = convert_timescale(period.end.value, TimeScale.JD, scale)
        duration = end_val - start_val
        print(f"   {label:<10} [{start_val:>18.9f} -> {end_val:>18.9f}]  Δ = {duration:.6f} days")

    # UTC datetime period
    utc_start = jd.to_datetime()
    utc_end = jd_end.to_datetime()
    utc_duration_s = (utc_end - utc_start).total_seconds()
    print(
        f"   {'UTC':<10} [{utc_start.isoformat()} -> {utc_end.isoformat()}]  "
        f"Δ = {utc_duration_s / 86400:.6f} days ({utc_duration_s:.0f} s)"
    )
    print()

    # ── 4) Conversions from a UTC window ──────────────────────────────
    print("4) UTC window conversions:")

    utc_window_start = utc_ref
    utc_window_end = utc_ref + timedelta(hours=6)

    jd_start = JulianDate.from_datetime(utc_window_start)
    jd_window_end = JulianDate.from_datetime(utc_window_end)
    utc_dur_days = (utc_window_end - utc_window_start).total_seconds() / 86400.0

    print(
        f"   UTC window: [{utc_window_start.isoformat()} -> "
        f"{utc_window_end.isoformat()}]  Δ = {utc_dur_days:.6f} days"
    )

    for label, scale in [
        ("JD", TimeScale.JD),
        ("MJD", TimeScale.MJD),
        ("UT", TimeScale.UT),
        ("Unix", TimeScale.UnixTime),
    ]:
        s_val = convert_timescale(jd_start.value, TimeScale.JD, scale)
        e_val = convert_timescale(jd_window_end.value, TimeScale.JD, scale)
        dur = e_val - s_val
        print(f"   {label:<10} [{s_val:>18.9f} -> {e_val:>18.9f}]  Δ = {dur:.6f} days")

    # MJD -> UTC roundtrip
    mjd_start = jd_start.to_mjd()
    mjd_end_val = jd_window_end.to_mjd()
    jd_rt_start = mjd_start.to_jd()
    jd_rt_end = mjd_end_val.to_jd()
    print(
        f"   UTC<-MJD  [{jd_rt_start.to_datetime().isoformat()} -> "
        f"{jd_rt_end.to_datetime().isoformat()}]"
    )
    print()

    # ── 5) TAI - UTC offset ───────────────────────────────────────────
    print("5) Leap-second offset (TAI - UTC):")
    tai_utc = tai_minus_utc(jd)
    print(f"   TAI - UTC at J2000.5: {tai_utc:.1f} s")
    print()

    # ── 6) Period intersection ────────────────────────────────────────
    print("6) Period intersection:")

    p1 = TimePeriod(JulianDate(jd_val), JulianDate(jd_val + 1.0))
    bounds = TimePeriod(JulianDate(jd_val + 0.5), JulianDate(jd_val + 1.5))

    result = intersect_periods([p1], bounds)
    print(f"   Period A: [{p1.start} -> {p1.end}]")
    print(f"   Bounds:   [{bounds.start} -> {bounds.end}]")
    for idx, r in enumerate(result):
        dur = r.duration_days()
        print(f"   Intersection {idx + 1}: [{r.start} -> {r.end}]  Δ = {dur:.6f} days")
    print()

    print("=== Example Complete ===")


if __name__ == "__main__":
    main()
