# Primary Hardware Envelope

**Machine**: Mac M5 Max (Apple Silicon)

- Model `Mac17,6`, 128 GB unified memory (`hw.memsize` = 137438953472), macOS 26.3.2, confirmed on the device on 2026-11-03.
- 18 CPU cores in two tiers (macmon reports a 12-core tier it labels `P` and a 6-core tier it labels `S`; `sysctl` reports `hw.perflevel0.physicalcpu` = 6 and `hw.perflevel1.physicalcpu` = 12, so which tier is the performance tier is an open question), 40 GPU cores (`ioreg gpu-core-count`).
- No discrete NVIDIA GPU; all acceleration uses the Apple GPU / Metal or CPU.

## Usage rules for Primness
- All independent D1 measurements (polynomial evaluation, smoothness counting, Dickman comparisons) must complete on this machine within the < 1 hour wall-clock success criterion stated in the roadmap for the target size, or be explicitly marked as scaled-down.
- External tools (CADO-NFS, msieve, YAFU) are invoked only as instruments; their resource use is observed, not inherited as a limit.
- Docker containers may be used for reproducible builds or isolation of external instruments.
- Dashboards may be added to surface measurement results and CI status without introducing brittle gates. The local bench is `docs/dashboard.md` (loopback port 9477).
- Every instrument run of 30 digits or more records a hardware trace (`hardware-sample/1`): CPU and GPU activity, unified memory and the GPU's share of it, temperature, fans and power, sampled each second. See `docs/hardware_telemetry.md`.

## Relation to Obstacle D
Hardware capacity is part of the concrete resource limits against which any claimed smoothness excess must be measured. If a direction requires resources beyond this Mac, it is stated as a pivot toward Obstacle H rather than assumed available.
