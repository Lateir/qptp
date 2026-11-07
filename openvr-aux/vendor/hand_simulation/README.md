# Valve hand simulation

`hand_simulation.h`, `hand_simulation.cpp` and `vrmath.h` are from Valve's OpenVR samples:

- https://github.com/ValveSoftware/openvr/tree/master/samples/drivers/drivers/handskeletonsimulation/src
- https://github.com/ValveSoftware/openvr/blob/master/samples/drivers/utils/vrmath/vrmath.h

Retrieved on 2026-11-07. Original copyright headers are retained. See LICENSE.
The sample notes its inspiration from Monado's hand simulation.
The sample is used for approximate hand geometry, not as measured Quest Pro controller calibration.
QPTP supplies auxiliary bones, normalization, sensor mapping and animation blending separately.

The source sample's wrist offset originated from an Index controller. Physical wrist alignment of
the synthesized fallback hand must be verified against the Quest Pro render model on hardware.

The OpenVR application header in `../openvr/openvr.h` comes from the same upstream SDK and is
used by the manual verification client. `driver_qptp/LICENSE.Valve.txt` accompanies the driver binary.
