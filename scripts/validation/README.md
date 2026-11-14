# Independent Siderust references

These scripts generate references for the Rust Siderust regression tests.
They call CSPICE, ERFA with jplephem, or Vallado's C++ SGP4 implementation
directly. Run commands from the siderust-py checkout with a separate Siderust
checkout that has the fixture directories below.

- [Lunar DE440 and surface references](cspice/README.md)
- Compact ephemeris and SGP4 commands below

## Compact ephemeris

Obtain DE440s from NAIF. The SHA-256 of `de440s.bsp` is
`c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2`.

```sh
siderust=/path/to/siderust
uv run --python 3.14 --with numpy==2.5.4 --with pyerfa==2.0.1.5 --with jplephem==2.24 \
  scripts/validation/compact_reference.py /path/to/de440s.bsp > /tmp/compact_reference.csv
cmp /tmp/compact_reference.csv "$siderust/tests/fixtures/compact_reference.csv"
```

## SGP4

The generator reads `vallado.txt` and overwrites `iau.txt` and
`synthetic.txt` in the specified directory. For verification, copy
`vallado.txt` into a temporary directory, pass that directory to
`--fixtures`, and compare the generated files against Siderust.

```sh
uv run --python 3.12 --with sgp4==2.25 scripts/validation/generate_iau.py \
  --fixtures "$siderust/tests/fixtures/sgp4"
```
