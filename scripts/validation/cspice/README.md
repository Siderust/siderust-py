# Independent CSPICE references

The generators use CSPICE N0067 through SpiceyPy 8.3.0 for frames, time
conversion, positions, and kernel excerpts. NumPy ENU calculations supply
surface angles and ranges independently of Siderust. Both generators check
the kernel hashes and toolkit version against Siderust's lunar provenance
manifest before loading the kernels.

Obtain the five kernels listed in Siderust's `doc/guides/spice-kernels.md`.
Run these commands from the siderust-py checkout.

```sh
siderust=/path/to/siderust
kernel_dir=/path/to/naif-kernels
fixtures="$siderust/tests/fixtures/lunar_de440"
uv venv --python 3.14 /tmp/lunar-reference-env
uv pip install --python /tmp/lunar-reference-env/bin/python \
  -r scripts/validation/cspice/requirements.txt
/tmp/lunar-reference-env/bin/python scripts/validation/cspice/cspice_lunar_de440_references.py \
  --fixtures "$fixtures" --kernel-dir "$kernel_dir" --verify
```

Verification compares newly generated artifacts byte for byte in a temporary
directory. Use `--output-dir /tmp/lunar-de440` in place of `--verify` to
inspect a regenerated set. Omit both options to update the fixture directory.

The surface generator prints the Moon orientation at J2000 TT, independent
lunar surface geometry, and Mars planetographic longitude conversions for
comparison with Siderust's `tests/body_surface_horizontal.rs`.

```sh
CSPICE_PCK="$kernel_dir/pck00011.tpc" CSPICE_LSK="$kernel_dir/naif0012.tls" \
  /tmp/lunar-reference-env/bin/python scripts/validation/cspice/cspice_surface_references.py \
  --fixtures "$fixtures"
```
