# Lunar DE440 CSPICE reference fixtures

Independent geometric reference data generated with **SpiceyPy 8.3.0 /
CSPICE N0067** (`abcorr=NONE`), copied from Siderust PR
[#125](https://github.com/Siderust/siderust/pull/125)
(`tests/fixtures/lunar_de440`).

| File | Role |
|------|------|
| `excerpt.bsp` / `excerpt.bpc` | Small original SPK/BPC excerpts (~16 KiB total) |
| `moon_de440_220930.tf` | Archived DE440 lunar FK |
| `naif0012.tls` | NAIF leap-second kernel |
| `test_rotation.tf` | Synthetic large fixed rotation for coverage tests |
| `reference.csv` | Offline CSPICE observation vectors |
| `external_reference.csv` | Optional full-kernel parity cases |
| `coverage_rotations.csv` | Optional BPC endpoint rotations |
| `time_reference.csv` | Historical UTC/TT/TDB conversion cases |
| `provenance.json` | Generator metadata and source SHA-256 hashes |

Full NAIF kernels (`de440s.bsp`, `moon_pa_de440_200625.bpc`, …) are **not**
committed. Optional tests load them from `SIDERUST_LUNAR_KERNEL_DIR`.

Redistribution of the small excerpts and text kernels follows their upstream
NAIF / CSPICE licensing and attribution. Do not treat these fixtures as a
substitute for the published DE440 kernel set in operational work.
